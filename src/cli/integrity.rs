//! `HighHarness integrity` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the integrity subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The integrity action to perform.
    pub cmd: IntegrityCmd,
}

/// Available integrity actions.
#[derive(Subcommand, Debug)]
pub enum IntegrityCmd {
    /// Verify the integrity log chain.
    Verify,
    /// Append an event to the integrity log.
    Append {
        /// Event name to record in the integrity log.
        event: String,
    },
}

/// Execute the integrity subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        IntegrityCmd::Verify => {
            let broken = crate::telemetry::integrity::verify(root)?;
            let s = serde_json::to_string(&broken)?;
            println!("{}", s);
            if broken.is_empty() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        IntegrityCmd::Append { event } => {
            let h = crate::telemetry::integrity::append(
                root,
                &event,
                serde_json::json!({"by": "human"}),
            )?;
            println!("{}", h);
            Ok(0)
        }
    }
}
