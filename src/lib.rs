//! HighHarness: a runtime-neutral agent harness that enforces the HARNESS_*
//! spec set. See `HARNESS_ENGINEERING.md` / `HARNESS_PRIMITIVES.md` for the
//! binding rules.

#![deny(missing_docs)]

/// Bootstrap protocol: 10-step self-test before any agent runs.
pub mod bootstrap;
/// SHA-256 canonical serialization for changelog entries and episodes.
pub mod canonical;
/// CLI dispatch: 22 subcommands mapped to module functions.
pub mod cli;
/// Error types: 14-variant `HxError` enum with `HxResult` alias.
pub mod error;
/// Eval runner: synthetic task fixtures for harness self-testing.
pub mod eval;
/// Verification gate runner: syntactic, functional, semantic, regression.
pub mod gates;
/// CSPRNG and pinned ID generators for run_id, agent_id, tool_call_id.
pub mod id;
/// Incident lifecycle: declare, list, acknowledge, close.
pub mod incident;
/// MCP server management: register, start, stop, sandbox isolation.
pub mod mcp;
/// KPI rollup computation: 11 KPIs, alerts, and artifact persistence.
pub mod metrics;
/// Model registry and OpenAI-compatible routing (config-only, inference stub).
pub mod models;
/// Permission engine: default-deny, scope narrowing, safety-critical forcing.
pub mod permissions;
/// Secret redaction vault: regex pattern scanning and token replacement.
pub mod redaction;
/// Report generation: HTML episode viewer and future report formats.
pub mod report;
/// Retrieval stub: grep-based filesystem search (no RAG, no embeddings).
pub mod retrieval;
/// Schema definitions: serde structs for all harness artifacts.
pub mod schema;
/// Storage backends: changelog, episodes, snapshots, spend, memory, approvals.
pub mod store;
/// Integrity log: line-chained JSONL with SHA-256 rolling hash.
pub mod telemetry;
/// Built-in tools: fs.read, fs.hash, fs.edit, git.*, shell.exec, test.run, lint.run, web.fetch.
pub mod tools;

pub use error::{HxError, HxResult};
