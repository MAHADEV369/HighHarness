//! Snapshot descriptor, per `HARNESS_PRIMITIVES.md` §8.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A point-in-time snapshot of the workspace state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// Unique identifier for this snapshot.
    pub snapshot_id: String,
    /// Run that produced this snapshot.
    pub run_id: String,
    /// Human-readable label for the snapshot.
    pub label: String,
    /// Git state at the time of the snapshot.
    pub git: SnapshotGit,
    /// Test results, if tests were run.
    pub tests: Option<SnapshotHash>,
    /// Type-check results, if type checking was run.
    pub types: Option<SnapshotHash>,
    /// Lint results, if linting was run.
    pub lint: Option<SnapshotHash>,
    /// Execution phase when the snapshot was taken.
    pub phase: String,
    /// ISO-8601 timestamp when the snapshot was taken.
    pub taken_at: String,
    /// W5: self-authentication hash for integrity verification.
    pub self_hash: Option<String>,
}

/// Git state captured at the time of a snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotGit {
    /// Git commit hash.
    pub commit: String,
    /// Whether the working tree had uncommitted changes.
    pub dirty: bool,
    /// One-line diff stat summary.
    pub diff_stat: String,
}

/// A hash and optional summary of a check result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotHash {
    /// Content hash of the check output.
    pub hash: String,
    /// Optional structured summary of the check result.
    pub summary: Option<Value>,
}
