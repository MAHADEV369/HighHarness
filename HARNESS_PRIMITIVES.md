# HARNESS_PRIMITIVES.md

**Interfaces, formats, and runtime contracts that turn `HARNESS_ENGINEERING.md` from a code of conduct into an enforceable runtime.**

`HARNESS_ENGINEERING.md` states the duties. This file specifies the interfaces and data formats those duties operate on. If the two appear to conflict, `HARNESS_ENGINEERING.md` wins on *intent*; this file wins on *mechanics*. Where an interface is unspecified, an agent MUST stop and ask — inventing a private implementation is a silent-action violation (§11 of the binding spec).

This file is paired with:
- `HARNESS_SECURITY.md` — threat model & mitigations.
- `HARNESS_METRICS.md` — KPIs & self-evaluation.
- `HARNESS_VERSIONING.md` — spec & artifact versioning, bootstrap.

---

## 0. Non-negotiable mandates

1. **No private protocols.** An artifact, tool, permission, snapshot, or model call that does not conform to a schema in this file is a harness violation, not a stylistic choice.
2. **Atomic writes.** Every artifact write is atomic (write-to-temp + rename) and crash-safe. Partial files are forbidden.
3. **Schema-versioned.** Every persisted artifact carries a `schema_version`. Readers reject unknown majors.
4. **Lock before write on shared state.** Changelog, memory store, snapshot store, and spend ledger are shared. Writers acquire the documented lock first.
5. **Deterministic IDs.** Run-ids, agent-ids, tool-call-ids, approval-ids follow the formats in §4 and are globally unique within the harness.
6. **Capability-gated.** An agent does not get a tool, a permission, or a model route it did not declare. Declarations are data, not comments.

---

## 1. Tool registry & schemas

### 1.1 Tool declaration

A tool is a named capability with a JSON-schema for arguments and returns, declared side-effect flags, and a stable version. Tools come from three sources:

- **Built-in**: shipped with the harness (filesystem, shell, search, git, test, etc.).
- **Extension**: registered by an extension via the tool-provider extension API (`vscode.ai.registerTool`).
- **MCP**: discovered via MCP servers in `.harness/mcp.toml`.

Declarations live in `.harness/tools/<tool-id>.toml` (built-ins are bundled; extensions/MCP write here on install). One file per tool.

```toml
id              = "fs.read"
schema_version  = 1
version         = "1.0.0"
source          = "builtin"          # builtin | extension | mcp
extension_id    = ""                 # set when source=extension
mcp_server      = ""                 # set when source=mcp
summary         = "Read a file as text or bytes."

[capabilities]
read      = true
write     = false
exec      = false
network   = false
destructive = false
secrets   = false                  # may surface secret-shaped content
side_effect = "none"                # none | read | write | exec | network | destructive

[argument_schema]
# JSON Schema (draft 2020-12). Reference, not inline prose.
"$ref" = ".harness/tools/schemas/fs.read.args.json"

[return_schema]
"$ref" = ".harness/tools/schemas/fs.read.returns.json"

[examples]
[examples.basic]
args   = { path = "src/index.ts" }
returns = { ok = true, bytes = 412, content = "…" }

[approval]
# default approval policy; superseded by permissions.toml
mode = "auto"                      # auto | ask | deny
reason = "read-only, path-scoped"

[rate_limit]
window  = "60s"
max     = 200
```

### 1.2 Required fields

`id`, `schema_version`, `version`, `source`, `summary`, `capabilities`, `argument_schema`, `return_schema`, `side_effect`, `approval.mode`. Missing any → registry rejects the tool; it is unreachable.

### 1.3 Tool ID format

`<namespace>.<name>` in lowercase, segments `[a-z0-9-]+`, dot-separated, ≤4 segments. Examples: `fs.read`, `fs.edit.diff`, `git.commit`, `shell.exec`, `web.fetch`, `test.run`, `mcp.linear.create_issue`.

### 1.4 Capability flags

The six flags in `[capabilities]` are the **only** inputs to the permission engine. A tool with `destructive = true` is never auto-approved regardless of its `[approval]` block; the permission engine overrides it.

### 1.5 Registry API (harness-facing)

```
tools.list()                       → [ToolDescriptor]
tools.get(id)                      → ToolDescriptor | null
tools.invoke(id, args, ctx)        → ToolResult
tools.describe(id)                  → markdown summary for the LLM
tools.reload()                      → re-scan .harness/tools/
```

`tools.invoke` is the **single** entry point. Bypassing it (e.g., shelling out directly) is F1 Gate-Bypass.

### 1.6 ToolResult shape

```json
{
  "schema_version": 1,
  "ok": true,
  "content": { "kind": "text|json|bytes|diff|error", "value": "…" },
  "meta": { "duration_ms": 12, "bytes": 412, "exit_code": null },
  "redactions": [{ "range": [0, 16], "reason": "secret:api_key" }],
  "approval_id": "appr_2026-06-29_001",
  "tool_call_id": "tc_2026-06-29T1432_014"
}
```

Every `tool.invoke` returns a `ToolResult`. There is no shortcut.

---

## 2. Permission model

### 2.1 Permission store

`.harness/permissions.toml` — the single source of truth. Edits are versioned (see §11 and `HARNESS_VERSIONING.md`). Agents MUST NOT edit this file from a run; out-of-band changes by the human only.

### 2.2 Rule format

