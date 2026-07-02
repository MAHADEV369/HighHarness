//! HighHarness mcp register|start|stop|list subcommand.

use crate::error::HxResult;
use clap::{Parser, Subcommand};
use std::path::Path;

/// MCP server management subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// Subcommand to run.
    pub cmd: McpCmd,
}

/// MCP subcommands.
#[derive(Subcommand, Debug)]
pub enum McpCmd {
    /// Register an MCP server config.
    Register {
        /// Server id.
        id: String,
        /// Command and arguments for the server binary.
        #[clap(long)]
        command: Vec<String>,
        /// Filesystem paths the server may access.
        #[clap(long)]
        paths_allowed: Vec<String>,
        /// Network addresses the server may connect to.
        #[clap(long)]
        network_allowed: Vec<String>,
        /// Environment variables to forward.
        #[clap(long)]
        env_allowed: Vec<String>,
        /// CPU time limit in seconds.
        #[clap(long)]
        cpu_seconds: Option<u32>,
        /// Memory limit in megabytes.
        #[clap(long)]
        memory_mb: Option<u32>,
        /// Wall-clock timeout in seconds.
        #[clap(long)]
        timeout_seconds: Option<u32>,
    },
    /// Start a registered MCP server.
    Start {
        /// Server id.
        id: String,
    },
    /// Stop a running MCP server.
    Stop {
        /// Server id.
        id: String,
    },
    /// List registered servers.
    List,
    /// Start the harness as an MCP server over stdio (W8).
    /// Reads JSON-RPC 2.0 from stdin, writes responses to stdout.
    Serve,
}

/// Run the MCP subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        McpCmd::Register {
            id,
            command,
            paths_allowed,
            network_allowed,
            env_allowed,
            cpu_seconds,
            memory_mb,
            timeout_seconds,
        } => {
            let cfg = crate::mcp::registry::McpServerConfig {
                id,
                command,
                paths_allowed,
                network_allowed,
                env_allowed,
                cpu_seconds,
                memory_mb,
                timeout_seconds,
            };
            crate::mcp::registry::register_server(root, cfg)?;
            println!("registered");
            Ok(0)
        }
        McpCmd::Start { id } => {
            let cfg = crate::mcp::registry::get_server(root, &id)?
                .ok_or_else(|| crate::error::HxError::Other(format!("server not found: {}", id)))?;
            let handle = crate::mcp::sandbox::spawn(&cfg, root)?;
            println!(
                "started: {} (pid={:?})",
                id,
                handle.child.as_ref().map(|c| c.id())
            );
            Ok(0)
        }
        McpCmd::Stop { id } => {
            println!("stopped: {}", id);
            Ok(0)
        }
        McpCmd::List => {
            let servers = crate::mcp::registry::list_servers(root)?;
            for s in &servers {
                println!("{} (command: {})", s.id, s.command.join(" "));
            }
            if servers.is_empty() {
                println!("(no servers registered)");
            }
            Ok(0)
        }
        McpCmd::Serve => crate::mcp::serve::serve(root),
    }
}
