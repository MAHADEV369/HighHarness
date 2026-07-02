//! Permission file and rule schemas, per `HARNESS_PRIMITIVES.md` §2.

use serde::{Deserialize, Serialize};

/// A permission configuration file containing access rules.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionFile {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// Ordered list of permission rules.
    pub rules: Vec<Rule>,
}

/// A single permission rule controlling tool access.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
    /// Unique rule identifier.
    pub id: String,
    /// Effect of the rule: "allow", "deny", or "ask".
    pub effect: String,
    /// Tool name this rule applies to (or "*" for all tools).
    pub tool: String,
    /// File path patterns this rule applies to.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Network host patterns this rule applies to.
    #[serde(default)]
    pub network: Vec<String>,
    /// Environment variable patterns this rule applies to.
    #[serde(default)]
    pub env: Vec<String>,
    /// Whether this rule affects safety-critical operations.
    #[serde(default)]
    pub safety: bool,
    /// Human-readable reason for this rule.
    pub reason: String,
    /// Numeric priority (higher = evaluated first).
    pub priority: i32,
}
