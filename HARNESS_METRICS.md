# HARNESS_METRICS.md

**How the harness measures itself.**

`HARNESS_ENGINEERING.md` opens by citing the ~46% agent-change rejection rate. That number is the motivation; this file is the metric layer that turns the motivation into a measured, improving quantity. A harness that does not measure itself cannot prove it is not still at 46%. A harness that does not improve its metrics is, by its own §1.8 (Verification) standard, unverifiable.

This file defines the KPI set, the collection mechanism, the dashboards, the alert conditions, the eval harness, and the review cadence. It is binding: gaps flagged by these metrics MUST either be acted on within the documented cadence or formally accepted as a known-limitation entry in `HARNESS_VERSIONING.md`.

Paired with:
- `HARNESS_PRIMITIVES.md` — produces the raw measurements (spend ledger, tool-call ledger, changelog, episodes, incidents).
- `HARNESS_SECURITY.md` — supplies the security-relevant metrics and the incident log format.

---

## 0. Non-negotiable mandates

1. **Measure or admit.** Every KPI in §1 either has a working meter or an explicit `not-yet-metered` entry in §8 with a target date.
2. **No vanity metrics.** Lines of code, number of commits, number of tool calls, raw model calls per day are NOT KPIs. They are inputs, at best.
3. **No denominator games.** The denominator-effect anti-pattern (§11 of the binding spec) is enforced numerically: rate KPIs use denominator = pre-change baseline, never denominator = post-change total volume.
4. **Cost is real.** Money spent is tracked to the cent by `spend/<YYYY-MM>.jsonl` and reconciled against provider invoices monthly. Discrepancies > 5% are incidents.
5. **Honest attribution.** When a metric regresses, the harness reports the regression, attributes it (agent/spec/env/flaky/pre-existing, mirroring the §7 fault table), and records the evidence. Hiding a regression is a silent-action violation.

---

## 1. The KPI set

Eleven KPIs. Each has: id, question it answers, formula, data source, target, alert.

### 1.1 Merge-rate (a.k.a. acceptance rate)

- **Question:** Of agent-proposed changes, what fraction survive review?
- **Formula:** `merge-rate = changes-merged / changes-proposed` where a proposed change is a `CHANGELOG.agent.md` entry reaching `status != reverted|denied`. Merged = entry still in HEAD after 7 days without revert.
- **Source:** `CHANGELOG.agent.md` + git history.
- **Target:** ≥ 80% (i.e., ≤ 20% rejection, down from the 46% baseline). Stretch ≥ 90%.
- **Alert:** trailing-30-day rate < 70% for two consecutive weeks.

### 1.2 Rollback rate

- **Question:** Of merged changes, what fraction get reverted later?
- **Formula:** `rollback-rate = reverted-within-30d / merged`.
- **Source:** `CHANGELOG.agent.md` entries with `status = reverted` whose `intent` cites an earlier entry id; plus git reverts/`git revert`.
- **Target:** ≤ 5%.
- **Alert:** trailing-30-day rate > 10%.

### 1.3 First-pass gate pass rate

- **Question:** How often does a change pass all four gates on first run?
- **Formula:** `fp-pass = runs-with-no-failed-gate / total-runs`.
- **Source:** episode `Verification report` sections.
- **Target:** ≥ 70% first-pass.
- **Alert:** trailing-30-day < 50%.

### 1.4 Gate-flip rate

- **Question:** How often does a gate that was initially green go red on re-run (indicates non-determinism, flaky tests, or environment drift)?
- **Formula:** `gate-flip = gate-passed-then-failed / gate-evaluations`.
- **Source:** `GateResult` records cross-referenced by snapshot id.
- **Target:** ≤ 3%.
- **Alert:** trailing-7-day > 8%.

### 1.5 Attribution accuracy

- **Question:** When the agent says "agent error" or "spec error", is it right?
- **Formula:** `attribution-accuracy = locus-correct-by-human / locus-judgments-human-reviewed`. Sampled: every reverted change and a random 10% of merged changes are human-reviewed within 7 days.
- **Source:** review queue + human-verdict records.
- **Target:** ≥ 85%.
- **Alert:** trailing-30-day < 70%.

