//! Spend ledger, per `HARNESS_PRIMITIVES.md` §9.1.

use std::fs;
use std::io::Write;
use std::path::Path;

use chrono::Datelike;
use serde_json::{json, Value};

use crate::error::{HxError, HxResult};
use crate::schema::spend::SpendLine;
use crate::store::locks::FileLock;
use crate::store::spend_dir;

use sha2::{Digest, Sha256};

/// Append a spend line. Serialized via `spend.lock`.
pub fn append(root: &Path, mut line: SpendLine) -> HxResult<()> {
    let lock_path = crate::store::locks_dir(root).join("spend.lock");
    let _lock = FileLock::acquire(&lock_path, 5000)?;
    // W5: compute self_hash before write
    let mut for_hash = line.clone();
    for_hash.self_hash = None;
    let json = serde_json::to_string(&for_hash)?;
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    line.self_hash = Some(format!("{:x}", hasher.finalize()));
    let month = line.ts.get(0..7).unwrap_or("1970-01");
    let dir = spend_dir(root);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.jsonl", month));
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let s = serde_json::to_string(&line)?;
    writeln!(f, "{}", s)?;
    f.sync_data()?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

/// Summarize spend for a given month (YYYY-MM). Returns a JSON object.
pub fn summary(root: &Path, month: &str) -> HxResult<Value> {
    let path = spend_dir(root).join(format!("{}.jsonl", month));
    let raw = fs::read_to_string(&path).unwrap_or_default();
    let mut total_usd = 0.0;
    let mut total_in = 0u64;
    let mut total_out = 0u64;
    let mut count = 0u64;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let v: SpendLine = serde_json::from_str(line)?;
        // W5: verify self_hash on non-legacy rows
        if let Some(ref h) = v.self_hash {
            let mut for_hash = v.clone();
            for_hash.self_hash = None;
            let json = serde_json::to_string(&for_hash)?;
            let mut hasher = Sha256::new();
            hasher.update(json.as_bytes());
            let computed = format!("{:x}", hasher.finalize());
            if &computed != h {
                return Err(HxError::AuditForgery(
                    "spend row self_hash mismatch".to_string(),
                ));
            }
        }
        total_usd += v.usd;
        total_in += v.input_tokens;
        total_out += v.output_tokens;
        count += 1;
    }
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(json!({
        "month": month,
        "count": count,
        "total_usd": total_usd,
        "total_input_tokens": total_in,
        "total_output_tokens": total_out,
    }))
}

/// Helper: current month string (YYYY-MM) in UTC.
#[allow(dead_code)]
/// fn `current_month` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn current_month() -> String {
    let now = chrono::Utc::now();
    format!("{:04}-{:02}", now.year(), now.month())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::spend::SpendLine;
    use serde_json::json;
    use tempfile::TempDir;

    fn mk_line(usd: f64) -> SpendLine {
        /// Variant `SpendLine` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        SpendLine {
            schema_version: 1,

            ts: format!("{}-01T00:00:00Z", "2026-01"),
            /// Field `run_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            run_id: "r".to_string(),
            /// Field `agent_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            agent_id: "a".to_string(),
            /// Field `model_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            model_id: "m".to_string(),
            /// Field `feature` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            feature: "chat".to_string(),

            input_tokens: 100,

            output_tokens: 50,

            reasoning_tokens: 0,
            usd,
            /// Field `routing_mode` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            routing_mode: "manual".to_string(),
            /// Field `provider` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            provider: "anthropic".to_string(),

            metadata: json!({}),

            self_hash: None,
        }
    }

    #[test]
    fn spend_append_and_summary_consistent() {
        let dir = TempDir::new().unwrap();
        append(dir.path(), mk_line(0.5)).unwrap();
        append(dir.path(), mk_line(0.5)).unwrap();
        let s = summary(dir.path(), "2026-01").unwrap();
        assert_eq!(s["count"], 2);
        assert!((s["total_usd"].as_f64().unwrap() - 1.0).abs() < 1e-9);
        assert_eq!(s["total_input_tokens"], 200);
        assert_eq!(s["total_output_tokens"], 100);
    }
}
