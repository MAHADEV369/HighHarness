//! `HighHarness eval` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

#[derive(Parser, Debug)]
/// struct `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: EvalCmd,
}

#[derive(Subcommand, Debug)]
/// enum `EvalCmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum EvalCmd {
    /// List all available evals.
    List,
    /// Run a single eval by id.
    Run {
        /// Eval id (directory name under .harness/evals/).
        id: String,
        /// Optional run id (auto-generated if omitted).
        #[clap(long)]
        /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        run_id: Option<String>,
    },
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        EvalCmd::List => {
            let summaries = crate::eval::list(root)?;
            for s in &summaries {
                println!("{} | {} | {}", s.id, s.kind, s.created_at);
            }
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        EvalCmd::Run { id, run_id: _ } => {
            let result = crate::eval::run(&id, root)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
    }
}
