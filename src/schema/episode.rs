//! Episode trace structs, per `HARNESS_ENGINEERING.md` §5.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A complete trace of one agent episode (task execution).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Episode {
    /// Run that produced this episode.
    pub run_id: String,
    /// Original task specification.
    pub task_spec: String,
    /// Plan generated for the task.
    pub plan: String,
    /// Log of task state transitions.
    pub task_state_log: String,
    /// All tool calls made during this episode.
    pub tool_calls: Vec<ToolCall>,
    /// Decisions made by the agent.
    pub decisions: Vec<Decision>,
    /// Failures encountered during execution.
    pub failures: Vec<Failure>,
    /// Interventions applied during execution.
    pub interventions: Vec<Intervention>,
    /// Pre-task checklist status.
    pub pre_task_checklist: String,
    /// Verification report produced at the end.
    pub verification_report: String,
    /// Files touched by this episode.
    pub files_touched: Vec<String>,
    /// Hash of this episode for integrity verification.
    pub episode_hash: String,
}

/// A single tool invocation within an episode.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call.
    pub tool_call_id: String,
    /// Name of the tool that was called.
    pub tool: String,
    /// Arguments passed to the tool.
    pub args: Value,
    /// Summary of the tool call result.
    pub result_summary: String,
    /// ISO-8601 timestamp when the call started.
    pub started_at: String,
    /// Duration of the call in milliseconds.
    pub duration_ms: u64,
    /// Approval request ID if approval was required.
    pub approval_id: Option<String>,
}

/// A decision made by the agent during episode execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Decision {
    /// Unique identifier for this decision.
    pub decision_id: String,
    /// The choice that was made.
    pub choice: String,
    /// Alternative choices that were considered.
    pub alternatives: Vec<String>,
    /// Reasoning behind the chosen option.
    pub reason: String,
}

/// A failure encountered during episode execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Failure {
    /// Unique identifier for this failure.
    pub failure_id: String,
    /// Description of what failed.
    pub what: String,
    /// Where the failure occurred.
    pub locus: String,
    /// Evidence of the failure.
    pub evidence: String,
    /// How the failure was resolved or mitigated.
    pub resolution: String,
}

/// An external intervention applied during episode execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Intervention {
    /// Unique identifier for this intervention.
    pub intervention_id: String,
    /// Type of intervention applied.
    pub kind: String,
    /// Description of what the intervention did.
    pub what: String,
    /// Context that prompted the intervention.
    pub context: Value,
    /// Correction applied by the intervention.
    pub correction: String,
    /// Agent or operator who applied the intervention.
    pub by: String,
}
