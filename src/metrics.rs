//! KPI rollup computation per HARNESS_METRICS.md §1-§4.

use crate::error::HxResult;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// --- KPI value types ---

/// Raw KPI value with evidence paths.
#[derive(Debug, Serialize)]
pub struct KpiValue {
    /// Numeric KPI value.
    pub value: f64,
    /// Unit of measurement (e.g., `rate`, `count`, `usd_per_call`).
    pub unit: String,
    /// Paths to evidence files supporting this value.
    pub evidence_paths: Vec<String>,
}

/// A KPI with its computed value, target threshold, and breach status.
#[derive(Debug, Serialize)]
pub struct Kpi {
    /// KPI identifier (e.g., `merge_rate`).
    pub id: String,
    /// Human-readable KPI name.
    pub name: String,
    /// Computed numeric value.
    pub value: f64,
    /// Unit of measurement.
    pub unit: String,
    /// Target threshold for this KPI.
    pub target: f64,
    /// Whether the KPI is below its target.
    pub breached: bool,
    /// Paths to evidence files.
    pub evidence_paths: Vec<String>,
    /// Lifecycle state: `cold-start`, `partial`, or `live`.
    pub state: String,
}

/// Time window for a KPI rollup.
#[derive(Debug, Serialize)]
pub struct Window {
    /// Window duration in days.
    pub days: u64,
}

/// Full KPI rollup containing all computed KPIs and overall status.
#[derive(Debug, Serialize)]
pub struct Rollup {
    /// Schema version for artifact compatibility.
    pub schema_version: u32,
    /// ISO-8601 timestamp when the rollup was produced.
    pub produced_at: String,
    /// Time window covered by this rollup.
    pub window: Window,
    /// All computed KPIs.
    pub kpis: Vec<Kpi>,
    /// Overall status: `cold-start`, `partial`, or `live`.
    pub status: String,
}

/// Alert generated when a KPI breaches its threshold or an incident is detected.
#[derive(Debug, Serialize)]
pub struct Alert {
    /// Alert kind (e.g., `kpi_breach_merge_rate`, `incident_count`).
    pub kind: String,
    /// Severity level: `warn` or `critical`.
    pub severity: String,
    /// Human-readable alert message.
    pub message: String,
    /// Suggested corrective action.
    pub suggested_action: String,
}

// --- Internal helpers ---

fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn now_ym() -> String {
    chrono::Utc::now().format("%Y-%m").to_string()
}

fn now_ymd() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

fn unix_ts_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn parse_iso_ts(s: &str) -> Option<u64> {
    // Accept "2026-06-29T09:51:08Z" or "2026-06-29T09:51:08"
    let cleaned = s.trim().trim_end_matches('Z');
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(cleaned, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt.and_utc().timestamp() as u64);
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(cleaned, "%Y-%m-%dT%H:%M:%S%.f") {
        return Some(dt.and_utc().timestamp() as u64);
    }
    None
}

fn is_within_window(ts_str: &str, window_secs: u64) -> bool {
    let entry_ts = match parse_iso_ts(ts_str) {
        Some(t) => t,
        None => return true,
    };
    let now = unix_ts_secs();
    now.saturating_sub(entry_ts) <= window_secs
}

/// Parse a window string like "7d" into Duration.
pub fn parse_window(s: &str) -> Duration {
    let s = s.trim();
    if let Some(days_str) = s.strip_suffix('d') {
        if let Ok(days) = days_str.parse::<u64>() {
            return Duration::from_secs(days * 86400);
        }
    }
    // Default: 30 days
    Duration::from_secs(30 * 86400)
}

fn window_secs(d: &Duration) -> u64 {
    d.as_secs()
}

// --- Changelog parsing ---

#[derive(Debug)]
struct ClEntry {
    _n: u64,
    ts: String,
    verification: String,
    status: String,
}

use crate::store::changelog as changelog_store;

fn read_changelog_entries(root: &Path) -> Vec<ClEntry> {
    let cpath = root.join("CHANGELOG.agent.md");
    let txt = fs::read_to_string(&cpath).unwrap_or_default();
    changelog_store::parse_all_entries(&txt)
        .into_iter()
        .map(|e| ClEntry {
            _n: e.n,
            ts: e.ts,
            verification: e.verification,
            status: e.status,
        })
        .collect()
}

// --- Episode helpers ---

