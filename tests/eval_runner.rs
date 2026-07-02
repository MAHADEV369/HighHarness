use assert_cmd::Command;

#[test]
fn eval_list_returns_three_synthetic_fixtures() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["eval", "list"]);
    cmd.assert().success();
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("synthetic-trivial-edit"));
    assert!(stdout.contains("synthetic-deny-harness-path"));
    assert!(stdout.contains("synthetic-semantic-orthogonality"));
}

#[test]
fn eval_run_synthetic_trivial_edit_passes() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["eval", "run", "synthetic-trivial-edit"]);
    cmd.assert().success();
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output JSON containing the eval_id
    assert!(stdout.contains("synthetic-trivial-edit"));
    assert!(stdout.contains("tool_call_0"));
}

#[test]
fn eval_run_synthetic_deny_exits_zero() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["eval", "run", "synthetic-deny-harness-path"]);
    cmd.assert().success();
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("synthetic-deny-harness-path"));
}

#[test]
fn eval_run_synthetic_orthogonality_exists() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["eval", "run", "synthetic-semantic-orthogonality"]);
    cmd.assert().success();
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("synthetic-semantic-orthogonality"));
}

#[test]
fn eval_run_writes_artifact_json() {
    use std::path::Path;
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["eval", "run", "synthetic-trivial-edit"]);
    cmd.assert().success();
    let artifact_dir = Path::new(".harness/artifacts/evals/synthetic-trivial-edit");
    assert!(artifact_dir.exists(), "artifact dir should exist");
    let mut found = false;
    if artifact_dir.exists() {
        for entry in std::fs::read_dir(artifact_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                found = true;
                let content = std::fs::read_to_string(entry.path()).unwrap();
                assert!(content.contains("synthetic-trivial-edit"));
            }
        }
    }
    assert!(found, "should have written at least one JSON artifact");
}
