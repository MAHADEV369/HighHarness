//! In-flight run record schema, per `HARNESS_PRIMITIVES.md` §3.4.

use serde::{Deserialize, Serialize};

/// A record of a currently running harness session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InFlight {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// Unique identifier for this run.
    pub run_id: String,
    /// Agent that is executing the run.
    pub agent_id: String,
    /// ISO-8601 timestamp when the run was opened.
    pub opened_at: String,
    /// Current execution phase.
    pub phase: String,
    /// Trust tier of the agent.
    pub tier: String,
    /// Current state of the run.
    pub state: String,
    /// OS process ID, if available.
    pub pid: Option<u32>,
}