```toml
schema_version = 1

[[rules]]
id       = "R-001"
effect   = "allow"                 # allow | deny | ask
tool     = "fs.read"               # glob "fs.*" or "*" supported
paths    = ["src/**", "docs/**"]   # glob; empty = unrestricted
network  = []                      # host globs; empty = unrestricted
env      = []                      # env-var name globs; empty = unrestricted
reason   = "read source tree"
priority = 100                     # higher wins; ties → deny

[[rules]]
id       = "R-002"
effect   = "deny"
tool     = "fs.read"
paths    = [".env", ".env.*", "**/secrets/**", "**/*.pem", "**/*.key"]
reason   = "secret redaction"
priority = 1000
```

### 2.3 Evaluation algorithm

1. Collect all rules whose `tool` glob matches the requested tool id.
2. Of those, collect rules whose path/network/env predicates match the request. A missing predicate matches everything.
3. Sort by `priority` desc, then by specificity (literal path > glob > bare tool > `*`).
4. Apply the first rule. If `effect = ask`, the harness opens an approval request (§10) and blocks until resolution.
5. **Default deny.** If no rule matches, the request is denied with reason "no-allow-rule".

### 2.4 Enforcement point

The harness enforces permissions **inside `tools.invoke`**, after argument validation and before dispatch. There is no code path from an agent to a tool that skips this gate. Tools that grow their own ad-hoc permission checks are redundant but not forbidden; the runtime gate is authoritative.

### 2.5 Deny-appeal flow

```
agent → tools.invoke denied
agent → logs denial in episode trace (tool, args, rule_id, reason)
agent → either: stop and request clarification, or
        request an approval override (§10) citing rule_id
human → approves/denies override (never auto)
```

A second denial of the same request inside the same run is a `bypass-on-deny` violation. Stop. Do not retry with reformatting.

### 2.6 Escalation ladder

| Trigger | Action |
|---|---|
| `effect = ask` rule | Open approval request, block tool. |
| Deny + agent requests override | Open approval request with override-context. |
| Override denied | Stop. Record in episode. |
| Recurring denial of same class in a run | Hard-stop the run. |
| Rule marked `priority = 9999` | Cannot be overridden. Fatal if it blocks. |

### 2.7 Per-call permission check API

```
permissions.check(tool_id, args, ctx) → Decision

Decision = {
  schema_version: 1,
  decision: "allow" | "deny" | "ask",
  rule_id, reason,
  effective_permissions: {paths, network, env, scope}, # what's in force after narrowing
  tool_call_id                     # allocated here, reused by tools.invoke
}
```

Callers SHOULD call `permissions.check` before announcing an action to the user / model so the model's planning reflects what the runtime will allow. But note: `permissions.check` is a **pure function over the rules + the request**; the authoritative gate runs inside `tools.invoke` (§1.5 & §2.4). A `check` ≠ authorization to bypass `tools.invoke`.

### 2.8 Per-call scope narrowing

`tools.invoke` accepts an optional `scope_narrow` field that **narrows** the effective permission set for this one call:

```
tools.invoke(id, args, ctx, scope_narrow?) → ToolResult

scope_narrow = {
  paths?:     ["src/auth/**"],   # intersect with rule.paths (set intersection)
  network?:   ["api.example.com"], # intersect with rule.network
  env?:       ["NODE_ENV"],        # intersect with rule.env
  ttl_tool_calls?: 1,            # optional: this narrowing auto-expires after N tool calls
}
```

Rules:
1. `scope_narrow` can only narrow, never widen. Any field in `scope_narrow` is intersected with the rule's effective scope; an empty intersection means the call is denied with `scope-empty`.
2. `scope_narrow` is recorded in the tool-call ledger line for this invocation.
3. Without `scope_narrow`, the permission rules apply unchanged.
4. Extensions and agents are encouraged to scope-narrow every `fs.write`, `shell.exec`, `git.commit` to the minimum needed set. This is what makes "least privilege" an enforceable property (rather than over-claimed).

### 2.9 Path-safety rules for `safety-critical` tier

A path declared as `safety-critical` in the rule's `safety = true` flag forces the decision to `ask` regardless of `approval.mode` on the tool declaration. Default safety-critical path globs (seeded at bootstrap, editable out-of-run):

```
[[rules]]
id = "R-SAFETY-CRITICAL"
effect = "ask"
tool = "*"
paths = ["**/auth/**", "**/secrets/**", "**/migrations/**", ".harness/**", "**/CODEOWNERS"]
safety = true
priority = 800
reason = "safety-critical tier path"
```

---

## 3. Artifact APIs

All artifacts live under `.harness/artifacts/` unless noted. Atomic write = write to `<name>.tmp` then `rename(<name>.tmp, <name>)`. Reads tolerate stale copies; writes never overwrite in place.

### 3.1 Store layout

```
.harness/
  config.toml
  permissions.toml
  models.toml
  mcp.toml
  tools/<tool-id>.toml
  tools/schemas/*.json
  artifacts/
    episodes/<run-id>.md
    changelog.agent.md              (symlinked from repo root for compatibility)
    memory/
      project.jsonl
      user.jsonl
      org.jsonl
    snapshots/<run-id>/              (per-run baseline + checkpoints)
    spend/<YYYY-MM>.jsonl            (cost ledger, append-only)
    approvals/<approval-id>.json
    interventions/<intervention-id>.json
    incidents/<incident-id>.json
    tool-calls.jsonl                 (append-only F2 cross-check ledger)
    harness.log                      (integrity log, line-chained)
    in-flight.jsonl                  (live runs; updated on episode open/close)
    bootstrap/bootstrap.json         (signed, written by bootstrap protocol)
  locks/
    changelog.lock
    memory.lock
    snapshot.lock
    spend.lock
    in-flight.lock
    approvals.lock
```

