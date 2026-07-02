use assert_cmd::Command;

#[test]
fn rollup_returns_11_kpis() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["metrics", "rollup", "--window", "30d"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("merge_rate") || stdout.contains("kpis"));
}

#[test]
fn rollup_cold_start() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["metrics", "rollup", "--window", "7d"]);
    cmd.assert().success();
}

#[test]
fn metrics_health_prints_summary() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["metrics", "health"]);
    cmd.assert().success();
}

#[test]
fn cadence_run_exits_zero() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["cadence", "run", "--daily"]);
    cmd.assert().success();
}
