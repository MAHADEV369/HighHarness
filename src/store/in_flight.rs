//! In-flight run tracking, per `HARNESS_PRIMITIVES.md` §3.4.

use std::fs;
use std::path::Path;

use crate::error::HxResult;
use crate::schema::in_flight::InFlight;
use crate::store::in_flight_path;
use crate::store::locks::FileLock;

/// Append a new in-flight line. Acquires `in-flight.lock`.
pub fn open(root: &Path, line: InFlight) -> HxResult<()> {
    let lock_path = crate::store::locks_dir(root).join("in-flight.lock");
    let _lock = FileLock::acquire(&lock_path, 5000)?;
    let path = in_flight_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    use std::io::Write;
    writeln!(f, "{}", serde_json::to_string(&line)?)?;
    f.sync_data()?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

/// Remove the in-flight line for a given run_id. Acquires `in-flight.lock`.
pub fn close(root: &Path, run_id: &str) -> HxResult<()> {
    let lock_path = crate::store::locks_dir(root).join("in-flight.lock");
    let _lock = FileLock::acquire(&lock_path, 5000)?;
    let path = in_flight_path(root);
    if !path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(&path)?;
    let mut kept: Vec<String> = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line).unwrap_or_default();
        if v.get("run_id").and_then(|x| x.as_str()) == Some(run_id) {
            continue;
        }
        kept.push(line.to_string());
    }
    let mut s = kept.join("\n");
    if !s.is_empty() {
        s.push('\n');
    }
    fs::write(&path, s)?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

/// Move any in-flight lines whose pid is not alive into a `reaped` line in-place.
pub fn reap_stale(root: &Path) -> HxResult<()> {
    let path = in_flight_path(root);
    if !path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(&path)?;
    let mut out_lines: Vec<String> = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut v: serde_json::Value = serde_json::from_str(line)?;
        if v.get("state").and_then(|x| x.as_str()) == Some("reaped") {
            out_lines.push(line.to_string());
            continue;
        }
        let pid = v.get("pid").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
        if pid == 0 || !crate::store::locks::pid_alive(pid) {
            v["state"] = serde_json::Value::String("reaped".to_string());
            v["reaped_at"] = serde_json::Value::String(crate::id::now_iso());
            out_lines.push(serde_json::to_string(&v)?);
        } else {
            out_lines.push(line.to_string());
        }
    }
    let mut s = out_lines.join("\n");
    if !s.is_empty() {
        s.push('\n');
    }
    fs::write(&path, s)?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::in_flight::InFlight;
    use tempfile::TempDir;

    fn mk_line(run_id: &str, state: &str) -> InFlight {
        /// Variant `InFlight` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        InFlight {
            schema_version: 1,

            run_id: run_id.to_string(),
            /// Field `agent_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            agent_id: "a".to_string(),

            opened_at: crate::id::now_iso(),
            /// Field `phase` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            phase: "highharness".to_string(),
            /// Field `tier` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            tier: "trivial".to_string(),

            state: state.to_string(),

            pid: None,
        }
    }

    #[test]
    fn in_flight_open_close_pair_consistent() {
        let dir = TempDir::new().unwrap();
        let l = mk_line("r1", "live");
        open(dir.path(), l).unwrap();
        let path = in_flight_path(dir.path());
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("\"r1\""));
        close(dir.path(), "r1").unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("\"r1\""));
    }

    #[test]
    fn in_flight_stale_line_reaped_on_startup() {
        let dir = TempDir::new().unwrap();
        // Write a line with a fake (definitely-dead) pid.
        let path = in_flight_path(dir.path());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let line = serde_json::json!({
            "schema_version": 1,
            "run_id": "stale",
            "agent_id": "a",
            "opened_at": crate::id::now_iso(),
            "phase": "highharness",
            "tier": "trivial",
            "state": "live",
            "pid": 9_999_999_u32,
        });
        std::fs::write(
            &path,
            format!("{}\n", serde_json::to_string(&line).unwrap()),
        )
        .unwrap();
        reap_stale(dir.path()).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("\"reaped\""));
    }
}