fn list_episode_files(root: &Path) -> Vec<std::path::PathBuf> {
    let dir = root.join("logs").join("episodes");
    let mut out = Vec::new();
    if !dir.is_dir() {
        return out;
    }
    if let Ok(rd) = fs::read_dir(&dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn episode_is_complete(path: &std::path::Path) -> bool {
    let txt = fs::read_to_string(path).unwrap_or_default();
    let required = [
        "## Task spec",
        "## Plan",
        "## Task state log",
        "## Tool calls",
        "## Decisions",
        "## Failures",
        "## Interventions",
        "## Pre-task checklist",
        "## Verification report",
        "## Files touched",
    ];
    for section in &required {
        if !txt.contains(section) {
            return false;
        }
    }
    true
}

// --- Intervention helpers ---

fn list_intervention_files(root: &Path) -> Vec<std::path::PathBuf> {
    let dir = crate::store::interventions_dir(root);
    let mut out = Vec::new();
    if !dir.is_dir() {
        return out;
    }
    if let Ok(rd) = fs::read_dir(&dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn count_budget_interventions(root: &Path) -> u64 {
    let mut count = 0u64;
    for path in list_intervention_files(root) {
        let txt = fs::read_to_string(&path).unwrap_or_default();
        let lower = txt.to_lowercase();
        if lower.contains("budget")
            || lower.contains("exhaust")
            || lower.contains("budget_exhausted")
        {
            count += 1;
        }
    }
    count
}

// --- Spend helpers ---

fn read_spend_files(root: &Path) -> HxResult<(f64, u64)> {
    let dir = crate::store::spend_dir(root);
    if !dir.is_dir() {
        return Ok((0.0, 0));
    }
    let mut total_usd = 0.0f64;
    let mut total_calls = 0u64;
    if let Ok(rd) = fs::read_dir(&dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                continue;
            }
            let raw = fs::read_to_string(&path).unwrap_or_default();
            for line in raw.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                    let usd = v.get("usd").and_then(|x| x.as_f64()).unwrap_or(0.0);
                    total_usd += usd;
                    total_calls += 1;
                }
            }
        }
    }
    Ok((total_usd, total_calls))
}

// --- 11 KPI functions ---

/// Compute the merge rate KPI: fraction of changelog entries with status `added` or `modified`.
pub fn kpi_merge_rate(root: &Path, window: &Duration) -> HxResult<KpiValue> {
    let ws = window_secs(window);
    let entries = read_changelog_entries(root);
    let filtered: Vec<&ClEntry> = entries
        .iter()
        .filter(|e| is_within_window(&e.ts, ws))
        .collect();
    if filtered.is_empty() {
        return Ok(KpiValue {
            value: 0.0,
            unit: "rate".to_string(),
            evidence_paths: vec![root
                .join("CHANGELOG.agent.md")
                .to_string_lossy()
                .to_string()],
        });
    }
    let total = filtered.len() as f64;
    let merged = filtered
        .iter()
        .filter(|e| e.status == "added" || e.status == "modified")
        .count() as f64;
    let rate = if total > 0.0 { merged / total } else { 0.0 };
    Ok(KpiValue {
        value: rate,
        unit: "rate".to_string(),
        evidence_paths: vec![root
            .join("CHANGELOG.agent.md")
            .to_string_lossy()
            .to_string()],
    })
}

/// Compute the rollback rate KPI: fraction of changelog entries with status `reverted`.
pub fn kpi_rollback_rate(root: &Path, window: &Duration) -> HxResult<KpiValue> {
    let ws = window_secs(window);
    let entries = read_changelog_entries(root);
    let filtered: Vec<&ClEntry> = entries
        .iter()
        .filter(|e| is_within_window(&e.ts, ws))
        .collect();
    if filtered.is_empty() {
        return Ok(KpiValue {
            value: 0.0,
            unit: "rate".to_string(),
            evidence_paths: vec![root
                .join("CHANGELOG.agent.md")
                .to_string_lossy()
                .to_string()],
        });
    }
    let total = filtered.len() as f64;
    let reverted = filtered.iter().filter(|e| e.status == "reverted").count() as f64;
    let rate = if total > 0.0 { reverted / total } else { 0.0 };
    Ok(KpiValue {
        value: rate,
        unit: "rate".to_string(),
        evidence_paths: vec![root
            .join("CHANGELOG.agent.md")
            .to_string_lossy()
            .to_string()],
    })
}

