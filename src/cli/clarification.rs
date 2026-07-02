//! `HighHarness clarification` subcommand (stub).

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the clarification subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The clarification action to perform.
    pub cmd: ClarificationCmd,
}

/// Available clarification actions.
#[derive(Subcommand, Debug)]
pub enum ClarificationCmd {
    /// List all open clarification requests.
    List,
    /// Resolve a clarification request with an answer.
    Resolve {
        /// ID of the clarification request.
        #[clap(long)]
        id: String,

        /// The answer to the clarification request.
        #[clap(long)]
        answer: String,
    },
}

/// Execute the clarification subcommand.
pub fn run(cmd: Cmd, _root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        ClarificationCmd::List => {
            println!("[]");
            Ok(0)
        }
        ClarificationCmd::Resolve { id, answer } => {
            println!("{} {}", id, answer);
            Ok(0)
        }
    }
}
