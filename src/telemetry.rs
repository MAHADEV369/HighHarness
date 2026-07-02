//! Telemetry layer: the integrity log (line-chained JSONL).

use std::fs;
use std::io::Write;
use std::path::Path;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::error::{HxError, HxResult};
use crate::schema::integrity::IntegrityLine;
use crate::store::integrity_log_path;

/// mod `integrity` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod integrity {
    use super::*;

    /// Append a line to the harness integrity log, computing its `this_hash`
    /// over the prior line + the new line body. Returns the `this_hash`.
    pub fn append(root: &Path, event: &str, details: Value) -> HxResult<String> {
        let path = crate::store::integrity_log_path(root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let prev_hash = read_latest_hash(&path).unwrap_or_else(|_| "0".repeat(64));
        let at = crate::id::now_iso();
        let line = IntegrityLine {
            schema_version: 1,

            event: event.to_string(),
            at,
            run_id: details
                .get("run_id")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),

            prev_hash: prev_hash.clone(),

            this_hash: String::new(),

            details: details.clone(),
        };
        let body = serde_json::to_string(&line)?;
        let mut h = Sha256::new();
        h.update(prev_hash.as_bytes());
        h.update(b"\n");
        h.update(body.as_bytes());
        let this_hash = format!("{:x}", h.finalize());
        let final_line = IntegrityLine {
            this_hash: this_hash.clone(),
            ..line
        };
        let final_body = serde_json::to_string(&final_line)?;
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(f, "{}", final_body)?;
        f.sync_data()?;
        /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        Ok(this_hash)
    }

    /// Append the very first line of the integrity log (no prior chain head).
    pub fn append_seed(root: &Path, event: &str, human: &str) -> HxResult<String> {
        append(root, event, json!({"human": human, "seed": true}))
    }

    /// Verify the integrity log chain. Returns indices of broken lines (1-based
    /// by line number). Empty Vec = healthy.
    pub fn verify(root: &Path) -> HxResult<Vec<usize>> {
        let path = crate::store::integrity_log_path(root);
        if !path.exists() {
            return Ok(vec![]);
        }
        let raw = fs::read_to_string(&path)?;
        let mut prev = "0".repeat(64);
        let mut broken = Vec::new();
        for (i, line) in raw.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let v: IntegrityLine = serde_json::from_str(line)?;
            // Reconstruct body with this_hash blanked and verify chain.
            let mut blanked = v.clone();
            blanked.this_hash = String::new();
            let body = serde_json::to_string(&blanked)?;
            let mut h = Sha256::new();
            h.update(prev.as_bytes());
            h.update(b"\n");
            h.update(body.as_bytes());
            let computed = format!("{:x}", h.finalize());
            if computed != v.this_hash || v.prev_hash != prev {
                broken.push(i + 1);
            }
            prev = v.this_hash;
        }
        /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        Ok(broken)
    }

    fn read_latest_hash(path: &Path) -> HxResult<String> {
        if !path.exists() {
            return Err(HxError::Other("no integrity log".to_string()));
        }
        let raw = fs::read_to_string(path)?;
        let last = raw
            .lines()
            .filter(|l| !l.trim().is_empty())
            .last()
            .ok_or_else(|| HxError::Other("empty log".to_string()))?;
        let v: IntegrityLine = serde_json::from_str(last)?;
        /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        Ok(v.this_hash)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use tempfile::TempDir;

        #[test]
        fn integrity_log_appends_and_verifies() {
            let dir = TempDir::new().unwrap();
            let h1 = append(dir.path(), "ev1", serde_json::json!({})).unwrap();
            let h2 = append(dir.path(), "ev2", serde_json::json!({})).unwrap();
            assert!(!h1.is_empty());
            assert_ne!(h1, h2);
            let broken = verify(dir.path()).unwrap();
            assert!(broken.is_empty(), "broken: {:?}", broken);
        }

        #[test]
        fn integrity_log_detects_tamper() {
            let dir = TempDir::new().unwrap();
            append(dir.path(), "ev1", serde_json::json!({})).unwrap();
            append(dir.path(), "ev2", serde_json::json!({})).unwrap();
            // Tamper with the log.
            let p = crate::store::integrity_log_path(dir.path());
            let raw = std::fs::read_to_string(&p).unwrap();
            let tampered = raw.replacen("ev1", "EVIL", 1);
            std::fs::write(&p, tampered).unwrap();
            let broken = verify(dir.path()).unwrap();
            assert!(!broken.is_empty());
        }
    }
}