/// Compute the first-pass rate KPI: fraction of entries with `verification = "full"`.
pub fn kpi_first_pass_rate(root: &Path, window: &Duration) -> HxResult<KpiValue> {
    let ws = window_secs(window);
    let entries = read_changelog_entries(root);
    let filtered: Vec<&ClEntry> = entries
        .iter()
        .filter(|e| is_within_window(&e.ts, ws))
        .collect();
    if filtered.is_empty() {
        return Ok(KpiValue {
            value: 0.0,
            unit: "rate".to_string(),
            evidence_paths: vec![root
                .join("CHANGELOG.agent.md")
                .to_string_lossy()
                .to_string()],
        });
    }
    let total = filtered.len() as f64;
    let full = filtered.iter().filter(|e| e.verification == "full").count() as f64;
    let rate = if total > 0.0 { full / total } else { 0.0 };
    Ok(KpiValue {
        value: rate,
        unit: "rate".to_string(),
        evidence_paths: vec![root
            .join("CHANGELOG.agent.md")
            .to_string_lossy()
            .to_string()],
    })
}

/// Compute the gate-flip rate KPI (stub: always 0 in v1).
pub fn kpi_gate_flip_rate(root: &Path, _window: &Duration) -> HxResult<KpiValue> {
    let eps = list_episode_files(root);
    if eps.is_empty() {
        return Ok(KpiValue {
            value: 0.0,
            unit: "rate".to_string(),
            evidence_paths: vec![],
        });
    }
    // Simple: no gate flips detected (no episode cross-referencing yet)
    Ok(KpiValue {
        value: 0.0,
        unit: "rate".to_string(),
        evidence_paths: vec![],
    })
}

/// Compute the attribution accuracy KPI (stub: always 1.0 in v1).
pub fn kpi_attribution_accuracy(_root: &Path, _window: &Duration) -> HxResult<KpiValue> {
    Ok(KpiValue {
        value: 1.0,
        unit: "rate".to_string(),
        evidence_paths: vec![],
    })
}

/// Compute the verification completeness KPI: fraction of episodes with all required sections.
pub fn kpi_verification_completeness(root: &Path, _window: &Duration) -> HxResult<KpiValue> {
    let eps = list_episode_files(root);
    if eps.is_empty() {
        return Ok(KpiValue {
            value: 0.0,
            unit: "rate".to_string(),
            evidence_paths: vec![],
        });
    }
    let total = eps.len() as f64;
    let complete = eps.iter().filter(|p| episode_is_complete(p)).count() as f64;
    let rate = if total > 0.0 { complete / total } else { 0.0 };
    let ev: Vec<String> = eps
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    Ok(KpiValue {
        value: rate,
        unit: "rate".to_string(),
        evidence_paths: ev,
    })
}

/// Compute the cost-per-merged-change KPI: total USD divided by total API calls.
pub fn kpi_cpmc(root: &Path, _window: &Duration) -> HxResult<KpiValue> {
    let (total_usd, total_calls) = read_spend_files(root)?;
    if total_calls == 0 {
        return Ok(KpiValue {
            value: 0.0,
            unit: "usd_per_call".to_string(),
            evidence_paths: vec![],
        });
    }
    let cpmc = total_usd / total_calls as f64;
    Ok(KpiValue {
        value: cpmc,
        unit: "usd_per_call".to_string(),
        evidence_paths: vec![],
    })
}

/// Compute the time-to-merge KPI (stub: always 0 in v1).
pub fn kpi_ttm(root: &Path, window: &Duration) -> HxResult<KpiValue> {
    let ws = window_secs(window);
    let entries = read_changelog_entries(root);
    let filtered: Vec<&ClEntry> = entries
        .iter()
        .filter(|e| is_within_window(&e.ts, ws))
        .collect();
    if filtered.is_empty() {
        return Ok(KpiValue {
            value: 0.0,
            unit: "seconds".to_string(),
            evidence_paths: vec![],
        });
    }
    // Simple: no time-differential available without git history, return 0
    Ok(KpiValue {
        value: 0.0,
        unit: "seconds".to_string(),
        evidence_paths: vec![],
    })
}

