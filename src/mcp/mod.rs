//! MCP (Model Context Protocol) server management and subprocess isolation.
//! Implements HARNESS_SECURITY.md §7 and HARNESS_PRIMITIVES.md §1.4.

pub mod registry;
pub mod sandbox;
/// MCP server mode: exposes the harness tool registry as an MCP server
/// over stdio with permission enforcement and episode recording.
pub mod serve;