### 3.2 Memory API

`memory` is append-mostly JSONL. Three streams. Each line is one entry.

```json
{"schema_version":1,"id":"mem_…","stream":"project","kind":"fact|path_tried|invariant|preference","subject":"src/auth","body":"…","evidence_run_id":"…","pinned":false,"tags":["auth"],"created_at":"…","ttl_days":null}
```

API:
```
memory.query(stream, subject?, tag?, since?)  → [Entry]
memory.write(stream, entry)                   → entry_id   # writes through memory.lock
memory.pin(id, true|false)
memory.forget(id)                              → appends a tombstone line (never delete)
memory.forget_subject(stream, subject)         → appends tombstones for matching
```

Forgetting is a tombstone, never a deletion. The changelog's append-only rule applies.

### 3.3 Task-state API

A run's live task-state is appended to the episode trace under `## Task state log` (§5 of the binding spec). Programmatic updates go through:

```
task_state.set(run_id, subtask_id, status, note?)   # status: pending|in_progress|done|blocked
task_state.get(run_id)                              → tree of subtasks + statuses
task_state.checkpoint(run_id)                       → snapshot current state to .harness/artifacts/snapshots/<run-id>/state-<n>.json
```

Updates are atomic appends to the episode file; the in-memory tree is derived.

### 3.4 Episode trace API

```
episode.open(run_id, task_spec)            → creates logs/episodes/<run-id>.md with header + pre-task checklist
episode.append(run_id, section, body)      → atomic append under <section>
episode.record_tool_call(run_id, tc)       → appends to ## Tool calls
episode.record_decision(run_id, d)        → appends to ## Decisions
episode.record_failure(run_id, f)         → appends to ## Failures
episode.record_intervention(run_id, i)     → appends to ## Interventions
episode.close(run_id, verification_report)→ writes verification report + files touched + episode_hash
```

`Tool call`, `Decision`, `Failure`, `Intervention` shapes:

```
Tool call:    { tool_call_id, tool, args, result_summary, started_at, duration_ms, approval_id? }
Decision:     { decision_id, choice, alternatives, reason }
Failure:      { failure_id, what, locus, evidence, resolution }
Intervention: { intervention_id, what_overridden, context, correction, by }
```

The episode file MUST be opened before any tool call. Violation is a silent-action anti-pattern.

**`in-flight.jsonl` lifecycle.** `episode.open` appends a line `{schema_version:1, run_id, agent_id, opened_at, phase, tier}` under `in-flight.lock`. `episode.close` rewrites the full file with the closed run's line removed, atomically (temp + rename). A run that crashes without close leaves a stale entry: on the next harness startup, lines whose `agent_id` has no live process are marked `(reaped)` and moved to `in-flight.reaped.jsonl`.

**`run_id` format.** `<ISO-8601 second-precision>-<short-slug>-<agent_id_short>-<rand4>`. The `rand4` is 4 hex chars from a CSPRNG. The harness refuses two opens with the same `run_id` in the same `.harness/` — collision returns `run-id-collision` and the agent MUST generate a fresh `run_id` (try once; a second collision is a harness incident).

### 3.4.1 `episode_hash` canonicalization

The `## Episode hash` section is the SHA-256 over the canonical byte range of the episode file, computed by `episode.close`. The canonical form is:

1. **Byte range.** From the first byte of `# Episode <run-id>\n` through the last byte before the `## Episode hash` section's first `\n` (i.e., the entire body *excluding* the `## Episode hash` section itself).
2. **Section order is fixed.** Required section order: `Task spec` → `Plan` → `Task state log` → `Tool calls` → `Decisions` → `Failures` → `Interventions` → `Pre-task checklist` → `Verification report` → `Files touched`. Per-tier additions and sub-agent sections append in the order they were created; missing sections are omitted (not emitted as empty headers).
3. **Normalization.** LF line endings (`\n`), no trailing whitespace on any line, no BOM, file ends with a single `\n` after the last non-hash section. Section separators are exactly `\n\n## `.
4. **Self-exclusion.** The `## Episode hash` section is excluded from its own input. Within the byte range above, any field literally named `episode_hash` (none exist in the body) would also be blanked — N/A today, but the rule is.
5. **Result placement.** The computed hex digest is written as the body of `## Episode hash` exactly as `SHA-256: <hex>\n`.

Hash mismatches between `episode.close` and any subsequent `episode.hash` read are F2 Audit-Forgery.

### 3.5 Changelog append API

```
changelog.append(entry)         → this_hash   # acquires .harness/locks/changelog.lock
changelog.latest()              → Entry      # the most recent committed entry
changelog.verify_chain()        → [broken_link_indices]   # empty = healthy
```

`changelog.append` flow — the **compare-and-append** primitive:

1. Acquire `.harness/locks/changelog.lock` (flock-style advisory lock; on conflict, fail-fast with `LockContention` after one retry at 200 ms — do not loop).
2. Read `changelog.latest()` (chain head).
3. Compute `prev_hash = latest.this_hash` (or, before any entry exists, the `this_hash` of the `## GENESIS` marker per `HARNESS_VERSIONING.md` §6.1).
4. Serialize the new entry in **canonical form** (see §3.5.1).
5. Compute `this_hash = SHA-256(canonical_text)`.
6. Re-read the chain head under the lock; if it has changed since step 2 (another writer committed while we hashed), release the lock and retry **exactly once** from step 2 with an updated `prev_hash`. A second lost race on the same append → release the lock and surface `harness-contention`; do not loop.
7. Append the entry atomically: a single `write()` call containing the entire canonical block under the lock. POSIX guarantees atomicity for small writes; do NOT rely on `O_APPEND` racing with the lock. Verify the write size matches.
8. Release the lock.
9. Re-read the latest entry to confirm `this_hash` matches what we wrote and `prev_hash` matches the prior entry's `this_hash` (or the `## GENESIS` marker's `this_hash` for entry 1). Mismatch → halt, surface `audit-forgery-suspected`, do not retry silently.

