use assert_cmd::Command;

#[test]
fn tools_invoke_accepts_inline_json_args() {
    // fs.read on Cargo.toml — args passed inline, not as a file path.
    Command::cargo_bin("HighHarness")
        .unwrap()
        .arg("tools")
        .arg("invoke")
        .arg("--tool")
        .arg("fs.read")
        .arg("--args")
        .arg(r#"{"path":"Cargo.toml"}"#)
        .assert()
        .success();
}

#[test]
fn tools_invoke_accepts_file_path_args() {
    // Write args to a temp file; pass the path.
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, r#"{{"path":"Cargo.toml"}}"#).unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
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
    use assert_cmd::Command;

    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args([
        "tools",
        "invoke",
        "--tool",
        "fs.edit",
        "--args",
        r#"{"path":".harness/x","old":"","new":""}"#,
    ]);
    let output = cmd.output().unwrap();

    // Exit code 3 per HARNESS_PRIMITIVES.md §2.4 (deny) and README install sample.
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
