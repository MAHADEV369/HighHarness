# HighHarness

**A runtime-neutral agent harness for AI coding agents.**

HighHarness sits between an LLM and your codebase. It enforces hash-chained audit trails, permission gates, episode traces, and four verification gates — so every agent change is auditable, attributable, and verifiable.

> The fix for unreliable AI agents is almost never a bigger model. It is a better harness.

---

## What it does

| Capability | What HighHarness enforces |
|---|---|
| **Hash-chained audit trail** | Every change is appended to `CHANGELOG.agent.md` with SHA-256 `prev_hash` → `this_hash` chaining. Tampering breaks the chain visibly. |
| **Permission engine** | Default-deny, priority-sorted rules with scope narrowing. Destructive operations always require approval. |
| **Episode traces** | Every run produces `logs/episodes/<run-id>.md` — the full story: plan, tool calls, decisions, failures, verification report. |
| **Four verification gates** | Syntactic → Functional → Semantic → Regression. A change passes only when all four pass. |
| **Bootstrap protocol** | The harness self-validates before any agent runs. No `bootstrap.json` with `passed: true` → no agent starts. |
| **Integrity log** | Line-chained JSONL (`harness.log`) records every harness event. Discontinuity on startup = integrity alert. |
| **Secret redaction** | Regex-based vault scans tool results, episodes, and memory. AWS keys, PEMs, GitHub PATs, JWTs, GCP keys caught automatically. |
| **Incident response** | F1–F4 failure detection, quarantine support, declare/ack/close lifecycle. |
| **Model routing** | Registry + routing policy with fallback, cost tracking, and degrade-on-budget. |
| **MCP integration** | Register and sandbox external MCP servers. Or expose the harness itself as an MCP server over JSON-RPC 2.0. |

---

## Quick start

```bash
# Build (requires Rust 1.78+)
git clone https://github.com/trident/HighHarness.git
cd HighHarness
cargo build --release

# Bootstrap (creates .harness/ with config, permissions, tools, genesis marker)
./target/release/HighHarness bootstrap init --human "$USER"

# Verify bootstrap passed
./target/release/HighHarness bootstrap verify

# Check the version
./target/release/HighHarness --version

# Run the canonical demo (end-to-end: episode → tools → gates → changelog)
make entry-1-demo
```

---

## CLI commands

```
HighHarness <command> [subcommand] [flags]
```

### Foundation

| Command | Purpose |
|---|---|
| `bootstrap init --human <name>` | 10-step bootstrap: skeleton, GENESIS, tools, permissions, models, config, eval |
| `bootstrap verify` | Check `bootstrap.json` exists with `passed: true` |
| `id-run --slug <slug>` | Generate a deterministic `run_id` |
| `id-agent` | Generate a deterministic `agent_id` |

### Audit trail

| Command | Purpose |
|---|---|
| `changelog append --entry <json>` | Compare-and-append with hash chain, race detection, post-write verify |
| `changelog latest` | Print the most recent changelog entry |
| `changelog get <n>` | Print entry N |
| `changelog verify-chain` | Walk the full chain, verify every `prev_hash` → `this_hash` link |
| `episode open --run-id <id> --task-spec-file <f>` | Create episode trace file with header |
| `episode append --run-id <id> --section <s> --body-file <f>` | Append a section (plan, decisions, failures, etc.) |
| `episode append-tool-call --run-id <id> --json <j>` | Append a structured tool-call row |
| `episode close --run-id <id> --verification-json <f> --files-touched <f>...` | Write verification report, compute `episode_hash` |
| `episode hash --run-id <id>` | Re-compute episode hash for verification |

### Integrity

| Command | Purpose |
|---|---|
| `integrity append --event <json>` | Append a line-chained entry to `harness.log` |
| `integrity verify` | Verify the rolling SHA-256 chain across `harness.log` |

### Verification gates

