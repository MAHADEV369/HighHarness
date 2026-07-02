use crate::error::HxResult;
use crate::id;
use crate::permissions;
use crate::schema::episode::ToolCall;
use crate::store;
use crate::tools::registry::{InvokeCtx, Registry};
use serde_json::Value;
use std::io::{BufRead, Read, Write};
use std::path::Path;

/// Session state tracked across MCP requests within a single connection.
struct McpSession {
    /// Unique run ID for this MCP session.
    run_id: String,
    /// Whether the episode file has been opened.
    episode_open: bool,
}

/// Run the MCP server over stdio. Reads JSON-RPC 2.0 requests from stdin,
/// dispatches to the tool registry with permission enforcement and episode
/// recording, and writes responses to stdout.
///
/// Permission enforcement:
/// - Loads `.harness/permissions.toml` on each `tools/call`
/// - Denied calls return an MCP error with the blocking rule
///
/// Episode recording:
/// - Opens an episode trace on the first `tools/call`
/// - Records every call (allowed and denied) to the trace
/// - Closes the episode on `shutdown` or EOF
pub fn serve(root: &Path) -> HxResult<i32> {
    let registry = Registry::load(root)?;
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let mut reader = std::io::BufReader::new(stdin.lock());
    let mut writer = std::io::BufWriter::new(stdout.lock());

    let mut line = String::new();
    let mut session: Option<McpSession> = None;

    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
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

        if method == "notifications/initialized" {
            continue;
        }

        let response = match method.as_str() {
            "initialize" => handle_initialize(id_val, &params),
            "ping" => json_rpc_result(id_val, Value::Null),
            "tools/list" => handle_tools_list(id_val, &registry, root),
            "tools/call" => {
                let resp = handle_tools_call(
                    id_val, &params, &registry, root, &mut session,
                );
                resp
            }
            "shutdown" => {
                if let Some(ref sess) = session {
                    if sess.episode_open {
                        let _ = store::episode::close(
                            root,
                            &sess.run_id,
                            "MCP session closed by shutdown request",
                            Vec::new(),
                        );
                    }
                }
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

    if let Some(ref sess) = session {
        if sess.episode_open {
            let _ = store::episode::close(
                root,
                &sess.run_id,
                "MCP session closed on EOF",
                Vec::new(),
            );
        }
    }

    Ok(0)
}

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

fn handle_tools_list(id: Option<Value>, registry: &Registry, root: &Path) -> Value {
    let descriptors = registry.list();
    let mut tools = Vec::new();
    for d in descriptors {
        let mut tool = serde_json::Map::new();
        tool.insert("name".to_string(), Value::String(d.id.clone()));
        tool.insert("description".to_string(), Value::String(d.summary.clone()));

        let schema_path = root.join(&d.argument_schema_path);
        let input_schema = match std::fs::read_to_string(&schema_path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| default_input_schema(&d.id)),
            Err(_) => default_input_schema(&d.id),
        };

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

fn handle_tools_call(
    id: Option<Value>,
    params: &Value,
    registry: &Registry,
    root: &Path,
    session: &mut Option<McpSession>,
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

    let tool_desc = match registry.get(&name) {
        Some(d) => d.clone(),
        None => {
            return json_rpc_error(id, -32604, format!("Tool not found: {}", name));
        }
    };

    let agent_id = format!("mcp-client-{}", id::now_iso());

    if session.is_none() {
        let run_id = format!("mcp-{}", id::now_iso());
        let _ = store::episode::open(root, &run_id, &agent_id, "MCP governance session", "any", "mcp");
        *session = Some(McpSession {
            run_id,
            episode_open: true,
        });
    }

    let sess = session.as_ref().unwrap();
    let run_id = &sess.run_id;
    let tool_call_id = format!("mcp-call-{}", id::now_iso());

    let ctx = InvokeCtx {
        run_id: run_id.to_string(),
        agent_id: agent_id.clone(),
        tool_call_id: tool_call_id.clone(),
    };

    let args = if arguments.is_null() {
        serde_json::json!({})
    } else {
        arguments
    };

    let perm_file = match permissions::load(root) {
        Ok(pf) => pf,
        Err(_) => {
            return json_rpc_error(id, -32603, "Failed to load permissions".to_string());
        }
    };

    match permissions::check(&perm_file, &tool_desc, &args, None, &tool_call_id) {
        Ok(decision) => {
            if decision.decision == "deny" {
                if sess.episode_open {
                    let tc = ToolCall {
                        tool_call_id: tool_call_id.clone(),
                        tool: name.clone(),
                        args: args.clone(),
                        result_summary: format!("DENIED: {}", decision.reason),
                        started_at: id::now_iso(),
                        duration_ms: 0,
                        approval_id: None,
                    };
                    let _ = store::episode::append_tool_call(root, run_id, tc);
                }
                return json_rpc_error(
                    id,
                    -32000,
                    format!("Permission denied: {} (rule: {})", decision.reason, decision.rule_id),
                );
            }
        }
        Err(e) => {
            return json_rpc_error(id, -32603, format!("Permission check error: {}", e));
        }
    }

    match registry.invoke_raw(&name, args.clone(), &ctx, None, root) {
        Ok(result) => {
            if sess.episode_open {
                let tc = ToolCall {
                    tool_call_id: tool_call_id.clone(),
                    tool: name.clone(),
                    args: args.clone(),
                    result_summary: result
                        .content
                        .value
                        .as_str()
                        .map(|s| s.chars().take(120).collect())
                        .unwrap_or_else(|| format!("{:?}", result.content.value).chars().take(120).collect()),
                    started_at: id::now_iso(),
                    duration_ms: result.meta.duration_ms,
                    approval_id: None,
                };
                let _ = store::episode::append_tool_call(root, run_id, tc);
            }

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
            if sess.episode_open {
                let tc = ToolCall {
                    tool_call_id: tool_call_id.clone(),
                    tool: name.clone(),
                    args: args.clone(),
                    result_summary: format!("Error: {}", e).chars().take(120).collect(),
                    started_at: id::now_iso(),
                    duration_ms: 0,
                    approval_id: None,
                };
                let _ = store::episode::append_tool_call(root, run_id, tc);
            }

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

/// Run the MCP server over HTTP. Listens on `host:port` and handles
/// JSON-RPC 2.0 requests via HTTP POST. Each connection gets its own
/// episode session.
pub fn serve_http(root: &Path, host: String, port: u16) -> HxResult<i32> {
    let addr = format!("{}:{}", host, port);
    let listener = std::net::TcpListener::bind(&addr)
        .map_err(|e| crate::error::HxError::Other(format!("bind {addr}: {e}")))?;

    listener.set_nonblocking(true)
        .map_err(|e| crate::error::HxError::Other(format!("set_nonblocking: {e}")))?;

    eprintln!("HighHarness MCP server listening on http://{addr}");

    loop {
        // Accept new connections with a short timeout
        match listener.accept() {
            Ok((stream, peer)) => {
                eprintln!("MCP connection from {peer}");
                stream.set_nonblocking(false).ok();
                if let Err(e) = handle_http_connection(root, stream) {
                    eprintln!("MCP connection error: {e}");
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                eprintln!("MCP accept error: {e}");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
}

/// Handle a single HTTP connection. Reads a JSON-RPC request from the
/// HTTP POST body, processes it, and writes the response.
fn handle_http_connection(root: &Path, mut stream: std::net::TcpStream) -> HxResult<()> {
    let registry = Registry::load(root)?;
    let mut buffer = [0u8; 65536];
    let n = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    // Parse Content-Length
    let body_start = request.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let body = &request[body_start..].trim();

    if body.is_empty() {
        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Length: 15\r\n\r\nEmpty request\n";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    let req: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            let err = json_rpc_error(None, -32700, format!("Parse error: {e}"));
            let out = serde_json::to_string(&err)?;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                out.len(), out
            );
            stream.write_all(resp.as_bytes())?;
            return Ok(());
        }
    };

    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let id_val = req.get("id").cloned();
    let params = req.get("params").cloned().unwrap_or(Value::Null);

    let mut session = McpSession {
        run_id: format!("mcp-http-{}", id::now_iso()),
        episode_open: false,
    };

    let response = match method {
        "initialize" => handle_initialize(id_val, &params),
        "ping" => json_rpc_result(id_val, Value::Null),
        "tools/list" => handle_tools_list(id_val, &registry, root),
        "tools/call" => {
            if !session.episode_open {
                let agent_id = format!("mcp-http-client-{}", id::now_iso());
                let _ = store::episode::open(
                    root, &session.run_id, &agent_id,
                    "MCP HTTP governance session", "any", "mcp",
                );
                session.episode_open = true;
            }
            handle_http_tools_call(id_val, &params, &registry, root, &session.run_id)
        }
        "shutdown" => {
            close_http_session(root, &session);
            json_rpc_result(id_val, Value::Null)
        }
        _ => json_rpc_error(id_val, -32601, format!("Method not found: {method}")),
    };

    let out = serde_json::to_string(&response)?;
    let http_resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        out.len(), out
    );
    stream.write_all(http_resp.as_bytes())?;

    if method == "shutdown" {
        close_http_session(root, &session);
    }

    Ok(())
}

/// Handle a tools/call request over HTTP with permission enforcement.
fn handle_http_tools_call(
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

    let tool_desc = match registry.get(&name) {
        Some(d) => d.clone(),
        None => {
            return json_rpc_error(id, -32604, format!("Tool not found: {name}"));
        }
    };

    let agent_id = format!("mcp-http-client-{}", id::now_iso());
    let tool_call_id = format!("mcp-http-call-{}", id::now_iso());

    let ctx = InvokeCtx {
        run_id: run_id.to_string(),
        agent_id: agent_id.clone(),
        tool_call_id: tool_call_id.clone(),
    };

    let args = if arguments.is_null() {
        serde_json::json!({})
    } else {
        arguments
    };

    let perm_file = match permissions::load(root) {
        Ok(pf) => pf,
        Err(_) => return json_rpc_error(id, -32603, "Failed to load permissions".to_string()),
    };

    match permissions::check(&perm_file, &tool_desc, &args, None, &tool_call_id) {
        Ok(decision) => {
            if decision.decision == "deny" {
                let tc = ToolCall {
                    tool_call_id: tool_call_id.clone(),
                    tool: name.clone(),
                    args: args.clone(),
                    result_summary: format!("DENIED: {}", decision.reason),
                    started_at: id::now_iso(),
                    duration_ms: 0,
                    approval_id: None,
                };
                let _ = store::episode::append_tool_call(root, run_id, tc);
                return json_rpc_error(
                    id,
                    -32000,
                    format!("Permission denied: {} (rule: {})", decision.reason, decision.rule_id),
                );
            }
        }
        Err(e) => return json_rpc_error(id, -32603, format!("Permission check error: {e}")),
    }

    match registry.invoke_raw(&name, args.clone(), &ctx, None, root) {
        Ok(result) => {
            let tc = ToolCall {
                tool_call_id: tool_call_id.clone(),
                tool: name.clone(),
                args: args.clone(),
                result_summary: result.content.value.as_str()
                    .map(|s| s.chars().take(120).collect())
                    .unwrap_or_else(|| format!("{:?}", result.content.value).chars().take(120).collect()),
                started_at: id::now_iso(),
                duration_ms: result.meta.duration_ms,
                approval_id: None,
            };
            let _ = store::episode::append_tool_call(root, run_id, tc);

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
            let tc = ToolCall {
                tool_call_id: tool_call_id.clone(),
                tool: name.clone(),
                args: args.clone(),
                result_summary: format!("Error: {e}").chars().take(120).collect(),
                started_at: id::now_iso(),
                duration_ms: 0,
                approval_id: None,
            };
            let _ = store::episode::append_tool_call(root, run_id, tc);

            let mut content = Vec::new();
            let mut entry = serde_json::Map::new();
            entry.insert("type".to_string(), Value::String("text".to_string()));
            entry.insert("text".to_string(), Value::String(format!("Error: {e}")));
            content.push(Value::Object(entry));
            let mut res = serde_json::Map::new();
            res.insert("content".to_string(), Value::Array(content));
            res.insert("isError".to_string(), Value::Bool(true));
            json_rpc_result(id, Value::Object(res))
        }
    }
}

/// Close an HTTP MCP session episode.
fn close_http_session(root: &Path, session: &McpSession) {
    if session.episode_open {
        let _ = store::episode::close(
            root,
            &session.run_id,
            "MCP HTTP session closed",
            Vec::new(),
        );
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
