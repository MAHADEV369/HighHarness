//! MCP server registry — load/save McpServerConfig from .harness/mcp.toml.

use crate::error::HxResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration for a single MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Unique server identifier.
    pub id: String,
    /// Command and arguments (first element is the binary path).
    pub command: Vec<String>,
    /// Filesystem paths the server is allowed to access.
    #[serde(default)]
    pub paths_allowed: Vec<String>,
    /// Network addresses/ranges the server is allowed to connect to.
    #[serde(default)]
    pub network_allowed: Vec<String>,
    /// Environment variable names to forward from the harness.
    #[serde(default)]
    pub env_allowed: Vec<String>,
    /// CPU time limit in seconds (soft).
    pub cpu_seconds: Option<u32>,
    /// Memory limit in megabytes (soft).
    pub memory_mb: Option<u32>,
    /// Wall-clock timeout in seconds.
    pub timeout_seconds: Option<u32>,
}

/// The MCP server registry file format.
#[derive(Debug, Serialize, Deserialize)]
pub struct McpRegistry {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// Registered server configs.
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

/// Load the MCP server registry from `.harness/mcp.toml`.
///
/// Returns an empty registry if the file does not exist.
pub fn load(root: &Path) -> HxResult<McpRegistry> {
    let path = root.join(".harness").join("mcp.toml");
    if !path.exists() {
        return Ok(McpRegistry {
            schema_version: 1,
            servers: Vec::new(),
        });
    }
    let raw = fs::read_to_string(&path)?;
    let reg: McpRegistry = toml::from_str(&raw)?;
    Ok(reg)
}

/// Save the MCP server registry to `.harness/mcp.toml`.
pub fn save(root: &Path, reg: &McpRegistry) -> HxResult<()> {
    let path = root.join(".harness").join("mcp.toml");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = toml::to_string(&reg).map_err(|e| crate::error::HxError::Other(e.to_string()))?;
    fs::write(&path, raw)?;
    Ok(())
}

/// List all registered MCP servers.
pub fn list_servers(root: &Path) -> HxResult<Vec<McpServerConfig>> {
    let reg = load(root)?;
    Ok(reg.servers)
}

/// Register (add or replace) an MCP server config.
pub fn register_server(root: &Path, cfg: McpServerConfig) -> HxResult<()> {
    let mut reg = load(root)?;
    reg.servers.retain(|s| s.id != cfg.id);
    reg.servers.push(cfg);
    save(root, &reg)
}

/// Get a registered server by id.
pub fn get_server(root: &Path, id: &str) -> HxResult<Option<McpServerConfig>> {
    let reg = load(root)?;
    Ok(reg.servers.into_iter().find(|s| s.id == id))
}
