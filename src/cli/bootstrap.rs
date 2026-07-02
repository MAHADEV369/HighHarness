//! `HighHarness bootstrap` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

#[derive(Parser, Debug)]
/// struct `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: BootstrapCmd,
}

#[derive(Subcommand, Debug)]
/// enum `BootstrapCmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum BootstrapCmd {
    /// Variant `Init` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Init {
        /// Name of the human performing the bootstrap.
        #[clap(long)]
        /// Field `human` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        human: String,
    },
    /// Variant `Verify` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Verify,
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        BootstrapCmd::Init { human } => {
            let bs = crate::bootstrap::init(root, &human)?;
            println!("{}", serde_json::to_string_pretty(&bs).unwrap());
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        BootstrapCmd::Verify => match crate::bootstrap::verify(root) {
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(bs) => {
                println!("{}", serde_json::to_string_pretty(&bs).unwrap());
                /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
                Ok(0)
            }
            /// Variant `Err` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Err(e) => {
                eprintln!("bootstrap verify failed: {}", e);
                /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
                Ok(4)
            }
        },
    }
}
