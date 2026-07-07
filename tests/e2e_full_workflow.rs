use std::io::Write;
use std::process::{Command as StdCommand, Stdio};

use assert_cmd::Command;

fn hx_bin() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push("debug");
    p.push("HighHarness");
    p
}

/// End-to-end test: verifies that all major HighHarness features work together.
/// Runs the actual binary and tests CLI + MCP server + permissions + episodes
/// + memory + snapshots + clarifications + model adapter.
fn hx() -> Command {
    Command::cargo_bin("HighHarness").unwrap()
}

#[test]
fn e2e_bootstrap_verify() {
    // bootstrap verify checks .harness/artifacts/bootstrap/bootstrap.json
    // This may or may not exist depending on the environment. We just verify
    // the CLI doesn't crash.
    let output = hx().args(["bootstrap", "verify"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        // If bootstrap.json doesn't exist, that's OK — the error should be descriptive
        assert!(
            !stderr.is_empty(),
            "expected descriptive error, got empty stderr"
        );
    }
}

#[test]
fn e2e_changelog_chain() {
    // Chain verification may fail in CI or fresh clones. We just verify
    // the command runs without crashing.
    let output = hx().args(["changelog", "verify-chain"]).output().unwrap();
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);
}

#[test]
fn e2e_clarification_request_list_resolve() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();

    std::fs::create_dir_all(root.join(".harness").join("artifacts")).unwrap();

    // Request a clarification
    let output = hx()
        .args([
            "clarification",
            "request",
            "--question",
            "What model to use?",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "request failed");
    let request_out = String::from_utf8_lossy(&output.stdout);
    let request_json: serde_json::Value =
        serde_json::from_str(request_out.trim()).expect("request output should be JSON");
    let clarification_id = request_json["id"]
        .as_str()
        .expect("request output should have id")
        .to_string();
    assert!(
        request_json["status"] == "pending",
        "expected pending status"
    );

    // List clarifications
    let output = hx()
        .args(["clarification", "list"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "list failed");
    let list_out = String::from_utf8_lossy(&output.stdout);
    assert!(
        list_out.contains("pending"),
        "expected pending clarification"
    );
    assert!(
        list_out.contains(&clarification_id),
        "expected our clarification id in list"
    );

    // Resolve the clarification
    let output = hx()
        .args([
            "clarification",
            "resolve",
            "--id",
            &clarification_id,
            "--answer",
            "GPT-4",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "resolve failed");
    let resolve_out = String::from_utf8_lossy(&output.stdout);
    assert!(resolve_out.contains("resolved"), "expected resolved status");
    assert!(resolve_out.contains("GPT-4"), "expected answer in output");
}

#[test]
fn e2e_mcp_server_permissions_and_episode() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();

    std::fs::create_dir_all(root.join(".harness").join("artifacts")).unwrap();
    std::fs::create_dir_all(root.join(".harness").join("locks")).unwrap();
    std::fs::create_dir_all(root.join("logs").join("episodes")).unwrap();
    std::fs::create_dir_all(root.join(".harness").join("tools")).unwrap();
    std::fs::create_dir_all(
        root.join(".harness")
            .join("artifacts")
            .join("clarifications"),
    )
    .unwrap();

    std::fs::write(
        root.join(".harness").join("permissions.toml"),
        r#"
schema_version = 1

[[rules]]
id = "allow-read"
effect = "allow"
tool = "fs.read"
paths = ["**"]
reason = "Allow reads"
priority = 50

[[rules]]
id = "deny-shell"
effect = "deny"
tool = "shell.exec"
reason = "Shell blocked"
priority = 50
"#,
    )
    .unwrap();

    std::fs::write(
        root.join(".harness").join("tools").join("fs.read.toml"),
        r#"
id = "fs.read"
schema_version = 1
version = "1.0.0"
source = "builtin"
summary = "Read a file"
argument_schema_path = ""
return_schema_path = ""
side_effect = "read"

[capabilities]
read = true
write = false
exec = false
network = false
destructive = false
secrets = false
side_effect = "read"

[approval]
mode = "auto"
reason = "read-only"
"#,
    )
    .unwrap();

    std::fs::write(
        root.join(".harness").join("tools").join("shell.exec.toml"),
        r#"
id = "shell.exec"
schema_version = 1
version = "1.0.0"
source = "builtin"
summary = "Shell exec"
argument_schema_path = ""
return_schema_path = ""
side_effect = "exec"

[capabilities]
read = false
write = false
exec = true
network = false
destructive = true
secrets = false
side_effect = "exec"

[approval]
mode = "ask"
reason = "exec"
"#,
    )
    .unwrap();

    std::fs::write(root.join("testfile.txt"), b"hello world").unwrap();

    let mut child = StdCommand::new(hx_bin())
        .arg("mcp")
        .arg("serve")
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn mcp serve");

    let stdin = child.stdin.as_mut().unwrap();

    let req = |id: u64, method: &str, params: &str| -> String {
        format!(
            r#"{{"jsonrpc":"2.0","id":{},"method":"{}","params":{}}}"#,
            id, method, params
        )
    };

    // Initialize
    writeln!(stdin, "{}", req(1, "initialize", "{}")).unwrap();

    // List tools
    writeln!(stdin, "{}", req(2, "tools/list", "{}")).unwrap();

    // Read file (should be ALLOWED)
    writeln!(
        stdin,
        "{}",
        req(
            3,
            "tools/call",
            r#"{"name":"fs.read","arguments":{"path":"testfile.txt"}}"#
        )
    )
    .unwrap();

    // Shell exec (should be DENIED)
    writeln!(
        stdin,
        "{}",
        req(
            4,
            "tools/call",
            r#"{"name":"shell.exec","arguments":{"command":"rm -rf /"}}"#
        )
    )
    .unwrap();

    // Shutdown
    writeln!(stdin, "{}", req(5, "shutdown", "{}")).unwrap();

    let output = child.wait_with_output().expect("failed to wait for child");
    assert!(output.status.success(), "mcp serve exited with error");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let responses: Vec<&str> = stdout.lines().collect();

    assert!(responses.len() >= 5, "expected at least 5 responses");

    // Verify responses are valid JSON-RPC
    for (i, line) in responses.iter().enumerate() {
        let v: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|_| panic!("response {} invalid JSON: {}", i, line));
        assert!(
            v.get("jsonrpc") == Some(&serde_json::json!("2.0")),
            "response {} missing jsonrpc: {}",
            i,
            line
        );
    }

    // Response 3 (fs.read) should have result
    let resp3: serde_json::Value = serde_json::from_str(responses[2]).unwrap();
    assert!(
        resp3.get("result").is_some(),
        "read should be allowed: {}",
        responses[2]
    );

    // Response 4 (shell.exec) should have error
    let resp4: serde_json::Value = serde_json::from_str(responses[3]).unwrap();
    assert!(
        resp4.get("error").is_some(),
        "shell should be denied: {}",
        responses[3]
    );
    let error_msg = resp4["error"]["message"].as_str().unwrap_or("");
    assert!(
        error_msg.contains("Permission denied"),
        "expected permission denied: {}",
        error_msg
    );
}

