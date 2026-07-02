//! Tool descriptor and result schemas, per `HARNESS_PRIMITIVES.md` §1.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Describes a tool that can be invoked by an agent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDescriptor {
    /// Unique tool identifier.
    pub id: String,
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// Tool version string.
    pub version: String,
    /// Source of the tool: "builtin", "extension", "mcp", etc.
    pub source: String,
    /// Extension identifier if the tool comes from an extension.
    pub extension_id: Option<String>,
    /// MCP server name if the tool is an MCP tool.
    pub mcp_server: Option<String>,
    /// Short human-readable description of the tool.
    pub summary: String,
    /// Capabilities this tool requires.
    pub capabilities: Capabilities,
    /// Path to the JSON schema for tool arguments.
    pub argument_schema_path: String,
    /// Path to the JSON schema for tool return value.
    pub return_schema_path: String,
    /// Side-effect classification: "none", "read-only", "write", "destructive".
    pub side_effect: String,
    /// Approval configuration for this tool.
    #[serde(default = "default_approval")]
    pub approval: ApprovalConfig,
}

/// Returns the default approval configuration (auto mode).
fn default_approval() -> ApprovalConfig {
    ApprovalConfig {
        mode: "auto".to_string(),

        reason: String::new(),
    }
}

/// Capability flags for a tool.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capabilities {
    /// Tool can read files.
    pub read: bool,
    /// Tool can write files.
    pub write: bool,
    /// Tool can execute commands.
    pub exec: bool,
    /// Tool can access the network.
    pub network: bool,
    /// Tool can perform destructive operations.
    pub destructive: bool,
    /// Tool can access secrets.
    pub secrets: bool,
    /// Side-effect classification.
    pub side_effect: String,
}

/// Approval configuration for a tool.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovalConfig {
    /// Approval mode: "auto", "always", or "never".
    pub mode: String,
    /// Reason why approval is required.
    pub reason: String,
}

/// The result returned by a tool execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResult {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// Whether the tool call succeeded.
    pub ok: bool,
    /// Content of the tool result.
    pub content: ToolContent,
    /// Execution metadata.
    pub meta: ToolMeta,
    /// Redactions applied to the result content.
    #[serde(default)]
    pub redactions: Vec<Redaction>,
    /// Approval request ID, if approval was involved.
    pub approval_id: Option<String>,
    /// Identifier of the tool call this result corresponds to.
    pub tool_call_id: String,
}

/// Content payload returned by a tool.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolContent {
    /// Content type: "text", "json", "file", etc.
    pub kind: String,
    /// The content value.
    pub value: Value,
}

/// Execution metadata for a tool call.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolMeta {
    /// Duration of the tool call in milliseconds.
    pub duration_ms: u64,
    /// Bytes transferred or processed, if applicable.
    pub bytes: Option<u64>,
    /// Exit code for command executions, if applicable.
    pub exit_code: Option<i32>,
}

/// A redaction applied to tool result content.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Redaction {
    /// Byte range [start, end) that was redacted.
    pub range: [usize; 2],
    /// Reason for the redaction.
    pub reason: String,
}

/// Returns a default `Capabilities` with all flags set to false.
#[allow(dead_code)]
pub fn default_capabilities() -> Capabilities {
    Capabilities {
        read: false,

        write: false,

        exec: false,

        network: false,

        destructive: false,

        secrets: false,
        side_effect: "none".to_string(),
    }
}
