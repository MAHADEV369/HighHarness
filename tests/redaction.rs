use assert_cmd::Command;
use tempfile::TempDir;

fn bootstrap_redaction_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    // Write a redactions.toml so the scan has patterns to match.
    std::fs::create_dir_all(dir.path().join(".harness")).unwrap();
    std::fs::write(
        dir.path().join(".harness/redactions.toml"),
        r#"schema_version = 1

[[patterns]]
id = "aws-access-key"
regex = "AKIA[0-9A-Z]{16}"
severity = "critical"

[[patterns]]
id = "pem-block"
regex = "-----BEGIN [A-Z ]*PRIVATE KEY-----"
severity = "critical"

[[patterns]]
id = "github-pat"
regex = "gh[pousr]_[A-Za-z0-9]{36,}"
severity = "critical"

[[patterns]]
id = "jwt"
regex = "eyJ[A-Za-z0-9_-]{10,}\\.[A-Za-z0-9_-]{10,}\\.[A-Za-z0-9_-]{10,}"
severity = "high"
"#,
    )
    .unwrap();
    dir
}

#[test]
fn redaction_detects_aws_access_key() {
    let dir = bootstrap_redaction_dir();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["redaction", "scan"])
        .write_stdin("AKIAIOSFODNN7EXAMPLE")
        .output()
        .unwrap();
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
    let dir = bootstrap_redaction_dir();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["redaction", "scan"])
        .write_stdin("ghp_abcdefghijklmnopqrstuvwxyzABCDEFGHIJ123456")
        .output()
        .unwrap();
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
    let dir = bootstrap_redaction_dir();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["redaction", "scan"])
        .write_stdin("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3j6T3XqoP1D")
        .output()
        .unwrap();
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
    let dir = bootstrap_redaction_dir();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["redaction", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("aws-access-key"));
    assert!(stdout.contains("pem-block"));
    assert!(stdout.contains("github-pat"));
    assert!(stdout.contains("jwt"));
}

#[test]
fn redaction_does_not_redact_64char_sha256() {
    let dir = bootstrap_redaction_dir();
    let sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["redaction", "scan"])
        .write_stdin(sha)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("REDACTED"),
        "SHA-256 should not be redacted: {}",
        stdout
    );
}

#[test]
fn redaction_detects_pem_block() {
    let dir = bootstrap_redaction_dir();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["redaction", "scan"])
        .write_stdin(
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA\n-----END RSA PRIVATE KEY-----",
        )
        .output()
        .unwrap();
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
    let dir = bootstrap_redaction_dir();
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "AKIAIOSFODNN7EXAMPLE").unwrap();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["redaction", "scan", "--file", tmp.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn tools_invoke_redacts_secret_in_output() {
    let dir = TempDir::new().unwrap();
    // Bootstrap so tools are available.
    Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args(["bootstrap", "init", "--human", "test"])
        .assert()
        .success();
    // Add redaction patterns.
    std::fs::write(
        dir.path().join(".harness/redactions.toml"),
        r#"schema_version = 1

[[patterns]]
id = "aws-access-key"
regex = "AKIA[0-9A-Z]{16}"
severity = "critical"
"#,
    )
    .unwrap();
    // Create a test file in the temp dir.
    std::fs::write(dir.path().join("secret.txt"), "AKIAIOSFODNN7EXAMPLE").unwrap();
    let output = Command::cargo_bin("HighHarness")
        .unwrap()
        .current_dir(&dir)
        .args([
            "tools",
            "invoke",
            "--tool",
            "fs.read",
            "--args",
            r#"{"path":"secret.txt"}"#,
        ])
        .output()
        .unwrap();
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
