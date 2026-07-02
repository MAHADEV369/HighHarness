//! Command-line interface for the HighHarness binary.
//!
//! Subcommands map 1:1 to the harness surface in `BUILD_PHASE_1.md`.

/// mod `bootstrap` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod bootstrap;
/// mod `cadence` — Implements HARNESS_METRICS.md §6 (review cadence).
pub mod cadence;
/// mod `changelog` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod changelog;
/// mod `clarification` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod clarification;
/// mod `episode` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod episode;
/// mod `eval` — Implements HARNESS_ENGINEERING.md / HARNESS_PRIMITIVES.md.
pub mod eval;
/// mod `gates` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod gates;
/// mod `hooks` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod hooks;
/// mod `id_cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod id_cmd;
/// mod `incident` — Implements HARNESS_SECURITY.md §9 (incident response).
pub mod incident;
/// mod `integrity` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod integrity;
/// mod `mcp` — MCP server management (Workstream 7).
pub mod mcp;
/// mod `metrics` — Implements HARNESS_METRICS.md §1-§4.
pub mod metrics;
/// mod `models` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod models;
/// mod `permissions` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod permissions;
/// mod `redaction` — Implements HARNESS_SECURITY.md §5 (redaction vault).
pub mod redaction;
/// mod `snapshot` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod snapshot;
/// mod `spend` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod spend;
/// mod `tools` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod tools;
/// mod `util` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod util;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Top-level CLI dispatcher.
#[derive(Parser, Debug)]
#[clap(name = "HighHarness", about = "Runtime-neutral agent harness.", version = env!("CARGO_PKG_VERSION"))]
/// struct `Cli` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cli {
    /// Working directory (defaults to current).
    #[clap(long, global = true, default_value = ".")]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub root: PathBuf,

    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
/// enum `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum Cmd {
    /// Variant `Bootstrap` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Bootstrap(bootstrap::Cmd),
    /// Variant `Changelog` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Changelog(changelog::Cmd),
    /// Variant `Episode` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Episode(episode::Cmd),
    /// Variant `Snapshot` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Snapshot(snapshot::Cmd),
    /// Variant `Gates` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Gates(gates::Cmd),
    /// Variant `Tools` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Tools(tools::Cmd),
    /// Variant `Permissions` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Permissions(permissions::Cmd),
    /// Variant `Spend` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Spend(spend::Cmd),
    /// Variant `Hook` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Hook(hooks::Cmd),
    /// Variant `Integrity` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Integrity(integrity::Cmd),
    /// Variant `Clarification` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Clarification(clarification::Cmd),
    /// Variant `Eval` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Eval(eval::Cmd),
    /// ID helpers (Phase 2 thin CLI wrappers).
    #[clap(name = "id-run")]
    IdRun(id_cmd::IdRunCmd),
    /// Variant `IdAgent` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    #[clap(name = "id-agent")]
    IdAgent(id_cmd::IdAgentCmd),
    /// Variant `Metrics` — Implements HARNESS_METRICS.md §1-§4.
    Metrics(metrics::Cmd),
    /// Variant `Cadence` — Implements HARNESS_METRICS.md §6.
    Cadence(cadence::Cmd),
    /// Variant `Redaction` — Implements HARNESS_SECURITY.md §5.
    Redaction(redaction::Cmd),
    /// Variant `Incident` — Implements HARNESS_SECURITY.md §9.
    Incident(incident::Cmd),
    /// Variant `Models` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Models(models::Cmd),
    /// Variant `Mcp` — MCP server management (Workstream 7).
    Mcp(mcp::Cmd),
}

/// Run the CLI. Returns the process exit code (0 = success).
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let root = cli.root.canonicalize().unwrap_or_else(|_| cli.root.clone());
    let rc = match cli.cmd {
        Cmd::Bootstrap(c) => bootstrap::run(c, &root)?,
        Cmd::Changelog(c) => changelog::run(c, &root)?,
        Cmd::Episode(c) => episode::run(c, &root)?,
        Cmd::Snapshot(c) => snapshot::run(c, &root)?,
        Cmd::Gates(c) => gates::run(c, &root)?,
        Cmd::Tools(c) => tools::run(c, &root)?,
        Cmd::Permissions(c) => permissions::run(c, &root)?,
        Cmd::Spend(c) => spend::run(c, &root)?,
        Cmd::Hook(c) => hooks::run(c, &root)?,
        Cmd::Integrity(c) => integrity::run(c, &root)?,
        Cmd::Clarification(c) => clarification::run(c, &root)?,
        Cmd::IdRun(c) => id_cmd::run_id_cmd(c)?,
        Cmd::IdAgent(c) => id_cmd::run_id_agent_cmd(c)?,
        Cmd::Metrics(c) => metrics::run(c, &root)?,
        Cmd::Cadence(c) => cadence::run(c, &root)?,
        Cmd::Redaction(c) => redaction::run(c, &root)?,
        Cmd::Eval(c) => eval::run(c, &root)?,
        Cmd::Incident(c) => incident::run(c, &root)?,
        Cmd::Models(c) => models::run(c, &root)?,
        Cmd::Mcp(c) => mcp::run(c, &root)?,
    };
    std::process::exit(rc);
}
