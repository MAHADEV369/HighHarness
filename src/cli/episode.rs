//! `HighHarness episode` subcommand.

use std::path::Path;

use clap::{Parser, Subcommand};

use crate::error::HxResult;

#[derive(Parser, Debug)]
/// struct `Cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Cmd {
    #[clap(subcommand)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cmd: EpisodeCmd,
}

#[derive(Subcommand, Debug)]
/// enum `EpisodeCmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub enum EpisodeCmd {
    /// Variant `Open` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Open {
        #[clap(long)]
        /// Field `run_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        run_id: String,

        #[clap(long)]
        /// Field `agent_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        agent_id: String,

        #[clap(long)]
        /// Field `task_spec_file` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        task_spec_file: std::path::PathBuf,

        #[clap(long)]
        /// Field `tier` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        tier: String,

        #[clap(long)]
        /// Field `phase` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        phase: String,
    },
    /// Variant `Append` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Append {
        #[clap(long)]
        /// Field `run_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        run_id: String,

        #[clap(long)]
        /// Field `section` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        section: String,

        #[clap(long)]
        /// Field `body_file` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        body_file: std::path::PathBuf,
    },
    /// Variant `AppendToolCall` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    AppendToolCall {
        #[clap(long)]
        /// Field `run_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        run_id: String,

        #[clap(long)]
        /// Field `tool_call_json` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        tool_call_json: std::path::PathBuf,
    },
    /// Variant `Close` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Close {
        #[clap(long)]
        /// Field `run_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        run_id: String,

        #[clap(long)]
        /// Field `verification_json` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        verification_json: std::path::PathBuf,

        #[clap(long)]
        /// Field `files_touched` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        files_touched: Vec<String>,
    },
    /// Variant `Hash` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Hash {
        #[clap(long)]
        /// Field `run_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        run_id: String,
    },
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        EpisodeCmd::Append {
            run_id,
            section,
            body_file,
        } => {
            let body = std::fs::read_to_string(&body_file)?;
            crate::store::episode::append(root, &run_id, &section, &body)?;
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        EpisodeCmd::AppendToolCall {
            run_id,
            tool_call_json,
        } => {
            let raw = std::fs::read_to_string(&tool_call_json)?;
            let tc: crate::schema::episode::ToolCall = serde_json::from_str(&raw)?;
            crate::store::episode::append_tool_call(root, &run_id, tc)?;
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
        EpisodeCmd::Hash { run_id } => {
            let h = crate::store::episode::hash(root, &run_id)?;
            println!("{}", h);
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(0)
        }
    }
}
