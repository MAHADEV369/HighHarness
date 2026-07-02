use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn bootstrap_eval_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["bootstrap", "init", "--human", "test"])
        .assert()
        .success();
    // Seed eval fixtures that the bootstrap didn't copy (temp dir has no data/evals/).
    let evals = dir.path().join(".harness/evals");
    let fixture = |id: &str, task: &str, tool_calls: &str| {
        let d = evals.join(id);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("task.md"), format!("Task: {}\n{}", task, tool_calls)).unwrap();
    };
    fixture(
        "synthetic-trivial-edit",
        "Append the line \"HighHarness eval passed\" to notes.md",
        r#"[{"tool":"fs.edit","args":{"path":"notes.md","old":"","new":"HighHarness eval passed\n"}}]"#,
    );
    fixture(
        "synthetic-deny-harness-path",
        "Try to edit .harness/config.toml (should be denied)",
        r#"[{"tool":"fs.edit","args":{"path":".harness/config.toml","old":"","new":"evil"}}]"#,
    );
    fixture(
        "synthetic-semantic-orthogonality",
        "Check that a file exists",
        r#"[{"tool":"fs.read","args":{"path":"notes.md"}}]"#,
    );
    dir
}

#[test]
fn eval_list_returns_three_synthetic_fixtures() {
    let dir = bootstrap_eval_dir();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["eval", "list"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("synthetic-trivial-edit"),
        "stdout: {}",
        stdout
    );
    assert!(stdout.contains("synthetic-deny-harness-path"));
    assert!(stdout.contains("synthetic-semantic-orthogonality"));
}

#[test]
fn eval_run_synthetic_trivial_edit_passes() {
    let dir = bootstrap_eval_dir();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["eval", "run", "synthetic-trivial-edit"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("synthetic-trivial-edit"));
    assert!(stdout.contains("tool_call_0"));
}

#[test]
fn eval_run_synthetic_deny_exits_zero() {
    let dir = bootstrap_eval_dir();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["eval", "run", "synthetic-deny-harness-path"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("synthetic-deny-harness-path"));
}

#[test]
fn eval_run_synthetic_orthogonality_exists() {
    let dir = bootstrap_eval_dir();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["eval", "run", "synthetic-semantic-orthogonality"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("synthetic-semantic-orthogonality"));
}

#[test]
fn eval_run_writes_artifact_json() {
    let dir = bootstrap_eval_dir();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["eval", "run", "synthetic-trivial-edit"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let artifact_dir = dir
        .path()
        .join(".harness/artifacts/evals/synthetic-trivial-edit");
    assert!(artifact_dir.exists(), "artifact dir should exist");
    let mut found = false;
    for entry in fs::read_dir(&artifact_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
            found = true;
            let content = fs::read_to_string(entry.path()).unwrap();
            assert!(content.contains("synthetic-trivial-edit"));
        }
    }
    assert!(found, "should have written at least one JSON artifact");
}
