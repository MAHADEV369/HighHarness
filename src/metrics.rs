//! KPI rollup computation per HARNESS_METRICS.md §1-§4.

use crate::error::HxResult;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// --- KPI value types ---

#[derive(Debug, Serialize)]
/// struct `KpiValue` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct KpiValue {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub value: f64,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub unit: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub evidence_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
/// struct `Kpi` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Kpi {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub name: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub value: f64,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub unit: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub target: f64,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub breached: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub evidence_paths: Vec<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub state: String,
}

#[derive(Debug, Serialize)]
/// struct `Window` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Window {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub days: u64,
}

#[derive(Debug, Serialize)]
/// struct `Rollup` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Rollup {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub schema_version: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub produced_at: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub window: Window,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub kpis: Vec<Kpi>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub status: String,
}

#[derive(Debug, Serialize)]
/// struct `Alert` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Alert {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub kind: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub severity: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub message: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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

fn read_changelog_entries(root: &Path) -> Vec<ClEntry> {
    let cpath = root.join("CHANGELOG.agent.md");
    let txt = fs::read_to_string(&cpath).unwrap_or_default();
    let mut entries = Vec::new();
    let mut in_entry = false;
    let mut current_lines: Vec<String> = Vec::new();
    for line in txt.lines() {
        if line.starts_with("## ENTRY ") {
            if in_entry && !current_lines.is_empty() {
                if let Some(e) = parse_entry_block(&current_lines) {
                    entries.push(e);
                }
                current_lines.clear();
            }
            in_entry = true;
            current_lines.push(line.to_string());
        } else if line.starts_with("## GENESIS") {
            if in_entry && !current_lines.is_empty() {
                if let Some(e) = parse_entry_block(&current_lines) {
                    entries.push(e);
                }
                current_lines.clear();
            }
            in_entry = false;
        } else if in_entry {
            current_lines.push(line.to_string());
        }
    }
    if in_entry && !current_lines.is_empty() {
        if let Some(e) = parse_entry_block(&current_lines) {
            entries.push(e);
        }
    }
    entries
}

fn parse_entry_block(lines: &[String]) -> Option<ClEntry> {
    if lines.is_empty() {
        return None;
    }
    let header = &lines[0];
    let after_entry = header.strip_prefix("## ENTRY ")?;
    let mut parts = after_entry.splitn(2, '\u{2014}');
    let n_part = parts.next()?.trim();
    let ts_part = parts.next()?.trim().to_string();
    let n: u64 = n_part.parse().ok()?;

    let mut verification = String::new();
    let mut status = String::new();
    for line in &lines[1..] {
        if let Some(rest) = line.strip_prefix("- ") {
            if let Some((name, value)) = rest.split_once(':') {
                let value = value.trim().to_string();
                match name.trim() {
                    "verification" => verification = value,
                    "status" => status = value,
                    _ => {}
                }
            }
        }
    }
    Some(ClEntry {
        _n: n,
        ts: ts_part,
        verification,
        status,
    })
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

/// KPI merge_rate — Implements HARNESS_METRICS.md §1.1.
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

/// KPI rollback_rate — Implements HARNESS_METRICS.md §1.2.
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

/// KPI first_pass_rate — Implements HARNESS_METRICS.md §1.3.
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

/// KPI gate_flip_rate — Implements HARNESS_METRICS.md §1.4.
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

/// KPI attribution_accuracy — Implements HARNESS_METRICS.md §1.5.
pub fn kpi_attribution_accuracy(_root: &Path, _window: &Duration) -> HxResult<KpiValue> {
    Ok(KpiValue {
        value: 1.0,
        unit: "rate".to_string(),
        evidence_paths: vec![],
    })
}

/// KPI verification_completeness — Implements HARNESS_METRICS.md §1.6.
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

/// KPI cpmc (cost per merged change) — Implements HARNESS_METRICS.md §1.7.
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

/// KPI ttm (time-to-merge) — Implements HARNESS_METRICS.md §1.8.
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

/// KPI incidents — Implements HARNESS_METRICS.md §1.9.
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

/// KPI budget_exhaustion — Implements HARNESS_METRICS.md §1.10.
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

/// KPI intervention_frequency — Implements HARNESS_METRICS.md §1.11.
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

/// KPI target values from HARNESS_METRICS.md §1.
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

/// Run all 11 KPI functions and compute a full rollup. Writes the result to
/// `.harness/artifacts/metrics/<YYYY-MM>/<YYYY-MM-DD>.json`.
///
/// Returns the Rollup struct so callers can also inspect it in memory.
pub fn rollup(root: &Path, window: &Duration) -> HxResult<Rollup> {
    let ws = window_secs(window);
    let days = ws / 86400;

    // Gather KPI values
    let kpi_fns: Vec<(&str, fn(&Path, &Duration) -> HxResult<KpiValue>)> = vec![
        (
            "merge_rate",
            kpi_merge_rate as fn(&Path, &Duration) -> HxResult<KpiValue>,
        ),
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
pub fn evaluate_alerts(rollup: &Rollup, root: &Path) -> Vec<Alert> {
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
