//! Tool registry and built-in tool implementations.
//!
//! Built-in tools are described in `.harness/tools/<id>.toml` per
//! `HARNESS_PRIMITIVES.md` §1.1. This module loads the registry, dispatches
//! invocations through the permission gate, and writes tool-call ledger lines.

/// mod `fs_edit` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod fs_edit;
/// mod `fs_hash` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod fs_hash;
/// mod `fs_read` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod fs_read;
/// mod `git_blame` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod git_blame;
/// mod `git_diff` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod git_diff;
/// mod `git_status` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod git_status;
/// mod `lint_run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod lint_run;
/// mod `registry` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod registry;
/// mod `shell_exec` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod shell_exec;
/// mod `test_run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod test_run;
/// mod `web_fetch` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod web_fetch;
