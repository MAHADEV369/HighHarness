//! Clarification request schema, per `HARNESS_PRIMITIVES.md` §10.6.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A clarification request raised by an agent during a run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clarification {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// Unique identifier for this clarification request.
    pub id: String,
    /// Run that produced this request.
    pub run_id: String,
    /// Agent identifier that raised the clarification.
    pub by: String,
    /// The question being asked.
    pub question: String,
    /// Additional context for the question.
    pub context: Value,
    /// Urgency level: "low", "medium", "high", or "blocking".
    pub urgency: String,
    /// Current state: "open", "answered", "denied", or "expired".
    pub state: String,
    /// The answer provided, if any.
    pub answer: Option<String>,
    /// Rationale behind the answer.
    pub rationale: Option<String>,
    /// Modified tool arguments, if any.
    pub modified_args: Option<Value>,
    /// How the clarification was resolved.
    pub resolution_kind: Option<String>,
    /// ISO-8601 timestamp when the request was created.
    pub created_at: String,
    /// ISO-8601 timestamp when the request expires.
    pub expires_at: String,
}
