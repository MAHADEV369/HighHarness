//! Tool descriptor and result schemas, per `HARNESS_PRIMITIVES.md` §1.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `ToolDescriptor` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct ToolDescriptor {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub schema_version: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub version: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub source: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub extension_id: Option<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub mcp_server: Option<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub summary: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub capabilities: Capabilities,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub argument_schema_path: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub return_schema_path: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub side_effect: String,
    #[serde(default = "default_approval")]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub approval: ApprovalConfig,
}

fn default_approval() -> ApprovalConfig {
    /// Variant `ApprovalConfig` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    ApprovalConfig {
        /// Field `mode` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        mode: "auto".to_string(),

        reason: String::new(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `Capabilities` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Capabilities {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub read: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub write: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub exec: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub network: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub destructive: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub secrets: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub side_effect: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `ApprovalConfig` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct ApprovalConfig {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub mode: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `ToolResult` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct ToolResult {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub schema_version: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub ok: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub content: ToolContent,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub meta: ToolMeta,
    #[serde(default)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub redactions: Vec<Redaction>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub approval_id: Option<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tool_call_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `ToolContent` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct ToolContent {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub kind: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub value: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `ToolMeta` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct ToolMeta {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub duration_ms: u64,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub bytes: Option<u64>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub exit_code: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `Redaction` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Redaction {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub range: [usize; 2],
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub reason: String,
}

#[allow(dead_code)]
/// fn `default_capabilities` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn default_capabilities() -> Capabilities {
    /// Variant `Capabilities` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Capabilities {
        read: false,

        write: false,

        exec: false,

        network: false,

        destructive: false,

        secrets: false,
        /// Field `side_effect` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        side_effect: "none".to_string(),
    }
}