### 1.6 Verification completeness

- **Question:** Does the episode actually contain all four gate reports + the six completion requirements?
- **Formula:** `verif-completeness = complete-episodes / total-episodes` where "complete" = all 10 checklist items answered.
- **Source:** episode files.
- **Target:** 100%.
- **Alert:** any incomplete episode > 24h old.

### 1.7 Cost per merged change

- **Question:** Is the harness getting cheaper at producing accepted work?
- **Formula:** `cpmc = spend(usd) over merged changes / merged-changes`. Counts all spend attributable to the change-run + revision runs + review runs, traced via `run_id` and `metadata.run_id`.
- **Source:** `spend/<YYYY-MM>.jsonl`.
- **Target:** monotone non-increasing over a 90-day window (per feature: autocomplete, chat, agent, review, embed).
- **Alert:** rolling-30-day cpmc up > 25% vs the previous 30-day window.

### 1.8 Time-to-merge

- **Question:** From "task given" to "merge accepted", how long?
- **Formula:** `ttm = median(merged_at - first_tool_call_at)` per change.
- **Source:** episode timestamps + git history.
- **Target:** no target; track and surface. Used as a sanity signal vs cpmc — falling cpmc with rising ttm is bad efficiency, not good.
- **Alert:** week-over-week increase > 50% with no corresponding scope increase.

### 1.9 Incident rate (security)

- **Question:** How often does the harness detect F1–F4 or a vector from `HARNESS_SECURITY.md`?
- **Formula:** `incidents-30d = count(incidents in trailing 30 days)` and `incidents-with-impact = incidents-30d where an action was executed before detection`.
- **Source:** `.harness/artifacts/incidents/*.json` rolled up.
- **Target:** `incidents-with-impact = 0` (hard). `incidents-30d` target < 5 (detections without impact are healthy).
- **Alert:** any incident with impact is an immediate incident (§9 of `HARNESS_SECURITY.md`).

### 1.10 Budget exhaustion rate

- **Question:** How often does a run hit a hard budget?
- **Formula:** `budget-exhausted-runs = runs-ending-due-to-budget / total-runs`.
- **Source:** `interventions/*.json` with `kind = budget_exhausted`.
- **Target:** < 5%. High rate means budgets are miscalibrated, not that users are "spending too much".
- **Alert:** > 15% for trailing 14 days triggers a budget recalibration suggestion to the human.

### 1.11 Intervention frequency

- **Question:** How often does a human need to pause/correct/cancel an agent run?
- **Formula:** `intervened-runs = runs-with-any-intervention / total-runs`.
- **Source:** `interventions/*.json`.
- **Target:** < 20% sustained. Occasional intervention is normal; high rates indicate drift or approval friction.
- **Alert:** > 40% for trailing 14 days. Also alert on the inverse: < 1% for trailing 14 days, which suggests approvals are being auto-resolved or silently skipped (audit risk).

---

## 2. Collection mechanism

### 2.1 Production meters

Each KPI's source is a concrete artifact produced by `HARNESS_PRIMITIVES.md`. There are no separate "metrics pipelines". The meters are:

| Meter | Produces | Where |
|---|---|---|
| Spend ledger | one row per inference | `.harness/artifacts/spend/<YYYY-MM>.jsonl` |
| Tool-call ledger | one row per `tools.invoke` | `.harness/artifacts/tool-calls.jsonl` |
| Episode writer | one file per run | `logs/episodes/<run-id>.md` |
| Changelog appender | one row per change | `CHANGELOG.agent.md` |
| Snapshot store | one record per snapshot | `.harness/artifacts/snapshots/<run-id>/` |
| Approvals store | one file per approval | `.harness/artifacts/approvals/<id>.json` |
| Interventions store | one file per intervention | `.harness/artifacts/interventions/<id>.json` |
| Incidents store | one file per incident | `.harness/artifacts/incidents/<id>.json` |
| Harness integrity log | one line per harness event | `.harness/artifacts/harness.log` |

