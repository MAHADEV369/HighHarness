//! Snapshot descriptor, per `HARNESS_PRIMITIVES.md` §8.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `Snapshot` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Snapshot {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub schema_version: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub snapshot_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub run_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub label: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub git: SnapshotGit,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tests: Option<SnapshotHash>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub types: Option<SnapshotHash>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub lint: Option<SnapshotHash>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub phase: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub taken_at: String,
    /// W5: self-authentication hash for integrity verification.
    pub self_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `SnapshotGit` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct SnapshotGit {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub commit: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub dirty: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub diff_stat: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `SnapshotHash` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct SnapshotHash {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub hash: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub summary: Option<Value>,
}
