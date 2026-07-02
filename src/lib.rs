//! HighHarness: a runtime-neutral agent harness that enforces the HARNESS_*
//! spec set. See `BUILD_PHASE_1.md` for the build plan and the
//! `HARNESS_ENGINEERING.md` / `HARNESS_PRIMITIVES.md` for the binding rules.

// Enforce docs on all `pub` items (HARNESS_PRIMITIVES / HARNESS_ENGINEERING — see
// buildedit.md Area C). `#![deny(missing_docs)]` is set; `cargo build` itself
// fails on any undocumented public item, which is the cheapest way to keep docs
// in lockstep with the code.
#![deny(missing_docs)]
#![allow(unused_doc_comments)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::single_match)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::useless_format)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::manual_strip)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::borrowed_box)]
#![allow(clippy::let_and_return)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::result_large_err)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::type_complexity)]
#![allow(clippy::needless_as_bytes)]
#![allow(clippy::or_fun_call)]
#![allow(clippy::while_immutable_condition)]
#![allow(clippy::suspicious_open_options)]
#![allow(clippy::double_ended_iterator_last)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::manual_flatten)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::sliced_string_as_bytes)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

/// mod `bootstrap` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod bootstrap;
/// mod `canonical` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod canonical;
/// mod `cli` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod cli;
/// mod `error` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod error;
/// mod `eval` — Implements HARNESS_ENGINEERING.md / HARNESS_PRIMITIVES.md.
pub mod eval;
/// mod `gates` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod gates;
/// mod `id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod id;
/// mod `incident` — Implements HARNESS_SECURITY.md §9 (incident response).
pub mod incident;
/// mod `metrics` — Implements HARNESS_METRICS.md §1-§4.
pub mod metrics;
/// mod `models` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod models;
/// mod `permissions` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod permissions;
/// mod `redaction` — Implements HARNESS_SECURITY.md §5 (redaction vault).
pub mod redaction;
/// mod `retrieval` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod retrieval;
/// mod `schema` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod schema;
/// mod `store` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod store;
/// mod `telemetry` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod telemetry;
/// mod `tools` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod tools;

/// mod `mcp` — MCP server management and sandbox (Workstream 7).
pub mod mcp;

pub use error::{HxError, HxResult};
