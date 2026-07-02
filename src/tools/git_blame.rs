//! `git.blame` tool.

use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

use crate::error::{HxError, HxResult};
use crate::schema::tool::{ToolContent, ToolMeta, ToolResult};

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(args: Value, root: &Path) -> HxResult<ToolResult> {
    let path = args
        .get("path")
        .and_then(|x| x.as_str())
        .ok_or_else(|| HxError::Other("git.blame: missing 'path'".to_string()))?;
    let line_range = args.get("lines").and_then(|x| x.as_str()).unwrap_or("");
    let mut cmd = Command::new("git");
    cmd.arg("blame");
    if !line_range.is_empty() {
        cmd.args(["-L", line_range]);
    }
    cmd.arg("--").arg(path);
    let out = cmd
        .current_dir(root)
        .output()
        .map_err(|e| HxError::Other(format!("git.blame spawn: {}", e)))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let exit = out.status.code().unwrap_or(-1);
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(ToolResult {
        schema_version: 1,

        ok: out.status.success(),
        content: ToolContent {
            /// Field `kind` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            kind: "text".to_string(),

            value: Value::String(stdout),
        },
        meta: ToolMeta {
            duration_ms: 0,

            bytes: None,

            exit_code: Some(exit),
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
        "id": "git.blame",
        "schema_version": 1,
        "version": "1.0.0",
        "source": "builtin",
        "summary": "Run `git blame` on a file (optionally a line range).",
        "capabilities": {
            "read": true, "write": false, "exec": false,
            "network": false, "destructive": false, "secrets": false,
            "side_effect": "read"
        },
        "side_effect": "read",
        "approval": { "mode": "auto", "reason": "read-only" }
    })
}