| Command | Purpose |
|---|---|
| `gates run --phase <p> --gate <g> --run-id <r> --changes <json>` | Run syntactic/functional/regression gate |
| `gates run --gate semantic --verification <json>` | Structured judgment with evidence orthogonality check |

### Tools & permissions

| Command | Purpose |
|---|---|
| `tools list` | List registered tools |
| `tools invoke --tool <id> --args <json\|path> --run-id <r> --agent-id <a>` | Single gated dispatch: permission check → execute → ledger → episode |
| `permissions list` | Show all permission rules |
| `permissions check --tool <id> --args <json>` | Dry-run permission check |

### Built-in tools

| Tool | Capabilities | Description |
|---|---|---|
| `fs.read` | read | Read file as text or bytes |
| `fs.hash` | read | SHA-256 hash a file |
| `fs.edit` | read, write | Atomic in-place text replacement |
| `git.status` | read, exec | Run `git status` |
| `git.diff` | read, exec | Run `git diff` |
| `git.blame` | read, exec | Run `git blame` |
| `shell.exec` | exec | Spawn shell command with timeout |
| `test.run` | exec | Run configured test command |
| `lint.run` | exec | Run configured lint command |
| `web.fetch` | network | Fetch URL via curl |

### Budget & cost

| Command | Purpose |
|---|---|
| `spend append --json <j>` | Log an inference cost row |
| `spend summary --month <YYYY-MM>` | Totals by model/feature |

### Hooks

| Command | Purpose |
|---|---|
| `hook session-start` | Bootstrap verify + integrity log startup |
| `hook pre-tool --tool <t> --args <a>` | Permission check + cost charge + tool-call ledger |
| `hook post-tool --tool-call-id <id>` | Post-dispatch processing |

### Human interface

| Command | Purpose |
|---|---|
| `clarification list` | List open clarifications |
| `clarification resolve --id <id>` | Resolve a clarification |
| `incident declare --severity <s> --failure-mode <f>` | Declare a security incident |
| `incident list` | List incidents |
| `incident ack --id <id>` | Acknowledge an incident |
| `incident close --id <id>` | Close an incident |

### Security

| Command | Purpose |
|---|---|
| `redaction scan --content <text>` | Scan for secrets (AWS, PEM, GitHub PAT, JWT, GCP) |
| `redaction list` | List registered redaction patterns |
| `redaction add --regex <r> --id <id> --severity <s>` | Add a new pattern |

### Model routing

| Command | Purpose |
|---|---|
| `models list` | List configured models from `models.toml` |
| `models complete` | Model inference (stub) |

### MCP

| Command | Purpose |
|---|---|
| `mcp register --name <n> --command <c>` | Register an MCP server |
| `mcp start --name <n>` | Start a registered server |
| `mcp stop --name <n>` | Stop a running server |
| `mcp list` | List registered servers |

### Metrics & cadence

| Command | Purpose |
|---|---|
| `metrics rollup` | Aggregate KPI data |
| `metrics alert` | Check alert thresholds |
| `metrics health` | Harness health summary |
| `cadence run --period <daily\|weekly\|monthly>` | Enforce freshness gates |

### Eval

| Command | Purpose |
|---|---|
| `eval list` | List available evals |
| `eval run --id <id>` | Run a single eval |

---

## Repo structure

