//! `HighHarness changelog` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

#[derive(Parser, Debug)]
/// struct `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: ChangelogCmd,
}

#[derive(Subcommand, Debug)]
/// enum `ChangelogCmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum ChangelogCmd {
    /// Append an entry from a JSON file.
    Append {
        /// Path to a JSON file conforming to schema::changelog::Entry.
        #[clap(long)]
        /// Field `entry` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        entry: std::path::PathBuf,
    },
    /// Print the latest entry.
    Latest,
    /// Verify the chain; exit 0 if healthy.
    VerifyChain {
        #[clap(long)]
        /// Field `tail` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        tail: Option<usize>,
    },
    /// Get entry N.
    Get {
        /// 1-based entry number to fetch.
        n: u64,
    },
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        ChangelogCmd::Latest => match crate::store::changelog::latest(root) {
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(e) => {
                println!("{}", serde_json::to_string_pretty(&e)?);
                /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
                Ok(0)
            }
            /// Variant `Err` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Err(e) => {
                eprintln!("latest: {}", e);
                /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
                Ok(1)
            }
        },
        ChangelogCmd::VerifyChain { tail } => {
            let broken = crate::store::changelog::verify_chain(root, tail)?;
            let s = serde_json::to_string(&broken)?;
            println!("{}", s);
            if broken.is_empty() {
                /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
                Ok(0)
            } else {
                /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
                Ok(1)
            }
        }
        ChangelogCmd::Get { n } => match crate::store::changelog::get(n, root) {
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(e) => {
                println!("{}", serde_json::to_string_pretty(&e)?);
                /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
                Ok(0)
            }
            /// Variant `Err` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Err(e) => {
                eprintln!("get: {}", e);
                /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
