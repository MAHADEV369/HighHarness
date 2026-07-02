//! `web.fetch` tool — fetch a URL. Capability: network.

use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

use crate::error::{HxError, HxResult};
use crate::schema::tool::{ToolContent, ToolMeta, ToolResult};

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(args: Value, _root: &Path) -> HxResult<ToolResult> {
    let url = args
        .get("url")
        .and_then(|x| x.as_str())
        .ok_or_else(|| HxError::Other("web.fetch: missing 'url'".to_string()))?;
    // Use curl — it's universally available.
    let out = Command::new("curl")
        .args(["-sSL", "--max-time", "30", url])
        .output()
        .map_err(|e| HxError::Other(format!("web.fetch spawn: {}", e)))?;
    let body = String::from_utf8_lossy(&out.stdout).to_string();
    let exit = out.status.code().unwrap_or(-1);
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(ToolResult {
        schema_version: 1,

        ok: out.status.success(),
        content: ToolContent {
            /// Field `kind` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            kind: "text".to_string(),

            value: Value::String(body),
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
        "id": "web.fetch",
        "schema_version": 1,
        "version": "1.0.0",
        "source": "builtin",
        "summary": "Fetch a URL with curl.",
        "capabilities": {
            "read": false, "write": false, "exec": false,
            "network": true, "destructive": false, "secrets": false,
            "side_effect": "network"
        },
        "side_effect": "network",
        "approval": { "mode": "ask", "reason": "network egress" }
    })
}
