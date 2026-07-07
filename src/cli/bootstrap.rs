//! `HighHarness bootstrap` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the bootstrap subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The bootstrap action to perform.
    pub cmd: BootstrapCmd,
}

/// Available bootstrap actions.
#[derive(Subcommand, Debug)]
pub enum BootstrapCmd {
    /// Initialise a new harness workspace.
    Init {
        /// Name of the human performing the bootstrap.
        #[clap(long)]
        human: String,
    },
    /// Verify an existing bootstrap is intact.
    Verify,
}

/// Execute the bootstrap subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        BootstrapCmd::Init { human } => {
            let bs = crate::bootstrap::init(root, &human)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&bs)
                    .expect("Bootstrap JSON serialization should not fail")
            );
            Ok(0)
        }
        BootstrapCmd::Verify => match crate::bootstrap::verify(root) {
            Ok(bs) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&bs)
                        .expect("Bootstrap JSON serialization should not fail")
                );
                Ok(0)
            }
            Err(e) => {
                eprintln!("bootstrap verify failed: {}", e);
                Ok(4)
            }
        },
    }
}
