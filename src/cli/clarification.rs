//! `HighHarness clarification` subcommand (stub).

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

#[derive(Parser, Debug)]
/// struct `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: ClarificationCmd,
}

#[derive(Subcommand, Debug)]
/// enum `ClarificationCmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum ClarificationCmd {
    /// Variant `List` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    List,
    /// Variant `Resolve` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Resolve {
        #[clap(long)]
        /// Field `id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        id: String,

        #[clap(long)]
        /// Field `answer` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        answer: String,
    },
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        ClarificationCmd::List => {
            println!("[]");
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        ClarificationCmd::Resolve { id, answer } => {
            println!("{} {}", id, answer);
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
    }
}
