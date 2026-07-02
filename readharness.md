# readharness.md

**What a developer needs to know about harness engineering in this repo.**

This file is a plain-English explainer for humans. It does not tell agents how to work — that is `HARNESS_ENGINEERING.md`. It only explains what kind of harness engineering is happening here and what the artifacts it produces look like.

---

## File map

| File / Directory | Who reads it | What it is |
|---|---|---|
| `readharness.md` | You (developer), now | Friendly explainer of the discipline and our setup |
| `HARNESS_ENGINEERING.md` | Agents, while working | Binding rules + mandatory checklists agents must follow (the **constitution**) |
| `HARNESS_PRIMITIVES.md` | Agents + runtime implementers | Interfaces & formats that make the rules enforceable (tools, permissions, artifacts, concurrency, retrieval, models, gates, snapshots, budgets, interventions, config) |
| `HARNESS_SECURITY.md` | Agents + security reviewers | Threat model & mitigations; F1–F4 with detection rules; attack vectors V1.x–V7.x |
| `HARNESS_METRICS.md` | Agents, you, dashboards | KPIs the harness measures itself on; eval harness; review cadence |
| `HARNESS_VERSIONING.md` | Runtime + you (on upgrade) | Spec & artifact versioning, upgrade protocol, bootstrap (the pre-Entry-1 self-test) |
| `CHANGELOG.agent.md` | You, audits, regression | Append-only log of every change any agent makes |
| `logs/episodes/<run-id>.md` | You, debugging, post-mortems | One trace file per agent run (task → plan → actions → verification). **Canonical Entry 1:** `logs/episodes/20260629T095108Z-add-version-flag-agent-pin0.md` (Phase 3 demo with `--pin`; byte-reproducible per `scripts/entry-1-repro.json`). |
| `.harness/evals/` | Eval runner | Synthetic eval task fixtures consumed by `HighHarness eval run` (W1). |
| `.harness/redactions.toml` | Redaction vault (W3) | Default regex patterns for secret detection (AWS keys, PEM, GitHub PATs, JWTs, GCP keys). |
| `.harness/mcp.toml` | MCP server registry (W7) | Registered MCP server configurations (command, allowed paths, allowed networks, env vars). |
| `.harness/artifacts/incidents/` | Incident response (W6) | Declared incidents with severity, source, timestamps. |
| `.harness/artifacts/notifications/` | Incident notifications (W6) | Notification files produced on incident declaration. |
| `.harness/artifacts/quarantine/` | Quarantine records (W6) | Quarantined sources (extensions, MCP servers, model routes). |

Read this file first. Then skim `HARNESS_ENGINEERING.md` for the rules and `HARNESS_SECURITY.md` for what those rules defend against. The other three paired files (`PRIMITIVES`, `METRICS`, `VERSIONING`) are the operating code that turns rules into a runtime — read them when you implement or upgrade the harness, not on every task. The log files under `logs/` and `CHANGELOG.agent.md` are produced automatically as agents work — you read them when you audit or debug. The bootstrap artifacts under `.harness/artifacts/bootstrap/` are produced once, at harness install time, by `HARNESS_VERSIONING.md` §6.

---

## 1. What harness engineering is

A **harness** is the runtime layer that sits between a foundation model (the LLM) and the environment (the repo, the shell, the tests). It decides what the model can see, what it can do, how it gets feedback, and how a change is judged "done."

This matters because raw model capability is not enough. Study after study shows agent-generated changes get rejected around **46% of the time** when the harness is weak — wrong implementation, broken CI, lost session, missed root cause. The fix is almost never "bigger model." It is "better harness." So we treat the harness as a first-class engineering asset.

