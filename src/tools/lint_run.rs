//! `lint.run` tool — thin wrapper around `shell.exec`.

use std::path::Path;

use serde_json::{json, Value};

use crate::error::HxResult;
use crate::schema::tool::ToolResult;
use crate::store::config_path;

/// Execute the configured lint command via `shell.exec`.
pub fn run(args: Value, root: &Path) -> HxResult<ToolResult> {
    let phase = args
        .get("phase")
        .and_then(|x| x.as_str())
        .unwrap_or("highharness");
    let _ = phase;
    let cmd = read_lint_cmd(root).unwrap_or_else(|| "true".to_string());
    super::shell_exec::run(json!({"cmd": cmd, "timeout_ms": 60_000}), root)
}

fn read_lint_cmd(root: &Path) -> Option<String> {
    let path = config_path(root);
    let raw = std::fs::read_to_string(&path).ok()?;
    let v: Value = toml::from_str(&raw).ok()?;
    v.get("lint_cmd")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
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