The chain is **tamper-evident under serial writers**, and seriality is enforced by the lock + compare-and-append. There is no "best-effort" mode. An agent that writes the changelog without going through `compare-and-append` has committed F2 Audit-Forgery.

### 3.5.1 Canonical entry form

The canonical byte sequence for an entry is:

```
## ENTRY <N> — <ISO-8601 timestamp>\n
- agent:        <value>\n
- run_id:       <value>\n
- tier:         <trivial|standard|safety-critical>\n
- files:        <comma-separated, in declared order, no spaces around commas unless in a value>\n
- intent:       <one-line string; no embedded newlines>\n
- diff_summary: <one or two lines; embedded newlines allowed after bullet position, indented 16 spaces to align under opening value column>\n
- evidence:     <same indent rules as diff_summary>\n
- attribution:  <one of agent | spec | env | flaky | pre-existing | none>\n
- verification: <one of syntactic | functional | semantic | regression | full>\n
- status:       <one of added | modified | reverted | deleted>\n
- prev_hash:    <hex OR "" if excluded>\n
- this_hash:    <"" if excluded>\n
```

Rules:

1. **Field order is fixed** and matches the list above. Out-of-order fields are rejected at registry parse time.
2. **Indentation:** the bullet dash is in column 1, the field name begins in column 3, the value begins at column 16 (right-padded with spaces). Exclusive of the leading `\n` after the header.
3. **Line endings** are LF (`\n`), never CRLF.
4. **No trailing whitespace** on any line.
5. **No BOM.** The block starts at the first byte of `## ENTRY`.
6. **Hash-input exclusion:** before computing `this_hash`, the `this_hash` field is blanked (`""`); the `prev_hash` field is **left in** because it references another entry (it is part of the input). After hashing, the `this_hash` and `prev_hash` fields are filled with the real hex.
7. **Final byte:** the block ends with a single `\n` after the `this_hash` value line. No extra blank line.
8. **`this_hash` computation:** `this_hash = SHA-256(canonical_block_bytes)`, where `canonical_block_bytes` is the UTF-8 byte sequence above with the `this_hash` value as `""`.
9. **`prev_hash` of entry 1:** equals `SHA-256("GENESIS<ISO-8601 from the GENESIS marker>")` — exactly the `this_hash` recorded on the `## GENESIS` line written by the bootstrap protocol (`HARNESS_VERSIONING.md` §6.1 step 3). Agents MUST NOT compute this themselves; they MUST read it from `changelog.latest_or_genesis()`.

Hand-rolled canonicalization elsewhere is forbidden. Every writer routes through `changelog.append`.

### 3.6 Snapshot API

```
snapshot.take(run_id, label)          → snapshot_id   # acquires snapshot.lock
snapshot.get(snapshot_id)            → Snapshot
snapshot.diff(before_id, after_id)   → Diff
snapshot.revert(snapshot_id)         → applies inverse via git
```

`Snapshot` payload (§8 defines fields):

```json
{"schema_version":1,"snapshot_id":"…","run_id":"…","label":"baseline|pre-edit|checkpoint-<n>|pre-agent",
 "git":{"commit":"…","dirty":false},"tests":{"hash":"…","summary":{...}},"lint":{"hash":"…"},"types":{"hash":"…"},
 "taken_at":"…"}
```

---

## 4. Concurrency & sub-agent contract

### 4.1 IDs

- **agent_id**: `agent_<random8>_<iso8601>` e.g. `agent_3f9a2c_2026-06-29T1432Z`. Stable per agent process.
- **run_id**: `<ISO-8601 second-precision>-<short-slug>-<agent_id_short>-<rand4>` (matches episode file name and the `in-flight.jsonl` entry). One per top-level task. `<rand4>` is 4 hex chars from a CSPRNG. See §3.4.1 for the collision rule.
- **sub_run_id**: `<run_id>.<n>` for the n-th spawned sub-agent.
- **tool_call_id**: `tc_<run_id>_<seq>` monotonic within a run.
- **approval_id**: `appr_<iso8601>_<seq>` allocated by the approval store.
- **clarification_id**: `clr_<iso8601>_<seq>` allocated by the clarification store.
- **intervention_id**: `int_<iso8601>_<seq>`.

### 4.2 Spawn protocol

A parent agent spawns a sub-agent via:

```
subagents.spawn({
  parent_run_id,
  name,                     # short label
  task_spec,                # verbatim, the child copies it into its own episode
  tools_allowed,            # subset of parent's tools (cannot exceed)
  permissions_overlay,      # additional deny/ask rules; cannot relax parent's
  budget,                   # token + tool-call + wallclock budget for the child
  approval_mode,            # auto|ask|manual; default ask
})
→ sub_run_id
```

Sub-agents inherit the parent's permission rules restricted by the overlay. The overlay can only narrow scope; widening is rejected by the harness. Sub-agents get their own episode file under `logs/episodes/<sub_run_id>.md`. The parent's episode links to each child via a `## Sub-agents` section.

### 4.3 Lock primitives

