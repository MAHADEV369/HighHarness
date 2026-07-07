//! `web.fetch` tool — fetch a URL. Capability: network.

use std::net::IpAddr;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use serde_json::{json, Value};

use crate::error::{HxError, HxResult};
use crate::schema::tool::{ToolContent, ToolMeta, ToolResult};

const MAX_FETCH_BYTES: u64 = 10 * 1024 * 1024;

fn validate_url(url: &str) -> HxResult<String> {
    let parsed = url::Url::parse(url)
        .map_err(|e| HxError::Other(format!("web.fetch: invalid URL: {}", e)))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(HxError::Other(format!(
            "web.fetch: scheme '{}' not allowed (only http/https)",
            scheme
        )));
    }
    if let Some(host_str) = parsed.host_str() {
        if host_str.eq_ignore_ascii_case("localhost")
            || host_str == "127.0.0.1"
            || host_str == "::1"
        {
            return Err(HxError::Other(
                "web.fetch: localhost not allowed".to_string(),
            ));
        }
        if let Ok(ip) = IpAddr::from_str(host_str) {
            match ip {
                IpAddr::V4(v4)
                    if v4.is_loopback()
                        || v4.is_private()
                        || v4.is_unspecified()
                        || v4.is_multicast() =>
                {
                    return Err(HxError::Other(format!(
                        "web.fetch: private/loopback/multicast IPv4 not allowed: {}",
                        host_str
                    )));
                }
                IpAddr::V6(v6) if v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() => {
                    return Err(HxError::Other(format!(
                        "web.fetch: loopback/multicast IPv6 not allowed: {}",
                        host_str
                    )));
                }
                _ => {}
            }
        }
    }
    Ok(url.to_string())
}

/// Fetch a URL using curl with SSRF protection and size limits.
pub fn run(args: Value, _root: &Path) -> HxResult<ToolResult> {
    let url = args
        .get("url")
        .and_then(|x| x.as_str())
        .ok_or_else(|| HxError::Other("web.fetch: missing 'url'".to_string()))?;
    let url = validate_url(url)?;
    let out = Command::new("curl")
        .args([
            "-sS",
            "-L",
            "--max-time",
            "30",
            "--max-filesize",
            &MAX_FETCH_BYTES.to_string(),
            &url,
        ])
        .output()
        .map_err(|e| HxError::Other(format!("web.fetch spawn: {}", e)))?;
    let body = String::from_utf8_lossy(&out.stdout).to_string();
    let exit = out.status.code().unwrap_or(-1);
    Ok(ToolResult {
        schema_version: 1,

        ok: out.status.success(),
        content: ToolContent {
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
/// Return the tool descriptor for `web.fetch`.
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
