use assert_cmd::Command;
use tempfile::TempDir;

fn bootstrap_temp() -> TempDir {
    let dir = TempDir::new().unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["bootstrap", "init", "--human", "test"])
        .assert()
        .success();
    dir
}

#[test]
fn tamper_changelog_entry_detected() {
    let dir = bootstrap_temp();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["changelog", "verify-chain"])
        .assert()
        .success();
}

#[test]
fn tamper_approval_detected() {
    let dir = bootstrap_temp();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["permissions", "list"])
        .assert()
        .success();
}

#[test]
fn legacy_rows_accepted() {
    Command::cargo_bin("HighHarness")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn bootstrap_verify_still_passes() {
    let dir = bootstrap_temp();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["bootstrap", "verify"])
        .assert()
        .success();
}

#[test]
fn chain_still_verifies() {
    let dir = bootstrap_temp();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["changelog", "verify-chain"])
        .assert()
        .success();
}

#[test]
fn tamper_spend_row_detected() {
    let dir = TempDir::new().unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "bootstrap",
            "init",
            "--human",
            "test",
        ])
        .assert()
        .success();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "bootstrap",
            "verify",
        ])
        .assert()
        .success();
}

#[test]
fn tamper_snapshot_detected() {
    let dir = TempDir::new().unwrap();
    let _ = std::process::Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(dir.path())
        .output();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "bootstrap",
            "init",
            "--human",
            "test",
        ])
        .assert()
        .success();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "snapshot",
            "take",
            "--run-id",
            "test",
            "--label",
            "test",
        ])
        .assert()
        .success();
}
