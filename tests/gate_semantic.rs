//! Semantic gate tests, per buildedit.md Area B.

use highharness::gates;
use serde_json::json;
use tempfile::TempDir;

#[test]
fn semantic_gate_passes_on_all_met_with_orthogonal_evidence() {
    let dir = TempDir::new().unwrap();
    let run_id = "test-semantic-orthogonal";
    let work = dir
        .path()
        .join(".harness/artifacts/episodes-work")
        .join(run_id);
    std::fs::create_dir_all(&work).unwrap();
    // Functional log content MUST NOT overlap with any semantic evidence line.
    std::fs::write(
        work.join("gate-highharness-functional.log"),
        "Finished test profile; cargo test --workspace --no-run exit 0\n",
    )
    .unwrap();
    let ver = json!({
        "schema_version": 1,
        "phase": "highharness",
        "mappings": [{ "criterion": "x", "outcome": "met", "evidence": "git-show-diff: src/cli/mod.rs changed" }],
        "all_met": true
    });
    let r = gates::run_semantic("highharness", run_id, ver, dir.path()).unwrap();
    assert_eq!(r.status, "pass");
}

#[test]
fn semantic_gate_fails_on_evidence_overlapping_with_functional() {
    let dir = TempDir::new().unwrap();
    let run_id = "test-semantic-violation";
    let work = dir
        .path()
        .join(".harness/artifacts/episodes-work")
        .join(run_id);
    std::fs::create_dir_all(&work).unwrap();
    // Functional log content that the semantic evidence cites (overlap).
    std::fs::write(
        work.join("gate-highharness-functional.log"),
        "tests/cli_version.rs:5 version_prints_highharness\n",
    )
    .unwrap();
    let ver = json!({
        "schema_version": 1,
        "phase": "highharness",
        "mappings": [{ "criterion": "x", "outcome": "met", "evidence": "tests/cli_version.rs:5 version_prints_highharness" }],
        "all_met": true
    });
    let r = gates::run_semantic("highharness", run_id, ver, dir.path()).unwrap();
    assert_eq!(r.status, "fail");
    assert!(r
        .reason
        .unwrap()
        .contains("semantic-orthogonality-violation"));
}

#[test]
fn semantic_gate_fails_on_unmet_criterion() {
    let dir = TempDir::new().unwrap();
    let run_id = "test-semantic-unmet";
    let work = dir
        .path()
        .join(".harness/artifacts/episodes-work")
        .join(run_id);
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(work.join("gate-highharness-functional.log"), "ok\n").unwrap();
    let ver = json!({
        "schema_version": 1,
        "phase": "highharness",
        "mappings": [{ "criterion": "x", "outcome": "unmet", "evidence": "git-show-diff" }],
        "all_met": false
    });
    let r = gates::run_semantic("highharness", run_id, ver, dir.path()).unwrap();
    assert_eq!(r.status, "fail");
}

#[test]
fn semantic_gate_cli_requires_verification_flag() {
    use assert_cmd::Command;
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .args([
            "gates",
            "run",
            "--phase",
            "highharness",
            "--gate",
            "semantic",
            "--run-id",
            "x",
            "--changes",
            "{}",
        ])
        .current_dir(dir.path())
        .assert()
        .failure();
}
