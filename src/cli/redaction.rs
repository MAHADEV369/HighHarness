//! HighHarness redaction scan|list|add subcommand.

use crate::error::HxResult;
use clap::{Parser, Subcommand};
use std::path::Path;

/// CLI arguments for the redaction subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The redaction action to perform.
    pub cmd: RedactionCmd,
}

/// Available redaction actions.
#[derive(Subcommand, Debug)]
pub enum RedactionCmd {
    /// Scan content for redactable secrets.
    Scan {
        /// Path to file (reads stdin if omitted).
        #[clap(long)]
        file: Option<std::path::PathBuf>,
    },
    /// List registered redaction patterns.
    List,
    /// Add a new pattern (denied inside a run via R-DENY-HARNESS).
    Add {
        /// Pattern id.
        #[clap(long)]
        id: String,
        /// Regex pattern.
        #[clap(long)]
        regex: String,
        /// Severity level.
        #[clap(long)]
        severity: String,
    },
}

/// Run the redaction subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        RedactionCmd::Scan { file } => {
            let content = if let Some(path) = file {
                std::fs::read_to_string(&path)?
            } else {
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
                    .map_err(crate::error::HxError::Io)?;
                buf
            };
            let redactions = crate::redaction::Redactions::load(root)?;
            let results = redactions.scan(&content);
            println!("{}", serde_json::to_string_pretty(&results).unwrap());
            Ok(0)
        }
        RedactionCmd::List => {
            let redactions = crate::redaction::Redactions::load(root)?;
            for p in &redactions.patterns {
                println!("{} (severity={}) regex={}", p.id, p.severity, p.regex_str);
            }
            Ok(0)
        }
        RedactionCmd::Add {
            id,
            regex,
            severity,
        } => {
            let path = root.join(".harness").join("redactions.toml");
            let mut raw = std::fs::read_to_string(&path).unwrap_or_default();
            raw.push_str(&format!(
                "\n[[patterns]]\nid = \"{}\"\nregex = \"{}\"\nseverity = \"{}\"\n",
                id, regex, severity
            ));
            std::fs::write(&path, raw)?;
            println!("added pattern: {}", id);
            Ok(0)
        }
    }
}
