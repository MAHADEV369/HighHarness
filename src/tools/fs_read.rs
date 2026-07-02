//! `fs.read` tool — read a file as text or bytes (≤ 1 MiB default).

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::error::{HxError, HxResult};
use crate::schema::tool::{ToolContent, ToolMeta, ToolResult};

const DEFAULT_MAX_BYTES: u64 = 1024 * 1024;

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(args: Value, root: &Path) -> HxResult<ToolResult> {
    let path = args
        .get("path")
        .and_then(|x| x.as_str())
        .ok_or_else(|| HxError::Other("fs.read: missing 'path'".to_string()))?;
    let max = args
        .get("max_bytes")
        .and_then(|x| x.as_u64())
        .unwrap_or(DEFAULT_MAX_BYTES);
    let full = root.join(path);
    let meta = fs::metadata(&full)?;
    if meta.len() > max {
        return Err(HxError::Other(format!(
            "fs.read: file too large ({} > {})",
            meta.len(),
            max
        )));
    }
    let bytes = fs::read(&full)?;
    let kind = if path.ends_with(".bin") || path.ends_with(".png") || path.ends_with(".jpg") {
        "bytes"
    } else {
        "text"
    };
    let value = if kind == "text" {
        Value::String(String::from_utf8_lossy(&bytes).to_string())
    } else {
        Value::String(format!("(base64, {} bytes)", bytes.len()))
    };
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(ToolResult {
        schema_version: 1,

        ok: true,
        content: ToolContent {
            kind: kind.to_string(),
            value,
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
        "id": "fs.read",
        "schema_version": 1,
        "version": "1.0.0",
        "source": "builtin",
        "summary": "Read a file as text or bytes.",
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
    fn fs_read_returns_text_content() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("hello.txt"), "hi\n").unwrap();
        let args = serde_json::json!({"path": "hello.txt"});
        let r = run(args, dir.path()).unwrap();
        assert!(r.ok);
        assert_eq!(r.content.kind, "text");
        assert_eq!(r.content.value, serde_json::json!("hi\n"));
    }

    #[test]
    fn fs_read_missing_path_returns_error() {
        let dir = TempDir::new().unwrap();
        let args = serde_json::json!({"path": "nope.txt"});
        let r = run(args, dir.path());
        assert!(r.is_err());
    }

    #[test]
    fn fs_read_too_large_returns_error() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("big.bin"), vec![0u8; 100]).unwrap();
        let args = serde_json::json!({"path": "big.bin", "max_bytes": 50});
        let r = run(args, dir.path());
        assert!(r.is_err());
    }
}
