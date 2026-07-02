//! Tool registry, per `HARNESS_PRIMITIVES.md` §1.5.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::error::{HxError, HxResult};
use crate::incident;
use crate::redaction::Redactions;
use crate::schema::tool::{ToolDescriptor, ToolMeta, ToolResult};
use crate::store::tool_calls_path;
use crate::tools;

/// Context for a single tool invocation.
#[derive(Debug, Clone, Serialize)]
pub struct InvokeCtx {
    /// Unique identifier for the current run.
    pub run_id: String,
    /// Identifier of the agent making the call.
    pub agent_id: String,
    /// Unique identifier for this tool call.
    pub tool_call_id: String,
}

/// Scope restrictions applied to a tool invocation.
#[derive(Debug, Clone, Serialize)]
pub struct ScopeNarrow {
    /// Allowed file path globs.
    pub paths: Option<Vec<String>>,
    /// Allowed network targets.
    pub network: Option<Vec<String>>,
    /// Allowed environment variables.
    pub env: Option<Vec<String>>,
    /// Maximum number of tool calls before expiry.
    pub ttl_tool_calls: Option<u32>,
}

/// Registry of available tools loaded from disk.
pub struct Registry {
    /// Map of tool ID to tool descriptor.
    pub tools: HashMap<String, ToolDescriptor>,
    /// Optional redaction vault for sensitive content.
    pub redactions: Option<Redactions>,
}

impl Registry {
    /// Load the registry from `.harness/tools/*.toml`.
    pub fn load(root: &Path) -> HxResult<Registry> {
        let redactions = Redactions::load(root).ok();
        let dir = crate::store::tools_dir(root);
        let mut tools = HashMap::new();
        if !dir.exists() {
            return Ok(Registry { tools, redactions });
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let p = entry.path();
            if p.extension().and_then(|x| x.to_str()) != Some("toml") {
                continue;
            }
            let raw = fs::read_to_string(&p)?;
            let d: ToolDescriptor = match toml::from_str(&raw) {
                Ok(d) => d,
                Err(e) => {
                    return Err(HxError::SchemaRejected {
                        artifact: format!("tools/{}", p.display()),

                        saw: e.to_string(),
                    });
                }
            };
            if d.schema_version != 1 {
                return Err(HxError::SchemaRejected {
                    artifact: format!("tools/{}", d.id),

                    saw: format!("schema_version={}", d.schema_version),
                });
            }
            tools.insert(d.id.clone(), d);
        }
        Ok(Registry { tools, redactions })
    }

    /// Look up a tool descriptor by ID.
    pub fn get(&self, id: &str) -> Option<&ToolDescriptor> {
        self.tools.get(id)
    }

    /// List all registered tool descriptors sorted by ID.
    pub fn list(&self) -> Vec<&ToolDescriptor> {
        let mut v: Vec<&ToolDescriptor> = self.tools.values().collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    }

    /// Invoke a tool by id. Does NOT enforce permissions — that gate is
    /// `crate::permissions::check` and is called by the CLI flow. This
    /// function assumes the call has been authorized.
    pub fn invoke_raw(
        &self,

        id: &str,

        args: Value,

        ctx: &InvokeCtx,

        _scope_narrow: Option<ScopeNarrow>,

        root: &Path,
    ) -> HxResult<ToolResult> {
        let _desc = self
            .get(id)
            .ok_or_else(|| HxError::Other(format!("tool not found: {}", id)))?;

        // W6: F1 detection — verify tool_call_id is non-empty and valid
        if ctx.tool_call_id.is_empty() {
            let incident_id = incident::declare(
                root,
                "F1",
                "V2.3",
                &ctx.run_id,
                &ctx.agent_id,
                None,
                None,
                "critical",
                false,
                Vec::new(),
            )?;
            return Err(HxError::Incident(format!(
                "F1 gate-bypass: tool_call_id empty, incident={}",
                incident_id
            )));
        }

        let started = std::time::Instant::now();
        let result = match id {
            "fs.read" => tools::fs_read::run(args.clone(), root)?,
            "fs.hash" => tools::fs_hash::run(args.clone(), root)?,
            "fs.edit" => tools::fs_edit::run(args.clone(), root)?,
            "shell.exec" => tools::shell_exec::run(args.clone(), root)?,
            "git.status" => tools::git_status::run(args.clone(), root)?,
            "git.diff" => tools::git_diff::run(args.clone(), root)?,
            "git.blame" => tools::git_blame::run(args.clone(), root)?,
            "test.run" => tools::test_run::run(args.clone(), root)?,
            "lint.run" => tools::lint_run::run(args.clone(), root)?,
            "web.fetch" => tools::web_fetch::run(args.clone(), root)?,
            _ => {
                return Err(HxError::Other(format!("tool not implemented: {}", id)));
            }
        };
        let duration_ms = started.elapsed().as_millis() as u64;
        // Re-stamp duration in meta.
        let mut result = ToolResult {
            schema_version: 1,

            ok: result.ok,

            content: result.content,
            meta: ToolMeta {
                duration_ms,

                bytes: result.meta.bytes,

                exit_code: result.meta.exit_code,
            },

            redactions: result.redactions,

            approval_id: result.approval_id,

            tool_call_id: ctx.tool_call_id.clone(),
        };
        // Apply redaction vault (W3)
        if let Some(ref r) = self.redactions {
            if let serde_json::Value::String(ref mut s) = result.content.value {
                r.apply(s);
            }
        }

        // Append to tool-calls ledger.
        let path = tool_calls_path(root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        use std::io::Write;
        let mut line = serde_json::json!({
            "schema_version": 1,
            "tool_call_id": ctx.tool_call_id,
            "run_id": ctx.run_id,
            "agent_id": ctx.agent_id,
            "tool": id,
            "args": args,
            "result_summary": match result.content.kind.as_str() {
                "text" => result.content.value.as_str().map(|s| s.chars().take(80).collect::<String>()).unwrap_or_default(),
                _ => format!("{:?}", result.content.value).chars().take(80).collect::<String>(),
            },
            "started_at": crate::id::now_iso(),
            "duration_ms": duration_ms,
        });
        // W5: add self_hash to tool-calls row
        let mut for_hash = line.clone();
        for_hash.as_object_mut().map(|o| o.remove("self_hash"));
        let line_str = serde_json::to_string(&for_hash)?;
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        hasher.update(line_str.as_bytes());
        let self_hash = format!("{:x}", hasher.finalize());
        line.as_object_mut()
            .map(|o| o.insert("self_hash".to_string(), serde_json::json!(self_hash)));
        writeln!(f, "{}", serde_json::to_string(&line)?)?;
        Ok(result)
    }
}
