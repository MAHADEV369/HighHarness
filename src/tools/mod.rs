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
