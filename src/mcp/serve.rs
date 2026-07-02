//! MCP server mode — exposes the harness as an MCP server over stdio.
//!
//! Reads JSON-RPC 2.0 requests from stdin, dispatches to the tool registry,
//! and writes JSON-RPC 2.0 responses to stdout. This allows consumers to
//! connect via the Model Context Protocol instead of subprocess + JSON.
//!
//! Supported methods:
//! - `initialize` — protocol negotiation
//! - `notifications/initialized` — confirm init (no response)
//! - `ping` — health check
//! - `tools/list` — list all registered tools
//! - `tools/call` — invoke a tool by name
//! - `shutdown` — graceful exit

use crate::error::{HxError, HxResult};
use crate::id;
use crate::tools::registry::{InvokeCtx, Registry, ScopeNarrow};
use serde_json::Value;
use std::io::{BufRead, Write};
use std::path::Path;

/// Run the MCP server over stdio. Reads lines, dispatches, writes responses.
pub fn serve(root: &Path) -> HxResult<i32> {
    let registry = Registry::load(root)?;
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let mut reader = std::io::BufReader::new(stdin.lock());
    let mut writer = std::io::BufWriter::new(stdout.lock());

    let mut line = String::new();
    let run_id = format!("mcp-serve-{}", id::now_iso());

    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break; // EOF
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let err = json_rpc_error(None, -32700, format!("Parse error: {}", e));
                let out = serde_json::to_string(&err)?;
                writeln!(writer, "{}", out)?;
                writer.flush()?;
                continue;
            }
        };

        let id_val = req.get("id").cloned();
        let method = req
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let params = req.get("params").cloned().unwrap_or(Value::Null);

        // notifications have no id
        if method == "notifications/initialized" {
            continue;
        }

        let response = match method.as_str() {
            "initialize" => handle_initialize(id_val, &params),
            "ping" => json_rpc_result(id_val, Value::Null),
            "tools/list" => handle_tools_list(id_val, &registry, root),
            "tools/call" => handle_tools_call(id_val, &params, &registry, root, &run_id),
            "shutdown" => {
                let resp = json_rpc_result(id_val, Value::Null);
                let out = serde_json::to_string(&resp)?;
                writeln!(writer, "{}", out)?;
                writer.flush()?;
                return Ok(0);
            }
            _ => json_rpc_error(id_val, -32601, format!("Method not found: {}", method)),
        };

        let out = serde_json::to_string(&response)?;
        writeln!(writer, "{}", out)?;
        writer.flush()?;
    }

    Ok(0)
}

/// Build a JSON-RPC 2.0 success response.
fn json_rpc_result(id: Option<Value>, result: Value) -> Value {
    let mut resp = serde_json::Map::new();
    resp.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
    if let Some(id) = id {
        resp.insert("id".to_string(), id);
    } else {
        resp.insert("id".to_string(), Value::Null);
    }
    resp.insert("result".to_string(), result);
    Value::Object(resp)
}

/// Build a JSON-RPC 2.0 error response.
fn json_rpc_error(id: Option<Value>, code: i64, message: String) -> Value {
    let mut resp = serde_json::Map::new();
    resp.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
    if let Some(id) = id {
        resp.insert("id".to_string(), id);
    } else {
        resp.insert("id".to_string(), Value::Null);
    }
    let mut err = serde_json::Map::new();
    err.insert("code".to_string(), Value::Number(code.into()));
    err.insert("message".to_string(), Value::String(message));
    resp.insert("error".to_string(), Value::Object(err));
    Value::Object(resp)
}

/// Handle the MCP `initialize` method.
fn handle_initialize(id: Option<Value>, _params: &Value) -> Value {
    let mut result = serde_json::Map::new();
    result.insert(
        "protocolVersion".to_string(),
        Value::String("2024-11-05".to_string()),
    );
    let mut capabilities = serde_json::Map::new();
    let mut tools = serde_json::Map::new();
    tools.insert("listChanged".to_string(), Value::Bool(false));
    capabilities.insert("tools".to_string(), Value::Object(tools));
    result.insert("capabilities".to_string(), Value::Object(capabilities));
    let mut server_info = serde_json::Map::new();
    server_info.insert("name".to_string(), Value::String("HighHarness".to_string()));
    server_info.insert(
        "version".to_string(),
        Value::String(env!("CARGO_PKG_VERSION").to_string()),
    );
    result.insert("serverInfo".to_string(), Value::Object(server_info));
    json_rpc_result(id, Value::Object(result))
}

/// Handle the MCP `tools/list` method.
fn handle_tools_list(id: Option<Value>, registry: &Registry, root: &Path) -> Value {
    let descriptors = registry.list();
    let mut tools = Vec::new();
    for d in descriptors {
        let mut tool = serde_json::Map::new();
        tool.insert("name".to_string(), Value::String(d.id.clone()));
        tool.insert("description".to_string(), Value::String(d.summary.clone()));

        // Try to load the argument schema; fall back to a generic object schema
        let schema_path = root.join(&d.argument_schema_path);
        let input_schema = match std::fs::read_to_string(&schema_path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| default_input_schema(&d.id)),
            Err(_) => default_input_schema(&d.id),
        };

        // Add capability annotations as a custom field
        let mut caps = serde_json::Map::new();
        caps.insert("read".to_string(), Value::Bool(d.capabilities.read));
        caps.insert("write".to_string(), Value::Bool(d.capabilities.write));
        caps.insert("exec".to_string(), Value::Bool(d.capabilities.exec));
        caps.insert("network".to_string(), Value::Bool(d.capabilities.network));
        caps.insert(
            "destructive".to_string(),
            Value::Bool(d.capabilities.destructive),
        );

        tool.insert("inputSchema".to_string(), input_schema);
        tool.insert("_capabilities".to_string(), Value::Object(caps));

        tools.push(Value::Object(tool));
    }

    let mut result = serde_json::Map::new();
    result.insert("tools".to_string(), Value::Array(tools));
    json_rpc_result(id, Value::Object(result))
}

