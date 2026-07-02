//! `HighHarness permissions` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the permissions subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The permission action to perform.
    pub cmd: PermSub,
}

/// Available permission actions.
#[derive(Subcommand, Debug)]
pub enum PermSub {
    /// List all permission rules.
    List,
    /// Check a tool invocation against permission rules.
    Check {
        /// Tool identifier to check.
        #[clap(long)]
        tool: String,

        /// Path to a JSON file with the tool arguments.
        #[clap(long)]
        args: std::path::PathBuf,
    },
}

/// Execute the permissions subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        PermSub::List => {
            let pf = crate::permissions::load(root)?;
            for r in &pf.rules {
                println!("{}\t{}\t{}\t{}", r.id, r.effect, r.priority, r.tool);
            }
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
            Ok(0)
        }
    }
}
