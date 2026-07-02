//! Changelog entry struct, per `HARNESS_ENGINEERING.md` §4 and
//! `HARNESS_PRIMITIVES.md` §3.5.

use serde::{Deserialize, Serialize};

/// A single changelog entry recording a code change.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    /// Monotonic sequence number within the changelog.
    #[serde(default)]
    pub n: u64,
    /// ISO-8601 timestamp of the change.
    pub ts: String,
    /// Agent identifier that authored the change.
    pub agent: String,
    /// Run that produced this entry.
    pub run_id: String,
    /// Trust tier of the agent at the time of change.
    pub tier: String,
    /// Files modified by this change.
    pub files: Vec<String>,
    /// Human-readable intent of the change.
    pub intent: String,
    /// Summary of the diff produced.
    pub diff_summary: String,
    /// Evidence supporting the change.
    pub evidence: String,
    /// Attribution for the change.
    pub attribution: String,
    /// Verification result for the change.
    pub verification: String,
    /// Status of the change: "applied", "rolled-back", etc.
    pub status: String,
    /// Hash of the previous changelog entry.
    pub prev_hash: String,
    /// Hash of this changelog entry.
    pub this_hash: String,
}
