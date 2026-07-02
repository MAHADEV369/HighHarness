//! Episode trace persistence, per `HARNESS_PRIMITIVES.md` §3.4.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::canonical;
use crate::error::{HxError, HxResult};
use crate::schema::episode::{Decision, Failure, Intervention, ToolCall};
use crate::store::{episodes_dir, in_flight_path};

/// Open a new episode file. Refuses if `run_id` already exists on disk
/// (collision → `RunIdCollision`).
pub fn open(
    root: &Path,

    run_id: &str,

    agent_id: &str,

    task_spec: &str,

    tier: &str,

    phase: &str,
) -> HxResult<()> {
    let dir = episodes_dir(root);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", run_id));
    if path.exists() {
        return Err(HxError::RunIdCollision(run_id.to_string()));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write in fixed-section order per §3.4.1.
    let mut content = String::new();
    content.push_str(&format!("# Episode {}\n\n", run_id));
    content.push_str("## Task spec\n");
    content.push_str(task_spec);
    if !task_spec.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\n## Plan\n\n## Task state log\n\n## Tool calls\n\n## Decisions\n\n## Failures\n\n## Interventions\n\n## Pre-task checklist\n\n## Verification report\n\n## Files touched\n\n");
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)?;
    f.write_all(content.as_bytes())?;
    f.sync_data()?;

    // Add in-flight line.
    let ifp = in_flight_path(root);
    if let Some(parent) = ifp.parent() {
        fs::create_dir_all(parent)?;
    }
    let line = serde_json::json!({
        "schema_version": 1,
        "run_id": run_id,
        "agent_id": agent_id,
        "opened_at": crate::id::now_iso(),
        "phase": phase,
        "tier": tier,
        "state": "live",
        "pid": std::process::id(),
    });
    let mut f2 = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&ifp)?;
    writeln!(f2, "{}", serde_json::to_string(&line)?)?;

    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

/// Append a free-form body to a named section of the episode file.
pub fn append(root: &Path, run_id: &str, section: &str, body: &str) -> HxResult<()> {
    let path = episodes_dir(root).join(format!("{}.md", run_id));
    let mut txt = fs::read_to_string(&path)?;
    let marker = format!("\n## {}", section);
    if let Some(idx) = txt.find(&marker) {
        // Find the end of the section header line.
        let after = idx + marker.len();
        let insert_at = txt[after..]
            .find('\n')
            .map(|i| after + i + 1)
            .unwrap_or(txt.len());
        let body = if body.ends_with('\n') {
            body.to_string()
        } else {
            format!("{}\n", body)
        };
        txt.insert_str(insert_at, &body);
    } else {
        // Section missing; append a new section at the end.
        txt.push_str(&format!("\n## {}\n{}\n", section, body));
    }
    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)?;
    f.write_all(txt.as_bytes())?;
    f.sync_data()?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

/// Append a tool call to `## Tool calls`.
pub fn append_tool_call(root: &Path, run_id: &str, tc: ToolCall) -> HxResult<()> {
    let s = serde_json::to_string(&tc)?;
    append(root, run_id, "Tool calls", &format!("- {}\n", s))
}

/// Append a decision to `## Decisions`.
pub fn append_decision(root: &Path, run_id: &str, d: Decision) -> HxResult<()> {
    let s = serde_json::to_string(&d)?;
    append(root, run_id, "Decisions", &format!("- {}\n", s))
}

/// Append a failure to `## Failures`.
pub fn append_failure(root: &Path, run_id: &str, f: Failure) -> HxResult<()> {
    let s = serde_json::to_string(&f)?;
    append(root, run_id, "Failures", &format!("- {}\n", s))
}

/// Append an intervention to `## Interventions`.
pub fn append_intervention(root: &Path, run_id: &str, i: Intervention) -> HxResult<()> {
    let s = serde_json::to_string(&i)?;
    append(root, run_id, "Interventions", &format!("- {}\n", s))
}

