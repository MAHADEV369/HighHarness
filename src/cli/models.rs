//! HighHarness models subcommand — model listing and completion.

use crate::error::HxResult;
use clap::{Parser, Subcommand};
use std::path::Path;

/// `HighHarness models` subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    /// Subcommand.
    #[clap(subcommand)]
    pub cmd: ModelsCmd,
}

/// Models subcommands.
#[derive(Subcommand, Debug)]
pub enum ModelsCmd {
    /// List configured models from .harness/models.toml
    List,
    /// Complete a model request (stub)
    Complete {
        /// Model id.
        #[clap(long)]
        model: String,
        /// File with messages JSON.
        #[clap(long)]
        messages_file: Option<std::path::PathBuf>,
        /// Inline messages JSON.
        #[clap(long)]
        messages: Option<String>,
    },
}

/// Run the models subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        ModelsCmd::List => {
            let models = crate::models::load_models(root)?;
            for m in &models {
                println!("{} (provider={})", m.id, m.provider);
            }
            Ok(0)
        }
        ModelsCmd::Complete {
            model,
            messages_file,
            messages,
        } => {
            let _ = model;
            let _ = messages_file;
            let _ = messages;
            let redactions = crate::redaction::Redactions::load(root)?;
            let req = crate::models::openai_compat::CompleteRequest {
                model_id: model,
                messages: vec![],
                tools: None,
                system: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                prefill: None,
                stream: false,
            };
            let events = crate::models::openai_compat::complete(&req, &redactions, root)?;
            for e in &events {
                println!("{}", serde_json::to_string(e).unwrap());
            }
            Ok(0)
        }
    }
}
