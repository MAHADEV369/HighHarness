use assert_cmd::Command;

#[test]
fn changelog_append_accepts_entry_flag() {
    // Use an existing entry file from tests/canonical/fixtures or generate one with no prev_hash;
    // append and verify it landed with chained prev_hash.
    // (If you prefer not to dirty the real CHANGELOG, run inside a tempdir.)
    use std::fs;
    let dir = tempfile::tempdir().unwrap();
    // First bootstrap
    Command::cargo_bin("HighHarness")
        .unwrap()
        .args(["bootstrap", "init", "--human", "test"])
        .current_dir(dir.path())
        .assert()
        .success();
    // Write an entry JSON
    let entry_path = dir.path().join("e.json");
    fs::write(&entry_path, r#"{"n":1,"ts":"2026-06-29T10:00:00Z","agent":"test","run_id":"test","tier":"trivial","files":["a"],"intent":"test","diff_summary":"s","evidence":"e","attribution":"none","verification":"syntactic","status":"added","prev_hash":"","this_hash":""}"#).unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .args([
            "changelog",
            "append",
            "--entry",
            entry_path.to_str().unwrap(),
        ])
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn changelog_append_positional_no_longer_accepted() {
    // Old positional form must now fail (clap rejects unknown positional).
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("HighHarness")
        .unwrap()
        .args(["changelog", "append", "some.json"])
        .current_dir(dir.path())
        .assert()
        .failure();
}
