//! `git.status` tool — spawn `git status --porcelain=v2 -b`.

use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

use crate::error::{HxError, HxResult};
use crate::schema::tool::{ToolContent, ToolMeta, ToolResult};

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(_args: Value, root: &Path) -> HxResult<ToolResult> {
    let out = Command::new("git")
        .args(["status", "--porcelain=v2", "-b"])
        .current_dir(root)
        .output()
        .map_err(|e| HxError::Other(format!("git.status spawn: {}", e)))?;
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
        "id": "git.status",
        "schema_version": 1,
        "version": "1.0.0",
        "source": "builtin",
        "summary": "Run `git status --porcelain=v2 -b`.",
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
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn git_status_runs_in_git_repo() {
        let dir = TempDir::new().unwrap();
        let _ = Command::new("git")
            .arg("init")
            .arg("-q")
            .arg(dir.path())
            .output();
        let r = run(serde_json::json!({}), dir.path()).unwrap();
        assert!(r.ok, "stderr: {}", r.content.value);
    }
}
