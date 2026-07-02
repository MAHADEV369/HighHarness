//! Changelog entry struct, per `HARNESS_ENGINEERING.md` §4 and
//! `HARNESS_PRIMITIVES.md` §3.5.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `Entry` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Entry {
    #[serde(default)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub n: u64,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub ts: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub agent: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub run_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tier: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub files: Vec<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub intent: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub diff_summary: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub evidence: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub attribution: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub verification: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub status: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub prev_hash: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub this_hash: String,
}
