# Episode _EXAMPLE

> **Superseded as canonical real-world reference** by `logs/episodes/20260629T095108Z-add-version-flag-agent-pin0.md` (canonical Entry 1 in this repo's own `CHANGELOG.agent.md`, produced by `make entry-1-demo` with `--pin`). This file remains as the static spec-compliant template; copy it for new runs that need a starting template.
>
> **This is a template, not a real run.** Copy it as `logs/episodes/<ISO-8601 timestamp>-<short-slug>-<agent-short>-<rand4>.md` and fill in real values. Remove this blockquote once you start a real run.
>
> This template is **spec-compliant**: every section required by `HARNESS_ENGINEERING.md` §5 is present, the §2 pre-task checklist is placed *before* the first tool call (§2 forbids placing it after), the §3 verification report covers **all ten** items (not six), and `## Files touched` includes every file the run actually wrote — including `CHANGELOG.agent.md` and this episode file itself. Semantic-gate evidence (§6.3) is orthogonal to the functional-gate evidence (§6.2): the functional gate runs a single regression test; the semantic gate cites git-blame reasoning + a typed-AST hash, **not** the same test.
>
> The scenario assumes a fictitious `src/auth/session.js` token-leak fix, **standard tier** (§16.1).

---

## Task spec

Fix the session token leak: refresh tokens are logged on error in the auth module. Tokens must never appear in logs. Add a regression test.

Completion criteria (one sentence): no `refreshToken` substring appears in any log-call site in `src/**`; a new test in `tests/auth.spec.js` asserts the absence; the existing 140-test suite continues to pass.

**Tier:** standard (touches `src/auth/**`, which is a safety-critical path glob → §16 forced upgrade applies; see Interventions).

## Plan

Decomposition (each atom has its own pass/fail criterion — §1.12):

1. **Locate leak.** Pass criterion: identify the exact line(s) where the token reaches a log call.
2. **Redact.** Pass criterion: replace token with opaque error id; log call no longer contains the token; no other behavior change in the function.
3. **Regression test.** Pass criterion: a new test that fails on the old code (token in log) and passes on the new code (no token); the test is added under `tests/auth.spec.js`.
4. **Run full suite.** Pass criterion: all previously-passing tests still pass + new test passes.

**Per-task exploration budget (§1.13):** max 25 tool calls, max 8 files traversed, ~6 K tokens, wallclock 10 min. Stop and request budget revision if exceeded.

## Pre-task checklist (§2 — completed before any tool call)

1. Task restated with completion criteria: **Y** — "Patch leak + regression test, no token in logs, suite green." (above)
2. Checked `CHANGELOG.agent.md` + `logs/episodes/` for prior work: **Y** — `changelog.latest()` returns the GENESIS marker; no prior entries; no episodes in `logs/episodes/` other than `_EXAMPLE.md`; no prior work on `src/auth/`.
3. Confirmed files exist by direct observation: **Y** — `fs.read` of `src/auth/session.js` and `tests/auth.spec.js` confirmed before any edit (see Tool calls 1–2).
4. Implicit constraints identified: **Y** — backward compat: the redacted log line must still correlate client errors to server logs, so replacement must carry an opaque id (not delete the log line). Code style: 2-space, single quotes, consistent with the file. Security: `src/auth/**` is a §16 safety-critical trigger → forced upgrade applies before any edit.
5. Task-state log initialized: **Y** — see below.
6. Test runner + tests known: **Y** — `npm test` (mocha), `tests/auth.spec.js`.
7. Dangerous operations listed: **Y** — `fs.edit` on `src/auth/**` (safety-critical path → `ask` per `R-SAFETY-CRITICAL` rule, `HARNESS_PRIMITIVES.md` §2.9). Approval requests planned at tool calls 3 and 6.
8. Verification plan covering all four gates: **Y** — syntactic: `node -c`; functional: new redaction test + full suite; semantic: spec→outcome mapping with orthogonal evidence (git-blame reasoning + typed-AST hash, NOT the functional test); regression: 140 passing → 141 passing (new test added).
9. Decomposed to verifiable grain with baseline snapshot: **Y** — see Plan; baseline snapshot taken at `snapshot.take(run_id, "baseline")` immediately after this checklist; baseline = 140/140 passing, recorded as `snap_<run_id>_baseline`.
10. Per-task exploration budget stated: **Y** — max 25 tool calls / 8 files / 6 K tokens / 10 min (above).
11. Product context check: **Y** — searched for RFC / decision records in `docs/` and `.decisions/`; none exist for auth logging; assumption stated here.

## Task state log

| timestamp | subtask | status |
|---|---|---|
| T+00m | 0 Tier upgrade triggered by §16 forced-upgrade | done (intervention INT-1) |
| T+00m | 0 Baseline snapshot | done (snap baseline) |
| T+01m | 1 Locate leak | in_progress |
| T+02m | 1 Locate leak | done |
| T+02m | 2 Redact | blocked (awaiting approval appr_…_001) |
| T+03m | 2 Redact | unblocked → in_progress |
| T+04m | 2 Redact | done |
| T+04m | 3 Regression test | blocked (awaiting approval appr_…_002) |
| T+05m | 3 Regression test | unblocked → in_progress |
| T+06m | 3 Regression test | done |
| T+06m | 4 Run full suite | blocked (env: missing dev dep) → clarification requested (clr_…_001) |
| T+06m | 4 Run full suite | blocked → clarification resolved (lockfile fixed by human out-of-band) |
| T+07m | 4 Run full suite | unblocked → in_progress |
| T+09m | 4 Run full suite | done |

## Tool calls

| # | tool | args (summary) | scope_narrow | approval | result summary | duration_ms |
|---|---|---|---|---|---|---|
| 1 | `fs.read` | `path="src/auth/session.js", offset=70, limit=25` | — | auto (read-only) | returned 25 lines; `console.error(err.refreshToken)` at line 82 confirmed | 8 |
| 2 | `fs.read` | `path="tests/auth.spec.js", limit=30` | — | auto | existing redaction test slot pattern found | 6 |
| 3 | `fs.edit` | `path="src/auth/session.js", old="console.error(err.refreshToken)", new="console.error('auth_error_id=' + err.id)"` | `{paths:["src/auth/session.js"]}` | **ask** → appr_…_001 approved | 1 line changed | 2401 |
| 4 | `fs.hash` | `path="src/auth/session.js"` | — | auto | `sha256:9f…` (post-edit; for semantic-gate comparison vs baseline) | 4 |
| 5 | `git.blame` | `path="src/auth/session.js", line=82` | — | auto | last touched by `feat(auth): error path` commit `<sha>`, 2025-11-04; intent of the line was diagnostics, not secret exposure | 90 |
| 6 | `fs.edit` | `path="tests/auth.spec.js", insert_after=10, body="…assert.strictEqual(logOutput.includes(refreshToken), false)…"` | `{paths:["tests/auth.spec.js"]}` | **ask** → appr_…_002 approved | 1 test block added | 1820 |
| 7 | `permissions.check` | `tool="shell.exec", args=["npm","test"]` | — | decision=allow (rule R-SHELL-TEST) | decision returned | 4 |
| 8 | `shell.exec` | `cmd="npm test"` | `{paths:["."], env:["NODE_ENV","PATH"]}` | auto (per R-SHELL-TEST) | FAILED: `mocha: command not found` (env error — see Failures) | 980 |
| 9 | `clarification.request` | `question="npm test fails with 'mocha: not found' but package.json lists mocha in devDependencies — lockfile appears stale. Pausing per §7 env locus. Fix lockfile out-of-band?"`, `urgency="blocking"` | — | — | returned `clr_…_001` | 2 |
| 10 | (run blocked; human fixes lockfile out-of-band) | — | — | — | `clarification.resolve(clr_…_001, { answer:"lockfile fixed; rerun npm test", resolution_kind:"env-fix" })` | — |
| 11 | `shell.exec` | `cmd="npm test"` | `{paths:["."], env:["NODE_ENV","PATH"]}` | auto | 141/141 passing (140 prior + 1 new redaction test) | 18320 |
| 12 | `shell.exec` | `cmd="node -c src/auth/session.js && npm run lint"` | `{paths:["."], env:["PATH"]}` | auto | no syntax errors, 0 lint errors | 2105 |
| 13 | `snapshot.diff` | `{before:"snap_<run-id>_baseline", after:"current"}` | — | auto | tests 140→141 pass (added, no regressions); types hash unchanged (file is JS); lint hash unchanged | 22 |
| 14 | `changelog.append` | entry (see Changelog block below) | — | `compare-and-append` lock acquired first try | `this_hash: <hex>` | 18 |

Total tool calls: 14 / 25 budget. Files traversed: 3 / 8. Estimated tokens: ~3 K / 6 K. Wallclock: 9 min / 10 min. **Under budget** (§1.13).

## Decisions

- **D1 — Redacted via error id, not full removal.** The log line carries diagnostic value (correlating client errors to server logs via `err.id`) that wholesale deletion would lose. Chosen over silent removal because logs need to remain auditable. Reason recorded for any future run that sees the redaction and wonders why.
- **D2 — Tier-upgraded to `safety-critical` before first edit (`src/auth/**` trigger).** Per §16.3 forced-upgrade flow. Recorded as Intervention INT-1 below. Re-ran the §2 checklist at the new tier (all 11 items).
- **D3 — Blocked on env error, did NOT paper-over with `npm install --no-save mocha`.** §7 names "environment error → Report; do not paper over with code changes." Previous versions of this template did exactly that and taught a spec violation. The corrected behavior is: log failure → classify env → `clarification.request` with `urgency=blocking` → wait for human env-fix → resume.
- **D4 — Semantic evidence chosen to be orthogonal to functional evidence (§6.3).** Functional = new redaction test. Semantic = (a) `git.blame` on line 82 confirming the original line's intent was diagnostics (not secret display), making the redaction semantically faithful, plus (b) typed-AST hash of `src/auth/session.js` showing the only structural change is the argument to one `CallExpression` (no control-flow change). Neither reuses the functional test.

## Failures

| id | what | locus | evidence | resolution |
|---|---|---|---|---|
| F-1 | `npm test` failed: `mocha: not found` | **env** (per §7) | tool call 8 output: `sh: mocha: command not found`; `package.json` lists `mocha` under `devDependencies`; `package-lock.json` timestamp pre-dates the `mocha` entry → lockfile stale | Did NOT modify `package.json` or install with `--no-save` (that would be the §7 forbidden paper-over). Filed a blocking clarification (clr_…_001, see Decisions D3). Human fixed lockfile out-of-band (`npm ci`). Re-ran `npm test` (tool call 11): 141/141 passing. |

No other failures. No agent or spec errors. No flaky tests.

## Interventions

| id | kind | what | context | correction | by |
|---|---|---|---|---|---|
| INT-1 | tier-upgrade | Discovered `src/auth/**` is a §16.1 safety-critical trigger *after* declaring the task `standard` | Plan-time tier declaration was `standard` because the *line count* qualified for trivial but the *path* does not (§16.1 is disjunctive on safety paths) | Upgraded tier to `safety-critical`; re-ran the §2 pre-task checklist at the new tier; recorded the upgrade here per §16.3 | agent (self, mandated) |
| INT-2 | clarification (env-fix) | `mocha: not found` env error blocks functional gate | Stale `package-lock.json` | Human ran `npm ci` out-of-band; `clarification.resolve` with `resolution_kind: "env-fix"`; agent resumed without editing any code | human |

## Verification report

- **Syntactic gate (§6.1):** **Y** — `node -c src/auth/session.js` → no syntax errors; `npm run lint` → 0 errors, 0 warnings. Evidence: tool call 12 output, captured at `.harness/artifacts/episodes-work/<run-id>/gate-syntactic.log`.
- **Functional gate (§6.2):** **Y** — `npm test` → 141/141 passing, including the new redaction test at `tests/auth.spec.js:31`. The no-test-suite escape (§6.2) was *not* invoked — a new test was added as part of this change. Evidence: tool call 11 output.
- **Semantic gate (§6.3):** **Y** — spec → outcome mapping, **with evidence orthogonal to the functional gate**:
  - "tokens never appear in logs" → `git.blame` (tool call 5) confirms the original line was a diagnostics call; the redaction preserves the diagnostic id while removing the secret. **Orthogonal evidence #1: git-blame reasoning.**
  - "add regression test" → typed-AST hash of `src/auth/session.js` changed by exactly one `CallExpression` argument sub-tree (no control-flow, no return, no signature change). **Orthogonal evidence #2: typed-AST diff.**
  - Neither citation reuses the functional test from §6.2.
  Evidence artifacts: `.harness/artifacts/episodes-work/<run-id>/gate-semantic blame.txt`, `…/gate-semantic-ast-diff.json`.
- **Regression gate (§6.4):** **Y** — `snapshot.diff` (tool call 13) confirms baseline 140 passing → after 141 passing; the +1 is the new test; no previously-passing test changed outcome. types/lint hashes unchanged.
- **Attribution (§3.5):** **Y** — one failure (F-1, env locus), recorded with evidence, resolved via INT-2 without code changes. See Failures table.
- **Memory (§3.6):** **Y** — `memory.write(stream="project", kind="invariant", subject="src/auth/session.js:82", body="refresh tokens are no longer logged; redaction uses err.id", evidence_run_id=this, pinned=false, ttl_days=null)` recorded.
- **Changelog (§3.7):** **Y** — entry appended via `compare-and-append` (tool call 14). `prev_hash` chained to the `## GENESIS` marker's `this_hash` (`changelog.latest_or_genesis()` was called; the agent did NOT compute the genesis hash itself, per `HARNESS_PRIMITIVES.md` §3.5.1 rule 9). `this_hash` computed over the canonical form with `this_hash=""` blanked. Entry block reproduced below for the audit reader.
- **Episode (§3.8):** **Y** — this file is complete and located at `logs/episodes/<run-id>.md`. `episode.close` will compute the `episode_hash` over the byte range defined in `HARNESS_PRIMITIVES.md` §3.4.1 (the `## Episode hash` section is excluded from its own input).
- **Permission (§3.9):** **Y** — two `ask` approvals (appr_…_001 for the source edit, appr_…_002 for the test edit), both granted under the `R-SAFETY-CRITICAL` `ask` rule (`HARNESS_PRIMITIVES.md` §2.9). Every other tool call was auto-allowed (read-only or R-SHELL-TEST). No bypass-on-deny.
- **No-side-effects (§3.10):** **Y** — files touched (listed below) are exactly: the source file, the test file, the changelog, and this episode. No edits to `package.json`, `package-lock.json`, `.harness/**`, or any file outside `src/auth/` and `tests/`.

All ten items **Y**. Task complete.

## Files touched

1. `src/auth/session.js` (modified — line 82 redaction)
2. `tests/auth.spec.js` (modified — added redaction assertion at line 31)
3. `CHANGELOG.agent.md` (appended — Entry 1)
4. `logs/episodes/<run-id>.md` (created — this file)

No other files were modified by this run. (Prior versions of this template omitted files 3 and 4 from this list; that was a §3.10 no-side-effects violation and has been corrected.)

## Changelog entry (reproduced for the audit reader)

```
## ENTRY 1 — 2026-06-29T10:14Z
- agent:        opencode-go/glm-5.2
- run_id:       2026-06-29T1014Z-fix-auth-leak-3f9a-a1b2
- tier:         safety-critical
- files:        src/auth/session.js, tests/auth.spec.js, CHANGELOG.agent.md, logs/episodes/2026-06-29T1014Z-fix-auth-leak-3f9a-a1b2.md
- intent:       Patch session token leak where refresh tokens were logged on error and add a regression test.
- diff_summary: Replaced `console.error(err.refreshToken)` at src/auth/session.js:82 with
                `console.error('auth_error_id=' + err.id)`; added redaction assertion at
                tests/auth.spec.js:31; appended this changelog entry and the run's episode file.
- evidence:     npm test → 141/141 passing; npm run lint → 0 errors; gate-syntactic.log +
                gate-semantic blame.txt + gate-semantic-ast-diff.json attached; snapshot.diff
                shows +1 test, 0 regressions, types/lint hashes unchanged.
- attribution:  env   (one env failure: stale package-lock.json; resolved by human env-fix, no code change)
- verification: full
- status:       modified
- prev_hash:    <this_hash from ## GENESIS marker, read via changelog.latest_or_genesis()>
- this_hash:    <SHA-256 of canonical form with this_hash blanked (""), computed after writing>
```

## Episode hash

`SHA-256: <computed by episode.close over the canonical byte range defined in HARNESS_PRIMITIVES.md §3.4.1 — the body of this section is excluded from its own input>`