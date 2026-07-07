//! Tool registry and built-in tool implementations.
//!
//! Built-in tools are described in `.harness/tools/<id>.toml` per
//! `HARNESS_PRIMITIVES.md` §1.1. This module loads the registry, dispatches
//! invocations through the permission gate, and writes tool-call ledger lines.

pub mod fs_edit;
pub mod fs_hash;
pub mod fs_read;
pub mod git_blame;
pub mod git_diff;
pub mod git_status;
pub mod lint_run;
pub mod registry;
pub mod shell_exec;
pub mod test_run;
pub mod web_fetch;

use std::path::Path;
use serde_json::Value;

/// Read a config value from `.harness/config.toml` by key.
pub fn read_tool_cmd(root: &Path, key: &str) -> Option<String> {
    let path = crate::store::config_path(root);
    let raw = std::fs::read_to_string(&path).ok()?;
    let v: Value = toml::from_str(&raw).ok()?;
    v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string())
}
