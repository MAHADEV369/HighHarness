//! `HighHarness spend` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

#[derive(Parser, Debug)]
/// struct `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: SpendSub,
}

#[derive(Subcommand, Debug)]
/// enum `SpendSub` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum SpendSub {
    /// Variant `Append` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Append {
        /// Path to the JSON file describing the spend line.
        line: std::path::PathBuf,
    },
    /// Variant `Summary` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Summary {
        /// Month in `YYYY-MM` format.
        month: String,
    },
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        SpendSub::Append { line } => {
            let raw = std::fs::read_to_string(&line)?;
            let l: crate::schema::spend::SpendLine = serde_json::from_str(&raw)?;
            crate::store::spend::append(root, l)?;
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        SpendSub::Summary { month } => {
            let s = crate::store::spend::summary(root, &month)?;
            println!("{}", serde_json::to_string_pretty(&s)?);
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
    }
}