- **File advisory locks** (`flock` on POSIX, `LockFileEx` on Windows). Stored under `.harness/locks/`.
- **Lock naming**: `.<artifact>.lock` colocated with the resource or under `locks/`.
- **Acquire timeout**: 5s default, configurable. On timeout → harness returns `LockContention` to the caller; the caller decides (wait / abort / raise intervention).
- **pidfile**: lock files contain `<pid>\n<iso8601>\n<agent_id>\n`. Stale locks (pid not alive > 60s) are reclaimable with a warning + changelog note.

### 4.4 Serialized writers

**Mandatory serialized writers**: changelog, memory store, snapshot store, spend ledger, approvals store, interventions store. Concurrent writes go through the lock; no exceptions.

**Concurrent readers** are permitted everywhere; readers must tolerate the last append being absent (eventual consistency) but never read a torn write.

### 4.5 Approval routing with multiple agents

- Each `ask` rule creates an approval request in `.harness/artifacts/approvals/` keyed by `approval_id`.
- Approvals target a specific `sub_run_id`. The human (or a delegated approver extension) sees a queue sorted by `created_at` then by `priority` (destructive > exec > network > write > read).
- On resolution, the approval file is updated atomically and the blocked tool.invoke unblocks.
- A parent cannot auto-approve its own children unless `approval_mode = auto` for that sub-agent (and never for destructive tools).

### 4.6 Failure propagation

- Child failure → recorded in child's episode. Child returns failure result to parent.
- Parent records the child's `sub_run_id` and a one-line summary; full detail lives in the child's episode.
- Exception: child violates harness (F1–F4). Parent MUST not retry; parent MUST stop and surface the violation. Continuing past a child's harness violation is itself a harness violation.

---

## 5. Context-selection / retrieval interface

### 5.1 Source registry

`.harness/sources.toml` declares where context comes from. Addressed by `@source` mentions in chat and by the retriever.

```toml
schema_version = 1

[[sources]]
id       = "repo"
kind     = "filesystem"
root     = "${workspaceFolder}"
weight   = 1.0
ignore   = [".git/**", "node_modules/**", "**/*.lock"]

[[sources]]
id       = "docs"
kind     "web"
seed     = ["https://react.dev", "https://fastapi Tantri"]
crawl    = false
refresh  = "weekly"

[[sources]]
id       "linear"
kind     = "connector"
connector = "linear"
auth    = "secret:linear_token"
scope   = ["team:ENG"]

[[sources]]
id       = "local-rules"
kind     = "rules"
path     = ".harness/rules"
```

### 5.2 Retriever API

```
retrieve(query, opts?) → RetrieveResult

RetrieveResult = {
  schema_version: 1,
  hits: [{
    source, ref, score, kind, snippet, tokens,
    taken_at, retriever_version
  }],
  spent: { tokens, tool_calls, ms },
  budget: { tokens, tool_calls }   # the budget in effect
}
```

`opts` includes: `sources` (filter), `max_hits`, `min_score`, `token_budget`, `tool_call_budget`, `since`, `suppressed_pii` (default true).

### 5.3 Retrieval budget enforcement

The retriever decrements both `token_budget` and `tool_call_budget` per probe. When either hits zero, it returns the current `hits` with `budget_exhausted = true`. The caller (agent or chat runtime) decides whether to ask for more — not the retriever.

Budget defaults come from `.harness/config.toml` `[retrieval]` and can be tightened per-run.

### 5.4 Citation contract

Every chat answer and every agent plan step that retrieved anything MUST emit citations matching `RetrieveResult.hits[].ref`. Uncited claims derived from retrieval are an attribution failure. The runtime supplies the citations, not the model.

### 5.5 PII scrubbing at retrieval

Before any hit leaves the retriever, a redaction pass runs over `snippet` using the secret dictionary (`.harness/redactions.toml`) plus entropy-based detectors (AWS keys, GCP tokens, JWTs, PEM blocks). Scrubbed ranges are reported in `ToolResult.redactions` when the hit flows through a tool.

---

## 6. Model / router abstraction

### 6.1 Inference interface (unified)

```
model.complete({
  model_id, messages, tools?, system?, max_tokens?, temperature?, reasoning_effort?,
  prefill?, stream: true, metadata?
}) → AsyncIterator<ModelEvent>

ModelEvent =
  | { kind: "text", delta }
  | { kind: "tool_call", id, name, args }       # partial or final
  | { kind: "tool_call_final", id, name, args }
  | { kind: "reasoning", delta }                # only if reasoning enabled
  | { kind: "usage", input_tokens, output_tokens, reasoning_tokens }
  | { kind: "cost", usd }
  | { kind: "done", finish_reason }
  | { kind: "error", code, message, retryable }
```

There is exactly one inference interface. Provider adapters produce `ModelEvent` streams. Agents and chat surfaces consume `ModelEvent` only.

### 6.2 Model registry

`.harness/models.toml`:

```toml
schema_version = 1

[[models]]
id = "claude-opus-4"
provider = "anthropic"
context_window = 200000
capabilities = { vision = true, tools = true, reasoning = true, prefill = true }
pricing = { input_usd_per_1m = 15.0, output_usd_per_1m = 75.0, reasoning_usd_per_1m = 22.0 }
privacy = { retention = "zero", training = "opt-out", residency = "any" }
auth = "secret:anthropic_key"
tier = "flagship"

[[models]]
id = "llama-3.3-70b-local"
provider = "ollama"
context_window = 128000
capabilities = { vision = false, tools = true, reasoning = false, prefill = true }
pricing = { input_usd_per_1m = 0, output_usd_per_1m = 0 }
privacy = { retention = "local", training = "none", residency = "device" }
auth = "none"
tier = "local"
```

