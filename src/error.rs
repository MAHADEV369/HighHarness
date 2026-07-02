//! Error types and Result alias for the HighHarness binary.
//!
//! Implements the error surface required by `HARNESS_ENGINEERING.md` and
//! `HARNESS_PRIMITIVES.md`. All public APIs return [`HxResult`]; the
//! [`HxError`] enum is the single error vocabulary across the harness.

use thiserror::Error;

/// Result alias used across the harness.
pub type HxResult<T> = Result<T, HxError>;

/// The harness's single error vocabulary. Each variant maps to a documented
/// failure mode in the spec; the harness surfaces a specific variant instead
/// of generic `anyhow::Error` so callers can branch on the failure locus.
#[derive(Debug, Error)]
pub enum HxError {
    /// I/O failure (file open, read, write).
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parse/encode failure.
    #[error("serde_json: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parse failure.
    #[error("toml: {0}")]
    Toml(#[from] toml::de::Error),

    /// Lock acquisition timed out (per `HARNESS_PRIMITIVES.md` §4.3).
    #[error("lock contention on {resource}")]
    LockContention {
        /// Resource the lock was being acquired on.
        resource: String,
    },

    /// `bootstrap.json` missing or `passed != true` (`HARNESS_VERSIONING.md` §6.1).
    #[error("not bootstrapped")]
    NotBootstrapped,

    /// Lost two races on `compare-and-append`; surface, do not loop
    /// (`HARNESS_PRIMITIVES.md` §3.5).
    #[error("harness contention (lost race twice)")]
    HarnessContention,

    /// Reader rejected the artifact's `schema_version` (`HARNESS_VERSIONING.md` §2.2).
    #[error("schema rejected: artifact={artifact} saw={saw}")]
    SchemaRejected {
        /// Artifact kind (e.g., `permissions.toml`).
        artifact: String,
        /// What the reader saw (e.g., `schema_version=2`).
        saw: String,
    },

    /// Spec describes a property the runtime does not yet enforce
    /// (`HARNESS_ENGINEERING.md` §11 — over-claim guardrail).
    #[error("not-yet-enforced: {what}")]
    NotYetEnforced {
        /// What's not yet enforced.
        what: String,
    },

    /// Hash chain link mismatch (`HARNESS_PRIMITIVES.md` §3.5).
    #[error("chain broken at entry {index}: expected {expected} got {got}")]
    ChainBroken {
        /// 1-based entry index of the broken link.
        index: usize,
        /// Expected `prev_hash` / `this_hash`.
        expected: String,
        /// Computed `prev_hash` / `this_hash`.
        got: String,
    },

    /// Permission rule denied the call (`HARNESS_PRIMITIVES.md` §2).
    #[error("permission denied: tool={tool} rule={rule} reason={reason}")]
    PermissionDenied {
        /// Tool id that was denied.
        tool: String,
        /// Rule id that matched.
        rule: String,
        /// Human-readable reason.
        reason: String,
    },

    /// Permission rule required human approval (`HARNESS_PRIMITIVES.md` §2).
    #[error("permission ask (blocking): tool={tool} rule={rule}")]
    PermissionAsk {
        /// Tool id that needs approval.
        tool: String,
        /// Rule id that matched.
        rule: String,
    },

    /// Run is blocked (clarification pending, hard budget hit, etc.)
    /// (`HARNESS_PRIMITIVES.md` §10.7).
    #[error("run blocked: run_id={run_id} reason={reason}")]
    RunBlocked {
        /// The blocked run's id.
        run_id: String,
        /// Why the run is blocked.
        reason: String,
    },

    /// `run_id` collision — a second open with the same id
    /// (`HARNESS_PRIMITIVES.md` §3.4).
    #[error("run-id collision: {0}")]
    RunIdCollision(String),

    /// F2 Audit-Forgery suspected (`HARNESS_SECURITY.md`).
    #[error("audit-forgery suspected: {0}")]
    AuditForgery(String),

    /// Incident detected (`HARNESS_SECURITY.md`).
    #[error("incident: {0}")]
    Incident(String),

    /// Canonical form violation (`HARNESS_PRIMITIVES.md` §3.5.1).
    #[error("canonical form violation: {0}")]
    CanonicalForm(String),

    /// Generic catch-all for errors that don't fit a more specific variant.
    #[error("other: {0}")]
    Other(String),
}
