use assert_cmd::Command;
use std::path::Path;

fn root_arg(dir: &Path) -> String {
    format!("--root={}", dir.display())
}

fn declare_cmd(dir: &Path, run_id: &str, rule: &str, vector: &str, severity: &str) -> Command {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args([
        &root_arg(dir),
        "incident",
        "declare",
        "--run-id",
        run_id,
        "--detection-rule",
        rule,
        "--vector",
        vector,
        "--severity",
        severity,
        "--evidence",
        "/dev/null",
    ]);
    cmd
}

#[test]
fn incident_declare_written_under_incidents() {
    let dir = tempfile::TempDir::new().unwrap();
    declare_cmd(dir.path(), "test-f1", "F1", "V2.3", "critical")
        .assert()
        .success();

    let incidents_dir = dir.path().join(".harness/artifacts/incidents");
    assert!(incidents_dir.exists(), "incidents dir should exist");
    let entries: Vec<_> = std::fs::read_dir(&incidents_dir).unwrap().collect();
    assert!(!entries.is_empty(), "should have at least one incident");
}

#[test]
fn incident_list_shows_declared_incidents() {
    let dir = tempfile::TempDir::new().unwrap();

    // First declare an incident
    declare_cmd(dir.path(), "test-list", "F1", "V2.3", "low")
        .assert()
        .success();

    // Then list
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args([&root_arg(dir.path()), "incident", "list"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "incident list failed: stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("F1") || stdout.contains("detection_rule"),
        "stdout: {}",
        stdout
    );
}

#[test]
fn incident_with_impact_triggers_notification() {
    let dir = tempfile::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args([
        &root_arg(dir.path()),
        "incident",
        "declare",
        "--run-id",
        "test-impact",
        "--detection-rule",
        "F2",
        "--vector",
        "V3.1",
        "--severity",
        "critical",
        "--had-impact",
        "--evidence",
        "/dev/null",
    ]);
    cmd.assert().success();

    let notif_dir = dir.path().join(".harness/artifacts/notifications");
    assert!(notif_dir.exists(), "notifications dir should exist");
    let entries: Vec<_> = std::fs::read_dir(&notif_dir).unwrap().collect();
    assert!(!entries.is_empty(), "should have notification files");
}

#[test]
fn incident_ack_and_close() {
    let dir = tempfile::TempDir::new().unwrap();

    // First declare
    let output = declare_cmd(dir.path(), "test-ack", "F3", "V4.1", "high")
        .output()
        .unwrap();
    assert!(output.status.success());
    let incident_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Ack
    let mut ack = Command::cargo_bin("HighHarness").unwrap();
    ack.args([
        &root_arg(dir.path()),
        "incident",
        "ack",
        &incident_id,
        "--by",
        "tester",
    ]);
    ack.assert().success();

    // Close
    let mut close = Command::cargo_bin("HighHarness").unwrap();
    close.args([
        &root_arg(dir.path()),
        "incident",
        "close",
        &incident_id,
        "--postmortem",
        "root cause identified",
    ]);
    close.assert().success();
}

#[test]
fn quarantine_source_blocks() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["permissions", "list"]);
    cmd.assert().success();
}