### 6.3 Routing policy

`.harness/routing.toml`:

```toml
schema_version = 1

[[routes]]
feature   = "autocomplete"
primary   = "llama-3.3-70b-local"
fallback  = ["claude-haiku-4"]
mode      = "cheapest-with-tools"

[[routes]]
feature   = "chat"
primary   = "claude-sonnet-4"
fallback  = ["gpt-4.1", "claude-opus-4"]
mode      = "rotate-on-error"

[[routes]]
feature   = "agent"
primary   = "claude-opus-4"
fallback  = ["gpt-4.1", "claude-sonnet-4"]
mode      = "fallback"

[[routes]]
feature   = "review"
primary   = "claude-opus-4"
fallback  = []
mode      = "primary-only"           # no fallback for review consistency
```

Routing modes: `primary-only`, `cheapest-with-tools`, `rotate-on-error`, `fallback`, `manual`. The router is the only thing that picks a model. Hardcoded model ids in agent prompts are forbidden (model-drift, untrackable).

### 6.4 Fallback & retry

- Transient errors (`retryable = true`): exponential backoff with jitter, max 3 retrits. Switch to next model in `fallback` after the 3rd transient error.
- Non-retryable: route to fallback immediately if available; else surface error to caller.
- Cost thresholds in §9 can force a downgrade before the request is dispatched.

### 6.5 Reasoning controls

`reasoning_effort` is one of `minimal | low | medium | high | max` and a `reasoning_token_budget`. The router maps these to provider-specific shapes (OpenAI `reasoning_effort`, Anthropic `thinking.budget_tokens`, etc.). The model registry's `capabilities.reasoning` gates whether the argument is sent at all.

### 6.6 Cancellation

Every `model.complete` stream MUST be cancelable. On cancellation the harness:
1. Stops consuming the upstream.
2. Logs partial usage & cost to the spend ledger.
3. Records the cancellation in the calling episode as an intervention.
No partial token counts are lost.

---

## 7. Verification execution contract

### 7.1 Per-phase command table

`.harness/config.toml` declares a `[gates]` table per phase (a phase = a top-level subagent bundle, e.g. `editor-shell`, `ai-runtime`, `tooling-python`). Each gate maps to a concrete command.

```toml
[gates.editor-shell]
syntactic  = { cmd = "pnpm -C editor run typecheck", timeout = "120s" }
functional = { cmd = "pnpm -C editor test --filter=$CHANGED", timeout = "300s" }
lint       = { cmd = "pnpm -C editor run lint",       timeout = "120s" }
regression = { inherit = "functional" }              # same as functional by default

[gates.ai-runtime]
syntactic  = { cmd = "cargo check -p ai-runtime",      timeout = "180s" }
functional = { cmd = "cargo nextest run -p ai-runtime", timeout = "600s" }
lint       = { cmd = "cargo clippy -p ai-runtime -- -D warnings", timeout = "180s" }
regression = { cmd = "cargo nextest run --workspace", timeout = "900s" }

[gates.tooling-python]
syntactic  = { cmd = "uv run ruff check .",      timeout = "60s" }
functional = { cmd = "uv run pytest -q $PYTEST_TARGETS", timeout = "300s" }
lint       = { cmd = "uv run ruff check --fix .", timeout = "60s" }
regression = { cmd = "uv run pytest -q",         timeout = "600s" }
```

The harness substitutes `$CHANGED`, `$PYTEST_TARGETS`, `$WORKSPACE_FILTER` from the snapshot diff. An unspecified phase → no gate can run → semantic gate forced to "could-not-verify"; the agent MUST stop and request a phase declaration.

### 7.2 Gate runner

```
gates.run(phase, gate_name, changes) → GateResult

GateResult = {
  schema_version: 1,
  phase, gate: "syntactic"|"functional"|"semantic"|"regression",
  status: "pass"|"fail"|"skipped"|"blocked",
  command, exit_code, output_truncated, duration_ms,
  evidence_path,                   # full output written under artifacts/
  reason                           # for non-pass
}
```

`output_truncated` is always along with `evidence_path`; gates never discard output.

### 7.3 Semantic gate concretely

Semantic is not a shell command; it's a structured judgment. The agent produces a `spec → outcome` mapping in the episode. The harness requires:

```json
{ "schema_version":1, "phase":"editor-shell",
  "mappings":[
    {"criterion":"rename updates references in tests","outcome":"met","evidence":"leaflet.test.ts:42"},
    {"criterion":"no new files outside src/editor","outcome":"met","evidence":"git show --stat"}
  ],
  "all_met": true }
```

If `all_met = false`, the semantic gate is `fail` regardless of tests.

### 7.4 Regression gate fallback when no tests exist

If the touched area has no tests, the agent MUST state so explicitly in the episode and substitute one of:

- A typed-AST hash unchanged (for non-behavioral changes), or
- A smoke check via a documented runbook command, or
- A new test added as part of the change and run under `functional`.

Silent skip is a verification violation.

### 7.5 Gate ordering & short-circuit

`syntactic → functional → regression → semantic`. A failure in an earlier gate halts the sequence; subsequent gates are `status: blocked`. The agent MUST fix or escalate before re-running. A flaky retry of the same gate is allowed once with a recorded rationale.

---

## 8. Baseline snapshot format

### 8.1 Required fields

