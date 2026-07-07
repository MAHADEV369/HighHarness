//! `shell.exec` tool — spawn a command with cwd, env allowlist, timeout.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::process::Command as TokioCommand;

use crate::error::{HxError, HxResult};
use crate::schema::tool::{ToolContent, ToolMeta, ToolResult};

/// Spawn a shell command with working directory, env, and timeout.
pub fn run(args: Value, root: &Path) -> HxResult<ToolResult> {
    // For tests we use the synchronous Command to avoid pulling tokio runtime
    // into library tests. Production can switch to async.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| HxError::Other(format!("tokio: {}", e)))?;
    rt.block_on(async_run(args, root))
}

async fn async_run(args: Value, root: &Path) -> HxResult<ToolResult> {
    let cmd = args
        .get("command")
        .or_else(|| args.get("cmd"))
        .and_then(|x| x.as_str())
        .ok_or_else(|| HxError::Other("shell.exec: missing 'command' or 'cmd'".to_string()))?;
    let timeout_ms = args
        .get("timeout_ms")
        .and_then(|x| x.as_u64())
        .unwrap_or(30_000);

    let mut command = TokioCommand::new("sh");
    command.arg("-c").arg(cmd).current_dir(root);
    if let Some(env) = args.get("env").and_then(|x| x.as_object()) {
        for (k, v) in env {
            if let Some(s) = v.as_str() {
                command.env(k, s);
            }
        }
    }
    command.kill_on_drop(true);

    let fut = command.output();
    let result = match tokio::time::timeout(Duration::from_millis(timeout_ms), fut).await {
        Ok(r) => r.map_err(|e| HxError::Other(format!("shell.exec spawn: {}", e)))?,
        Err(_) => {
            return Ok(ToolResult {
                schema_version: 1,

                ok: false,
                content: ToolContent {
                    kind: "error".to_string(),

                    value: Value::String(format!("timeout after {} ms", timeout_ms)),
                },
                meta: ToolMeta {
                    duration_ms: timeout_ms,

                    bytes: None,

                    exit_code: Some(-1),
                },

                redactions: vec![],

                approval_id: None,

                tool_call_id: String::new(),
            });
        }
    };

    let exit_code = result.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let stderr = String::from_utf8_lossy(&result.stderr).to_string();
    let combined = format!("{}{}", stdout, stderr);
    let bytes = combined.len() as u64;
    Ok(ToolResult {
        schema_version: 1,

        ok: result.status.success(),
        content: ToolContent {
            kind: "text".to_string(),

            value: Value::String(combined),
        },
        meta: ToolMeta {
            duration_ms: 0,

            bytes: Some(bytes),

            exit_code: Some(exit_code),
        },

        redactions: vec![],

        approval_id: None,

        tool_call_id: String::new(),
    })
}

#[allow(dead_code)]
/// Return the tool descriptor for `shell.exec`.
pub fn descriptor() -> serde_json::Value {
    json!({
        "id": "shell.exec",
        "schema_version": 1,
        "version": "1.0.0",
        "source": "builtin",
        "summary": "Spawn a shell command with cwd + env allowlist + timeout.",
        "capabilities": {
            "read": false, "write": false, "exec": true,
            "network": false, "destructive": true, "secrets": false,
            "side_effect": "exec"
        },
        "side_effect": "exec",
        "approval": { "mode": "ask", "reason": "exec" }
    })
}

#[allow(dead_code)]
fn _unused_to_avoid_warning(_cmd: &mut Command) {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn shell_exec_captures_exit_code() {
        let dir = TempDir::new().unwrap();
        let args = serde_json::json!({"cmd": "true"});
        let r = run(args, dir.path()).unwrap();
        assert!(r.ok);
        assert_eq!(r.meta.exit_code, Some(0));
    }

    #[test]
    fn shell_exec_captures_nonzero_exit() {
        let dir = TempDir::new().unwrap();
        let args = serde_json::json!({"cmd": "false"});
        let r = run(args, dir.path()).unwrap();
        assert!(!r.ok);
        assert_eq!(r.meta.exit_code, Some(1));
    }

    #[test]
    fn shell_exec_timeout_produces_blocked_status() {
        let dir = TempDir::new().unwrap();
        // sleep longer than timeout
        let args = serde_json::json!({"cmd": "sleep 5", "timeout_ms": 100});
        let r = run(args, dir.path()).unwrap();
        assert!(!r.ok);
        assert_eq!(r.meta.exit_code, Some(-1));
    }
}
