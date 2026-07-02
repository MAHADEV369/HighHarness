//! `HighHarness metrics` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

#[derive(Parser, Debug)]
/// struct `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: MetricsCmd,
}

#[derive(Subcommand, Debug)]
/// enum `MetricsCmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum MetricsCmd {
    /// Variant `Rollup` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Rollup {
        /// Window size: "7d", "30d", or "90d".
        #[clap(long)]
        window: String,
    },
    /// Variant `Alert` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Alert {
        /// Window size: "7d", "30d", or "90d".
        #[clap(long)]
        window: String,
    },
    /// Variant `Health` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Health,
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        MetricsCmd::Rollup { window } => {
            let dur = crate::metrics::parse_window(&window);
            let r = crate::metrics::rollup(root, &dur)?;
            println!("{}", serde_json::to_string_pretty(&r)?);
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        MetricsCmd::Alert { window } => {
            let dur = crate::metrics::parse_window(&window);
            let r = crate::metrics::rollup(root, &dur)?;
            let alerts = crate::metrics::evaluate_alerts(&r, root);
            println!("{}", serde_json::to_string_pretty(&alerts)?);
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
    }
}
