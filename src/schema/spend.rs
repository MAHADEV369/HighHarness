//! Spend line schema, per `HARNESS_PRIMITIVES.md` §9.1.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single spend line recording token usage and cost.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpendLine {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// ISO-8601 timestamp of the spend event.
    pub ts: String,
    /// Run that incurred this spend.
    pub run_id: String,
    /// Agent that incurred this spend.
    pub agent_id: String,
    /// Model used for this spend event.
    pub model_id: String,
    /// Feature or operation that triggered the spend.
    pub feature: String,
    /// Number of input tokens consumed.
    pub input_tokens: u64,
    /// Number of output tokens produced.
    pub output_tokens: u64,
    /// Number of reasoning tokens used.
    pub reasoning_tokens: u64,
    /// Cost in USD.
    pub usd: f64,
    /// Routing mode used to select the model.
    pub routing_mode: String,
    /// Provider that served the request.
    pub provider: String,
    /// Additional metadata about the spend event.
    pub metadata: Value,
    /// W5: self-authentication hash for integrity verification.
    pub self_hash: Option<String>,
}
