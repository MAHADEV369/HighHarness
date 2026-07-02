//! HighHarness incident declare|list|ack|close subcommand.

use crate::error::HxResult;
use clap::{Parser, Subcommand};
use std::path::Path;

/// struct `Cmd` — Implements HARNESS_SECURITY.md §9 (incident subcommand).
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
    pub cmd: IncidentCmd,
}

/// enum `IncidentCmd` — Implements HARNESS_SECURITY.md §9.
#[derive(Subcommand, Debug)]
pub enum IncidentCmd {
    /// Variant `Declare` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
    Declare {
        #[clap(long)]
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        run_id: String,
        #[clap(long)]
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        detection_rule: String,
        #[clap(long)]
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        vector: String,
        #[clap(long)]
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        severity: String,
        #[clap(long)]
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        had_impact: bool,
        #[clap(long)]
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        evidence: Vec<String>,
    },
    /// Variant `List` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
    List {
        #[clap(long)]
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        open_only: bool,
    },
    /// Variant `Ack` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
    Ack {
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        id: String,
        #[clap(long)]
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        by: String,
    },
    /// Variant `Close` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
    Close {
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        id: String,
        #[clap(long)]
        /// item `?` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
        postmortem: Option<String>,
    },
}

/// fn `run` — Implements HARNESS_SECURITY.md / HARNESS_ENGINEERING.md.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        IncidentCmd::Declare {
            run_id,
            detection_rule,
            vector,
            severity,
            had_impact,
            evidence,
        } => {
            let id = crate::incident::declare(
                root,
                &detection_rule,
                &vector,
                &run_id,
                "cli",
                None,
                None,
                &severity,
                had_impact,
                evidence,
            )?;
            println!("{}", id);
            Ok(0)
        }
        IncidentCmd::List { open_only } => {
            let incidents = crate::incident::list(root, open_only)?;
            println!("{}", serde_json::to_string_pretty(&incidents)?);
            Ok(0)
        }
        IncidentCmd::Ack { id, by } => {
            crate::incident::acknowledge(root, &id, &by)?;
            println!("acknowledged: {}", id);
            Ok(0)
        }
        IncidentCmd::Close { id, postmortem } => {
            crate::incident::close(root, &id, &postmortem.unwrap_or_default())?;
            println!("closed: {}", id);
            Ok(0)
        }
    }
}
