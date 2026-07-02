//! `HighHarness changelog` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the changelog subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The changelog action to perform.
    pub cmd: ChangelogCmd,
}

/// Available changelog actions.
#[derive(Subcommand, Debug)]
pub enum ChangelogCmd {
    /// Append an entry from a JSON file.
    Append {
        /// Path to a JSON file conforming to schema::changelog::Entry.
        #[clap(long)]
        entry: std::path::PathBuf,
    },
    /// Print the latest entry.
    Latest,
    /// Verify the chain; exit 0 if healthy.
    VerifyChain {
        /// Optional number of trailing entries to check.
        #[clap(long)]
        tail: Option<usize>,
    },
    /// Get entry N.
    Get {
        /// 1-based entry number to fetch.
        n: u64,
    },
}

/// Execute the changelog subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        ChangelogCmd::Append { entry } => {
            let raw = std::fs::read_to_string(&entry)?;
            let mut e: crate::schema::changelog::Entry = serde_json::from_str(&raw)?;
            // Auto-fill prev_hash from chain head.
            e.prev_hash = crate::store::changelog::latest_or_genesis(root)?;
            // Auto-fill n if zero.
            if e.n == 0 {
                e.n = next_n(root);
            }
            let h = crate::store::changelog::append(&mut e, root)?;
            println!("{}", h);
            Ok(0)
        }
        ChangelogCmd::Latest => match crate::store::changelog::latest(root) {
            Ok(e) => {
                println!("{}", serde_json::to_string_pretty(&e)?);
                Ok(0)
            }
            Err(e) => {
                eprintln!("latest: {}", e);
                Ok(1)
            }
        },
        ChangelogCmd::VerifyChain { tail } => {
            let broken = crate::store::changelog::verify_chain(root, tail)?;
            let s = serde_json::to_string(&broken)?;
            println!("{}", s);
            if broken.is_empty() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        ChangelogCmd::Get { n } => match crate::store::changelog::get(n, root) {
            Ok(e) => {
                println!("{}", serde_json::to_string_pretty(&e)?);
                Ok(0)
            }
            Err(e) => {
                eprintln!("get: {}", e);
                Ok(1)
            }
        },
    }
}

fn next_n(root: &Path) -> u64 {
    crate::store::changelog::latest(root)
        .map(|e| e.n + 1)
        .unwrap_or(1)
}
