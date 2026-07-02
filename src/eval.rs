//! Eval suite — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{HxError, HxResult};
use crate::id;
use crate::tools::registry::{InvokeCtx, Registry};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// struct `EvalSummary` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct EvalSummary {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub kind: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub commit: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub created_at: String,
}

#[derive(Debug, Serialize)]
/// struct `EvalResult` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct EvalResult {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub schema_version: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub eval_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub run_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub started_at: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub finished_at: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub outcomes: Vec<EvalOutcome>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub passed: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub artifact_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
/// struct `EvalOutcome` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct EvalOutcome {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub check: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub status: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub evidence: String,
}

/// fn `list` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn list(root: &Path) -> HxResult<Vec<EvalSummary>> {
    let evals_dir = crate::store::harness_dir(root).join("evals");
    if !evals_dir.exists() {
        /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        return Ok(Vec::new());
    }
    let mut summaries = Vec::new();
    for entry in fs::read_dir(&evals_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let task_path = path.join("task.md");
        if !task_path.exists() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        let content = fs::read_to_string(&task_path)?;
        let first_line = content.lines().next().unwrap_or("").to_string();
        let kind = first_line.trim_start_matches("Task:").trim().to_string();
        let metadata = fs::metadata(&task_path)?;
        let created_at = metadata
            .created()
            .ok()
            .map(|t| {
                let dt: chrono::DateTime<chrono::Utc> = t.into();
                dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
            })
            .unwrap_or_default();

        summaries.push(EvalSummary {
            id,
            kind,
            commit: String::new(),
            created_at,
        });
    }
    summaries.sort_by(|a, b| a.id.cmp(&b.id));
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(summaries)
}

/// fn `run` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run(id: &str, root: &Path) -> HxResult<EvalResult> {
    let started_at = id::now_iso();
    let run_id = id::run_id("eval", "hh");
    let evals_dir = crate::store::harness_dir(root).join("evals");
    let eval_dir = evals_dir.join(id);
    let task_path = eval_dir.join("task.md");
    if !task_path.exists() {
        return Err(HxError::Other(format!("eval not found: {}", id)));
    }

    let content = fs::read_to_string(&task_path)?;
    let mut lines = content.lines();
    let _task_desc = lines.next().unwrap_or("").to_string();
    let rest: String = lines.collect::<Vec<&str>>().join("\n").trim().to_string();

    let tool_calls: Vec<Value> = if rest.is_empty() {
        Vec::new()
    } else {
        serde_json::from_str(&rest)?
    };

    let registry = Registry::load(root)?;
    let mut outcomes = Vec::new();

    for (i, call) in tool_calls.iter().enumerate() {
        let tool = call.get("tool").and_then(|v| v.as_str()).unwrap_or("");
        let args = call.get("args").cloned().unwrap_or(Value::Null);
        let ctx = InvokeCtx {
            run_id: run_id.clone(),
            agent_id: "eval-agent".to_string(),
            tool_call_id: format!("eval-tc-{}", i),
        };
        let result = registry.invoke_raw(tool, args, &ctx, None, root);
        match result {
            /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Ok(r) => {
                outcomes.push(EvalOutcome {
                    check: format!("tool_call_{}", i),
                    status: if r.ok {
                        "pass".to_string()
                    } else {
                        "fail".to_string()
                    },
                    evidence: r.content.value.to_string(),
                });
            }
            /// Variant `Err` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            Err(e) => {
                outcomes.push(EvalOutcome {
                    check: format!("tool_call_{}", i),
                    status: "blocked".to_string(),
                    evidence: e.to_string(),
                });
            }
        }
    }

    let golden_dir = eval_dir.join("golden");
    if golden_dir.exists() {
        for entry in fs::read_dir(&golden_dir)? {
            let entry = entry?;
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname == ".gitkeep" || entry.path().is_dir() {
                continue;
            }
            let golden_path = entry.path();
            let golden_content = fs::read_to_string(&golden_path)?;
            let target_path = root.join(&fname);
            if golden_content.starts_with("content_contains:") {
                let needle = golden_content
                    .trim_start_matches("content_contains:")
                    .trim();
                if target_path.exists() {
                    let actual = fs::read_to_string(&target_path)?;
                    if actual.contains(needle) {
                        outcomes.push(EvalOutcome {
                            check: format!("golden/{}", fname),
                            status: "pass".to_string(),
                            evidence: format!("content contains '{}'", needle),
                        });
                    } else {
                        outcomes.push(EvalOutcome {
                            check: format!("golden/{}", fname),
                            status: "fail".to_string(),
                            evidence: format!(
                                "expected content containing '{}' but got '{}'",
                                needle, actual
                            ),
                        });
                    }
                } else {
                    outcomes.push(EvalOutcome {
                        check: format!("golden/{}", fname),
                        status: "fail".to_string(),
                        evidence: format!("file {} does not exist", target_path.display()),
                    });
                }
            } else if target_path.exists() {
                outcomes.push(EvalOutcome {
                    check: format!("golden/{}", fname),
                    status: "pass".to_string(),
                    evidence: format!("file {} exists", target_path.display()),
                });
            } else {
                outcomes.push(EvalOutcome {
                    check: format!("golden/{}", fname),
                    status: "fail".to_string(),
                    evidence: format!("file {} does not exist", target_path.display()),
                });
            }
        }
    }

    let forbidden_dir = eval_dir.join("forbidden");
    if forbidden_dir.exists() {
        for entry in fs::read_dir(&forbidden_dir)? {
            let entry = entry?;
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname == ".gitkeep" || entry.path().is_dir() {
                continue;
            }
            let target_path = root.join(&fname);
            if target_path.exists() {
                outcomes.push(EvalOutcome {
                    check: format!("forbidden/{}", fname),
                    status: "fail".to_string(),
                    evidence: format!("file {} exists but should be absent", target_path.display()),
                });
            } else {
                outcomes.push(EvalOutcome {
                    check: format!("forbidden/{}", fname),
                    status: "pass".to_string(),
                    evidence: format!("file {} correctly absent", target_path.display()),
                });
            }
        }
    }

    let finished_at = id::now_iso();
    let passed = outcomes.iter().all(|o| o.status == "pass");

    let artifact_dir = crate::store::artifacts_dir(root).join("evals").join(id);
    fs::create_dir_all(&artifact_dir)?;
    let timestamp = finished_at.replace([':', '-'], "");
    let artifact_path = artifact_dir.join(format!("{}.json", timestamp));
    let result = EvalResult {
        schema_version: 1,
        eval_id: id.to_string(),
        run_id,
        started_at,
        finished_at: finished_at.clone(),
        outcomes,
        passed,
        artifact_paths: vec![artifact_path.to_string_lossy().to_string()],
    };
    let json = serde_json::to_string_pretty(&result)?;
    fs::write(&artifact_path, &json)?;
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(result)
}
