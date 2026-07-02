//! `HighHarness spend` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the spend subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The spend action to perform.
    pub cmd: SpendSub,
}

/// Available spend actions.
#[derive(Subcommand, Debug)]
pub enum SpendSub {
    /// Append a spend line from a JSON file.
    Append {
        /// Path to the JSON file describing the spend line.
        line: std::path::PathBuf,
    },
    /// Print a spend summary for a given month.
    Summary {
        /// Month in `YYYY-MM` format.
        month: String,
    },
}

/// Execute the spend subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        SpendSub::Append { line } => {
            let raw = std::fs::read_to_string(&line)?;
            let l: crate::schema::spend::SpendLine = serde_json::from_str(&raw)?;
            crate::store::spend::append(root, l)?;
            Ok(0)
        }
        SpendSub::Summary { month } => {
            let s = crate::store::spend::summary(root, &month)?;
            println!("{}", serde_json::to_string_pretty(&s)?);
            Ok(0)
        }
    }
}