### 2.2 Aggregation

A scheduled rollup task reads the meters (read-only) and writes read-only dashboards under `.harness/artifacts/metrics/<YYYY-MM>/<YYYY-MM-DD>.json` and `.html`. Rollup windows: daily, weekly, monthly, trailing-30-day.

Rollups are themselves recorded in the harness integrity log. Rollup code is small, signed, and read-only.

### 2.3 Sampling rules

- **KPIs 1.1, 1.2, 1.3, 1.6, 1.7, 1.8, 1.10, 1.11**: 100% sampling from artifacts. No estimation.
- **KPI 1.5 (attribution accuracy)**: judged sample — every reverted change in 7 days plus a random 10% of merged changes. Sample selection seeds are logged.
- **KPI 1.4 (gate-flip)**: every gate evaluation is recorded; cross-referencing is deterministic.

### 2.4 Privacy & retention

- Episodes + tool calls default to 90-day retention (see `HARNESS_PRIMITIVES.md` §11). Cost rollups are 2 years. Incidents are indefinite.
- Redacted content stays redacted in rollups. No re-identification.
- Telemetry that leaves the device is opt-in and is limited to rollup JSON (no episode content, no tool args, no memory bodies).

---

## 3. Dashboards

### 3.1 Inside the IDE

A built-in "Harness Health" panel in the secondary side bar surface:
- KPI sparklines (trailing 30 days).
- Spend waterfall per feature per day.
- Gate-flip heat map (phase × day).
- Active approval queue with priority sort.
- Regression log: last 10 reverted changes with one-click jump to their episode.
- Live incident indicator.

### 3.2 Org / team (opt-in)

A web dashboard synced from the local rollup JSONs (user-owned sink). Provides:
- Per-team cpmc trends.
- Attribution accuracy trends.
- Top regression sources (files / symbols / phases).
- Budget recapture opportunities.

### 3.3 Per-project README digest

A self-updating `HARNESS_HEALTH.md` (gitignored) committed only by deliberate choice. Digest of the current KPI snapshot for new contributors. One screen, no fluff.

---

## 4. Alerting

Alerts fire from the rollup task into the harness integrity log + the IDE notification surface. Alert types:

| Type | Channel | Severity |
|---|---|---|
| KPI threshold breached | IDE notification + harness integrity log | info |
| KPI trend breach (week-over-week or month-over-month) | IDE notification + rollup JSON | warn |
| Security incident with impact | All channels + configured security contact | critical |
| Rollup task failure | all-KPIs-stale warning in status bar | warn |
| Sample review overdue | notification to assigned reviewer | warn |

Alerts include: KPI id, current value, target, threshold crossed, trailing window length, link to the rollup artifact, and a suggested next action. Alerts that lack a suggested next action are treated as malformed.

No alert auto-resolves. Acknowledgement is a human action recorded in `harness.log`.

---

## 5. Eval harness

### 5.1 Purpose

A deterministic offline harness that exercises the agent runtime against a frozen set of tasks with golden outcomes. Used:
- Before the harness releases a new version (`HARNESS_VERSIONING.md` §4).
- On any change to `HARNESS_PRIMITIVES.md`'s interfaces, `HARNESS_SECURITY.md` mitigation set, or `HARNESS_ENGINEERING.md` rules.

### 5.2 Suite format

`.harness/evals/<eval-id>/`:

```
task.md            # the task spec, frozen
repo/              # a frozen fixture repo (git submodule or tarball)
golden/            # golden state: feature files that should now exist/be modified
forbidden/          # files or symbol-shapes that must NOT appear
budget.toml        # token + tool-call + wallclock caps
gates.toml         # phase → command overrides; usually inherits
rubric.toml        # rubric: which gates weigh how; which KPI thresholds gate the eval
```

The eval harness:

1. Restores `repo/` from the frozen fixture.
2. Boots the agent with the task spec; routing policy forced to a recorded `models.toml` snapshot.
3. Runs to completion or budget exhaustion.
4. Diffs against `golden/` and `forbidden/`.
5. Compiles the produced KPIs from the run's episodes/changelog/spend.
6. Emits an eval result under `.harness/artifacts/evals/<eval-id>/<timestamp>.json`.