/// Compute the incident count KPI: number of incident JSON files.
pub fn kpi_incidents(root: &Path, _window: &Duration) -> HxResult<KpiValue> {
    let dir = root.join(".harness").join("artifacts").join("incidents");
    if !dir.is_dir() {
        return Ok(KpiValue {
            value: 0.0,
            unit: "count".to_string(),
            evidence_paths: vec![],
        });
    }
    let mut count = 0u64;
    if let Ok(rd) = fs::read_dir(&dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                count += 1;
            }
        }
    }
    Ok(KpiValue {
        value: count as f64,
        unit: "count".to_string(),
        evidence_paths: vec![],
    })
}

/// Compute the budget exhaustion rate KPI: budget interventions divided by episodes.
pub fn kpi_budget_exhaustion(root: &Path, _window: &Duration) -> HxResult<KpiValue> {
    let eps = list_episode_files(root);
    let total_eps = eps.len() as f64;
    if total_eps == 0.0 {
        return Ok(KpiValue {
            value: 0.0,
            unit: "rate".to_string(),
            evidence_paths: vec![],
        });
    }
    let budget_count = count_budget_interventions(root) as f64;
    let rate = budget_count / total_eps;
    Ok(KpiValue {
        value: rate,
        unit: "rate".to_string(),
        evidence_paths: vec![],
    })
}

/// Compute the intervention frequency KPI: interventions divided by episodes.
pub fn kpi_intervention_frequency(root: &Path, _window: &Duration) -> HxResult<KpiValue> {
    let eps = list_episode_files(root);
    let total_eps = eps.len() as f64;
    if total_eps == 0.0 {
        return Ok(KpiValue {
            value: 0.0,
            unit: "rate".to_string(),
            evidence_paths: vec![],
        });
    }
    let interventions = list_intervention_files(root).len() as f64;
    let rate = interventions / total_eps;
    Ok(KpiValue {
        value: rate,
        unit: "rate".to_string(),
        evidence_paths: vec![],
    })
}

fn kpi_target(id: &str) -> f64 {
    match id {
        "merge_rate" => 0.80,
        "rollback_rate" => 0.05,
        "first_pass_rate" => 0.70,
        "gate_flip_rate" => 0.03,
        "attribution_accuracy" => 0.85,
        "verification_completeness" => 1.0,
        "cpmc" => 0.0,
        "ttm" => 0.0,
        "incidents" => 5.0,
        "budget_exhaustion" => 0.05,
        "intervention_frequency" => 0.20,
        _ => 0.0,
    }
}

#[allow(dead_code)]
fn kpi_unit(id: &str) -> String {
    match id {
        "cpmc" => "usd_per_call".to_string(),
        "incidents" => "count".to_string(),
        "ttm" => "seconds".to_string(),
        _ => "rate".to_string(),
    }
}

fn kpi_name(id: &str) -> String {
    match id {
        "merge_rate" => "Merge rate".to_string(),
        "rollback_rate" => "Rollback rate".to_string(),
        "first_pass_rate" => "First-pass gate pass rate".to_string(),
        "gate_flip_rate" => "Gate-flip rate".to_string(),
        "attribution_accuracy" => "Attribution accuracy".to_string(),
        "verification_completeness" => "Verification completeness".to_string(),
        "cpmc" => "Cost per merged change".to_string(),
        "ttm" => "Time-to-merge".to_string(),
        "incidents" => "Incident count".to_string(),
        "budget_exhaustion" => "Budget exhaustion rate".to_string(),
        "intervention_frequency" => "Intervention frequency".to_string(),
        _ => id.to_string(),
    }
}

/// Build a Kpi from a KpiValue and metadata.
fn build_kpi(id: &str, val: KpiValue, has_data: bool, target: f64) -> Kpi {
    let state = if !has_data {
        "cold-start".to_string()
    } else {
        "live".to_string()
    };
    let breached = has_data && val.value < target;
    Kpi {
        id: id.to_string(),
        name: kpi_name(id),
        value: val.value,
        unit: val.unit,
        target,
        breached,
        evidence_paths: val.evidence_paths,
        state,
    }
}

type KpiFn = fn(&Path, &Duration) -> HxResult<KpiValue>;

