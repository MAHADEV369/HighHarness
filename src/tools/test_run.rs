//! `test.run` tool — thin wrapper around `shell.exec` that uses the
//! configured test command from `.harness/config.toml`.

use std::path::Path;

use serde_json::{json, Value};

use crate::error::HxResult;
use crate::schema::tool::{ToolMeta, ToolResult};
use crate::store::config_path;

/// Execute the configured test command via `shell.exec`.
pub fn run(args: Value, root: &Path) -> HxResult<ToolResult> {
    let phase = args
        .get("phase")
        .and_then(|x| x.as_str())
        .unwrap_or("highharness");
    let cfg = read_config(root);
    let cmd = cfg
        .as_ref()
        .and_then(|c| c.get("test_cmd"))
        .and_then(|x| x.as_str())
        .unwrap_or("true")
        .to_string();
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

fn read_config(root: &Path) -> Option<Value> {
    let path = config_path(root);
    let raw = std::fs::read_to_string(&path).ok()?;
    let v: Value = toml::from_str(&raw).ok()?;
    v.get("test_run")
        .cloned()
        .or_else(|| v.get("test_cmd").map(|c| json!({"test_cmd": c})))
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
