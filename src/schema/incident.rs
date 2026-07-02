//! Incident record schema, per `HARNESS_SECURITY.md`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A security or integrity incident record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Incident {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// Unique identifier for this incident.
    pub id: String,
    /// Type of incident: "policy_violation", "integrity_failure", etc.
    pub kind: String,
    /// ISO-8601 timestamp when the incident was detected.
    pub at: String,
    /// Run associated with the incident, if any.
    pub run_id: Option<String>,
    /// Detailed information about the incident.
    pub details: Value,
}
