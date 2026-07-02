//! Integrity log line schema, per `HARNESS_SECURITY.md` §8.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single line in the integrity log chain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntegrityLine {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// Event type recorded in this line.
    pub event: String,
    /// ISO-8601 timestamp of the event.
    pub at: String,
    /// Run that produced this log line, if any.
    pub run_id: Option<String>,
    /// Hash of the previous integrity log line.
    pub prev_hash: String,
    /// Hash of this integrity log line.
    pub this_hash: String,
    /// Additional details for the event.
    #[serde(default)]
    pub details: Value,
}
