//! `HighHarness permissions` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

#[derive(Parser, Debug)]
/// struct `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: PermSub,
}

#[derive(Subcommand, Debug)]
/// enum `PermSub` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum PermSub {
    /// Variant `List` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    List,
    /// Variant `Check` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Check {
        #[clap(long)]
        /// Field `tool` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        tool: String,

        #[clap(long)]
        /// Field `args` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        args: std::path::PathBuf,
    },
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        PermSub::List => {
            let pf = crate::permissions::load(root)?;
            for r in &pf.rules {
                println!("{}\t{}\t{}\t{}", r.id, r.effect, r.priority, r.tool);
            }
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        PermSub::Check { tool, args } => {
            let raw = std::fs::read_to_string(&args)?;
            let args_value: serde_json::Value = serde_json::from_str(&raw)?;
            let reg = crate::tools::registry::Registry::load(root)?;
            let desc = reg
                .get(&tool)
                .cloned()
                .ok_or_else(|| crate::error::HxError::Other(format!("tool not found: {}", tool)))?;
            let pf = crate::permissions::load(root)?;
            let d = crate::permissions::check(&pf, &desc, &args_value, None, "ad-hoc")?;
            println!("{}", serde_json::to_string_pretty(&d)?);
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
    }
}
