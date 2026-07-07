//! Incident response automation per HARNESS_SECURITY.md §9.

use crate::error::HxResult;
use std::fs;
use std::path::Path;

/// Request parameters for declaring an incident.
pub struct DeclareRequest<'a> {
    /// Workspace root path.
    pub root: &'a Path,
    /// Detection rule identifier (e.g. "F1").
    pub detection_rule: &'a str,
    /// Attack vector identifier (e.g. "V2.3").
    pub vector: &'a str,
    /// Run identifier.
    pub run_id: &'a str,
    /// Agent identifier.
    pub agent_id: &'a str,
    /// Model identifier (optional).
    pub model_id: Option<&'a str>,
    /// Model route (optional).
    pub model_route: Option<&'a str>,
    /// Severity level: "low", "medium", "high", "critical".
    pub severity: &'a str,
    /// Whether the incident had impact.
    pub had_impact: bool,
    /// Paths to evidence files.
    pub evidence_paths: Vec<String>,
}

/// Declare a new incident. Creates incident JSON and optionally a notification file.
/// Returns the incident id.
#[allow(clippy::too_many_arguments)]
pub fn declare(
    root: &Path,
    detection_rule: &str,
    vector: &str,
    run_id: &str,
    agent_id: &str,
    model_id: Option<&str>,
    model_route: Option<&str>,
    severity: &str,
    had_impact: bool,
    evidence_paths: Vec<String>,
) -> HxResult<String> {
    declare_from(DeclareRequest {
        root,
        detection_rule,
        vector,
        run_id,
        agent_id,
        model_id,
        model_route,
        severity,
        had_impact,
        evidence_paths,
    })
}

/// Declare an incident from a structured request (M9).
pub fn declare_from(req: DeclareRequest) -> HxResult<String> {
    let incidents_dir = req
        .root
        .join(".harness")
        .join("artifacts")
        .join("incidents");
    fs::create_dir_all(&incidents_dir)?;

    let notifications_dir = req
        .root
        .join(".harness")
        .join("artifacts")
        .join("notifications");
    fs::create_dir_all(&notifications_dir)?;

    let id = format!(
        "inc_{}_{}",
        crate::id::now_compact(),
        &req.detection_rule
            .to_lowercase()
            .chars()
            .take(6)
            .collect::<String>()
    );

    let incident = serde_json::json!({
        "schema_version": 1,
        "id": id,
        "detection_rule": req.detection_rule,
        "vector": req.vector,
        "run_id": req.run_id,
        "agent_id": req.agent_id,
        "model_id": req.model_id,
        "model_route": req.model_route,
        "severity": req.severity,
        "had_impact": req.had_impact,
        "declared_at": crate::id::now_iso(),
        "status": "open",
        "evidence_paths": req.evidence_paths,
    });

    let path = incidents_dir.join(format!("{}.json", id));
    fs::write(&path, serde_json::to_string_pretty(&incident)?)?;

    // Write notification file if had_impact
    if req.had_impact {
        let notif_path = notifications_dir.join(format!("{}.txt", id));
        fs::write(
            &notif_path,
            format!(
                "INCIDENT WITH IMPACT: {} (rule={}, vector={}, run={})\n",
                id, req.detection_rule, req.vector, req.run_id
            ),
        )?;
    }

    Ok(id)
}

/// List declared incidents, optionally filtering to open only.
pub fn list(root: &Path, open_only: bool) -> HxResult<Vec<serde_json::Value>> {
    let incidents_dir = root.join(".harness").join("artifacts").join("incidents");
    if !incidents_dir.exists() {
        return Ok(Vec::new());
    }
    let mut results = Vec::new();
    for entry in fs::read_dir(&incidents_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(&path)?;
        let v: serde_json::Value = serde_json::from_str(&raw)?;
        if open_only {
            if let Some(status) = v.get("status").and_then(|x| x.as_str()) {
                if status != "open" {
                    continue;
                }
            }
        }
        results.push(v);
    }
    Ok(results)
}

/// Acknowledge an incident, recording the acknowledger.
pub fn acknowledge(root: &Path, id: &str, by: &str) -> HxResult<()> {
    let path = root
        .join(".harness")
        .join("artifacts")
        .join("incidents")
        .join(format!("{}.json", id));
    let raw = fs::read_to_string(&path)?;
    let mut v: serde_json::Value = serde_json::from_str(&raw)?;
    v["status"] = serde_json::json!("acknowledged");
    v["acknowledged_by"] = serde_json::json!(by);
    v["acknowledged_at"] = serde_json::json!(crate::id::now_iso());
    fs::write(&path, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

/// Close an incident with a postmortem summary.
pub fn close(root: &Path, id: &str, postmortem: &str) -> HxResult<()> {
    let path = root
        .join(".harness")
        .join("artifacts")
        .join("incidents")
        .join(format!("{}.json", id));
    let raw = fs::read_to_string(&path)?;
    let mut v: serde_json::Value = serde_json::from_str(&raw)?;
    v["status"] = serde_json::json!("closed");
    v["closed_at"] = serde_json::json!(crate::id::now_iso());
    v["postmortem"] = serde_json::json!(postmortem);
    fs::write(&path, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

/// Quarantine a source (agent, model, etc.) by id.
pub fn quarantine_source(root: &Path, source_id: &str, reason: &str) -> HxResult<()> {
    let quarantine_dir = root.join(".harness").join("artifacts").join("quarantine");
    fs::create_dir_all(&quarantine_dir)?;
    let entry = serde_json::json!({
        "source_id": source_id,
        "reason": reason,
        "quarantined_at": crate::id::now_iso(),
    });
    fs::write(
        quarantine_dir.join(format!("{}.json", source_id)),
        serde_json::to_string_pretty(&entry)?,
    )?;
    Ok(())
}

/// Check if a source is quarantined.
pub fn is_quarantined(root: &Path, source_id: &str) -> HxResult<bool> {
    let path = root
        .join(".harness")
        .join("artifacts")
        .join("quarantine")
        .join(format!("{}.json", source_id));
    Ok(path.exists())
}

/// Mark a run as suspicious (for memory inheritance filtering).
pub fn mark_run_suspicious(root: &Path, run_id: &str) -> HxResult<()> {
    let quarantine_dir = root.join(".harness").join("artifacts").join("quarantine");
    fs::create_dir_all(&quarantine_dir)?;
    let entry = serde_json::json!({
        "run_id": run_id,
        "suspicious": true,
        "marked_at": crate::id::now_iso(),
    });
    fs::write(
        quarantine_dir.join(format!("run_{}.json", run_id)),
        serde_json::to_string_pretty(&entry)?,
    )?;
    Ok(())
}

/// Check if a run is marked suspicious (for memory inheritance filtering).
pub fn is_run_suspicious(root: &Path, run_id: &str) -> HxResult<bool> {
    let path = root
        .join(".harness")
        .join("artifacts")
        .join("quarantine")
        .join(format!("run_{}.json", run_id));
    Ok(path.exists())
}