/// Close the episode: writes the verification report + files touched, computes
/// the canonical `episode_hash`, removes the in-flight line. Returns the
/// computed hash.
pub fn close(
    root: &Path,

    run_id: &str,

    verification_report: &str,

    files_touched: Vec<String>,
) -> HxResult<String> {
    // Update the verification report section.
    let path = episodes_dir(root).join(format!("{}.md", run_id));
    let mut txt = fs::read_to_string(&path)?;
    // Replace placeholder "## Verification report" content if empty.
    let marker = "## Verification report";
    if let Some(idx) = txt.find(marker) {
        let after = idx + marker.len();
        let insert_at = txt[after..]
            .find('\n')
            .map(|i| after + i + 1)
            .unwrap_or(txt.len());
        // Truncate from insert_at to the next section header.
        let rest = &txt[insert_at..];
        let next_section = rest.find("\n## ").unwrap_or(rest.len());
        let end = insert_at + next_section;
        txt.replace_range(insert_at..end, "");
        let v = if verification_report.ends_with('\n') {
            verification_report.to_string()
        } else {
            format!("{}\n", verification_report)
        };
        txt.insert_str(insert_at, &v);
    }

    // Update the files touched section.
    let marker = "## Files touched";
    if let Some(idx) = txt.find(marker) {
        let after = idx + marker.len();
        let insert_at = txt[after..]
            .find('\n')
            .map(|i| after + i + 1)
            .unwrap_or(txt.len());
        let rest = &txt[insert_at..];
        let next_section = rest.find("\n## ").unwrap_or(rest.len());
        let end = insert_at + next_section;
        txt.replace_range(insert_at..end, "");
        let mut buf = String::new();
        for f in &files_touched {
            buf.push_str(f);
            buf.push('\n');
        }
        txt.insert_str(insert_at, &buf);
    }

    // Compute the canonical episode hash over the byte range that EXCLUDES
    // the `## Episode hash` section. We haven't written it yet, so the full
    // body is the input.
    let h = canonical::episode_hash(&txt);

    // Append the ## Episode hash section.
    txt.push_str(&format!("\n## Episode hash\nSHA-256: {}\n", h));

    fs::write(&path, &txt)?;

    // Remove the in-flight line for this run.
    let ifp = in_flight_path(root);
    if ifp.exists() {
        let raw = fs::read_to_string(&ifp)?;
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
        fs::write(&ifp, s)?;
    }

    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(h)
}

/// Return the canonical episode_hash of an already-closed episode.
pub fn hash(root: &Path, run_id: &str) -> HxResult<String> {
    let path = episodes_dir(root).join(format!("{}.md", run_id));
    let txt = fs::read_to_string(&path)?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(canonical::episode_hash(&txt))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn episode_open_creates_file_with_required_sections_in_order() {
        let dir = TempDir::new().unwrap();
        open(
            dir.path(),
            "2026-01-01T0000Z-test-agent_x-abcd",
            "agent_x",
            "spec body",
            "trivial",
            "highharness",
        )
        .unwrap();
        let path = episodes_dir(dir.path()).join("2026-01-01T0000Z-test-agent_x-abcd.md");
        let txt = std::fs::read_to_string(&path).unwrap();
        let order: Vec<&str> = vec![
            "Task spec",
            "Plan",
            "Task state log",
            "Tool calls",
            "Decisions",
            "Failures",
            "Interventions",
            "Pre-task checklist",
            "Verification report",
            "Files touched",
        ];
        let mut last = 0;
        for s in order {
            let idx = txt.find(&format!("## {}", s)).unwrap();
            assert!(idx >= last, "section out of order: {}", s);
            last = idx;
        }
    }

    #[test]
    fn episode_open_refuses_duplicate_run_id() {
        let dir = TempDir::new().unwrap();
        open(dir.path(), "dup", "a", "spec", "trivial", "highharness").unwrap();
        let err = open(dir.path(), "dup", "a", "spec", "trivial", "highharness").unwrap_err();
        assert!(matches!(err, HxError::RunIdCollision(_)));
    }

    #[test]
    fn episode_close_writes_episode_hash_and_removes_in_flight_line() {
        let dir = TempDir::new().unwrap();
        open(dir.path(), "r1", "a", "spec", "trivial", "highharness").unwrap();
        let h = close(
            dir.path(),
            "r1",
            "- syntactic: Y\n- functional: Y\n- semantic: Y\n- regression: Y\n- attribution: Y\n- memory: Y\n",
            vec!["a.txt".to_string()],
        )
        .unwrap();
        assert_eq!(h.len(), 64);
        let path = episodes_dir(dir.path()).join("r1.md");
        let txt = std::fs::read_to_string(&path).unwrap();
        assert!(txt.contains(&format!("SHA-256: {}", h)));
        let ifp = in_flight_path(dir.path());
        let raw = std::fs::read_to_string(&ifp).unwrap_or_default();
        assert!(!raw.contains("\"r1\""));
    }
}
