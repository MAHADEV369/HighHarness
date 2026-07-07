//! `test.run` tool — thin wrapper around `shell.exec` that uses the
//! configured test command from `.harness/config.toml`.

use std::path::Path;

use serde_json::{json, Value};

use crate::error::HxResult;
use crate::schema::tool::{ToolMeta, ToolResult};
use crate::tools::read_tool_cmd;

/// Execute the configured test command via `shell.exec`.
pub fn run(args: Value, root: &Path) -> HxResult<ToolResult> {
    let phase = args
        .get("phase")
        .and_then(|x| x.as_str())
        .unwrap_or("highharness");
    let cmd = read_tool_cmd(root, "test_cmd").unwrap_or_else(|| "true".to_string());
    let _ = phase;
    let result = super::shell_exec::run(json!({"cmd": cmd, "timeout_ms": 60_000}), root)?;
    Ok(ToolResult {
        schema_version: 1,

        ok: result.ok,

        content: result.content,
        meta: ToolMeta {
            duration_ms: result.meta.duration_ms,

            bytes: result.meta.bytes,

            exit_code: result.meta.exit_code,
        },

        redactions: result.redactions,

        approval_id: result.approval_id,

        tool_call_id: result.tool_call_id,
    })
}

#[allow(dead_code)]
/// Return the tool descriptor for `test.run`.
pub fn descriptor() -> serde_json::Value {
    json!({
        "id": "test.run",
        "schema_version": 1,
        "version": "1.0.0",
        "source": "builtin",
        "summary": "Run the configured test command for a phase.",
        "capabilities": {
            "read": false, "write": false, "exec": true,
            "network": false, "destructive": false, "secrets": false,
            "side_effect": "exec"
        },
        "side_effect": "exec",
        "approval": { "mode": "auto", "reason": "configured test runner" }
    })
}
