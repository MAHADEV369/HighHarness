//! `git.diff` tool.

use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

use crate::error::{HxError, HxResult};
use crate::schema::tool::{ToolContent, ToolMeta, ToolResult};

/// Run `git diff` against a target ref (default HEAD).
pub fn run(args: Value, root: &Path) -> HxResult<ToolResult> {
    let target = args
        .get("target")
        .and_then(|x| x.as_str())
        .unwrap_or("HEAD");
    let out = Command::new("git")
        .args(["diff", target])
        .current_dir(root)
        .output()
        .map_err(|e| HxError::Other(format!("git.diff spawn: {}", e)))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let exit = out.status.code().unwrap_or(-1);
    Ok(ToolResult {
        schema_version: 1,

        ok: out.status.success(),
        content: ToolContent {
            kind: "diff".to_string(),

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
/// Return the tool descriptor for `git.diff`.
pub fn descriptor() -> serde_json::Value {
    json!({
        "id": "git.diff",
        "schema_version": 1,
        "version": "1.0.0",
        "source": "builtin",
        "summary": "Run `git diff <target>` (default HEAD).",
        "capabilities": {
            "read": true, "write": false, "exec": false,
            "network": false, "destructive": false, "secrets": false,
            "side_effect": "read"
        },
        "side_effect": "read",
        "approval": { "mode": "auto", "reason": "read-only" }
    })
}
