use assert_cmd::Command;

#[test]
fn gates_run_passes_when_command_exits_zero() {
    // stub a gate via a temp config; simplest: phase=highharness gate=syntactic with cargo check
    Command::cargo_bin("HighHarness")
        .unwrap()
        .args([
            "gates",
            "run",
            "--phase",
            "highharness",
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
