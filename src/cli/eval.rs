//! `HighHarness eval` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the eval subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The eval action to perform.
    pub cmd: EvalCmd,
}

/// Available eval actions.
#[derive(Subcommand, Debug)]
pub enum EvalCmd {
    /// List all available evals.
    List,
    /// Run a single eval by id.
    Run {
        /// Eval id (directory name under .harness/evals/).
        id: String,
        /// Optional run id (auto-generated if omitted).
        #[clap(long)]
        run_id: Option<String>,
    },
}

/// Execute the eval subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        EvalCmd::List => {
            let summaries = crate::eval::list(root)?;
            for s in &summaries {
                println!("{} | {} | {}", s.id, s.kind, s.created_at);
            }
            Ok(0)
        }
        EvalCmd::Run { id, run_id: _ } => {
            let result = crate::eval::run(&id, root)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(0)
        }
    }
}
