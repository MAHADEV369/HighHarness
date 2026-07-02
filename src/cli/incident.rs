//! HighHarness incident declare|list|ack|close subcommand.

use crate::error::HxResult;
use clap::{Parser, Subcommand};
use std::path::Path;

/// CLI arguments for the incident subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The incident action to perform.
    pub cmd: IncidentCmd,
}

/// Available incident actions.
#[derive(Subcommand, Debug)]
pub enum IncidentCmd {
    /// Declare a new incident.
    Declare {
        /// Run identifier associated with the incident.
        #[clap(long)]
        run_id: String,
        /// Detection rule that triggered the incident.
        #[clap(long)]
        detection_rule: String,
        /// Attack or failure vector.
        #[clap(long)]
        vector: String,
        /// Severity level.
        #[clap(long)]
        severity: String,
        /// Whether the incident had impact.
        #[clap(long)]
        had_impact: bool,
        /// Evidence paths or URLs.
        #[clap(long)]
        evidence: Vec<String>,
    },
    /// List incidents (optionally only open ones).
    List {
        /// Only list incidents that are not yet closed.
        #[clap(long)]
        open_only: bool,
    },
    /// Acknowledge an incident.
    Ack {
        /// Incident identifier.
        id: String,
        /// Person or agent acknowledging the incident.
        #[clap(long)]
        by: String,
    },
    /// Close an incident with an optional postmortem.
    Close {
        /// Incident identifier.
        id: String,
        /// Optional postmortem document path.
        #[clap(long)]
        postmortem: Option<String>,
    },
}

/// Execute the incident subcommand.
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