```json
{
  "schema_version": 1,
  "snapshot_id": "snap_…",
  "run_id": "…",
  "label": "baseline",
  "git": { "commit": "<sha>", "dirty": false, "diff_stat": "M src/x.ts" },
  "tests": { "hash": "<sha256 of runner output>", "summary": { "passed": 142, "failed": 0, "skipped": 3 }, "duration_ms": 18320 },
  "types": { "hash": "<sha256 of typecheck output>" },
  "lint":  { "hash": "<sha256 of lint output>" },
  "phase": "editor-shell",
  "taken_at": "…"
}
```

### 8.2 When to snapshot

- `baseline`: at `episode.open`, before the first tool call.
- `pre-edit`: before any `fs.edit.*` or `shell.exec` that mutates code.
- `checkpoint-<n>`: after each completed subtask.
- `pre-agent`: when entering agent mode.

Skipping `baseline` is a harness violation; the regression gate cannot run without it.

### 8.3 Regression diff

`snapshot.diff(baseline_id, current_id)` returns the test/type/lint hash diffs plus git diffs. The regression gate passes iff: `baseline.tests.hash`-minus-failing-baseline-tests is a subset of `current.tests.hash` and no previously passing test now fails. Hash equality ≠ equality of passing set; the harness uses structured summaries, not just the hash.

### 8.4 Revert

`snapshot.revert(snap_id)` performs `git reset --hard <commit>` for code files only, never for `.harness/`. Reverting harness state is forbidden — harness state is monotonic.

---

## 9. Budget & cost metering

### 9.1 Spend ledger

`.harness/artifacts/spend/<YYYY-MM>.jsonl`, one JSON object per atom of inference, append-only, serialized via `spend.lock`.

```json
{"schema_version":1,"ts":"…","run_id":"…","agent_id":"…","model_id":"…","feature":"autocomplete|chat|agent|review|embed","input_tokens":…,"output_tokens":…,"reasoning_tokens":…,"usd":0.0034,"routing_mode":"…","provider":"…","metadata":{"file":"…"}}
```

### 9.2 Account scopes

- `per-run`, `per-feature`, `per-agent`, `per-day`, `per-month`. The harness enforces the most restrictive active budget.
- Budgets declared in `.harness/config.toml` `[budgets]`. Hard vs soft budgets: hard → block the next call; soft → warn + degrade.

### 9.3 Degrade-vs-stop policy

On budget exhaustion:

1. **Soft budget reached**: router downgrades to the next cheaper model in the route's `fallback` and logs a `degrade` event. Continues.
2. **Hard budget reached**: router refuses dispatch. The current tool/agent step aborts cleanly; episode records `budget_exhausted`. The run is paused for human review, not auto-cancelled.
3. **Cost spike guard**: a 10× increase in `usd/min` over the trailing 5-min average triggers an `intervention` regardless of budget.

### 9.4 Sampling

Every inference event is logged. There is no sampling-based accounting. The ledger is the source of truth for `HARNESS_METRICS.md`.

### 9.5 BYO-key passthrough

When a provider is reached via a user-supplied key (`auth = "secret:user_*"`), the ledger records `usd = 0` but still records tokens, so productivity analytics still work.

---

## 10. Human-intervention protocol

### 10.1 Intervention primitives

The harness exposes four first-class operations a human can invoke on a live run:

- `pause(run_id)` — the agent finishes its current tool call, then waits. No further tool dispatches.
- `resume(run_id)` — unblocks.
- `inject(run_id, { kind: "constraint"|"correction"|"message", body })` — adds content to the run's context as an intervention (§1.11 of the binding spec). The agent MUST read it before its next step.
- `cancel(run_id, reason?)` — aborts. Snapshots taken; partial state preserved; episode closed with verification `status = cancelled`.

### 10.2 Approval request

```
approval.request({
  run_id, tool, args, rule_id, reason, priority, destructive
}) → approval_id

approval.state(approval_id) → "pending"|"approved"|"denied"|"expired"
approval.resolve(approval_id, decision, rationale?, modified_args?)
approval.expire(approval_id, after = "30m")
```

Approved-with-`modified_args` is allowed: the human can change the tool arguments before approving (e.g., trim a `paths` list). Harness dispatches the modified form.

### 10.3 Intervention artifact

```
.harness/artifacts/interventions/<intervention-id>.json
{ schema_version, id, run_id, kind, body, by, at, resulting_decision }
```

Every intervention is referenced from the run's `## Interventions` episode section and from `CHANGELOG.agent.md` when it altered a landed change.

### 10.4 Out-of-band approval

A human may resolve an approval from a device other than the agent's host (CLI, mobile companion, web). The resolution is signed by the human's account key and the harness verifies the signature. Unsigned OOB resolutions are rejected.

### 10.5 Constraint as code

Out-of-band constraints can be applied to a run by pushing to `.harness/constraints/<run-id>.toml`. The harness polls the file on each tool invocation boundary. Constraints take precedence over default permissions and route rules, but cannot widen them. Useful for: emergency "stop touching Terraform" instructions.

### 10.6 Clarification protocol (the "stop and ask" primitive)

When the binding spec mandates "stop and ask" (§1.1 of `HARNESS_ENGINEERING.md`, §7 failure attribution, §16 forced upgrade, or any other ambiguity), the harness exposes a real protocol — not a phrase.

```
clarification.request({
  run_id, by: "agent" | "human",
  question: "<one-sentence, parseable>",
  context: { tool_call_id?, rule_id?, failed_gate?, files?, spec_criterion? },
  urgency: "blocking" | "advisory"               # blocking pauses the run
}) → clarification_id

clarification.state(clarification_id) → "pending" | "answered" | "superseded" | "expired"
clarification.resolve(clarification_id, { answer, rationale?, modified_args?, resolution_kind: "answer"|"spec-revision"|"approval"|"tier-upgrade"|"env-fix" })
clarification.expire(clarification_id, after = "24h")
clarification.list(run_id?, open? = true) → [Clarification]
```

