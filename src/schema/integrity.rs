//! Integrity log line schema, per `HARNESS_SECURITY.md` §8.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `IntegrityLine` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct IntegrityLine {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub schema_version: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub event: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub at: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub run_id: Option<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub prev_hash: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub this_hash: String,
    #[serde(default)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub details: Value,
}
