//! `HighHarness integrity` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

#[derive(Parser, Debug)]
/// struct `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: IntegrityCmd,
}

#[derive(Subcommand, Debug)]
/// enum `IntegrityCmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum IntegrityCmd {
    /// Variant `Verify` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Verify,
    /// Variant `Append` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Append {
        /// Event name to record in the integrity log.
        event: String,
    },
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        IntegrityCmd::Verify => {
            let broken = crate::telemetry::integrity::verify(root)?;
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
        IntegrityCmd::Append { event } => {
            let h = crate::telemetry::integrity::append(
                root,
                &event,
                /// Field `serde_json` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
                serde_json::json!({"by": "human"}),
            )?;
            println!("{}", h);
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
    }
}
