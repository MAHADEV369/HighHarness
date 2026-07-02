use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn gates_run_passes_when_command_exits_zero() {
    let dir = TempDir::new().unwrap();
    // Bootstrap so .harness/config.toml exists.
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["bootstrap", "init", "--human", "test"])
        .assert()
        .success();
    // Append a gate config for a phase that doesn't exist yet.
    let cfg = dir.path().join(".harness/config.toml");
    let mut raw = std::fs::read_to_string(&cfg).unwrap();
    raw.push_str("\n[gates.testphase]\nsyntactic = \"true\"\n");
    std::fs::write(&cfg, raw).unwrap();
    // Run the gate.
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args([
            "gates",
            "run",
            "--phase",
            "testphase",
            "--gate",
            "syntactic",
            "--run-id",
            "gates-test",
            "--changes",
            r#"{"files":[]}"#,
        ])
        .assert()
        .success();
}

#[test]
fn gates_flat_form_no_longer_accepted() {
    // Old shape: gates --phase ... (no run subcommand). Must exit non-zero (clap rejects).
    Command::cargo_bin("HighHarness")
        .unwrap()
        .args(["gates", "--phase", "x"])
        .assert()
        .failure();
}
