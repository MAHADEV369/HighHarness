//! `HighHarness gates` subcommand.
//!
//! Implements the `gates run` subcommand shape per `buildedit.md` Area A.2.
//! The old flat form (`gates --phase X --gate Y --run-id Z --changes <path>`)
//! is rejected by clap because the only subcommand is now `Run`.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::{HxError, HxResult};

/// CLI arguments for the gates subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The gate action to perform.
    pub cmd: GatesCmd,
}

/// Available gate actions.
#[derive(Subcommand, Debug)]
pub enum GatesCmd {
    /// Run a single gate; exit 0 on pass, non-zero on fail/blocked.
    Run {
        /// Build phase identifier.
        #[clap(long)]
        phase: String,

        /// Gate name to evaluate.
        #[clap(long)]
        gate: String,

        /// Run identifier for the gate check.
        #[clap(long)]
        run_id: String,
        /// Inline JSON or path to a JSON file describing changes.
        #[clap(long)]
        changes: String,
        /// Inline JSON or path to a JSON file with the §7.3 verification
        /// mapping. Required iff `--gate semantic`.
        #[clap(long)]
        verification: Option<std::path::PathBuf>,
    },
}

/// Execute the gates subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        GatesCmd::Run {
            phase,
            gate,
            run_id,
            changes,
            verification,
        } => {
            // 1. Parse --changes (inline or path).
            let changes_raw = crate::cli::util::read_json_or_path(&changes)?;
            let changes_value: serde_json::Value = serde_json::from_str(&changes_raw)?;

            // 2. Branch on gate kind.
            if gate == "semantic" {
                let ver_path = verification.ok_or_else(|| {
                    HxError::Other(
                        "semantic gate requires --verification <path-to-judgment-json>; see HARNESS_PRIMITIVES.md §7.3".into(),
                    )
                })?;
                let ver_raw = std::fs::read_to_string(&ver_path)?;
                let ver_value: serde_json::Value = serde_json::from_str(&ver_raw)?;
                let r = crate::gates::run_semantic(&phase, &run_id, ver_value, root)?;
                println!("{}", serde_json::to_string_pretty(&r)?);
                Ok(if r.status == "pass" { 0 } else { 1 })
            } else {
                let r = crate::gates::run(&phase, &gate, &run_id, changes_value, root)?;
                println!("{}", serde_json::to_string_pretty(&r)?);
                Ok(if r.status == "pass" { 0 } else { 1 })
            }
        }
    }
}
