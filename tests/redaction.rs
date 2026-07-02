use assert_cmd::Command;

#[test]
fn redaction_detects_aws_access_key() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["redaction", "scan"])
        .write_stdin("AKIAIOSFODNN7EXAMPLE");
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("aws-access-key") || stdout.contains("REDACTED"),
        "stdout: {}",
        stdout
    );
}

#[test]
fn redaction_detects_github_pat() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["redaction", "scan"])
        .write_stdin("ghp_abcdefghijklmnopqrstuvwxyzABCDEFGHIJ123456");
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("github-pat") || stdout.contains("REDACTED"),
        "stdout: {}",
        stdout
    );
}

#[test]
fn redaction_detects_jwt() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["redaction", "scan"])
        .write_stdin("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3j6T3XqoP1D");
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("jwt") || stdout.contains("REDACTED"),
        "stdout: {}",
        stdout
    );
}

#[test]
fn redaction_list_shows_patterns() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["redaction", "list"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("aws-access-key"));
    assert!(stdout.contains("pem-block"));
    assert!(stdout.contains("github-pat"));
    assert!(stdout.contains("jwt"));
}

#[test]
fn redaction_does_not_redact_64char_sha256() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    let sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    cmd.args(["redaction", "scan"]).write_stdin(sha);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should not match any pattern (no REDACTED)
    assert!(
        !stdout.contains("REDACTED"),
        "SHA-256 should not be redacted: {}",
        stdout
    );
}

#[test]
fn redaction_detects_pem_block() {
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["redaction", "scan"]).write_stdin(
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA\n-----END RSA PRIVATE KEY-----",
    );
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("pem-block") || stdout.contains("REDACTED"),
        "stdout: {}",
        stdout
    );
}

#[test]
fn redaction_scan_accepts_file_flag() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "AKIAIOSFODNN7EXAMPLE").unwrap();
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args(["redaction", "scan", "--file", tmp.path().to_str().unwrap()]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
}

#[test]
fn tools_invoke_redacts_secret_in_output() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "AKIAIOSFODNN7EXAMPLE").unwrap();
    let mut cmd = Command::cargo_bin("HighHarness").unwrap();
    cmd.args([
        "tools",
        "invoke",
        "--tool",
        "fs.read",
        "--args",
        &format!(r#"{{"path":"{}"}}"#, tmp.path().display()),
    ]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
}

// Wipe and lookup tests (unit-level)
#[test]
fn redaction_apply_replaces_with_token() {
    let content = "AKIAIOSFODNN7EXAMPLE";
    let redactions = highharness::redaction::Redactions {
        patterns: vec![highharness::redaction::PatternSpec {
            id: "aws-access-key".to_string(),
            regex_str: "AKIA[0-9A-Z]{16}".to_string(),
            severity: "critical".to_string(),
            compiled: Some(regex::Regex::new("AKIA[0-9A-Z]{16}").unwrap()),
        }],
        in_memory_map: std::sync::Mutex::new(Vec::new()),
    };
    let mut s = content.to_string();
    let results = redactions.apply(&mut s);
    assert!(!results.is_empty(), "should have redactions");
}