Semantics:

- **Blocking clarification** sets the run into `blocked` task-state (`HARNESS_ENGINEERING.md` §1.5). The harness blocks further `tools.invoke` for that run until resolve or expire. The run does NOT terminate; the harness records an `## Interventions` entry with `{ kind: "clarification", clarification_id, question, context }` and writes a corresponding `.harness/artifacts/interventions/<intervention-id>.json`.
- **Advisory clarification** does not block; the agent may continue with a stated assumption logged in the episode.
- **Resolve with `resolution_kind = "spec-revision"`** is the only path that can change a §2 pre-task checklist answer; it forces the agent to re-run the checklist at the new spec.
- **Resolve with `resolution_kind = "tier-upgrade"`** triggers the §16 forced-upgrade flow.
- **Resolve with `resolution_kind = "env-fix"`** marks an environment problem as resolved by a human out-of-band (e.g., lockfile fixed, missing dep installed); the agent may resume `functional`/`regression` gates without re-editing code.
- **Resolve with `modified_args`** lets the human correct the tool args the agent proposed (mirrors §10.2 approval-with-modified-args) — used when the agent's args are right in spirit but wrong in scope.
- **Expiry.** A pending clarification past `after` is moved to `expired`; the run is hard-stopped (not auto-cancelled) and surfaces as an `interventions` intervention with `kind = "clarification-expired"`. The agent does NOT default-decide on expire; it MUST stay blocked until human resolve or human cancel.
- **Out-of-band resolve**, like approvals, requires a signed resolution from the human's account key.

### 10.7 Blocked run-state contract

A run's task-state may be `pending | in_progress | done | blocked` (per `HARNESS_ENGINEERING.md` §1.5). `blocked` is specifically entered by:
- a blocking `clarification.request`,
- a paid approval awaiting resolution (`§10.2`),
- a lost race on `compare-and-append` while waiting on a second attempt,
- a hard budget exhaustion (§9),
- a forced §16 tier upgrade pending re-checklist.

While blocked, the harness:
1. Refuses every `tools.invoke` for that run with `decision = "deny"`, `reason = "run-blocked"`. This is not a `bypass-on-deny` trigger for the agent; it is the runtime enforcing the block.
2. Continues to accept `clarification.resolve`, approval resolutions, and `task_state.set(..., "in_progress", note="unblocked by …")` from the human/resolver.
3. Cannot be unblocked by the agent itself. An agent that attempts to self-unblock (e.g., by re-issuing the same tool call without context change) commits a `bypass-on-deny` violation.

---

## 11. Harness config schema

`.harness/config.toml` — the only configuration file the harness reads at startup. Other files (`permissions.toml`, `models.toml`, `routing.toml`, `sources.toml`, `mcp.toml`, `redactions.toml`) are referenced from here and have their own schemas.

```toml
schema_version = 2
harness_version = "0.1.0"

[identity]
# Filled by `HighHarness bootstrap init --org <name> --project <name>`.
# Defaults: --org=default, --project=default. The identity is label-only; it does not
# affect any runtime decision. It appears in the bootstrap.json record and in the
# harness integrity log first line to help humans identify the .harness instance.
org = "${ORG}"
project = "${PROJECT}"
phase = "default"               # default phase for gates when not detected from changes

[retrieval]
default_token_budget       = 12000
default_tool_call_budget   = 12
redact_secrets             = true
redactions_file            = ".harness/redactions.toml"

[budgets]
per_run_usd_hard   = 5.0
per_run_usd_soft   = 2.0
per_feature_chat   = { per_day_usd_hard = 10.0 }
per_feature_agent  = { per_day_usd_hard = 25.0 }

[approval]
default_mode             = "ask"
destructive_needs_human   = true
default_expiry            = "30m"
oob_signature_required    = true

[episodes]
dir          = "logs/episodes"
auto_compact_after_tokens = 60000
retention_days = 90

[changelog]
path        = "CHANGELOG.agent.md"
lock_path   = ".harness/locks/changelog.lock"

[snapshots]
dir = ".harness/artifacts/snapshots"
max_per_run = 50
```

Profile overrides: `.harness/config.<phase>.toml` layered on top, narrow only (cannot widen permissions, cannot raise budgets above base).

---

## 12. Quick reference

```
TOOLS         .harness/tools/<id>.toml, schema-versioned, capability-gated
PERMISSIONS   .harness/permissions.toml — allow|deny|ask, priority, default-deny
ARTIFACTS     .harness/artifacts/{episodes,memory,snapshots,spend,approvals,interventions}
CONCURRENCY   flock under .harness/locks/, serialized writers, concurrent readers
RETRIEVAL     .harness/sources.toml, retrieve(query, budget) → cited hits
MODELS        .harness/models.toml + routing.toml, one ModelEvent stream
GATES         .harness/config.toml [gates.<phase>], 4-gate short-circuit
SNAPSHOTS     baseline / pre-edit / checkpoint / pre-agent, never revert .harness/
BUDGET        per-run / per-feature / per-day / per-month, soft→degrade, hard→pause
INTERVENTION  pause / resume / inject / cancel, approval.request/resolve
CONFIG        .harness/config.toml is the only entry point at startup
INVARIANTS    schema_version everywhere, atomic writes, default-deny, no private protocols
```

---

*End of primitives. See `HARNESS_SECURITY.md` for the threat model that these primitives are designed to withstand.*