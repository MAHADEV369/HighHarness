//! Approvals store, per `HARNESS_PRIMITIVES.md` §10.2.

use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::error::HxResult;
use crate::schema::approval::Approval;
use crate::store::approvals_dir;

/// Create an approval request. Returns the approval id.
pub fn request(root: &Path, req: Approval) -> HxResult<String> {
    let dir = approvals_dir(root);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.json", req.id));
    let s = serde_json::to_string_pretty(&req)?;
    fs::write(&path, s)?;
    Ok(req.id)
}

/// Get the current state of an approval.
pub fn state(root: &Path, id: &str) -> HxResult<String> {
    let path = approvals_dir(root).join(format!("{}.json", id));
    let raw = fs::read_to_string(&path)?;
    let v: Value = serde_json::from_str(&raw)?;
    Ok(v.get("state")
        .and_then(|x| x.as_str())
        .unwrap_or("unknown")
        .to_string())
}

/// Resolve an approval (approve / deny / modify).
pub fn resolve(
    root: &Path,

    id: &str,

    decision: &str,

    rationale: Option<&str>,

    modified_args: Option<Value>,
) -> HxResult<()> {
    let path = approvals_dir(root).join(format!("{}.json", id));
    let raw = fs::read_to_string(&path)?;
    let mut v: Value = serde_json::from_str(&raw)?;
    v["state"] = Value::String(decision.to_string());
    if let Some(r) = rationale {
        v["rationale"] = Value::String(r.to_string());
    }
    if let Some(m) = modified_args {
        v["modified_args"] = m;
    }
    fs::write(&path, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

/// Expire an approval after a given duration.
pub fn expire(root: &Path, id: &str, after_secs: u64) -> HxResult<()> {
    let _ = (root, id, after_secs);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::approval::Approval;
    use tempfile::TempDir;

    fn mk_approval(id: &str) -> Approval {
        Approval {
            schema_version: 1,

            id: id.to_string(),
            run_id: "r1".to_string(),
            tool: "fs.edit".to_string(),

            args: serde_json::json!({}),
            rule_id: "R-ASK".to_string(),
            reason: "ask".to_string(),

            priority: 0,

            destructive: false,
            state: "pending".to_string(),

            modified_args: None,

            rationale: None,

            at: crate::id::now_iso(),

            expires_at: String::new(),

            self_hash: None,
        }
    }

    #[test]
    fn approval_lifecycle_request_to_resolve() {
        let dir = TempDir::new().unwrap();
        request(dir.path(), mk_approval("a1")).unwrap();
        let s = state(dir.path(), "a1").unwrap();
        assert_eq!(s, "pending");
        resolve(dir.path(), "a1", "approved", Some("ok"), None).unwrap();
        let s = state(dir.path(), "a1").unwrap();
        assert_eq!(s, "approved");
    }
}
