//! Approval request schema, per `HARNESS_PRIMITIVES.md` §10.2.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A pending tool-execution approval request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Approval {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// Unique identifier for this approval request.
    pub id: String,
    /// Run that produced this request.
    pub run_id: String,
    /// Tool name being requested.
    pub tool: String,
    /// Original arguments to the tool.
    pub args: Value,
    /// Permission rule that triggered this approval.
    pub rule_id: String,
    /// Human-readable reason the approval was raised.
    pub reason: String,
    /// Numeric priority (higher = more urgent).
    pub priority: i32,
    /// Whether the tool call is destructive.
    pub destructive: bool,
    /// Current state: "pending", "approved", "denied", "expired", or "modified".
    pub state: String,
    /// Modified arguments if state is "modified".
    pub modified_args: Option<Value>,
    /// Agent rationale when state is "approved" or "denied".
    pub rationale: Option<String>,
    /// ISO-8601 timestamp when the request was created.
    pub at: String,
    /// ISO-8601 timestamp when the request expires.
    pub expires_at: String,
    /// W5: self-authentication hash for integrity verification.
    pub self_hash: Option<String>,
}
