use assert_cmd::Command;
use tempfile::TempDir;

fn bootstrap_tool_invoke_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("test.txt"), "hello world").unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["bootstrap", "init", "--human", "test"])
        .assert()
        .success();
    dir
}

#[test]
fn tools_invoke_accepts_inline_json_args() {
    let dir = bootstrap_tool_invoke_dir();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .arg("tools")
        .arg("invoke")
        .arg("--tool")
        .arg("fs.read")
        .arg("--args")
        .arg(r#"{"path":"test.txt"}"#)
        .assert()
        .success();
}

#[test]
fn tools_invoke_accepts_file_path_args() {
    let dir = bootstrap_tool_invoke_dir();
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, r#"{{"path":"test.txt"}}"#).unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .arg("tools")
        .arg("invoke")
        .arg("--tool")
        .arg("fs.read")
        .arg("--args")
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn tools_invoke_deny_returns_exit_3_on_harness_path_via_inline_json() {
    let dir = bootstrap_tool_invoke_dir();
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.current_dir(&dir).args([
        "tools",
        "invoke",
        "--tool",
        "fs.edit",
        "--args",
        r#"{"path":".harness/x","old":"","new":""}"#,
    ]);
    let output = cmd.output().unwrap();

    assert_eq!(
        output.status.code(),
        Some(3),
        "deny must exit 3, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);
    assert!(
        combined.contains("R-DENY-HARNESS") || combined.contains("deny"),
        "Expected R-DENY-HARNESS rule_id in output, got: {}",
        combined
    );
}