The mental model:

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   MODEL     │◄────►│   HARNESS   │◄────►│ ENVIRONMENT │
│ (the LLM)   │      │ (this repo's│      │ (code, tests│
│             │      │  machinery) │      │  shell, git)│
└─────────────┘      └─────────────┘      └─────────────┘
```

The harness is the mediating layer. Every design choice in it shapes how reliably the model can do its job.

---

## 2. Which harness level we operate at

There are four levels (H0 → H3). We commit to **H2: Stateful Orchestration** for everything in this repo.

- **H0 — Prompt-only:** model gets a task, emits a patch, no feedback. We do not operate here.
- **H1 — Tool-augmented:** model has tools (read, edit, run tests) but no memory between steps.
- **H2 — Stateful orchestration (ours):** tools **plus** explicit task state, project memory, basic failure attribution, structured logging. Multi-step work holds together.
- **H3 — Full runtime substrate:** deterministic verification, formal invariants, entropy auditing, episode packaging. Heaviest. Reserved for safety-critical paths we may adopt later.

H2 is the sweet spot for general work: enough memory and attribution that long tasks don't drift, without the upfront cost of H3 formal verification. If something fails at H2, the ladder tells us where the fault is — model capability, tool use, memory/state, or verification.

---

## 3. The thirteen responsibilities

Every harness has these thirteen jobs. Each one in one line:

1. **Task specification** — turn intent into a representation the model can act on; resolve ambiguity before acting.
2. **Context selection** — choose what the model sees at each step (the whole repo never fits); look beyond source code to product specs, decision records, and customer signals.
3. **Tool access** — define which actions the agent may take and through what interfaces.
4. **Project memory** — persist explored paths, failed approaches, verified invariants across runs.
5. **Task state** — track what subtask is done, in progress, blocked, so the agent doesn't drift.
6. **Observability** — log every action, decision, and intermediate artifact for humans and audits.
7. **Failure attribution** — when a test breaks, decide whether it is the agent, the test, the environment, or a pre-existing bug.
8. **Verification** — establish the change is done: syntactic, functional, semantic, regression.
9. **Permissions** — least-privilege access control; dangerous actions need approval.
10. **Entropy auditing** — control stochastic variation; seed, monitor variance, fall back when outputs diverge.
11. **Intervention recording** — log every human override and feed the correction back in.
12. **Decomposition to a verifiable grain** — break tasks down until every subtask has its own pass/fail criterion; snapshot the baseline so regression is detectable.
13. **Budget-aware exploration** — set a per-task exploration budget (tool calls, files traversed, tokens); surface spend vs. progress; stop and ask when approaching the budget.

They are interdependent. A weakness in one propagates to the others.

---

## 4. What `CHANGELOG.agent.md` is

It is an **append-only, structured, hash-chained log of every change** any agent makes to this repo. The point is audit and regression: if something breaks next week, you can trace it back to the exact change, the agent, the evidence the agent had, and whether it was verified.

- One new entry per change (not per run — per change).
- Never edited or deleted. If a change is reverted, that is itself a new change with a new entry referencing the original in its `intent` field.
- Tamper-evident / hash-chained: each entry's `prev_hash` is the SHA-256 of the previous entry's `this_hash`, forming a chain. The first entry's `prev_hash` equals the `this_hash` of the `## GENESIS` marker that the bootstrap protocol (`HARNESS_VERSIONING.md` §6.1) writes *before* any agent run begins. The marker is not an entry and is not numbered. Edit history and the chain breaks visibly. Verification runs on every read, not just on demand.
- Agents append to it; humans read it.

