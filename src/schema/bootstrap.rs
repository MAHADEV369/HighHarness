//! Bootstrap record schema, per `HARNESS_VERSIONING.md` §6.1 step 10.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A bootstrap record created during initial harness setup.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bootstrap {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// ISO-8601 timestamp of the bootstrap run.
    pub bootstrapped_at: String,
    /// Human operator who triggered the bootstrap.
    pub bootstrap_human: String,
    /// Git commit that was bootstrapped.
    pub bootstrap_commit: String,
    /// Spec versions recorded at bootstrap time.
    pub spec_versions: Value,
    /// Genesis hash of the integrity chain.
    pub genesis_hash: String,
    /// Evaluation results from the bootstrap.
    pub eval: Value,
    /// Hash of the initial integrity log seed.
    pub integrity_log_seed_hash: String,
    /// Whether the bootstrap passed all checks.
    pub passed: bool,
}