```
HighHarness/
├── HARNESS_ENGINEERING.md      # Binding rules + checklists (the constitution)
├── HARNESS_PRIMITIVES.md       # Interfaces & formats (961 lines)
├── HARNESS_SECURITY.md         # Threat model & mitigations (337 lines)
├── HARNESS_METRICS.md          # KPIs & self-evaluation (322 lines)
├── HARNESS_VERSIONING.md       # Bootstrap & upgrade protocol (307 lines)
├── readharness.md              # Human-friendly explainer
├── README.md                   # This file
├── LICENSE                     # MIT
├── Cargo.toml                  # Rust project manifest
├── Makefile                    # entry-1-demo, repro, docs targets
├── CHANGELOG.agent.md          # Append-only hash-chained change log
├── rust-toolchain.toml         # Pinned Rust version (1.78)
│
├── src/
│   ├── main.rs                 # Entry point → cli::run
│   ├── lib.rs                  # Crate root (22 pub modules)
│   ├── bootstrap.rs            # 10-step bootstrap protocol
│   ├── canonical.rs            # SHA-256 canonical serialization
│   ├── error.rs                # HxError (14 variants)
│   ├── gates.rs                # 4-gate runner + semantic gate
│   ├── id.rs                   # CSPRNG/pinned ID generators
│   ├── permissions.rs          # Permission engine
│   ├── redaction.rs            # Secret pattern scanning
│   ├── retrieval.rs            # Retrieval stub (grep-based)
│   ├── telemetry.rs            # Integrity log (chained JSONL)
│   ├── incident.rs             # Incident lifecycle
│   ├── metrics.rs              # KPI rollup & alerts
│   ├── eval.rs                 # Eval runner
│   ├── models/                 # Model registry & routing
│   ├── mcp/                    # MCP server management
│   ├── cli/                    # 21 CLI dispatch modules
│   ├── schema/                 # 13 serde struct modules
│   ├── store/                  # 8 storage backends
│   └── tools/                  # 12 built-in tool implementations
│
├── tests/                      # 17 integration test files
├── scripts/                    # Demo scaffolding + repro check
├── logs/episodes/              # Episode traces
└── evals/                      # Eval fixtures
```

---

## The five binding specs

| File | What it covers | Lines |
|---|---|---|
| `HARNESS_ENGINEERING.md` | Rules, checklists, anti-patterns, task tiers | 379 |
| `HARNESS_PRIMITIVES.md` | Tools, permissions, artifacts, concurrency, retrieval, models, gates, budgets | 961 |
| `HARNESS_SECURITY.md` | Threat model, 7 attack surfaces, F1–F4 detection, incident response | 337 |
| `HARNESS_METRICS.md` | 11 KPIs, collection, eval harness, review cadence | 322 |
| `HARNESS_VERSIONING.md` | Schema versioning, upgrade protocol, bootstrap 10-step | 307 |

Read order: `readharness.md` → `HARNESS_ENGINEERING.md` → `HARNESS_PRIMITIVES.md` → `HARNESS_SECURITY.md` → `HARNESS_METRICS.md` → `HARNESS_VERSIONING.md`.

---

## Verification gates

A change passes verification only when **all four gates pass** in order:

1. **Syntactic** — compiles, parses, or type-checks
2. **Functional** — relevant tests pass
3. **Semantic** — satisfies the original task spec (not just the tests)
4. **Regression** — previously passing tests still pass

Plus six completion requirements: attribution, memory, changelog entry, episode trace, permission check, no-side-effects.

---

## Testing

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test gate_semantic        # Semantic orthogonality
cargo test concurrency          # Race detection + lock contention
cargo test e2e_entry_1          # Full demo flow
cargo test canonical_golden     # SHA-256 golden fixtures
cargo test redaction            # Secret pattern scanning

# Run the canonical demo
make entry-1-demo

# Verify byte-level reproducibility
make repro

# Enforce documentation
cargo doc --no-deps -D missing-docs
```

---

## How it works

```
User Request
      ↓
Instructions Loaded (AGENTS.md, HARNESS_ENGINEERING.md)
      ↓
Episode Opened (logs/episodes/<run-id>.md)
      ↓
Tool Calls ← Permission Check ← Redaction ← Ledger
      ↓
Verification Gates (syntactic → functional → semantic → regression)
      ↓
Changelog Append (hash-chained, compare-and-append)
      ↓
Episode Closed (episode_hash computed)
```

Every step is logged. Every change is hash-chained. Every gate produces evidence. The harness mediates between the model and the environment.

---

## License

MIT — see [LICENSE](LICENSE).
