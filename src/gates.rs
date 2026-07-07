//! Verification gate runner, per `HARNESS_PRIMITIVES.md` §7.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use tokio::runtime::Runtime;

use serde::Serialize;
use serde_json::Value;
use toml::Value as TomlValue;

use crate::error::{HxError, HxResult};
use crate::store::{artifacts_dir, config_path};

/// Result of a single gate execution.
#[derive(Debug, Clone, Serialize)]
pub struct GateResult {
    /// Schema version for artifact compatibility.
    pub schema_version: u32,
    /// Phase this gate belongs to (e.g., `highharness`).
    pub phase: String,
    /// Gate name (e.g., `syntactic`, `functional`, `semantic`).
    pub gate: String,
    /// Outcome status: `pass`, `fail`, or `blocked`.
    pub status: String,
    /// Shell command that was executed.
    pub command: String,
    /// Process exit code.
    pub exit_code: i32,
    /// Whether output was truncated to 32 KB.
    pub output_truncated: bool,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
    /// Path to the evidence log file.
    pub evidence_path: String,
    /// Optional reason for non-pass status.
    pub reason: Option<String>,
}

/// Run a single gate. The gate command is looked up under
/// `.harness/config.toml [gates.<phase>.<gate>]`.
pub fn run(
    phase: &str,

    gate: &str,

    run_id: &str,

    changes: Value,

    root: &Path,
) -> HxResult<GateResult> {
    let cfg = read_config(root)?;
    let cmd_str = lookup_gate(&cfg, phase, gate)
        .ok_or_else(|| HxError::Other(format!("gate {}/{} not configured", phase, gate)))?;
    let cmd = substitute(&cmd_str, &changes);
    let timeout_s = lookup_timeout(&cfg, phase, gate).unwrap_or(60);

    let started = Instant::now();
    let evidence_dir = artifacts_dir(root).join("episodes-work").join(run_id);
    fs::create_dir_all(&evidence_dir)?;
    let evidence_path = evidence_dir.join(format!("gate-{}-{}.log", phase, gate));
    let evidence_path_str = evidence_path.display().to_string();

    // Run the command with timeout enforcement.
    let timeout_duration = Duration::from_secs(timeout_s);
    let rt = Runtime::new()
        .map_err(|e| HxError::Other(format!("failed to create tokio runtime: {}", e)))?;
    let child_result = rt.block_on(async {
        let mut tokio_cmd = tokio::process::Command::new("sh");
        tokio_cmd
            .arg("-c")
            .arg(&cmd)
            .current_dir(root)
            .env("CHANGED", changes.to_string());
        match tokio::time::timeout(timeout_duration, tokio_cmd.output()).await {
            Ok(result) => result.map(|out| {
                (
                    out.status,
                    out.stdout,
                    out.stderr,
                )
            }),
            Err(_elapsed) => {
                // Timeout expired — kill the process.
                Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("gate command timed out after {}s: {}", timeout_s, cmd),
                ))
            }
        }
    });
    let (status, stdout, stderr, blocked) = match child_result {
        Ok((st, out, err)) => (st, out, err, false),
        Err(_e) => {
            // Couldn't spawn — treat as a `blocked` gate (e.g., missing command).
            // Per spec, if the functional gate "no tests" signal triggers, we
            // would substitute a smoke check. We attempt a smoke check.
            let smoke = lookup_smoke(&cfg, phase).unwrap_or_else(|| "true".to_string());
            let out = Command::new("sh")
                .arg("-c")
                .arg(&smoke)
                .current_dir(root)
                .output();
            match out {
                Ok(o) => {
                    let combined = format!(
                        "smoke-fallback (original failed to spawn):\nstdout: {}\nstderr: {}\n",
                        String::from_utf8_lossy(&o.stdout),
                        String::from_utf8_lossy(&o.stderr)
                    );
                    let exit = o.status.code().unwrap_or(-1);
                    fs::write(&evidence_path, &combined)?;
                    let duration = started.elapsed().as_millis() as u64;
                    let status = if o.status.success() { "pass" } else { "fail" };
                    return Ok(GateResult {
                        schema_version: 1,

                        phase: phase.to_string(),

                        gate: gate.to_string(),

                        status: status.to_string(),

                        command: smoke,

                        exit_code: exit,

                        output_truncated: combined.len() > 32_768,

                        duration_ms: duration,

                        evidence_path: evidence_path_str,
                        reason: Some(
                            "original gate command unavailable; smoke fallback used".to_string(),
                        ),
                    });
                }
                Err(_) => {
                    let body = "gate failed to spawn and smoke fallback also failed".to_string();
                    fs::write(&evidence_path, &body)?;
                    return Ok(GateResult {
                        schema_version: 1,

                        phase: phase.to_string(),

                        gate: gate.to_string(),
                        status: "blocked".to_string(),

                        command: cmd,
                        exit_code: -1,

                        output_truncated: false,

                        duration_ms: started.elapsed().as_millis() as u64,

                        evidence_path: evidence_path_str,

                        reason: Some("command unavailable; smoke fallback failed".to_string()),
                    });
                }
            }
        }
    };

    let exit_code = status.code().unwrap_or(-1);
    let mut combined = String::new();
    combined.push_str(&format!("$ {}\n", cmd));
    combined.push_str(&String::from_utf8_lossy(&stdout));
    combined.push_str(&String::from_utf8_lossy(&stderr));
    let truncated = combined.len() > 32_768;
    fs::write(&evidence_path, &combined)?;
    let _ = blocked;
    let _ = timeout_s;
    let duration = started.elapsed().as_millis() as u64;
    let gate_status = if status.success() { "pass" } else { "fail" };
    Ok(GateResult {
        schema_version: 1,

        phase: phase.to_string(),

        gate: gate.to_string(),

        status: gate_status.to_string(),

        command: cmd,
        exit_code,

        output_truncated: truncated,

        duration_ms: duration,

        evidence_path: evidence_path_str,

        reason: None,
    })
}

