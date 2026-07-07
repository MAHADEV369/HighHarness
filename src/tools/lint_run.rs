//! `lint.run` tool — thin wrapper around `shell.exec`.

use std::path::Path;

use serde_json::{json, Value};

use crate::error::HxResult;
use crate::schema::tool::ToolResult;
use crate::tools::read_tool_cmd;

/// Execute the configured lint command via `shell.exec`.
pub fn run(args: Value, root: &Path) -> HxResult<ToolResult> {
    let phase = args
        .get("phase")
        .and_then(|x| x.as_str())
        .unwrap_or("highharness");
    let _ = phase;
    let cmd = read_tool_cmd(root, "lint_cmd").unwrap_or_else(|| "true".to_string());
    super::shell_exec::run(json!({"cmd": cmd, "timeout_ms": 60_000}), root)
}

#[allow(dead_code)]
/// Return the tool descriptor for `lint.run`.
pub fn descriptor() -> serde_json::Value {
    json!({
        "id": "lint.run",
        "schema_version": 1,
        "version": "1.0.0",
        "source": "builtin",
        "summary": "Run the configured lint command.",
        "capabilities": {
            "read": false, "write": false, "exec": true,
            "network": false, "destructive": false, "secrets": false,
            "side_effect": "exec"
        },
        "side_effect": "exec",
        "approval": { "mode": "auto", "reason": "configured lint runner" }
    })
}
