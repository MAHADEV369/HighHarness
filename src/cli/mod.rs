//! Command-line interface for the HighHarness binary.
//!
//! Subcommands map 1:1 to the harness surface in `BUILD_PHASE_1.md`.

/// Bootstrap initialisation and verification subcommand.
pub mod bootstrap;
/// Metrics cadence rollup subcommand.
pub mod cadence;
/// Append-only changelog subcommand.
pub mod changelog;
/// Clarification request subcommand (stub).
pub mod clarification;
/// Episode recording subcommand.
pub mod episode;
/// Eval listing and execution subcommand.
pub mod eval;
/// Gate evaluation subcommand.
pub mod gates;
/// Pre-tool / post-tool / session-start hooks subcommand.
pub mod hooks;
/// ID generation subcommands (id-run, id-agent).
pub mod id_cmd;
/// Incident lifecycle subcommand.
pub mod incident;
/// Integrity log verification and append subcommand.
pub mod integrity;
/// MCP server integration subcommand.
pub mod mcp;
/// Metrics rollup and alerts subcommand.
pub mod metrics;
/// Model catalogue subcommand.
pub mod models;
/// Permissions listing and checking subcommand.
pub mod permissions;
/// Redaction scanning and pattern management subcommand.
pub mod redaction;
/// Snapshot creation, diffing, and revert subcommand.
pub mod snapshot;
/// Spend tracking subcommand.
pub mod spend;
/// Tool registry listing and invocation subcommand.
pub mod tools;
/// CLI utility helpers (JSON-or-path reading).
pub mod util;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Top-level CLI dispatcher.
#[derive(Parser, Debug)]
#[clap(name = "HighHarness", about = "Runtime-neutral agent harness.", version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Working directory (defaults to current).
    #[clap(long, global = true, default_value = ".")]
    pub root: PathBuf,

    /// The subcommand to execute.
    #[clap(subcommand)]
    pub cmd: Cmd,
}

/// Top-level subcommand dispatcher.
#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Bootstrap initialisation or verification.
    Bootstrap(bootstrap::Cmd),
    /// Append or inspect the changelog.
    Changelog(changelog::Cmd),
    /// Open, append, close, or hash an episode.
    Episode(episode::Cmd),
    /// Create, diff, or revert file snapshots.
    Snapshot(snapshot::Cmd),
    /// Run a gate check.
    Gates(gates::Cmd),
    /// List or invoke registered tools.
    Tools(tools::Cmd),
    /// List or check permission rules.
    Permissions(permissions::Cmd),
    /// Append or summarise spend lines.
    Spend(spend::Cmd),
    /// Execute a lifecycle hook.
    Hook(hooks::Cmd),
    /// Verify or append to the integrity log.
    Integrity(integrity::Cmd),
    /// List or resolve clarification requests.
    Clarification(clarification::Cmd),
    /// List or run evals.
    Eval(eval::Cmd),
    /// ID helpers (Phase 2 thin CLI wrappers).
    #[clap(name = "id-run")]
    IdRun(id_cmd::IdRunCmd),
    /// ID helpers (Phase 2 thin CLI wrappers).
    #[clap(name = "id-agent")]
    IdAgent(id_cmd::IdAgentCmd),
    /// Roll up metrics or check alerts.
    Metrics(metrics::Cmd),
    /// Run a cadence rollup check.
    Cadence(cadence::Cmd),
    /// Scan content or manage redaction patterns.
    Redaction(redaction::Cmd),
    /// Declare, list, acknowledge, or close incidents.
    Incident(incident::Cmd),
    /// List or inspect registered models.
    Models(models::Cmd),
    /// Start or query the MCP server.
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
