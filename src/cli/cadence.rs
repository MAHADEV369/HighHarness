//! `HighHarness cadence` subcommand.

use std::fs;
use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the cadence subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The cadence action to perform.
    pub cmd: CadenceCmd,
}

/// Available cadence actions.
#[derive(Subcommand, Debug)]
pub enum CadenceCmd {
    /// Run a cadence rollup for the specified window.
    Run {
        /// Run daily cadence.
        #[clap(long)]
        daily: bool,
        /// Run weekly cadence.
        #[clap(long)]
        weekly: bool,
        /// Run monthly cadence.
        #[clap(long)]
        monthly: bool,
    },
}

/// Execute the cadence subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        CadenceCmd::Run {
            daily,
            weekly,
            monthly,
        } => {
            // Determine window from flags
            let window_str = if daily {
                "1d"
            } else if weekly {
                "7d"
            } else if monthly {
                "30d"
            } else {
                "7d"
            };

            // Check .last-rollup freshness
            let last_rollup_path = root
                .join(".harness")
                .join("artifacts")
                .join("metrics")
                .join(".last-rollup");
            let stale = if last_rollup_path.exists() {
                if let Ok(m) = fs::metadata(&last_rollup_path) {
                    if let Ok(modified) = m.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            let max_age = if daily {
                                86400
                            } else if weekly {
                                7 * 86400
                            } else {
                                30 * 86400
                            };
                            elapsed.as_secs() > max_age
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                } else {
                    true
                }
            } else {
                true
            };

            if stale {
                let dur = crate::metrics::parse_window(window_str);
                let r = crate::metrics::rollup(root, &dur)?;
                // Update .last-rollup
                if let Some(parent) = last_rollup_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&last_rollup_path, &r.produced_at)?;
                println!("rollup: fresh (computed {})", r.produced_at);
            } else {
                println!("ok: rollup fresh");
            }

            Ok(0)
        }
    }
}
