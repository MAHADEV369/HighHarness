//! `fs.hash` tool — SHA-256 of a file at `path`.

use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use serde_json::{json, Value};

use crate::error::{HxError, HxResult};
use crate::schema::tool::{ToolContent, ToolMeta, ToolResult};

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(args: Value, root: &Path) -> HxResult<ToolResult> {
    let path = args
        .get("path")
        .and_then(|x| x.as_str())
        .ok_or_else(|| HxError::Other("fs.hash: missing 'path'".to_string()))?;
    let full = root.join(path);
    let bytes = fs::read(&full)?;
    let mut h = Sha256::new();
    h.update(&bytes);
    let hex = format!("{:x}", h.finalize());
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(ToolResult {
        schema_version: 1,

        ok: true,
        content: ToolContent {
            /// Field `kind` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            kind: "text".to_string(),

            value: Value::String(hex),
        },
        meta: ToolMeta {
            duration_ms: 0,

            bytes: Some(bytes.len() as u64),

            exit_code: None,
        },

        redactions: vec![],

        approval_id: None,

        tool_call_id: String::new(),
    })
}

#[allow(dead_code)]
/// fn `descriptor` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn descriptor() -> serde_json::Value {
    json!({
        "id": "fs.hash",
        "schema_version": 1,
        "version": "1.0.0",
        "source": "builtin",
        "summary": "SHA-256 of a file at path.",
        "capabilities": {
            "read": true, "write": false, "exec": false,
            "network": false, "destructive": false, "secrets": false,
            "side_effect": "read"
        },
        "side_effect": "read",
        "approval": { "mode": "auto", "reason": "read-only" }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn fs_hash_returns_correct_sha256() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        let args = serde_json::json!({"path": "a.txt"});
        let r = run(args, dir.path()).unwrap();
        assert!(r.ok);
        // SHA-256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        let s = r.content.value.as_str().unwrap();
        assert_eq!(
            s,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
