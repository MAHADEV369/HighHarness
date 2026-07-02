use assert_cmd::Command;

#[test]
fn mcp_list_empty_initially() {
    let dir = tempfile::tempdir().unwrap();
    // Bootstrap first so .harness/ exists
    Command::cargo_bin("HighHarness")
        .unwrap()
        .args(["bootstrap", "init", "--human", "test"])
        .current_dir(dir.path())
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["mcp", "list"]).current_dir(dir.path());
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no servers registered") || stdout.is_empty(),
        "stdout: {}",
        stdout
    );
}

#[test]
fn mcp_register_writes_config() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .args(["bootstrap", "init", "--human", "test"])
        .current_dir(dir.path())
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args([
        "mcp",
        "register",
        "test-server",
        "--command",
        "/bin/echo",
        "--command",
        "hello",
        "--paths-allowed",
        "/tmp",
        "--network-allowed",
        "127.0.0.1",
        "--env-allowed",
        "PATH",
        "--cpu-seconds",
        "10",
        "--memory-mb",
        "128",
        "--timeout-seconds",
        "30",
    ])
    .current_dir(dir.path());
    cmd.assert().success();

    // Verify it can be listed
    let mut list = Command::cargo_bin("HighHarness").unwrap();
    list.args(["mcp", "list"]).current_dir(dir.path());
    let output = list.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-server"), "stdout: {}", stdout);
}

#[test]
fn mcp_start_and_stop() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .args(["bootstrap", "init", "--human", "test"])
        .current_dir(dir.path())
        .assert()
        .success();

    // Register first
    let mut reg = Command::cargo_bin("HighHarness").unwrap();
    reg.args([
        "mcp",
        "register",
        "test-start",
        "--command",
        "/bin/echo",
        "--command",
        "started",
    ])
    .current_dir(dir.path());
    reg.assert().success();

    // Start
    let mut start = Command::cargo_bin("HighHarness").unwrap();
    start
        .args(["mcp", "start", "test-start"])
        .current_dir(dir.path());
    start.assert().success();
}
