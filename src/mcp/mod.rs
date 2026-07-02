//! MCP (Model Context Protocol) server management and subprocess isolation.
//! Implements HARNESS_SECURITY.md §7 and HARNESS_PRIMITIVES.md §1.4.

/// mod `registry` — MCP server config registry (read/write .harness/mcp.toml).
pub mod registry;
/// mod `sandbox` — MCP server subprocess isolation.
pub mod sandbox;
/// mod `serve` — Expose the harness as an MCP server over stdio (W8).
pub mod serve;