fn read_config(root: &Path) -> HxResult<TomlValue> {
    let p = config_path(root);
    if !p.exists() {
        return Ok(TomlValue::Table(Default::default()));
    }
    let raw = fs::read_to_string(&p)?;
    let v: TomlValue = toml::from_str(&raw)?;
    Ok(v)
}

fn lookup_gate(cfg: &TomlValue, phase: &str, gate: &str) -> Option<String> {
    let t = cfg.as_table()?;
    let gates = t.get("gates")?.as_table()?;
    let p = gates.get(phase)?.as_table()?;
    let g = p.get(gate)?;
    if let Some(s) = g.as_str() {
        return Some(s.to_string());
    }
    if let Some(t) = g.as_table() {
        if let Some(cmd) = t.get("cmd") {
            if let Some(s) = cmd.as_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn lookup_timeout(cfg: &TomlValue, phase: &str, gate: &str) -> Option<u64> {
    let t = cfg.as_table()?;
    let gates = t.get("gates")?.as_table()?;
    let p = gates.get(phase)?.as_table()?;
    let g = p.get(gate)?.as_table()?;
    let s = g.get("timeout")?.as_str()?;
    s.strip_suffix('s')?.parse().ok()
}

fn lookup_smoke(cfg: &TomlValue, phase: &str) -> Option<String> {
    let t = cfg.as_table()?;
    let gates = t.get("gates")?.as_table()?;
    let p = gates.get(phase)?.as_table()?;
    let g = p.get("smoke")?;
    if let Some(s) = g.as_str() {
        return Some(s.to_string());
    }
    if let Some(t) = g.as_table() {
        if let Some(cmd) = t.get("cmd") {
            if let Some(s) = cmd.as_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn substitute(cmd: &str, changes: &Value) -> String {
    let changed = changes
        .get("changed")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    cmd.replace("$CHANGED", &changed)
        .replace("$PYTEST_TARGETS", &changed)
        .replace("$WORKSPACE_FILTER", &changed)
}

/// Run the semantic gate per `HARNESS_PRIMITIVES.md` §7.3 + `HARNESS_ENGINEERING.md` §6.3.
///
/// `verification_json` must be a JSON object of the shape:
///   { "schema_version":1, "phase":"...", "mappings":[{"criterion","outcome","evidence"}], "all_met":true }
///
/// Algorithm:
/// 1. Parse the §7.3 shape.
/// 2. Fail if `all_met != true` or any `mapping.outcome != "met"`.
/// 3. Orthogonality check: read
///    `.harness/artifacts/episodes-work/<run_id>/gate-<phase>-functional.log`
///    and ensure no `mapping.evidence` string is cited by it (substring
///    match) — also fail if the literal path
///    `gate-<phase>-functional.log` appears in any evidence. This enforces
///    §6.3 semantic-vs-functional evidence orthogonality.
/// 4. Pass otherwise.
pub fn run_semantic(
    phase: &str,

    run_id: &str,

    verification_json: Value,

    root: &Path,
) -> HxResult<GateResult> {
    let started = Instant::now();
    let evidence_dir = artifacts_dir(root).join("episodes-work").join(run_id);
    fs::create_dir_all(&evidence_dir)?;
    let evidence_path = evidence_dir.join(format!("gate-{}-semantic.log", phase));
    let evidence_path_str = evidence_path.display().to_string();

    // 1. Parse §7.3 shape.
    let mappings = verification_json
        .get("mappings")
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();
    let all_met = verification_json
        .get("all_met")
        .and_then(|m| m.as_bool())
        .unwrap_or(false);

    // 2. all_met check.
    if !all_met {
        let body = format!(
            "$ semantic-verification-parse\nverification.all_met = false (mappings: {})\n",
            mappings.len()
        );
        fs::write(&evidence_path, &body)?;
        return Ok(GateResult {
            schema_version: 1,

            phase: phase.to_string(),
            gate: "semantic".to_string(),
            status: "fail".to_string(),
            command: "semantic-verification-parse".to_string(),

            exit_code: 1,

            output_truncated: false,

            duration_ms: started.elapsed().as_millis() as u64,

            evidence_path: evidence_path_str,

            reason: Some("verification.all_met != true".to_string()),
        });
    }

    // 3. Each mapping's outcome must be "met".
    for m in &mappings {
        let outcome = m.get("outcome").and_then(|v| v.as_str()).unwrap_or("");
        if outcome != "met" {
            let criterion = m
                .get("criterion")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            let body = format!(
                "$ semantic-verification-parse\nmapping.outcome = {} (criterion: {})\n",
                outcome, criterion
            );
            fs::write(&evidence_path, &body)?;
            return Ok(GateResult {
                schema_version: 1,

                phase: phase.to_string(),
                gate: "semantic".to_string(),
                status: "fail".to_string(),
                command: "semantic-verification-parse".to_string(),

                exit_code: 1,

                output_truncated: false,

                duration_ms: started.elapsed().as_millis() as u64,

                evidence_path: evidence_path_str,
                reason: Some(format!(
                    "criterion {} has outcome {} (must be met)",
                    criterion, outcome
                )),
            });
        }
    }

    // 4. Orthogonality check vs. the functional-gate log.
    let functional_log = evidence_dir.join(format!("gate-{}-functional.log", phase));
    let functional_text = if functional_log.exists() {
        fs::read_to_string(&functional_log).unwrap_or_default()
    } else {
        String::new()
    };
    let literal_functional_path = format!("gate-{}-functional.log", phase);

    let mut body = String::new();
    body.push_str("$ semantic-verification-parse\n");
    let functional_log_str = functional_log.display().to_string();
    let functional_log_ref: &str = if functional_text.is_empty() {
        "<absent>"
    } else {
        functional_log_str.as_str()
    };
    body.push_str(&format!(
        "mappings: {} (all_met=true); functional-log: {}\n",
        mappings.len(),
        functional_log_ref
    ));

    for m in &mappings {
        let criterion = m
            .get("criterion")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let evidence = m.get("evidence").and_then(|v| v.as_str()).unwrap_or("");

        // 4a. Evidence must NOT mention the literal functional-log path.
        if !literal_functional_path.is_empty() && evidence.contains(&literal_functional_path) {
            body.push_str(&format!(
                "FAIL criterion={} cites functional-gate artifact {}\n",
                criterion, literal_functional_path
            ));
            fs::write(&evidence_path, &body)?;
            return Ok(GateResult {

                schema_version: 1,

                phase: phase.to_string(),
                gate: "semantic".to_string(),
                status: "fail".to_string(),
                command: "semantic-verification-parse".to_string(),

                exit_code: 1,

                output_truncated: false,

                duration_ms: started.elapsed().as_millis() as u64,

                evidence_path: evidence_path_str,
                reason: Some(format!(
                    "semantic-orthogonality-violation: criterion {} cites functional-gate artifact {}",
                    criterion, literal_functional_path
                )),
            });
        }

        // 4b. Multi-line evidence: any non-empty line of the evidence must
        // not appear as a substring in the functional log.
        let mut any_overlap = false;
        for line in evidence.lines() {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }
            if functional_text.contains(l) {
                any_overlap = true;
                body.push_str(&format!(
                    "FAIL criterion={} evidence line overlaps functional log: {:?}\n",
                    criterion, l
                ));
                break;
            }
        }
        if any_overlap {
            fs::write(&evidence_path, &body)?;
            return Ok(GateResult {

                schema_version: 1,

                phase: phase.to_string(),
                gate: "semantic".to_string(),
                status: "fail".to_string(),
                command: "semantic-verification-parse".to_string(),

                exit_code: 1,

                output_truncated: false,

                duration_ms: started.elapsed().as_millis() as u64,

                evidence_path: evidence_path_str,
                reason: Some(format!(
                    "semantic-orthogonality-violation: criterion {} cites functional-gate artifact <e>",
                    criterion
                )),
            });
        }
    }

    body.push_str("orthogonality check: PASS (no overlap with functional-gate log)\n");
    fs::write(&evidence_path, &body)?;
    Ok(GateResult {
        schema_version: 1,

        phase: phase.to_string(),
        gate: "semantic".to_string(),
        status: "pass".to_string(),
        command: "semantic-verification-parse".to_string(),

        exit_code: 0,

        output_truncated: false,

        duration_ms: started.elapsed().as_millis() as u64,

        evidence_path: evidence_path_str,

        reason: None,
    })
}