/// Handle the MCP `tools/call` method.
fn handle_tools_call(
    id: Option<Value>,
    params: &Value,
    registry: &Registry,
    root: &Path,
    run_id: &str,
) -> Value {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

    if name.is_empty() {
        return json_rpc_error(id, -32602, "Missing tool name".to_string());
    }

    let ctx = InvokeCtx {
        run_id: run_id.to_string(),
        agent_id: format!("mcp-client-{}", id::now_iso()),
        tool_call_id: format!("mcp-call-{}", id::now_iso()),
    };

    // Use the arguments as-is (already a Value)
    let args = if arguments.is_null() {
        serde_json::json!({})
    } else {
        arguments
    };

    match registry.invoke_raw(&name, args, &ctx, None, root) {
        Ok(result) => {
            let mut content = Vec::new();
            let text = match &result.content.value {
                Value::String(s) => s.clone(),
                Value::Object(o) => serde_json::to_string_pretty(o).unwrap_or_default(),
                Value::Array(a) => serde_json::to_string_pretty(a).unwrap_or_default(),
                other => other.to_string(),
            };
            let mut entry = serde_json::Map::new();
            entry.insert("type".to_string(), Value::String("text".to_string()));
            entry.insert("text".to_string(), Value::String(text));
            content.push(Value::Object(entry));

            let mut res = serde_json::Map::new();
            res.insert("content".to_string(), Value::Array(content));
            if !result.ok {
                res.insert("isError".to_string(), Value::Bool(true));
            }
            json_rpc_result(id, Value::Object(res))
        }
        Err(e) => {
            let mut content = Vec::new();
            let mut entry = serde_json::Map::new();
            entry.insert("type".to_string(), Value::String("text".to_string()));
            entry.insert("text".to_string(), Value::String(format!("Error: {}", e)));
            content.push(Value::Object(entry));
            let mut res = serde_json::Map::new();
            res.insert("content".to_string(), Value::Array(content));
            res.insert("isError".to_string(), Value::Bool(true));
            json_rpc_result(id, Value::Object(res))
        }
    }
}

/// Return a default JSON Schema for a built-in tool's arguments.
fn default_input_schema(tool_id: &str) -> Value {
    match tool_id {
        "fs.read" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "File path to read"}
            },
            "required": ["path"]
        }),
        "fs.edit" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "File path to edit"},
                "old": {"type": "string", "description": "Text to replace"},
                "new": {"type": "string", "description": "Replacement text"}
            },
            "required": ["path", "old", "new"]
        }),
        "fs.hash" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "File path to hash"}
            },
            "required": ["path"]
        }),
        "shell.exec" => serde_json::json!({
            "type": "object",
            "properties": {
                "command": {"type": "string", "description": "Shell command to execute"}
            },
            "required": ["command"]
        }),
        "git.status" => serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        "git.diff" => serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        "git.blame" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "File path to blame"}
            },
            "required": ["path"]
        }),
        "test.run" => serde_json::json!({
            "type": "object",
            "properties": {
                "filter": {"type": "string", "description": "Test filter pattern"}
            },
            "required": []
        }),
        "lint.run" => serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path to lint"}
            },
            "required": []
        }),
        "web.fetch" => serde_json::json!({
            "type": "object",
            "properties": {
                "url": {"type": "string", "description": "URL to fetch"}
            },
            "required": ["url"]
        }),
        _ => serde_json::json!({
            "type": "object",
            "properties": {},
            "description": format!("Arguments for {}", tool_id)
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_error_has_proper_structure() {
        let err = json_rpc_error(
            Some(Value::Number(1.into())),
            -32601,
            "Method not found".to_string(),
        );
        assert_eq!(err["jsonrpc"], "2.0");
        assert_eq!(err["id"], 1);
        assert_eq!(err["error"]["code"], -32601);
        assert_eq!(err["error"]["message"], "Method not found");
    }

    #[test]
    fn test_json_rpc_result_has_proper_structure() {
        let res = json_rpc_result(
            Some(Value::Number(1.into())),
            serde_json::json!({"ok": true}),
        );
        assert_eq!(res["jsonrpc"], "2.0");
        assert_eq!(res["id"], 1);
        assert_eq!(res["result"]["ok"], true);
    }

    #[test]
    fn test_json_rpc_no_id_notification() {
        let res = json_rpc_result(None, Value::Null);
        assert_eq!(res["jsonrpc"], "2.0");
        assert!(res["id"].is_null());
        assert!(res["result"].is_null());
    }

    #[test]
    fn test_initialize_response_has_required_fields() {
        let resp = handle_initialize(Some(Value::Number(1.into())), &Value::Null);
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(resp["result"]["serverInfo"]["name"], "HighHarness");
        assert_eq!(resp["result"]["serverInfo"]["version"], "0.1.0");
        assert!(resp["result"]["capabilities"]["tools"]["listChanged"] == false);
    }

    #[test]
    fn test_default_input_schema_returns_object() {
        let schema = default_input_schema("fs.read");
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["path"]["type"] == "string");

        let schema2 = default_input_schema("unknown.tool");
        assert_eq!(schema2["type"], "object");
    }
}
