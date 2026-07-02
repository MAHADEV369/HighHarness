//! `HighHarness metrics` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the metrics subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The metrics action to perform.
    pub cmd: MetricsCmd,
}

/// Available metrics actions.
#[derive(Subcommand, Debug)]
pub enum MetricsCmd {
    /// Compute a metrics rollup for the given window.
    Rollup {
        /// Window size: "7d", "30d", or "90d".
        #[clap(long)]
        window: String,
    },
    /// Evaluate alerts over the given window.
    Alert {
        /// Window size: "7d", "30d", or "90d".
        #[clap(long)]
        window: String,
    },
    /// Print a simple health summary.
    Health,
}

/// Execute the metrics subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        MetricsCmd::Rollup { window } => {
            let dur = crate::metrics::parse_window(&window);
            let r = crate::metrics::rollup(root, &dur)?;
            println!("{}", serde_json::to_string_pretty(&r)?);
            Ok(0)
        }
        MetricsCmd::Alert { window } => {
            let dur = crate::metrics::parse_window(&window);
            let r = crate::metrics::rollup(root, &dur)?;
            let alerts = crate::metrics::evaluate_alerts(&r, root);
            println!("{}", serde_json::to_string_pretty(&alerts)?);
            Ok(0)
        }
        MetricsCmd::Health => {
            // Print a simple health summary
            let metrics_dir = root.join(".harness").join("artifacts").join("metrics");
            let last_rollup = metrics_dir.join("..").join(".last-rollup");
            let fresh = if last_rollup.exists() {
                if let Ok(m) = std::fs::metadata(&last_rollup) {
                    if let Ok(modified) = m.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            elapsed.as_secs() < 86400
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };
            if fresh {
                println!("health: ok — rollup fresh (< 24h old)");
            } else {
                println!("health: stale — no rollup in the last 24h");
            }
            Ok(0)
        }
    }
}
