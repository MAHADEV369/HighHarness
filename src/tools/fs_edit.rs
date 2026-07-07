//! `fs.edit` tool — atomic in-place file edit.
//!
//! Accepts either:
//! - `{"path":..., "old":"...", "new":"..."}` (substring replace)
//! - `{"path":..., "replace_start":N, "replace_end":N, "new":"..."}` (byte range)
//! - `{"path":..., "insert_after_line":N, "new":"..."}` (line insert)
//!
//! Atomic write = write-to-temp + rename. Always commits or fails cleanly.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde_json::{json, Value};

use crate::error::{HxError, HxResult};
use crate::schema::tool::{ToolContent, ToolMeta, ToolResult};

/// Execute an atomic in-place file edit.
pub fn run(args: Value, root: &Path) -> HxResult<ToolResult> {
    let path = args
        .get("path")
        .and_then(|x| x.as_str())
        .ok_or_else(|| HxError::Other("fs.edit: missing 'path'".to_string()))?;
    let full = crate::tools::resolve_safe_path(root, path)
        .map_err(|e| HxError::Other(format!("fs.edit path error: {}", e)))?;
    let original = fs::read_to_string(&full).unwrap_or_default();

    let new_content = if let Some(old) = args.get("old").and_then(|x| x.as_str()) {
        let new = args
            .get("new")
            .and_then(|x| x.as_str())
            .ok_or_else(|| HxError::Other("fs.edit: 'new' required with 'old'".to_string()))?;
        if !original.contains(old) {
            return Err(HxError::Other(format!(
                "fs.edit: 'old' substring not found in {}",
                path
            )));
        }
        original.replacen(old, new, 1)
    } else if let (Some(start), Some(end)) = (
        args.get("replace_start").and_then(|x| x.as_u64()),
        args.get("replace_end").and_then(|x| x.as_u64()),
    ) {
        let new = args
            .get("new")
            .and_then(|x| x.as_str())
            .ok_or_else(|| HxError::Other("fs.edit: 'new' required with byte range".to_string()))?;
        let s = start as usize;
        let e = end as usize;
        if s > original.len() || e > original.len() || s > e {
            return Err(HxError::Other("fs.edit: invalid byte range".to_string()));
        }
        let mut buf = String::with_capacity(original.len() - (e - s) + new.len());
        buf.push_str(&original[..s]);
        buf.push_str(new);
        buf.push_str(&original[e..]);
        buf
    } else if let Some(after) = args.get("insert_after_line").and_then(|x| x.as_u64()) {
        let new = args
            .get("new")
            .and_then(|x| x.as_str())
            .ok_or_else(|| HxError::Other("fs.edit: 'new' required".to_string()))?;
        let lines: Vec<&str> = original.split('\n').collect();
        let n = (after as usize).min(lines.len());
        let mut buf = String::new();
        for (i, l) in lines.iter().enumerate() {
            if i == n {
                buf.push_str(new);
                if !new.ends_with('\n') {
                    buf.push('\n');
                }
            }
            buf.push_str(l);
            if i + 1 < lines.len() {
                buf.push('\n');
            }
        }
        if n == lines.len() {
            buf.push_str(new);
            if !new.ends_with('\n') {
                buf.push('\n');
            }
        }
        buf
    } else {
        return Err(HxError::Other(
            "fs.edit: must provide (old+new) or (replace_start+replace_end+new) or (insert_after_line+new)"
                .to_string(),
        ));
    };

    // Atomic write: temp file + rename.
    let tmp = full.with_extension("hx.tmp");
    {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)?;
        f.write_all(new_content.as_bytes())?;
        f.sync_data()?;
    }
    fs::rename(&tmp, &full)?;

    Ok(ToolResult {
        schema_version: 1,

        ok: true,
        content: ToolContent {
            kind: "text".to_string(),

            value: Value::String(format!("wrote {} bytes to {}", new_content.len(), path)),
        },
        meta: ToolMeta {
            duration_ms: 0,

            bytes: Some(new_content.len() as u64),

            exit_code: None,
        },

        redactions: vec![],

        approval_id: None,

        tool_call_id: String::new(),
    })
}

#[allow(dead_code)]
/// Return the tool descriptor for `fs.edit`.
pub fn descriptor() -> serde_json::Value {
    json!({
        "id": "fs.edit",
        "schema_version": 1,
        "version": "1.0.0",
        "source": "builtin",
        "summary": "Atomic in-place file edit (substring, byte range, or line insert).",
        "capabilities": {
            "read": true, "write": true, "exec": false,
            "network": false, "destructive": false, "secrets": false,
            "side_effect": "write"
        },
        "side_effect": "write",
        "approval": { "mode": "auto", "reason": "writes path-scoped" }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn fs_edit_old_new_basic_replace() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("a.txt");
        std::fs::write(&p, "hello world").unwrap();
        let args = serde_json::json!({"path":"a.txt","old":"world","new":"rust"});
        let r = run(args, dir.path()).unwrap();
        assert!(r.ok);
        let after = std::fs::read_to_string(&p).unwrap();
        assert_eq!(after, "hello rust");
    }

    #[test]
    fn fs_edit_old_not_found_returns_error() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        let args = serde_json::json!({"path":"a.txt","old":"nope","new":"x"});
        let r = run(args, dir.path());
        assert!(r.is_err());
    }

    #[test]
    fn fs_edit_insert_after_line() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("a.txt");
        std::fs::write(&p, "line1\nline2\nline3\n").unwrap();
        let args = serde_json::json!({"path":"a.txt","insert_after_line":1,"new":"INSERTED"});
        let r = run(args, dir.path()).unwrap();
        assert!(r.ok);
        let after = std::fs::read_to_string(&p).unwrap();
        assert!(after.contains("INSERTED"));
    }

    #[test]
    fn fs_edit_replace_byte_range() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("a.txt");
        std::fs::write(&p, "hello world").unwrap();
        let args =
            serde_json::json!({"path":"a.txt","replace_start":6,"replace_end":11,"new":"rust"});
        let r = run(args, dir.path()).unwrap();
        assert!(r.ok);
        let after = std::fs::read_to_string(&p).unwrap();
        assert_eq!(after, "hello rust");
    }

    #[test]
    fn fs_edit_no_args_returns_error() {
        let dir = TempDir::new().unwrap();
        let args = serde_json::json!({"path":"a.txt"});
        let r = run(args, dir.path());
        assert!(r.is_err());
    }
}
