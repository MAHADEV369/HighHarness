use assert_cmd::Command;

#[test]
fn models_list_returns_configured_models() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["models", "list"]);
    cmd.assert().success();
}

#[test]
fn models_complete_returns_events() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args([
        "models",
        "complete",
        "--model",
        "llama-3.3-70b-local",
        "--messages",
        r#"[{"role":"user","content":"hi"}]"#,
    ]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("error") || stdout.contains("kind"),
        "stdout: {}",
        stdout
    );
}

#[test]
fn models_complete_accepts_messages_file() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        tmp,
        r#"[{{"role":"user","content":"test"}}]"#
    )
    .unwrap();
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args([
        "models",
        "complete",
        "--model",
        "llama-3.3-70b-local",
        "--messages-file",
        tmp.path().to_str().unwrap(),
    ]);
    cmd.assert().success();
}
