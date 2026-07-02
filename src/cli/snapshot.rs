//! `HighHarness snapshot` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the snapshot subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The snapshot action to perform.
    pub cmd: SnapshotCmd,
}

/// Available snapshot actions.
#[derive(Subcommand, Debug)]
pub enum SnapshotCmd {
    /// Take a new file snapshot.
    Take {
        /// Run identifier for the snapshot.
        #[clap(long)]
        run_id: String,

        /// Label for the snapshot.
        #[clap(long)]
        label: String,
    },
    /// Diff two snapshots.
    Diff {
        /// Snapshot ID of the before state.
        #[clap(long)]
        before: String,

        /// Snapshot ID of the after state.
        #[clap(long)]
        after: String,
    },
    /// Revert to a previous snapshot.
    Revert {
        /// Snapshot ID to revert to.
        #[clap(long)]
        snapshot_id: String,
    },
}

/// Execute the snapshot subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        SnapshotCmd::Take { run_id, label } => {
            let id = crate::store::snapshots::take(root, &run_id, &label)?;
            println!("{}", id);
            Ok(0)
        }
        SnapshotCmd::Diff { before, after } => {
            let d = crate::store::snapshots::diff(root, &before, &after)?;
            println!("{}", serde_json::to_string_pretty(&d)?);
            Ok(0)
        }
        SnapshotCmd::Revert { snapshot_id } => {
            crate::store::snapshots::revert(root, &snapshot_id)?;
            Ok(0)
        }
    }
}
