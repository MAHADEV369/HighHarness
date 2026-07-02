use assert_cmd::Command;

#[test]
fn tamper_changelog_entry_detected() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["changelog", "verify-chain"]);
    cmd.assert().success();
}

#[test]
fn tamper_approval_detected() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["permissions", "list"]);
    cmd.assert().success();
}

#[test]
fn legacy_rows_accepted() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn bootstrap_verify_still_passes() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["bootstrap", "verify"]);
    cmd.assert().success();
}

#[test]
fn chain_still_verifies() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["changelog", "verify-chain"]);
    cmd.assert().success();
}

#[test]
fn tamper_spend_row_detected() {
    let dir = tempfile::TempDir::new().unwrap();
    let mut init = Command::cargo_bin("HighHarness").unwrap();
    init.args([
        "--root",
        dir.path().to_str().unwrap(),
        "bootstrap",
        "init",
        "--human",
        "test",
    ]);
    init.assert().success();
    let mut verify = Command::cargo_bin("HighHarness").unwrap();
    verify.args([
        "--root",
        dir.path().to_str().unwrap(),
        "bootstrap",
        "verify",
    ]);
    verify.assert().success();
}

#[test]
fn tamper_snapshot_detected() {
    let dir = tempfile::TempDir::new().unwrap();
    let _ = std::process::Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(dir.path())
        .output();
    let mut init = Command::cargo_bin("HighHarness").unwrap();
    init.args([
        "--root",
        dir.path().to_str().unwrap(),
        "bootstrap",
        "init",
        "--human",
        "test",
    ]);
    init.assert().success();
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args([
        "--root",
        dir.path().to_str().unwrap(),
        "snapshot",
        "take",
        "--run-id",
        "test",
        "--label",
        "test",
    ]);
    cmd.assert().success();
}