**Entry schema** (the fields you'll see in each entry — `CHANGELOG.agent.md` is the source of truth for this schema; see that file for the canonical version):

```
## ENTRY <N> — <ISO-8601 timestamp>
- agent:        <agent id / model>
- run_id:       <run id, links to logs/episodes/<run-id>.md>
- files:        <paths touched, comma-separated>
- intent:       <one sentence — what this change was supposed to do>
- diff_summary: <one or two lines — what actually changed>
- evidence:     <test outputs, type check, lint results, links>
- attribution:  <agent | spec | env | flaky | pre-existing | none>
- verification: <syntactic | functional | semantic | regression | full>
- status:       <added | modified | reverted | deleted>
- prev_hash:    <SHA-256 of previous entry's canonical text; first entry links to the GENESIS marker's this_hash>
- this_hash:    <SHA-256 of this entry's canonical text, computed after writing>

> **Hashing concretely:** before hashing, the `this_hash` and `prev_hash` fields are blanked (`""`). Canonical text is byte-exact: LF line endings, 2-space indentation, fields in schema order, no trailing whitespace, file ends with a single `\n`. The full canonicalization rules live in `HARNESS_PRIMITIVES.md` §3.5 and §3.4. Hand-rolled hashing is a silent-action violation.
```

You read `CHANGELOG.agent.md` when you want to know **what changed and why**. You read `logs/episodes/` when you want to know **how a particular run unfolded**.

---

## 5. What `logs/episodes/` is

One trace file per agent run, named with a run id (timestamp + slug). It is the full story of a single task: the plan, every tool call, every output, every failure and its attribution, and the final verification report.

- One file per run, e.g. `logs/episodes/2026-06-28T1432-fix-auth-leak.md`.
- Captures the journey, not just the destination.
- The changelog records outcomes (changes landed); episode traces record the reasoning that got there.
- Required reading for post-mortems. If a change in `CHANGELOG.agent.md` looks wrong, open its `run_id` in `logs/episodes/` and walk the path the agent took.
- The trace file also contains the completed **pre-task checklist** (run before any tool call) and **post-task checklist** (run before declaring complete) — see `HARNESS_ENGINEERING.md` Sections 2 and 3.
- A filled-in template lives at `logs/episodes/_EXAMPLE.md` — but as of Phase 3 it is **superseded as the canonical real-world reference** by the Episode 1 file below. Use `_EXAMPLE.md` as a structural template for new runs; do not cite it as evidence of a real agent run.
- **Canonical Episode 1: `logs/episodes/2026-06-29T110448Z-add-version-flag-agent-9bd7.md`** — the demo produced by `make entry-1-demo` (per `BUILD_PHASE_3.md`). Its `CHANGELOG.agent.md` Entry 2 (`n=2`, `prev_hash` = the bootstrap-eval Entry 1's `this_hash`) chains to the GENESIS marker. Hashes are in `scripts/entry-1-repro.json`.

**Episode package** (what's inside each trace file — `HARNESS_ENGINEERING.md` Section 5 is the source of truth):

```
run_id
task_spec            — the task as given
plan                 — the decomposition attempted
task_state_log       — subtasks done / in-progress / blocked over time
tool_calls           — ordered list of tool invocations and their outputs
decisions            — why the agent chose A over B
failures             — every failure + attribution (agent / spec / env / flaky / pre-existing)
interventions        — any human overrides, with context
verification_report  — syntactic / functional / semantic / regression (4 gates)
                        plus attribution and memory (6 fields total)
files_touched        — ordered list of every file modified
episode_hash         — SHA-256 computed over all of the above, linkable from CHANGELOG.agent.md
```

---

## 6. Verification gates

A change is not "done" until it passes four **verification gates**, in order:

1. **Syntactic** — it compiles / parses / type-checks.
2. **Functional** — the relevant tests pass.
3. **Semantic** — the change actually satisfies the original intent (not just tests that happen to pass).
4. **Regression** — nothing that used to work now breaks.

Skipping any gate is how subtle bugs ship. But verification is necessary, not sufficient. Six **completion requirements** must also be satisfied before a task is declared complete: attribution, memory, changelog entry, episode trace, permission check, no-side-effects. The four gates establish technical correctness; the six requirements establish that the harness did its mediating job. Agents must produce a verification report covering all four gates plus the six completion items before declaring complete.

---

## 7. Failure attribution

When something fails, the first job is to find the fault, not to fix the symptom. Five possible loci:

- **Agent error** — the agent's edit is wrong.
- **Spec error** — the task itself was ambiguous or wrong.
- **Environment error** — misconfigured test env, missing dependency, wrong runner.
- **Flaky test** — passes/fails independent of the change.
- **Pre-existing bug** — already broken before this change.

Cargo-cult debugging (randomly changing code until tests turn green) is forbidden. The agent must record which locus it attributes the failure to and the evidence for that call.

---

## 8. Safety boundaries

Four failure modes the harness is built to catch (drawn from the security literature on agentic runtimes):

- **F1 Gate-Bypass** — agent executes an action the permission layer should have blocked.
- **F2 Audit-Forgery** — recorded action does not match what actually executed.
- **F3 Silent Host Failure** — the environment failed in a way the harness did not detect.
- **F4 Wrong-Target** — action hit an unintended file, recipient, or resource.

Operating principles:

- **The model is not trusted.** It is stochastic, hallucinates, and can be jailbroken.
- **The environment is only partially trusted.** It has bugs and changes.
- **The harness must be trusted.** It is small, immutable from the agent's side, observable, and minimal.
- **Least privilege.** Agents get the minimum access needed for the current subtask, nothing more.
- **Dangerous operations need approval.** Anything destructive (delete, force-push, network egress to arbitrary URLs, reading secrets) is gated.

---

## 9. Persistence: memory and task state

Two things make long tasks not fall apart:

- **Project memory** — what paths the agent already explored, what approaches already failed, what invariants are already verified. Persisted across runs.
- **Task state** — a live record of which subtasks are done, in progress, or blocked. Without it, agents drift (forget the original goal, redo work, get distracted).

Both are first-class. Both are logged.

---

## 10. Anti-patterns we reject

These are named and forbidden in this repo:

- **Vibe coding** — letting the agent generate freely without architectural oversight. Produces code bloat without quality. The "6.7% improvement" you sometimes see is a denominator effect from code growth, not real improvement.
- **Cargo-cult debugging** — randomly editing code until tests turn green, without understanding the fault.
- **The H0 trap** — using the model as a chat oracle, then spending more time reviewing its output than it would have taken to write the code. The failure is the harness, not the model.
- **Denominator-effect bloat** — measuring smells-per-line and celebrating a drop that actually came from the agent writing more code.
- **Drift** — losing the original goal mid-task because task state is not tracked.
- **Context hallucination** — inventing APIs or variables because the agent couldn't see the real definitions (poor context selection).
- **Silent action** — taking a step without logging it in the episode trace.
- **Bypass-on-deny** — routing around a denied permission instead of stopping and asking.

---

## 11. Quick reference

```
LEVEL         H2 — Stateful Orchestration
LOG CHANGES   append to CHANGELOG.agent.md (per change, via changelog.append API)
LOG RUNS      write logs/episodes/<run-id>.md (per run, via episode API)
DONE MEANS    4 verification gates pass + 6 completion requirements met
FAILURE FIRST attribute the fault before fixing the symptom
TRUST         model = not trusted, env = partial, harness = trusted
ACCESS        least privilege; dangerous ops need approval; enforced at tools.invoke
NEVER         vibe coding | cargo-cult | H0 trap | drift | context hallucination
              | silent action | bypass-on-deny | denominator-effect bloat
SCHEMA        every artifact carries schema_version; out-of-schema writes rejected
BOOTSTRAP     no agent runs until bootstrap.json with passed=true exists
SUBCMDS       bootstrap | changelog | episode | snapshot | gates | tools | permissions
              | spend | hook | integrity | clarification | eval | metrics | cadence
              | redaction | models | incident | mcp (register/start/stop/list/serve)
REDACTION     auto-applied to ToolResult, episode, changelog, and memory writes (W3)
VERIFY-ON-READ self_hash checked on every artifact load (W5)
INCIDENTS     auto-detect F1 (empty tool_call_id); manual declare/list/ack/close (W6)
EVALS         synthetic task runner for harness self-testing (W1)
MCP SERVE     harness exposes tools over JSON-RPC 2.0 stdio (W8)
```

If you remember nothing else: agents in this repo never act as lone oracles. They operate inside a harness, they log every change, they verify before declaring done, and they attribute failures before fixing them. Everything else is detail — and that detail lives in `HARNESS_PRIMITIVES.md`, `HARNESS_SECURITY.md`, `HARNESS_METRICS.md`, and `HARNESS_VERSIONING.md`.

---

## 12. Additional capabilities (Phase 2.5+)

Beyond the core harness, the HighHarness binary provides these subsystems:

### 12.1 Evaluation suite (`HighHarness eval`)
Synthetic task runner that tests the harness itself. Each eval is a `.harness/evals/<name>/` directory with a `Task:` header and golden/forbidden predicates. `HighHarness eval run --all` executes all evals and writes pass/fail results to `.harness/artifacts/evals/`. Built-in evals check: trivial edits, deny-harness-path enforcement, and semantic-gate orthogonality.

### 12.2 KPI rollups and cadence (`HighHarness metrics`, `HighHarness cadence`)
Eleven KPI functions measuring changelog health, episode completeness, spend tracking, incident counts, bootstrap integrity, and tool-usage volume. `HighHarness metrics rollup` aggregates, `metrics alert` evaluates thresholds, `metrics health` runs all checks. `HighHarness cadence run --daily|--weekly|--monthly` enforces freshness gates — if rollups are stale beyond the window, the command exits non-zero.

### 12.3 Redaction vault (`HighHarness redaction`)
Process-local secret detection via regex patterns loaded from `.harness/redactions.toml`. Five default patterns: AWS access keys, PEM-encoded private keys, GitHub PATs, JSON Web Tokens, GCP service account keys. `redaction scan` checks a string, `redaction list` shows configured patterns, and redactions are automatically applied to `ToolResult` content before it reaches the agent, to episode and changelog writes, and to memory store entries.

### 12.4 Provider adapter (`HighHarness models`)
OpenAI-compatible model provider interface (`model.complete`). Currently a stub that returns an error — the types (`CompleteRequest`, `ModelEvent`, `Message`, `Usage`, `Cost`) and structure are in place. Real HTTP wiring (via `reqwest`) requires API key configuration and is deferred.

### 12.5 Verify-on-read (W5, transparent)
Every artifact read (changelog entries, episode snapshots, spend rows, approvals, incidents, tool-calls ledger, memory entries) verifies its `self_hash` to detect tampering. Legacy rows written before W5 lack `self_hash` and are accepted with `state: "legacy"`. Schema structs implement a `SchemaVersion` trait with `compute_self_hash()`.

### 12.6 Incident automation (`HighHarness incident`)
Declare, list, acknowledge, and close security incidents. Incidents carry: id, severity (`low`/`medium`/`high`/`critical`), failure mode (F1–F4), attack vector (V1.x–V7.x), source run, and attribution. F1 detection (empty `tool_call_id`) is wired into `tools.invoke` — it auto-declares an incident before rejecting the call. Quarantine support: quarantined sources (extensions, MCP servers, model routes) are refused at load time.

### 12.7 MCP sandbox (`HighHarness mcp register|start|stop|list`)
Register, spawn, and stop MCP server subprocesses with least-privilege isolation: environment stripped to declared vars, command validated non-empty, timeout support. Server configs are stored in `.harness/mcp.toml`. This is the harness *consuming* MCP servers as tool providers.

### 12.8 MCP serve (`HighHarness mcp serve`)
Exposes the harness *itself* as an MCP server over stdio (JSON-RPC 2.0). Consumers connect via any MCP client instead of subprocess + JSON. Supported methods: `initialize`, `ping`, `tools/list` (all 10 built-in tools with input schemas and capability annotations), `tools/call`, `shutdown`. Each tool call goes through the full harness dispatch (permissions, redaction, ledger logging, F1 detection).