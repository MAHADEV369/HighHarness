//! Artifact store: disk-backed persistence for all harness artifacts.
//!
//! Every artifact lives under `.harness/artifacts/`. Atomic writes only
//! (write-to-temp + rename). All shared writers take the lock documented
//! in `HARNESS_PRIMITIVES.md` §4.3.

/// mod `approvals` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod approvals;
/// mod `changelog` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod changelog;
/// mod `episode` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod episode;
/// mod `in_flight` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod in_flight;
/// mod `interventions` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod interventions;
/// mod `locks` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod locks;
/// mod `memory` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod memory;
/// mod `snapshots` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod snapshots;
/// mod `spend` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod spend;

use std::path::{Path, PathBuf};

/// Returns the absolute path to the harness store root (`.harness/`).
pub fn harness_dir(root: &Path) -> PathBuf {
    root.join(".harness")
}

/// Returns the path to the artifact directory.
pub fn artifacts_dir(root: &Path) -> PathBuf {
    harness_dir(root).join("artifacts")
}

/// Returns the path to the lock directory.
pub fn locks_dir(root: &Path) -> PathBuf {
    harness_dir(root).join("locks")
}

/// Returns the path to the changelog file (CHANGELOG.agent.md at repo root).
pub fn changelog_path(root: &Path) -> PathBuf {
    root.join("CHANGELOG.agent.md")
}

/// Returns the path to the changelog lock file.
pub fn changelog_lock_path(root: &Path) -> PathBuf {
    locks_dir(root).join("changelog.lock")
}

/// Returns the path to the integrity log.
pub fn integrity_log_path(root: &Path) -> PathBuf {
    artifacts_dir(root).join("harness.log")
}

/// Returns the path to the in-flight JSONL.
pub fn in_flight_path(root: &Path) -> PathBuf {
    artifacts_dir(root).join("in-flight.jsonl")
}

/// Returns the path to the tool-call ledger.
pub fn tool_calls_path(root: &Path) -> PathBuf {
    artifacts_dir(root).join("tool-calls.jsonl")
}

/// Returns the path to the permissions TOML.
pub fn permissions_path(root: &Path) -> PathBuf {
    harness_dir(root).join("permissions.toml")
}

/// Returns the path to the config TOML.
pub fn config_path(root: &Path) -> PathBuf {
    harness_dir(root).join("config.toml")
}

/// Returns the path to the models TOML.
pub fn models_path(root: &Path) -> PathBuf {
    harness_dir(root).join("models.toml")
}

/// Returns the path to the routing TOML.
pub fn routing_path(root: &Path) -> PathBuf {
    harness_dir(root).join("routing.toml")
}

/// Returns the path to the bootstrap.json.
pub fn bootstrap_path(root: &Path) -> PathBuf {
    artifacts_dir(root).join("bootstrap").join("bootstrap.json")
}

/// Returns the path to the GENESIS marker file (in the changelog).
/// (The GENESIS marker is appended to `CHANGELOG.agent.md` per spec.)
pub fn genesis_marker_path(root: &Path) -> PathBuf {
    changelog_path(root)
}

/// Returns the path to the episodes directory.
pub fn episodes_dir(root: &Path) -> PathBuf {
    root.join("logs").join("episodes")
}

/// Returns the path to the approvals directory.
pub fn approvals_dir(root: &Path) -> PathBuf {
    artifacts_dir(root).join("approvals")
}

/// Returns the path to the interventions directory.
pub fn interventions_dir(root: &Path) -> PathBuf {
    artifacts_dir(root).join("interventions")
}

/// Returns the path to the spend directory.
pub fn spend_dir(root: &Path) -> PathBuf {
    artifacts_dir(root).join("spend")
}

/// Returns the path to the snapshots directory.
pub fn snapshots_dir(root: &Path) -> PathBuf {
    artifacts_dir(root).join("snapshots")
}

/// Returns the path to the memory directory.
pub fn memory_dir(root: &Path) -> PathBuf {
    artifacts_dir(root).join("memory")
}

/// Returns the path to the tools directory.
pub fn tools_dir(root: &Path) -> PathBuf {
    harness_dir(root).join("tools")
}

/// W5: Compute self_hash for a serde-serializable struct.
/// The hash is computed over the JSON serialization WITHOUT the self_hash field.
/// Caller is responsible for clearing self_hash before calling this function.
pub fn compute_self_hash<T: serde::Serialize>(value: &T) -> crate::error::HxResult<String> {
    let json = serde_json::to_string(value)?;
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(json.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

/// Ensure the harness skeleton exists. Idempotent. Used by `bootstrap init`.
pub fn ensure_skeleton(root: &Path) -> crate::error::HxResult<()> {
    use std::fs;
    for d in [
        harness_dir(root),
        artifacts_dir(root),
        locks_dir(root),
        episodes_dir(root),
        approvals_dir(root),
        interventions_dir(root),
        spend_dir(root),
        snapshots_dir(root),
        memory_dir(root),
        tools_dir(root),
        root.join(".harness").join("artifacts").join("incidents"),
        root.join(".harness")
            .join("artifacts")
            .join("notifications"),
        root.join(".harness").join("artifacts").join("quarantine"),
    ] {
        fs::create_dir_all(&d)?;
    }
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}
