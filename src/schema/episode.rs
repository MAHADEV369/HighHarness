//! Episode trace structs, per `HARNESS_ENGINEERING.md` §5.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `Episode` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Episode {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub run_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub task_spec: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub plan: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub task_state_log: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tool_calls: Vec<ToolCall>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub decisions: Vec<Decision>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub failures: Vec<Failure>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub interventions: Vec<Intervention>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub pre_task_checklist: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub verification_report: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub files_touched: Vec<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub episode_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `ToolCall` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct ToolCall {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tool_call_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tool: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub args: Value,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub result_summary: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub started_at: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub duration_ms: u64,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub approval_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `Decision` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Decision {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub decision_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub choice: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub alternatives: Vec<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `Failure` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Failure {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub failure_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub what: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub locus: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub evidence: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub resolution: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `Intervention` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Intervention {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub intervention_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub kind: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub what: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub context: Value,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub correction: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub by: String,
}
