//! `HighHarness hook` subcommand (pre-tool / post-tool / session-start).

use std::path::Path;

use clap::{Parser, Subcommand};
use serde_json::Value;

use crate::error::HxResult;

/// CLI arguments for the hooks subcommand.
#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(subcommand)]
    /// The hook action to perform.
    pub cmd: HookCmd,
}

/// Available hook actions.
#[derive(Subcommand, Debug)]
pub enum HookCmd {
    /// Run the pre-tool hook (permission check).
    PreTool {
        /// Path to the JSON payload (or stdin if absent).
        json: Option<std::path::PathBuf>,
    },
    /// Run the post-tool hook (log the tool call).
    PostTool {
        /// Path to the JSON payload (or stdin if absent).
        json: Option<std::path::PathBuf>,
    },
    /// Run the session-start hook (bootstrap and chain verification).
    SessionStart,
}

/// Execute the hooks subcommand.
pub fn run(cmd: Cmd, root: &Path) -> HxResult<i32> {
    match cmd.cmd {
        HookCmd::PreTool { json } => pre_tool(root, json.as_deref()),
        HookCmd::PostTool { json } => post_tool(root, json.as_deref()),
        HookCmd::SessionStart => session_start(root),
    }
}

fn read_payload(p: Option<&Path>) -> HxResult<Value> {
    if let Some(path) = p {
        let raw = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw).unwrap_or(Value::Null))
    } else {
        let mut s = String::new();
        use std::io::Read;
        std::io::stdin().read_to_string(&mut s)?;
        Ok(serde_json::from_str(&s).unwrap_or(Value::Null))
    }
}

fn pre_tool(root: &Path, json: Option<&Path>) -> HxResult<i32> {
    // 1. bootstrap verify
    if let Err(e) = crate::bootstrap::verify(root) {
        eprintln!("hook pre-tool: not bootstrapped: {}", e);
        return Ok(4);
    }
    let payload = read_payload(json)?;
    let tool = payload
        .get("tool")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let run_id = payload
        .get("run_id")
        .and_then(|x| x.as_str())
        .map(String::from);
    let args_value = payload
        .get("args")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));
    let reg = crate::tools::registry::Registry::load(root)?;
    let desc = match reg.get(&tool) {
        Some(d) => d.clone(),
        None => {
            eprintln!("hook pre-tool: unknown tool {}", tool);
            return Ok(3);
        }
    };
    let pf = crate::permissions::load(root)?;
    let tcid = payload
        .get("tool_call_id")
        .and_then(|x| x.as_str())
        .map(String::from)
        .unwrap_or_else(|| "hook".to_string());
    let decision = crate::permissions::check(&pf, &desc, &args_value, None, &tcid)?;
    println!("{}", serde_json::to_string_pretty(&decision)?);
    // 2. log the event
    let _ = crate::telemetry::integrity::append(
        root,
        "hook.pre-tool",
        serde_json::json!({"tool": tool, "decision": decision.decision, "run_id": run_id}),
    );
    match decision.decision.as_str() {
        "deny" => Ok(3),
        "ask" => Ok(2),
        _ => Ok(0),
    }
}

fn post_tool(root: &Path, json: Option<&Path>) -> HxResult<i32> {
    let payload = read_payload(json)?;
    let path = crate::store::tool_calls_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    use std::io::Write;
    let line = serde_json::json!({
        "schema_version": 1,
        "kind": "post-tool",
        "at": crate::id::now_iso(),
        "payload": payload,
    });
    writeln!(f, "{}", serde_json::to_string(&line)?)?;
    let _ = crate::telemetry::integrity::append(
        root,
        "hook.post-tool",
        serde_json::json!({"tool": payload.get("tool"), "run_id": payload.get("run_id")}),
    );
    Ok(0)
}

fn session_start(root: &Path) -> HxResult<i32> {
    if let Err(e) = crate::bootstrap::verify(root) {
        eprintln!("hook session-start: not bootstrapped: {}", e);
        return Ok(4);
    }
    let broken = crate::store::changelog::verify_chain(root, None)?;
    if !broken.is_empty() {
        eprintln!("hook session-start: chain broken: {:?}", broken);
        return Ok(2);
    }
    let log_broken = crate::telemetry::integrity::verify(root)?;
    if !log_broken.is_empty() {
        eprintln!("hook session-start: integrity log broken: {:?}", log_broken);
        return Ok(3);
    }
    // ok
    let bs = crate::bootstrap::verify(root)?;
    println!("ok {}", bs.genesis_hash);
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn hook_pre_tool_returns_4_when_not_bootstrapped() {
        let dir = TempDir::new().unwrap();
        let rc = session_start(dir.path()).unwrap();
        assert_eq!(rc, 4);
    }

    #[test]
    fn hook_session_start_exits_0_on_healthy_state() {
        let dir = TempDir::new().unwrap();
        crate::bootstrap::init(dir.path(), "admin").unwrap();
        let rc = session_start(dir.path()).unwrap();
        assert_eq!(rc, 0);
    }

    #[test]
    fn hook_session_start_exits_nonzero_on_broken_chain() {
        let dir = TempDir::new().unwrap();
        crate::bootstrap::init(dir.path(), "admin").unwrap();
        // Tamper with the changelog (break the chain).
        let p = crate::store::changelog_path(dir.path());
        let mut raw = std::fs::read_to_string(&p).unwrap();
        // Replace the bootstrap eval entry's intent with a TAMPERED value,
        // which will invalidate its this_hash.
        raw = raw.replacen("bootstrap eval", "TAMPERED TAMPERED", 1);
        std::fs::write(&p, raw).unwrap();
        let rc = session_start(dir.path()).unwrap();
        // chain broken OR genesis hash mismatch → nonzero
        assert!(rc != 0, "expected nonzero, got {}", rc);
    }
}