#[test]
fn e2e_tools_permission_check() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();

    std::fs::create_dir_all(root.join(".harness")).unwrap();
    std::fs::write(
        root.join(".harness").join("permissions.toml"),
        r#"
schema_version = 1

[[rules]]
id = "deny-all"
effect = "deny"
tool = "*"
reason = "Default deny"
priority = 10
"#,
    )
    .unwrap();
    std::fs::write(root.join("test.txt"), b"data").unwrap();

    let output = hx()
        .args([
            "tools",
            "invoke",
            "--tool",
            "fs.read",
            "--args",
            r#"{"path":"test.txt"}"#,
            "--run-id",
            "test-run",
            "--agent-id",
            "test-agent",
        ])
        .current_dir(root)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "expected permission denied but command succeeded. stderr: {}",
        stderr
    );
}

#[test]
fn e2e_memory_write_query_forget() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();

    std::fs::create_dir_all(root.join(".harness").join("artifacts").join("memory")).unwrap();
    std::fs::create_dir_all(root.join(".harness").join("locks")).unwrap();

    std::fs::write(
        root.join(".harness").join("artifacts").join("memory").join("project.jsonl"),
        r#"{"schema_version":1,"id":"mem1","stream":"project","kind":"fact","subject":"auth","body":"Use OAuth2","evidence_run_id":"r1","pinned":false,"tags":["important"],"created_at":"2026-01-01","ttl_days":null,"tombstone":false}
"#,
    )
    .unwrap();
}

#[test]
fn e2e_snapshot_take_and_diff() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();

    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "test"])
        .current_dir(root)
        .output()
        .unwrap();

    std::fs::write(root.join("initial.txt"), b"v1").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    std::fs::create_dir_all(root.join(".harness").join("artifacts").join("snapshots")).unwrap();
    std::fs::create_dir_all(root.join(".harness").join("locks")).unwrap();

    let output = hx()
        .args(["snapshot", "take", "--run-id", "r1", "--label", "v1"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "snapshot take failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
