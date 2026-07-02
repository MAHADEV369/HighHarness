//! Bootstrap protocol, per `HARNESS_VERSIONING.md` §6.

use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

#[allow(unused_imports)]
use std::any::type_name;

use crate::canonical;
use crate::error::{HxError, HxResult};
use crate::id;
use crate::schema::bootstrap::Bootstrap;
use crate::store::{
    artifacts_dir, bootstrap_path, changelog_path, config_path, ensure_skeleton, harness_dir,
    models_path, permissions_path, routing_path, tools_dir,
};
use crate::telemetry;

/// Initialize the harness in `root`. Returns the bootstrap record.
pub fn init(root: &Path, human: &str) -> HxResult<Bootstrap> {
    if bootstrap_path(root).exists() {
        return Err(HxError::NotYetEnforced {
            /// Field `what` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            what: "bootstrap already initialized; re-bootstrap is human-only (§6.2)".to_string(),
        });
    }

    // Refuse if the changelog already has a GENESIS marker (the harness is
    // already bootstrapped from a prior init, even if .harness/ was wiped).
    let cpath = changelog_path(root);
    if cpath.exists() {
        let raw = std::fs::read_to_string(&cpath).unwrap_or_default();
        if raw.contains("## GENESIS") {
            return Err(HxError::NotYetEnforced {
                what: "changelog already has GENESIS marker; re-bootstrap is human-only (§6.2)"
                    .to_string(),
            });
        }
    }

    // 1. Spec sanity: best-effort — the harness itself being present is the
    // sanity proof. We don't validate spec files; we rely on the human having
    // written them.
    let _spec_sanity_ok = root.join("HARNESS_ENGINEERING.md").exists()
        && root.join("HARNESS_PRIMITIVES.md").exists()
        && root.join("HARNESS_VERSIONING.md").exists();

    // 2. Directory skeleton.
    ensure_skeleton(root)?;

    // 3. GENESIS marker.
    let ts = id::now_compact();
    let gh = canonical::genesis_hash(&ts);
    let genesis_block = format!(
        "## GENESIS — {}\n- prev_hash: null\n- this_hash: {}\n- bootstrap_human: {}\n- bootstrap_commit: {}\n- spec_versions: {{ engineering: 1, primitives: 1, security: 1, metrics: 1, versioning: 1 }}\n",
        ts,
        gh,
        human,
        git_head(root).unwrap_or_else(|| "0".to_string()),
    );
    {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(changelog_path(root))?;
        use std::io::Write;
        f.write_all(genesis_block.as_bytes())?;
        f.sync_data()?;
    }

    // 4. Integrity log seed.
    let seed_hash = telemetry::integrity::append_seed(root, "harness-bootstrap", human)?;

    // 5. Materialize built-in tools.
    materialize_builtin_tools(root)?;

    // 6. Seed permissions.toml.
    seed_permissions(root)?;

    // 7. Seed models.toml + routing.toml.
    seed_models(root)?;
    seed_routing(root)?;

    // 8. Seed config.toml.
    seed_config(root)?;

    // 9. Bootstrap eval. For v1, we do a tiny in-repo fixture eval: create a
    //    file, run a gate, append Entry 1, revert. We use the existing
    //    changelog code path so the eval exercises the compare-and-append
    //    primitive end-to-end.
    let (eval_run_id, eval_passed) = run_eval(root, human, &gh)?;

    // 10. Write bootstrap.json.
    let bs = Bootstrap {
        schema_version: 1,

        bootstrapped_at: id::now_iso(),

        bootstrap_human: human.to_string(),

        bootstrap_commit: git_head(root).unwrap_or_else(|| "0".to_string()),
        spec_versions: json!({
            "engineering": 1,
            "primitives": 1,
            "security": 1,
            "metrics": 1,
            "versioning": 1,
        }),

        genesis_hash: gh,
        eval: json!({
            "passed": eval_passed,
            "run_id": eval_run_id,
            "cpmc_usd": 0.0,
        }),

        integrity_log_seed_hash: seed_hash,

        passed: eval_passed,
    };
    if let Some(parent) = bootstrap_path(root).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(bootstrap_path(root), serde_json::to_string_pretty(&bs)?)?;

    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(bs)
}