/// Run all 11 KPI functions and compute a full rollup. Writes the result to
/// `.harness/artifacts/metrics/<YYYY-MM>/<YYYY-MM-DD>.json`.
///
/// Returns the Rollup struct so callers can also inspect it in memory.
pub fn rollup(root: &Path, window: &Duration) -> HxResult<Rollup> {
    let ws = window_secs(window);
    let days = ws / 86400;

    // Gather KPI values
    let kpi_fns: Vec<(&str, KpiFn)> = vec![
        ("merge_rate", kpi_merge_rate as KpiFn),
        ("rollback_rate", kpi_rollback_rate),
        ("first_pass_rate", kpi_first_pass_rate),
        ("gate_flip_rate", kpi_gate_flip_rate),
        ("attribution_accuracy", kpi_attribution_accuracy),
        ("verification_completeness", kpi_verification_completeness),
        ("cpmc", kpi_cpmc),
        ("ttm", kpi_ttm),
        ("incidents", kpi_incidents),
        ("budget_exhaustion", kpi_budget_exhaustion),
        ("intervention_frequency", kpi_intervention_frequency),
    ];

    // Determine has_data per KPI (had entries/episodes to compute from)
    let entries = read_changelog_entries(root);
    let eps = list_episode_files(root);
    let has_entries = !entries.is_empty();
    let has_episodes = !eps.is_empty();
    let has_spend = {
        let dir = crate::store::spend_dir(root);
        dir.is_dir()
            && fs::read_dir(&dir)
                .map(|mut r| r.next().is_some())
                .unwrap_or(false)
    };
    let has_incidents = {
        let dir = root.join(".harness").join("artifacts").join("incidents");
        dir.is_dir()
            && fs::read_dir(&dir)
                .map(|mut r| r.next().is_some())
                .unwrap_or(false)
    };
    let has_interventions = {
        let dir = crate::store::interventions_dir(root);
        dir.is_dir()
            && fs::read_dir(&dir)
                .map(|mut r| r.next().is_some())
                .unwrap_or(false)
    };

    let mut kpis = Vec::new();
    for (id, func) in &kpi_fns {
        let val = func(root, window)?;
        let target = kpi_target(id);
        let has_data = match *id {
            "merge_rate" | "rollback_rate" | "first_pass_rate" | "ttm" => has_entries,
            "gate_flip_rate" | "verification_completeness" | "intervention_frequency" => {
                has_episodes
            }
            "attribution_accuracy" => false,
            "cpmc" => has_spend,
            "incidents" => has_incidents,
            "budget_exhaustion" => has_interventions,
            _ => false,
        };
        kpis.push(build_kpi(id, val, has_data, target));
    }

    // Count cold-start KPIs
    let cold_count = kpis.iter().filter(|k| k.state == "cold-start").count();
    let status = if cold_count >= 8 {
        "cold-start".to_string()
    } else if cold_count >= 1 {
        "partial".to_string()
    } else {
        "live".to_string()
    };

    let rollup = Rollup {
        schema_version: 1,
        produced_at: now_iso(),
        window: Window { days },
        kpis,
        status,
    };

    // Write to artifact path
    let artifact_dir = root
        .join(".harness")
        .join("artifacts")
        .join("metrics")
        .join(now_ym());
    fs::create_dir_all(&artifact_dir)?;
    let artifact_path = artifact_dir.join(format!("{}.json", now_ymd()));
    let json = serde_json::to_string_pretty(&rollup)?;
    fs::write(&artifact_path, &json)?;

    Ok(rollup)
}

/// Evaluate a completed rollup and produce alerts for any breached thresholds.
pub fn evaluate_alerts(rollup: &Rollup, _root: &Path) -> Vec<Alert> {
    let mut alerts = Vec::new();

    for kpi in &rollup.kpis {
        if kpi.breached {
            alerts.push(Alert {
                kind: format!("kpi_breach_{}", kpi.id),
                severity: "warn".to_string(),
                message: format!(
                    "KPI {} value {:.2} below target {:.2}",
                    kpi.name, kpi.value, kpi.target
                ),
                suggested_action: format!(
                    "Investigate {} degradation. Check {} evidence paths.",
                    kpi.name,
                    kpi.evidence_paths.len()
                ),
            });
        }
    }

    // Check incidents count
    for kpi in &rollup.kpis {
        if kpi.id == "incidents" && kpi.value > 0.0 {
            alerts.push(Alert {
                kind: "incident_count".to_string(),
                severity: "critical".to_string(),
                message: format!("{} incident(s) detected in rollup window", kpi.value as u64),
                suggested_action:
                    "Review incidents in .harness/artifacts/incidents/ and take corrective action."
                        .to_string(),
            });
        }
    }

    alerts
}
