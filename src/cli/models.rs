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
            let msgs: Vec<crate::models::openai_compat::Message> =
                if let Some(ref path) = messages_file {
                    let raw = std::fs::read_to_string(path)?;
                    serde_json::from_str(&raw)?
                } else if let Some(ref inline) = messages {
                    serde_json::from_str(inline)?
                } else {
                    vec![]
                };
            let redactions = crate::redaction::Redactions::load(root)?;
            let req = crate::models::openai_compat::CompleteRequest {
                model_id: model,
                messages: msgs,
                tools: None,
                system: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                prefill: None,
                stream: false,
            };
            match crate::models::openai_compat::complete(&req, &redactions, root) {
                Ok(events) => {
                    for e in &events {
                        println!(
                            "{}",
                            serde_json::to_string(e)
                                .expect("JSON serialization should not fail on model events")
                        );
                    }
                }
                Err(e) => {
                    let err_event = crate::models::openai_compat::ModelEvent {
                        kind: "error".to_string(),
                        delta: None,
                        tool_call: None,
                        usage: None,
                        cost: None,
                        finish_reason: None,
                        error: Some(e.to_string()),
                    };
                    println!(
                        "{}",
                        serde_json::to_string(&err_event)
                            .expect("JSON serialization should not fail on model events")
                    );
                }
            }
            Ok(0)
        }
    }
}
