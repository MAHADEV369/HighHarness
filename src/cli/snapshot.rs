//! `HighHarness snapshot` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

#[derive(Parser, Debug)]
/// struct `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: SnapshotCmd,
}

#[derive(Subcommand, Debug)]
/// enum `SnapshotCmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum SnapshotCmd {
    /// Variant `Take` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Take {
        #[clap(long)]
        /// Field `run_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        run_id: String,

        #[clap(long)]
        /// Field `label` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        label: String,
    },
    /// Variant `Diff` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Diff {
        #[clap(long)]
        /// Field `before` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        before: String,

        #[clap(long)]
        /// Field `after` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        after: String,
    },
    /// Variant `Revert` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Revert {
        #[clap(long)]
        /// Field `snapshot_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        snapshot_id: String,
    },
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        SnapshotCmd::Take { run_id, label } => {
            let id = crate::store::snapshots::take(root, &run_id, &label)?;
            println!("{}", id);
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        SnapshotCmd::Diff { before, after } => {
            let d = crate::store::snapshots::diff(root, &before, &after)?;
            println!("{}", serde_json::to_string_pretty(&d)?);
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        SnapshotCmd::Revert { snapshot_id } => {
            crate::store::snapshots::revert(root, &snapshot_id)?;
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
    }
}
