//! `HighHarness tools` subcommand.

use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use clap::{Parser, Subcommand};
use serde_json::Value;

use crate::error::HxResult;
use crate::tools::registry::{InvokeCtx, ScopeNarrow};

/// CLI arguments for the tools subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The tool action to perform.
    pub cmd: ToolsSub,
}

/// Available tool actions.
#[derive(Subcommand, Debug)]
pub enum ToolsSub {
    /// List all registered tools.
    List,
    /// Invoke a registered tool.
    Invoke {
        /// Tool identifier to invoke.
        #[clap(long)]
        tool: String,
        /// Inline JSON object, or a path to a JSON file (detected by first non-whitespace char: '{' or '[' → inline, else path).
        #[clap(long)]
        args: String,
        /// Inline JSON object, or a path to a JSON file.
        #[clap(long)]
        scope_narrow: Option<String>,

        /// Optional run identifier for the tool call.
        #[clap(long)]
        run_id: Option<String>,

        /// Optional agent identifier for the tool call.
        #[clap(long)]
        agent_id: Option<String>,
    },
}

static TOOL_CALL_SEQ: AtomicU32 = AtomicU32::new(1);

/// Execute the tools subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        ToolsSub::List => {
            let reg = crate::tools::registry::Registry::load(root)?;
            for d in reg.list() {
                println!("{}\t{}\t{}", d.id, d.side_effect, d.summary);
            }
            Ok(0)
        }
        ToolsSub::Invoke {
            tool,
            args,
            scope_narrow,
            run_id,
            agent_id,
        } => {
            // 1. Read args (inline JSON literal OR file path; heuristic: leading '{' or '[' means inline).
            let args_raw = crate::cli::util::read_json_or_path(&args)?;
            let args_value: Value = serde_json::from_str(&args_raw)?;

            // 2. Read scope_narrow (optional).
            let narrow = if let Some(p) = scope_narrow {
                let raw = crate::cli::util::read_json_or_path(&p)?;
                let v: Value = serde_json::from_str(&raw)?;
                Some(ScopeNarrow {
                    paths: v.get("paths").and_then(|x| x.as_array()).map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect()
                    }),
                    network: v.get("network").and_then(|x| x.as_array()).map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect()
                    }),
                    env: v.get("env").and_then(|x| x.as_array()).map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect()
                    }),
                    ttl_tool_calls: v
                        .get("ttl_tool_calls")
                        .and_then(|x| x.as_u64())
                        .map(|n| n as u32),
                })
            } else {
                None
            };

            // 3. Permission check.
            let reg = crate::tools::registry::Registry::load(root)?;
            let desc = reg
                .get(&tool)
                .cloned()
                .ok_or_else(|| crate::error::HxError::Other(format!("tool not found: {}", tool)))?;
            let pf = crate::permissions::load(root)?;
            let run_id = run_id.unwrap_or_else(|| crate::id::run_id("inv", "hx"));
            let agent_id = agent_id.unwrap_or_else(crate::id::agent_id);
            let seq = TOOL_CALL_SEQ.fetch_add(1, Ordering::Relaxed);
            let tool_call_id = crate::id::tool_call_id(&run_id, seq);
            let decision =
                crate::permissions::check(&pf, &desc, &args_value, narrow.as_ref(), &tool_call_id)?;
            match decision.decision.as_str() {
                "deny" => {
                    println!("{}", serde_json::to_string_pretty(&decision)?);
                    return Ok(3);
                }
                "ask" => {
                    // Create an approval request in the queue.
                    let approval_id = crate::id::approval_id(seq);
                    let ap = crate::schema::approval::Approval {
                        schema_version: 1,

                        id: approval_id.clone(),

                        run_id: run_id.clone(),

                        tool: tool.clone(),

                        args: args_value.clone(),

                        rule_id: decision.rule_id.clone(),

                        reason: decision.reason.clone(),

                        priority: 0,

                        destructive: desc.capabilities.destructive,
                        state: "pending".to_string(),

                        modified_args: None,

                        rationale: None,

                        at: crate::id::now_iso(),

                        expires_at: String::new(),

                        self_hash: None,
                    };
                    crate::store::approvals::request(root, ap)?;
                    let mut d = decision.clone();
                    d.reason = format!("approval pending: {}", approval_id);
                    println!("{}", serde_json::to_string_pretty(&d)?);
                    return Ok(2);
                }
                _ => {}
            }

            // 4. Invoke.
            let ctx = InvokeCtx {
                run_id,
                agent_id,
                tool_call_id,
            };
            let result = reg.invoke_raw(&tool, args_value, &ctx, narrow, root)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(0)
        }
    }
}