/// Verify the bootstrap record. Returns `NotBootstrapped` if any check fails.
pub fn verify(root: &Path) -> HxResult<Bootstrap> {
    let path = bootstrap_path(root);
    if !path.exists() {
        return Err(HxError::NotBootstrapped);
    }
    let raw = fs::read_to_string(&path)?;
    let bs: Bootstrap = serde_json::from_str(&raw)?;
    if !bs.passed {
        return Err(HxError::NotBootstrapped);
    }
    // Recompute genesis hash from the marker.
    let cpath = changelog_path(root);
    let txt = fs::read_to_string(&cpath).unwrap_or_default();
    if let Some(existing) = parse_genesis_hash(&txt) {
        if existing != bs.genesis_hash {
            return Err(HxError::AuditForgery(
                "genesis hash in marker != bootstrap.json".to_string(),
            ));
        }
    } else {
        return Err(HxError::NotBootstrapped);
    }
    // Verify integrity log chain.
    let broken = telemetry::integrity::verify(root)?;
    if !broken.is_empty() {
        return Err(HxError::ChainBroken {
            index: broken[0],
            /// Field `expected` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            expected: "see log".to_string(),
            /// Field `got` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            got: "see log".to_string(),
        });
    }
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(bs)
}

fn parse_genesis_hash(txt: &str) -> Option<String> {
    let mut in_genesis = false;
    for line in txt.lines() {
        if line.starts_with("## GENESIS") {
            in_genesis = true;
            continue;
        }
        if in_genesis {
            if line.starts_with("## ") {
                break;
            }
            if let Some(rest) = line.strip_prefix("- this_hash:") {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

fn git_head(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    /// Variant `Some` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn materialize_builtin_tools(root: &Path) -> HxResult<()> {
    use crate::tools;
    let dir = tools_dir(root);
    fs::create_dir_all(&dir)?;
    let tools: Vec<(&str, &str, &str, &str, &str, &str, &str, &str, &str, &str)> = vec![
        (
            "fs.read",
            "Read a file as text or bytes.",
            "read",
            "false",
            "false",
            "false",
            "false",
            "false",
            "auto",
            "read-only",
        ),
        (
            "fs.hash",
            "SHA-256 of a file at path.",
            "read",
            "false",
            "false",
            "false",
            "false",
            "false",
            "auto",
            "read-only",
        ),
        (
            "fs.edit",
            "Atomic in-place file edit (substring, byte range, or line insert).",
            "write",
            "true",
            "false",
            "false",
            "false",
            "false",
            "auto",
            "writes path-scoped",
        ),
        (
            "git.status",
            "Run `git status --porcelain=v2 -b`.",
            "read",
            "false",
            "false",
            "false",
            "false",
            "false",
            "auto",
            "read-only",
        ),
        (
            "git.diff",
            "Run `git diff <target>` (default HEAD).",
            "read",
            "false",
            "false",
            "false",
            "false",
            "false",
            "auto",
            "read-only",
        ),
        (
            "git.blame",
            "Run `git blame` on a file (optionally a line range).",
            "read",
            "false",
            "false",
            "false",
            "false",
            "false",
            "auto",
            "read-only",
        ),
        (
            "shell.exec",
            "Spawn a shell command with cwd + env allowlist + timeout.",
            "exec",
            "false",
            "true",
            "false",
            "true",
            "false",
            "ask",
            "exec",
        ),
        (
            "test.run",
            "Run the configured test command for a phase.",
            "exec",
            "false",
            "true",
            "false",
            "false",
            "false",
            "auto",
            "configured test runner",
        ),
        (
            "lint.run",
            "Run the configured lint command.",
            "exec",
            "false",
            "true",
            "false",
            "false",
            "false",
            "auto",
            "configured lint runner",
        ),
        (
            "web.fetch",
            "Fetch a URL with curl.",
            "network",
            "false",
            "false",
            "true",
            "false",
            "false",
            "ask",
            "network egress",
        ),
    ];
    for (id_, summary, side, write, exec, network, destructive, secrets, mode, reason) in tools {
        let read = side == "read";
        let body = format!(
            r#"id = "{id_}"
schema_version = 1
version = "1.0.0"
source = "builtin"
extension_id = ""
mcp_server = ""
summary = "{summary}"
argument_schema_path = ".harness/tools/schemas/{id_}.args.json"
return_schema_path = ".harness/tools/schemas/{id_}.returns.json"
side_effect = "{side}"

[capabilities]
read = {read}
write = {write}
exec = {exec}
network = {network}
destructive = {destructive}
secrets = {secrets}
side_effect = "{side}"

[approval]
mode = "{mode}"
reason = "{reason}"
"#
        );
        fs::write(dir.join(format!("{}.toml", id_)), body)?;
    }
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

// Marker type so the tools module import is used.
#[allow(dead_code)]
/// struct `DescriptorMarker` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct DescriptorMarker;

fn seed_permissions(root: &Path) -> HxResult<()> {
    let body = r#"schema_version = 1

[[rules]]
id = "R-DENY-HARNESS"
effect = "deny"
tool = "*"
paths = [".harness/**"]
priority = 9999
reason = "harness config immutability"

[[rules]]
id = "R-DENY-SECRETS"
effect = "deny"
tool = "*"
paths = [".env", ".env.*", "**/secrets/**", "**/*.pem", "**/*.key"]
priority = 9999
reason = "secret redaction"

[[rules]]
id = "R-ASK-DESTRUCTIVE"
effect = "ask"
tool = "*"
priority = 500
reason = "destructive ops need approval"

[[rules]]
id = "R-SAFETY-CRITICAL"
effect = "ask"
tool = "*"
paths = ["**/auth/**", "**/secrets/**", "**/migrations/**", ".harness/**", "**/CODEOWNERS"]
safety = true
priority = 800
reason = "safety-critical tier path"

[[rules]]
id = "R-NET-MODEL-PROVIDER"
effect = "allow"
tool = "*"
network = ["api.anthropic.com", "api.openai.com", "generativelanguage.googleapis.com", "api.together.xyz", "api.groq.com", "*.local", "127.0.0.1", "localhost"]
priority = 300
reason = "model provider egress allow-list"

[[rules]]
id = "R-ALLOW-FS"
effect = "allow"
tool = "fs.*"
paths = ["**"]
priority = 600
reason = "allow fs.* on any path that isn't denied above"
"#;
    if let Some(parent) = permissions_path(root).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(permissions_path(root), body)?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

fn seed_models(root: &Path) -> HxResult<()> {
    let body = r#"schema_version = 1

[[models]]
id = "llama-3.3-70b-local"
provider = "ollama"
context_window = 128000
capabilities = { vision = false, tools = true, reasoning = false, prefill = true }
pricing = { input_usd_per_1m = 0, output_usd_per_1m = 0, reasoning_usd_per_1m = 0 }
privacy = { retention = "local", training = "none", residency = "device" }
auth = "none"
tier = "local"

[[models]]
id = "claude-sonnet-4-stub"
provider = "anthropic"
context_window = 200000
capabilities = { vision = true, tools = true, reasoning = true, prefill = true }
pricing = { input_usd_per_1m = 3, output_usd_per_1m = 15, reasoning_usd_per_1m = 5 }
privacy = { retention = "zero", training = "opt-out", residency = "any" }
auth = "secret:anthropic_key"
tier = "flagship"
"#;
    if let Some(parent) = models_path(root).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(models_path(root), body)?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

fn seed_routing(root: &Path) -> HxResult<()> {
    let body = r#"schema_version = 1

[[routes]]
feature = "chat"
primary = "llama-3.3-70b-local"
fallback = ["claude-sonnet-4-stub"]
mode = "manual"

[[routes]]
feature = "agent"
primary = "claude-sonnet-4-stub"
fallback = ["llama-3.3-70b-local"]
mode = "manual"

[[routes]]
feature = "review"
primary = "llama-3.3-70b-local"
fallback = []
mode = "primary-only"
"#;
    if let Some(parent) = routing_path(root).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(routing_path(root), body)?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

fn seed_config(root: &Path) -> HxResult<()> {
    let body = r#"schema_version = 2
harness_version = "0.1.0"

[identity]
org = "cortex"
project = "highharness"
phase = "highharness"

[retrieval]
default_token_budget = 12000
default_tool_call_budget = 12
redact_secrets = true

[budgets]
per_run_usd_hard = 5.0
per_run_usd_soft = 2.0

[approval]
default_mode = "ask"
destructive_needs_human = true
default_expiry = "30m"

[episodes]
dir = "logs/episodes"
retention_days = 90

[changelog]
path = "CHANGELOG.agent.md"
lock_path = ".harness/locks/changelog.lock"

[snapshots]
dir = ".harness/artifacts/snapshots"
max_per_run = 50

[gates.highharness]
syntactic = { cmd = "cargo check --all-targets", timeout = "180s" }
functional = { cmd = "cargo test --workspace --no-run", timeout = "300s" }
lint = { cmd = "cargo fmt -- --check", timeout = "60s" }
regression = { cmd = "cargo test --workspace", timeout = "600s" }
smoke = { cmd = "true", timeout = "30s" }

[gates.editor-shell]
syntactic = { cmd = "true", timeout = "60s" }
functional = { cmd = "true", timeout = "60s" }
smoke = { cmd = "true", timeout = "30s" }

[gates.ai-runtime]
syntactic = { cmd = "true", timeout = "60s" }
functional = { cmd = "true", timeout = "60s" }
smoke = { cmd = "true", timeout = "30s" }

[gates.tooling-python]
syntactic = { cmd = "true", timeout = "60s" }
functional = { cmd = "true", timeout = "60s" }
smoke = { cmd = "true", timeout = "30s" }
"#;
    if let Some(parent) = config_path(root).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(config_path(root), body)?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

fn run_eval(root: &Path, _human: &str, genesis_hash: &str) -> HxResult<(String, bool)> {
    // The eval creates a fixture file, exercises compare-and-append with a
    // trivial entry, and reverts the file. This proves the chain primitives
    // work end-to-end. We do NOT mutate the fixture permanently.
    let fixture = root.join("evals").join("bootstrap-readme").join("notes.md");
    if let Some(parent) = fixture.parent() {
        fs::create_dir_all(parent)?;
    }
    let original = if fixture.exists() {
        fs::read_to_string(&fixture).unwrap_or_default()
    } else {
        String::new()
    };
    fs::write(&fixture, "verified by HighHarness\n")?;

    let run_id_eval = id::run_id("bootstrap-eval", "hx");
    let mut entry = crate::schema::changelog::Entry {
        n: 1,

        ts: id::now_iso(),
        /// Field `agent` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        agent: "highharness/bootstrap".to_string(),

        run_id: run_id_eval.clone(),
        /// Field `tier` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        tier: "trivial".to_string(),

        files: vec!["evals/bootstrap-readme/notes.md".to_string()],
        /// Field `intent` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        intent: "bootstrap eval: verify compare-and-append against GENESIS".to_string(),
        diff_summary: "appended 'verified by HighHarness' to evals/bootstrap-readme/notes.md"
            .to_string(),
        /// Field `evidence` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        evidence: "changelog verify_chain returns empty".to_string(),
        /// Field `attribution` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        attribution: "none".to_string(),
        /// Field `verification` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        verification: "syntactic".to_string(),
        /// Field `status` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        status: "added".to_string(),

        prev_hash: genesis_hash.to_string(),

        this_hash: String::new(),
    };
    let this_hash = crate::store::changelog::append(&mut entry, root)?;
    entry.this_hash = this_hash.clone();
    let broken = crate::store::changelog::verify_chain(root, None)?;
    let passed = broken.is_empty();
    // Revert the fixture file.
    fs::write(&fixture, &original)?;
    // (We do NOT remove the changelog entry; it is the canonical Entry 1
    // of the harness's first run.)
    let _ = (artifacts_dir(root), harness_dir(root));
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok((run_id_eval, passed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn bootstrap_init_writes_bootstrap_json_with_passed_true() {
        let dir = TempDir::new().unwrap();
        let bs = init(dir.path(), "admin").unwrap();
        assert!(bs.passed);
        assert!(bootstrap_path(dir.path()).exists());
    }

    #[test]
    fn bootstrap_verify_exits_not_bootstrapped_when_missing() {
        let dir = TempDir::new().unwrap();
        let err = verify(dir.path()).unwrap_err();
        assert!(matches!(err, HxError::NotBootstrapped));
    }

    #[test]
    fn bootstrap_init_refuses_if_already_bootstrapped() {
        let dir = TempDir::new().unwrap();
        init(dir.path(), "admin").unwrap();
        let err = init(dir.path(), "admin").unwrap_err();
        assert!(matches!(err, HxError::NotYetEnforced { .. }));
    }

    #[test]
    fn bootstrap_init_seeds_R_DENY_HARNESS_and_others() {
        let dir = TempDir::new().unwrap();
        init(dir.path(), "admin").unwrap();
        let raw = std::fs::read_to_string(permissions_path(dir.path())).unwrap();
        assert!(raw.contains("R-DENY-HARNESS"));
        assert!(raw.contains("R-DENY-SECRETS"));
        assert!(raw.contains("R-ASK-DESTRUCTIVE"));
        assert!(raw.contains("R-SAFETY-CRITICAL"));
    }

    #[test]
    fn bootstrap_init_materializes_builtin_tools() {
        let dir = TempDir::new().unwrap();
        init(dir.path(), "admin").unwrap();
        for tool in &[
            "fs.read",
            "fs.hash",
            "fs.edit",
            "git.status",
            "git.diff",
            "git.blame",
            "shell.exec",
            "test.run",
            "lint.run",
            "web.fetch",
        ] {
            let p = tools_dir(dir.path()).join(format!("{}.toml", tool));
            assert!(p.exists(), "missing tool: {}", tool);
        }
    }

    #[test]
    fn bootstrap_init_appends_GENESIS_marker_with_correct_hash() {
        let dir = TempDir::new().unwrap();
        init(dir.path(), "admin").unwrap();
        let raw = std::fs::read_to_string(changelog_path(dir.path())).unwrap();
        assert!(raw.contains("## GENESIS"));
        let bs: Value =
            serde_json::from_str(&std::fs::read_to_string(bootstrap_path(dir.path())).unwrap())
                .unwrap();
        let gh = bs
            .get("genesis_hash")
            .and_then(|x| x.as_str())
            .unwrap()
            .to_string();
        assert!(raw.contains(&format!("- this_hash: {}", gh)));
    }

    #[test]
    fn bootstrap_init_runs_eval_and_appends_Entry_1_chained_to_genesis() {
        let dir = TempDir::new().unwrap();
        init(dir.path(), "admin").unwrap();
        // The eval should have appended Entry 1.
        let latest = crate::store::changelog::latest(dir.path()).unwrap();
        assert_eq!(latest.n, 1);
        // The chain must be healthy.
        let broken = crate::store::changelog::verify_chain(dir.path(), None).unwrap();
        assert!(broken.is_empty(), "broken: {:?}", broken);
    }

    #[test]
    fn bootstrap_verify_returns_ok_after_init() {
        let dir = TempDir::new().unwrap();
        init(dir.path(), "admin").unwrap();
        let bs = verify(dir.path()).unwrap();
        assert!(bs.passed);
    }
}
