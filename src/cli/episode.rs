//! `HighHarness episode` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

/// CLI arguments for the episode subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The episode action to perform.
    pub cmd: EpisodeCmd,
}

/// Available episode actions.
#[derive(Subcommand, Debug)]
pub enum EpisodeCmd {
    /// Open a new episode for a run.
    Open {
        /// Run identifier.
        #[clap(long)]
        run_id: String,

        /// Agent identifier.
        #[clap(long)]
        agent_id: String,

        /// Path to the task spec file.
        #[clap(long)]
        task_spec_file: std::path::PathBuf,

        /// Budget tier for the run.
        #[clap(long)]
        tier: String,

        /// Build phase of the run.
        #[clap(long)]
        phase: String,
    },
    /// Append a section to an open episode.
    Append {
        /// Run identifier.
        #[clap(long)]
        run_id: String,

        /// Section name to append.
        #[clap(long)]
        section: String,

        /// Path to the body content file.
        #[clap(long)]
        body_file: std::path::PathBuf,
    },
    /// Append a tool call record to an episode.
    AppendToolCall {
        /// Run identifier.
        #[clap(long)]
        run_id: String,

        /// Path to the tool call JSON file.
        #[clap(long)]
        tool_call_json: std::path::PathBuf,
    },
    /// Close an episode with verification.
    Close {
        /// Run identifier.
        #[clap(long)]
        run_id: String,

        /// Path to the verification JSON file.
        #[clap(long)]
        verification_json: std::path::PathBuf,

        /// Files touched during the run.
        #[clap(long)]
        files_touched: Vec<String>,
    },
    /// Compute the episode hash.
    Hash {
        /// Run identifier.
        #[clap(long)]
        run_id: String,
    },
    /// Render an episode as a self-contained HTML report.
    Render {
        /// Run identifier.
        #[clap(long)]
        run_id: String,

        /// Output file path (default: stdout).
        #[clap(long)]
        output: Option<std::path::PathBuf>,
    },
}

/// Execute the episode subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        EpisodeCmd::Open {
            run_id,
            agent_id,
            task_spec_file,
            tier,
            phase,
        } => {
            let spec = std::fs::read_to_string(&task_spec_file)?;
            crate::store::episode::open(root, &run_id, &agent_id, &spec, &tier, &phase)?;
            println!("{{\"run_id\": \"{}\"}}", run_id);
            Ok(0)
        }
        EpisodeCmd::Append {
            run_id,
            section,
            body_file,
        } => {
            let body = std::fs::read_to_string(&body_file)?;
            crate::store::episode::append(root, &run_id, &section, &body)?;
            Ok(0)
        }
        EpisodeCmd::AppendToolCall {
            run_id,
            tool_call_json,
        } => {
            let raw = std::fs::read_to_string(&tool_call_json)?;
            let tc: crate::schema::episode::ToolCall = serde_json::from_str(&raw)?;
            crate::store::episode::append_tool_call(root, &run_id, tc)?;
            Ok(0)
        }
        EpisodeCmd::Close {
            run_id,
            verification_json,
            files_touched,
        } => {
            let v = std::fs::read_to_string(&verification_json)?;
            let h = crate::store::episode::close(root, &run_id, &v, files_touched)?;
            println!("{}", h);
            Ok(0)
        }
        EpisodeCmd::Hash { run_id } => {
            let h = crate::store::episode::hash(root, &run_id)?;
            println!("{}", h);
            Ok(0)
        }
        EpisodeCmd::Render { run_id, output } => {
            let path = crate::store::episodes_dir(root).join(format!("{}.md", run_id));
            crate::report::render(&path, output.as_deref())?;
            Ok(0)
        }
    }
}
