use std::fs;
use std::io::Write;
use std::path::Path;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::error::{HxError, HxResult};

/// CLI arguments for the clarification subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    /// The clarification action to perform.
    #[clap(subcommand)]
    pub cmd: ClarificationCmd,
}

/// Available clarification actions.
#[derive(Subcommand, Debug)]
pub enum ClarificationCmd {
    /// List all clarification requests.
    List,
    /// Request a clarification.
    Request {
        /// The question to ask.
        #[clap(long)]
        question: String,
    },
    /// Resolve a clarification request with an answer.
    Resolve {
        /// ID of the clarification request.
        #[clap(long)]
        id: String,
        /// The answer to the clarification.
        #[clap(long)]
        answer: String,
    },
}

/// A persisted clarification request.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Clarification {
    /// Schema version.
    schema_version: u32,
    /// Unique identifier.
    id: String,
    /// The question asked.
    question: String,
    /// The answer, if resolved.
    answer: Option<String>,
    /// Status: "pending" or "resolved".
    status: String,
    /// ISO-8601 creation timestamp.
    created_at: String,
    /// ISO-8601 resolution timestamp.
    resolved_at: Option<String>,
}

/// Return the path to the clarifications artifact directory.
fn clarifications_dir(root: &Path) -> std::path::PathBuf {
    root.join(".harness").join("artifacts").join("clarifications")
}

/// Execute the clarification subcommand with persistent storage.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        ClarificationCmd::List => {
            let dir = clarifications_dir(root);
            let mut results = Vec::new();
            if dir.exists() {
                for entry in fs::read_dir(&dir)? {
                    let entry = entry?;
                    let raw = fs::read_to_string(entry.path())?;
                    if let Ok(c) = serde_json::from_str::<Clarification>(&raw) {
                        results.push(c);
                    }
                }
            }
            results.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            println!("{}", serde_json::to_string_pretty(&results)?);
            Ok(0)
        }
        ClarificationCmd::Request { question } => {
            let dir = clarifications_dir(root);
            fs::create_dir_all(&dir)?;
            let id = format!("cl-{}", crate::id::now_iso());
            let now = crate::id::now_iso();
            let c = Clarification {
                schema_version: 1,
                id: id.clone(),
                question,
                answer: None,
                status: "pending".to_string(),
                created_at: now,
                resolved_at: None,
            };
            let path = dir.join(format!("{}.json", id));
            let mut f = fs::File::create(&path)?;
            f.write_all(serde_json::to_string_pretty(&c)?.as_bytes())?;
            f.sync_data()?;
            println!("{}", serde_json::to_string_pretty(&c)?);
            Ok(0)
        }
        ClarificationCmd::Resolve { id, answer } => {
            let dir = clarifications_dir(root);
            let path = dir.join(format!("{}.json", id));
            if !path.exists() {
                return Err(HxError::Other(format!("clarification not found: {}", id)));
            }
            let raw = fs::read_to_string(&path)?;
            let mut c: Clarification = serde_json::from_str(&raw)?;
            c.answer = Some(answer);
            c.status = "resolved".to_string();
            c.resolved_at = Some(crate::id::now_iso());
            let mut f = fs::File::create(&path)?;
            f.write_all(serde_json::to_string_pretty(&c)?.as_bytes())?;
            f.sync_data()?;
            println!("{}", serde_json::to_string_pretty(&c)?);
            Ok(0)
        }
    }
}