### 5.3 Pass criteria

An eval run passes iff:

- every gate pass/fail matches `golden/gates.toml` expectations,
- no `forbidden/` predicate fired,
- the rubric's KPI thresholds were met (e.g., `attribution-accuracy ≥ rubric.min_accuracy`),
- the run completed without an incident with impact, and
- cost within `rubric.max_cpmc` of the median of the prior 10 successful eval runs.

### 5.4 Suite growth

Every reverted real-world change is a candidate for becoming a regression eval: the failing task is captured (with redactions) and added to `.harness/evals/regressions/<n>/`. The suite MUST grow only from real incidents or eval-generated test cases, never from synthetic hypotheticals without an incident source.

---

## 6. Review cadence

| Cadence | Activity | Owner |
|---|---|---|
| Daily | acknowledgement of any open alert | on-call agent maintainer |
| Weekly | walk the trailing-7-day incident log | security+engineering |
| Bi-weekly | attribution-accuracy sample review + verdicts | human reviewers |
| Monthly | KPI trend review + budget recalibration | human + rollup dashboards |
| Quarterly | eval suite audit + harness version bump gate | engineering |
| Annual | full threat-model review against `HARNESS_SECURITY.md` | engineering + security |

Skipping a cadence entry is recorded in `harness.log` as `cadence-miss`. Two consecutive misses on security cadences open a `sustained-cadence-miss` incident.

---

## 7. Incident log format

`.harness/artifacts/incidents/<incident-id>.json`:

```json
{
  "schema_version": 1,
  "incident_id": "inc_2026-06-29T1432_001",
  "detected_at": "…",
  "detection_rule": "F4-wrong-target-exfil",
  "vector": "V4.4-tool-argument-exfiltration",
  "run_id": "…",
  "agent_id": "…",
  "model_id": "…",
  "model_route": "claude-opus-4",
  "severity": "high",
  "had_impact": false,
  "impact_summary": "",
  "evidence_paths": ["…"],
  "quarantined_sources": ["mcp.linear-local-mirror"],
  "postmortem": { "root_cause": "…", "fix": "…", "fix_commit": "…" },
  "status": "open|mitigated|closed"
}
```

Incidents roll up into §1.9. The incident log itself is signed entry-by-entry (rolling hash) like the changelog.

---

## 8. Known limitations (admitted-not-metered)

A section the harness carries explicitly. An entry here is not a failure; lack of one when metrics are incomplete is.

| KPI | Limitation | Target date | Owner |
|---|---|---|---|
| 1.5 attribution accuracy | sample size <10 in early weeks; confidence wide | 2026-09-30 | metrics |
| 1.8 time-to-merge | excludes human-review wall-clock when reviewer is offline | 2026-08-15 | metrics |
| 1.4 gate-flip | cannot yet distinguish env-drift from model nondeterminism | 2026-10-01 | runtime |

Each entry is dated and owned. Stale entries > target date escalate to a `cadence-miss`.

---

## 9. Quick reference

```
KPIs           merge-rate · rollback · first-pass · gate-flip · attribution · verif-completeness
               · cpmc · ttm · incidents · budget-exhaustion · intervention-frequency
METERS         spend · tool-calls · episodes · changelog · snapshots · approvals · interventions · incidents · harness.log
ROLLUP         daily/weekly/monthly/trailing-30d, signed, read-only, JSON + HTML
EVALS          frozen tasks, golden state, forbidden predicates, rubric thresholds
PASSES         all gates match golden · no forbidden · KPI thresholds met · no impact-incident · cost within ±N
ALERTING       KPI threshold · trend · incident · cadence-miss · rollup-failure; no auto-resolve
CADENCE        daily ack · weekly incidents · bi-weekly attribution · monthly trends · quarterly version
REPUDIATION    no vanity metrics, no denominator games, no hidden regressions
```

---

*End of metrics. See `HARNESS_VERSIONING.md` for how the harness upgrades itself and validates its own first run.*