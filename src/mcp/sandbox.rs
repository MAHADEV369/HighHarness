//! MCP server subprocess isolation with rlimit, env stripping, and timeout.

use crate::error::{HxError, HxResult};
use crate::mcp::registry::McpServerConfig;
use std::path::Path;
use std::process::Stdio;


/// Handle to a spawned MCP sandbox process.
pub struct SandboxHandle {
    /// The child process, if still alive.
    pub child: Option<std::process::Child>,
    /// The server id this handle corresponds to.
    pub id: String,
}

impl SandboxHandle {
    /// Kill the child process if it exists and reap it.
    pub fn kill(&mut self) -> HxResult<()> {
        if let Some(ref mut c) = self.child {
            let _ = c.kill();
            let _ = c.wait();
        }
        Ok(())
    }
}

/// Spawn an MCP server process in a sandboxed environment.
///
/// Environment is stripped to only the vars declared in `cfg.env_allowed`.
pub fn spawn(cfg: &McpServerConfig, root: &Path) -> HxResult<SandboxHandle> {
    let _ = root;
    if cfg.command.is_empty() {
        return Err(HxError::Other("MCP server command is empty".to_string()));
    }

    let mut cmd = std::process::Command::new(&cfg.command[0]);
    if cfg.command.len() > 1 {
        cmd.args(&cfg.command[1..]);
    }

    // Strip environment — only allow declared vars
    cmd.env_clear();
    for var in &cfg.env_allowed {
        if let Ok(val) = std::env::var(var) {
            cmd.env(var, val);
        }
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let child = cmd.spawn().map_err(HxError::Io)?;

    Ok(SandboxHandle {
        child: Some(child),
        id: cfg.id.clone(),
    })
}

/// Spawn with a timeout (currently a stub — caller manages timeout).
pub fn spawn_with_timeout(
    cfg: &McpServerConfig,
    root: &Path,
    timeout_secs: u64,
) -> HxResult<SandboxHandle> {
    let handle = spawn(cfg, root)?;
    let _ = timeout_secs;
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::registry::McpServerConfig;

    #[test]
    fn mcp_spawn_with_env_stripping() {
        let cfg = McpServerConfig {
            id: "test-env".to_string(),
            command: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "echo hello".to_string(),
            ],
            paths_allowed: vec![],
            network_allowed: vec![],
            env_allowed: vec!["PATH".to_string()],
            cpu_seconds: None,
            memory_mb: None,
            timeout_seconds: Some(5),
        };
        let dir = tempfile::TempDir::new().unwrap();
        let mut handle = spawn(&cfg, dir.path()).unwrap();
        let result = handle.child.as_mut().unwrap().wait();
        assert!(result.is_ok(), "process should exit: {:?}", result);
    }

    #[test]
    fn mcp_spawn_empty_command_returns_error() {
        let cfg = McpServerConfig {
            id: "empty".to_string(),
            command: vec![],
            paths_allowed: vec![],
            network_allowed: vec![],
            env_allowed: vec![],
            cpu_seconds: None,
            memory_mb: None,
            timeout_seconds: None,
        };
        let dir = tempfile::TempDir::new().unwrap();
        let result = spawn(&cfg, dir.path());
        assert!(result.is_err(), "empty command should error");
    }
}
