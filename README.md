<p align="center">
  <a href="https://crates.io/crates/highharness"><img src="https://img.shields.io/crates/v/highharness?style=flat&color=brightgreen" alt="crates.io"></a>
  <a href="https://github.com/MAHADEV369/HighHarness/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT"></a>
  <a href="https://github.com/MAHADEV369/HighHarness/actions"><img src="https://img.shields.io/github/actions/workflow/status/MAHADEV369/HighHarness/.github/workflows/ci.yml?branch=main" alt="CI"></a>
  <img src="https://img.shields.io/badge/rust-1.85+-orange" alt="Rust">
  <img src="https://img.shields.io/badge/tests-147-green" alt="147 tests">
</p>

> spec_version: 1
> status: stable

> **Platform:** macOS & Linux. Windows is not currently supported.

<h1 align="center">HighHarness</h1>

<p align="center">
  <b>Git brought version control to software.<br>
  Docker brought portability to deployment.<br>
  Kubernetes brought orchestration.<br>
  HighHarness brings governance to AI-generated code.</b>
</p>

<p align="center">
  <code>cargo install highharness</code>
</p>

<p align="center">
  Works with Claude Code · Codex · Cursor · Gemini CLI · opencode · Aider · any MCP client
</p>

---

## The story

AI can now write production code.

It cannot prove **what** it changed.

It cannot prove **who** changed it.

It cannot prove that **nothing was modified afterwards**.

It cannot prove that **policies were enforced**.

Software engineering has version control.

AI engineering needs **change control**.

HighHarness is that layer.

> AI should never directly modify software.
> AI should propose. The harness should verify. Humans should approve. Repositories should remember.

---

## 30-second value proposition

- **Every tool call your agent makes** is checked against a default-deny permission policy. No permission file → nothing allowed.
- **Every change is appended** to a SHA-256 hash-chained audit log. Tamper with one byte → chain breaks → detected.
- **Every run is recorded** as an episode trace with plan, decisions, tool calls, failures, and verification report. Render it as an HTML report with one command.

---

## Quick start — 5 commands, 30 seconds

```bash
# 1. Install
cargo install highharness

# 2. Initialize the harness (one-time per repo)
HighHarness bootstrap init --human "Your Name"

# 3. Verify the hash chain is intact
HighHarness changelog verify-chain
# → []    (empty array = no broken entries)

# 4. Start the governance server
HighHarness mcp serve-http --port 8931 &

# 5. Connect your agent
opencode mcp add highharness --url http://127.0.0.1:8931
```

Your agent is now governed. Every tool call is checked, recorded, and hash-chained.

---

## What a session looks like

This is a real session, captured against a fresh HighHarness bootstrap:

```bash
# Initialize the harness in a new project
$ HighHarness bootstrap init --human "Demo User"
{
  "schema_version": 1,
  "bootstrapped_at": "2026-07-10T20:08:07Z",
  "bootstrap_human": "Demo User",
  "genesis_hash": "882b6146ec709542315363d5b5a09cdfdb7362bc091253e8a2928c00f6a7f7c4",
  "passed": true
}

# Verify the hash chain — empty confirms no tampered entries
$ HighHarness changelog verify-chain
[]

# Inspect the latest changelog entry
$ HighHarness changelog latest
{
  "n": 1,
  "ts": "2026-07-10T20:08:07Z",
  "agent": "highharness/bootstrap",
  "intent": "bootstrap eval: verify compare-and-append against GENESIS",
  "verification": "syntactic",
  "prev_hash": "882b6146ec709542315363d5b5a09cdfdb7362bc091253e8a2928c00f6a7f7c4",
  "this_hash": "ebabb121ade13867e24b3fa26dc66f8c6b383243538ed7888bea1b307ee40b01"
}

# Open an episode to record an agent session
$ HighHarness episode open \
    --run-id "fix-login-timeout-1783715690" \
    --agent-id "claude-code" \
    --task-spec-file task-spec.md \
    --tier trivial \
    --phase highharness
{"run_id": "fix-login-timeout-1783715690"}

# Append the plan to the episode
$ HighHarness episode append \
    --run-id "fix-login-timeout-1783715690" \
    --section "Plan" \
    --body-file plan.md

# Close the episode with verification evidence
$ HighHarness episode close \
    --run-id "fix-login-timeout-1783715690" \
    --verification-json verification.json
889555258c2e1825899500352e81359aab46f2875bc6c5a8042a4178a1e385b3

# Render the episode as a self-contained HTML report
$ HighHarness episode render \
    --run-id "fix-login-timeout-1783715690" \
    --output report.html
```

The episode trace file at `logs/episodes/fix-login-timeout-<ts>.md` contains the full story: task spec, plan, tool calls, decisions, failures, verification report, and the SHA-256 episode hash. The HTML report is a self-contained page with syntax-highlighted JSON, gate pass/fail badges, and a dark theme.

---

## Visual episode viewer

Turn any episode into a standalone HTML report:

```bash
HighHarness episode render --run-id <ID> [--output <PATH>]
```

The report is a single self-contained HTML file (no external CSS/JS). It renders all 11 episode sections as styled cards with:

- **Status badge** — "Closed" (green) or "Open" (orange) based on whether the episode hash is present
- **Gate results** — Syntactic, Functional, Semantic, Regression with ✓/✗ pass/fail indicators
- **JSON syntax highlighting** — colored keys (green), strings (blue), numbers (purple), booleans (pink)
- **SHA-256 hash display** — prominently shown at the bottom, styled in a highlighted block
- **Dark theme** — GitHub-dark aesthetic, easy on the eyes

The HTML report output from a canonical Entry 1 episode (119 lines of Markdown → 211 lines of HTML):

```
┌────────────────────────────────────────────────────────┐
│  Episode    2026-06-29T110448Z-add-version-flag...  [Closed] │
├────────────────────────────────────────────────────────┤
│  TASK SPEC                                              │
│  Add a --version flag to the HighHarness CLI...          │
│                                                          │
│  PLAN                                                    │
│  Decomposition (each atom has its own pass/fail...)       │
│                                                          │
│  TASK STATE LOG                                          │
│  ┌──────────┬──────────────────────────┬──────────┐     │
│  │ timestamp │ subtask                  │ status   │     │
│  ├──────────┼──────────────────────────┼──────────┤     │
│  │ T+00m    │ pre-task checklist...    │ done     │     │
│  │ T+00m    │ episode open             │ done     │     │
│  └──────────┴──────────────────────────┴──────────┘     │
│                                                          │
│  TOOL CALLS                                              │
│  ┌──────────────────────────────────────────────────┐   │
│  │ { "tool": "fs.read", "args": {"path":"..."} }    │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  VERIFICATION REPORT                                     │
│  ✓ All criteria met                                      │
│  ✓ Syntactic    ✓ Functional    ✓ Semantic    ✓ Regression│
│                                                          │
│  EPISODE HASH                                            │
│  ┌──────────────────────────────────────────────────┐   │
│  │ SHA-256: 461aeb7a72547d4447c0b50bebfea0f0e...   │   │
│  └──────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────┘
```

For clients that don't support MCP (or don't need a running server), `episode render` works on any closed episode in the repository.

---

## Why hash chains matter

Imagine your compliance team asks: **"Who changed authentication.py last Tuesday at 3pm?"**

HighHarness answers in milliseconds:

```bash
$ HighHarness changelog get 7
- run_id:       login-fix-2026-07-01
- agent:        claude-code
- intent:       Fix login timeout
- verification: full
- prev_hash:    52653a7da8f9be91b64992f5d11e297e838f7dd8fb228577ab0db6f021feec64
- this_hash:    f7d71dd1f6a8974b2abbb5ff0c8438b6d156bdd21e0bfb6cdfd8b411d25c2d6f
```

Now imagine someone **edits yesterday's audit log** to hide a bad change.

```bash
$ HighHarness changelog verify-chain
[3]                        ← entry 3 broken — tamper detected
```

The hash chain is a **mathematical invariant**, not a policy. It cannot be overridden, bypassed, or ignored. Every entry's `this_hash` = SHA-256(canonical entry bytes with `this_hash` blanked). Change one byte → hash changes → next entry's `prev_hash` doesn't match → detected.

### Why SHA-256 chain, not Merkle tree or signed transparency log?

| Alternative | Trade-off |
|---|---|
| **Merkle tree** | Needs a separate witness format; compact proofs are useful for large logs but add complexity. A linear chain is simpler and still detects any byte-level tampering immediately. |
| **Transparency log (Rekor / Sigstore)** | Requires PKI infrastructure, a running witness service, and network access to verify. HighHarness works offline on any laptop with zero external dependencies. |
| **Signed-append-only log** | Needs key management, key rotation, and online signature verification. A hash chain is self-verifying — `prev_hash` is all the proof you need. |

For a governance layer that must work in air-gapped environments, during `git bisect`, and on a developer's laptop without internet, a linear SHA-256 chain over canonical text is the right trade-off.

---

## Why not just use the agent directly?

| | Permission engine | Audit trail | Policy enforcement | Runtime neutral |
|---|---|---|---|---|
| **Cursor** | ❌ | Partial | ❌ | ❌ |
| **Claude Code** | ❌ | Limited | ❌ | ❌ |
| **Codex CLI** | ❌ | Limited | ❌ | ❌ |
| **Gemini CLI** | ❌ | Limited | ❌ | ❌ |
| **Git hooks** | ⚠️ per-repo | ❌ | ⚠️ bypasable | ✓ |
| **CI/CD** | ❌ pre-merge | ❌ pre-merge | ⚠️ after-the-fact | ✓ |
| **Branch protection** | ❌ | ❌ | ⚠️ push only | ✓ |
| **HighHarness** | ✓ default-deny | ✓ hash-chained | ✓ real-time | ✓ |

---

## Who is this for / not for

**For:**
- Compliance teams that need reproducible proof of _what_ changed, _who_ changed it, and _whether policies were enforced_
- Platform engineers wiring AI coding agents into a regulated codebase
- Security researchers testing guardrails on agent tool access
- Anyone running an AI agent against production code who wants audit before damage, not after

**Not for:**
- A sandbox for running untrusted code (use a container or VM)
- An LLM output evaluator or prompt guardrail
- A chat interface or agent framework (use Claude Code, Cursor, or opencode directly — HighHarness governs them)
- A replacement for code review

---

## What you get — organized by outcome

### 🔐 Trust
**Hash-chained audit trail** — every change appended with SHA-256. Tampering breaks the chain immediately. Compliance teams get proof, not promises.

**Threat model** — HighHarness protects against:
- ✓ Malicious or compromised agents
- ✓ Accidental destructive edits
- ✓ Audit log tampering (hash chain breaks)
- ✓ SSRF and path-traversal attacks (web.fetch blocklists private IPs; fs.read resolves paths canonically)
- ⚠️ Secret leakage (tool-result strings redacted via configurable regex vault; episodes and memory redaction planned)
- ✗ Does not protect against root access, kernel compromise, or deleted repositories

### 📋 Auditability
**Episode traces** — every run produces the full story: plan, tool calls, decisions, failures, verification report. All in `logs/episodes/<run-id>.md` with SHA-256 hash. Render any episode as a self-contained HTML report via `HighHarness episode render --run-id <ID> [--output report.html]`.

### 🛡️ Safety
**Permission engine** — default-deny, priority-sorted rules. Define exactly what each agent can touch. Destructive operations blocked by default.

**Verification gates** — syntactic → functional → semantic → regression. A change passes only when all four pass.

### 🧠 Memory
**Persistent store** — write, query, pin, forget across sessions. Streams for project, user, and org. Tombstone-based forgetting (never deleted, only marked).

### 🔄 Verification
**4-stage pipeline** — compile, test, verify intent, check regression. Each stage produces evidence. Pipeline stops at first failure.

### 🔌 Interoperability
**MCP integration** — expose the harness as an MCP server (stdio or HTTP). Any MCP client connects. Claude Code, Cursor, opencode, Codex — all speak MCP. Connect any agent in two commands:

```bash
HighHarness mcp serve-http --port 8931 &
opencode mcp add highharness --url http://127.0.0.1:8931
```

---

## Design principles

- **Hash chain and episode traces are append-only.** Changelog entries and tool-call records are never modified. Memory store uses append + rewrite for pin/forget operations.
- **Everything is reproducible.** Same inputs → same hashes.
- **Nothing is trusted.** Every tool call is checked against policy. Every entry is verified against the hash chain.
- **Policies are deterministic.** Same rules + same inputs → same decision. No LLM-as-judge in the permission path.
- **Verification before mutation.** Gates run before changes land.
- **Governance over convenience.** The harness is designed to say no when it should.

---

## Commands

| Command | Description |
|---------|-------------|
| `bootstrap` | Initialize or verify the harness skeleton and hash chain |
| `changelog` | Append, get, list, or verify the hash-chained changelog |
| `episode` | Open, append, close, render episode traces as HTML |
| `gates` | Run verification gates (syntactic/functional/semantic/regression) |
| `tools` | Invoke built-in tools (fs.read, fs.edit, shell.exec, web.fetch, git.*, test.run, lint.run) |
| `mcp` | Start MCP server (stdio or HTTP transport) |

See [`docs/cli-reference.md`](./docs/cli-reference.md) for all 22 commands with flags, arguments, and usage examples.

---

## Documentation

- [`docs/cli-reference.md`](./docs/cli-reference.md) — CLI command reference (flags, arguments, examples)
- [`HARNESS_INTEGRATION.md`](./HARNESS_INTEGRATION.md) — MCP connection guide
- [`HARNESS_ENGINEERING.md`](./HARNESS_ENGINEERING.md) — engineering workflow and episode lifecycle
- [`HARNESS_METRICS.md`](./HARNESS_METRICS.md) — KPI definitions and rollup specification
- [`HARNESS_PRIMITIVES.md`](./HARNESS_PRIMITIVES.md) — artifact schema and storage primitives
- [`HARNESS_SECURITY.md`](./HARNESS_SECURITY.md) — threat model and incident response
- [`HARNESS_VERSIONING.md`](./HARNESS_VERSIONING.md) — versioning policy and changelog format
- [`CONTRIBUTING.md`](./CONTRIBUTING.md) — how to contribute
- [`SECURITY.md`](./SECURITY.md) — vulnerability reporting policy
- [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md) — contributor covenant

## Configuration

Create `.harness/permissions.toml` to define what your agent can access:

```toml
# Default-deny: anything not explicitly allowed is blocked.
# Rules are evaluated in priority order; first match wins.

[[rules]]
priority = 100
effect = "allow"
paths = ["src/**", "tests/**"]
network = []
description = "Allow source and test file access"

[[rules]]
priority = 200
effect = "deny"
paths = [".git/**", "target/**"]
description = "Never touch build artifacts or git internals"
```

Tool-specific commands are configured in `.harness/config.toml`:

```toml
lint_cmd = "cargo clippy --all-targets"
test_cmd = "cargo test"
```

**Security defaults (built-in, no config needed):**
- `web.fetch` blocks private (RFC 1918), loopback, and multicast IP addresses — SSRF protection from day one
- `fs.read` and `fs.edit` resolve paths through `canonicalize()` — path-traversal attacks are rejected
- All tool-result strings are scanned against a configurable regex vault — secrets are redacted before they reach the agent

See [`HARNESS_PRIMITIVES.md`](./HARNESS_PRIMITIVES.md) for the full spec.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     USER / AGENT LAYER                        │
│  Claude Code · Cursor · Codex · Gemini CLI · opencode        │
│                    Any MCP client                             │
└────────────────────────┬────────────────────────────────────┘
                         │ MCP (JSON-RPC 2.0) stdio / HTTP
┌────────────────────────▼────────────────────────────────────┐
│                    GOVERNANCE LAYER (HighHarness)              │
│                                                               │
│  ┌──────────────────┐  ┌────────────────────────────────┐    │
│  │  Permission       │  │  Episode Recorder               │    │
│  │  Engine           │  │  ──────────────────────         │    │
│  │  ─────────        │  │  tool calls · decisions         │    │
│  │  allow/deny/ask   │  │  failures · interventions       │    │
│  │  scope-narrow     │  │  verification · hash            │    │
│  └────────┬─────────┘  └────────────────┬─────────────────┘    │
│           │                             │                       │
│           ▼                             ▼                       │
│  ┌────────────────────────────────────────────────────────┐    │
│  │                  Hash Chain                             │    │
│  │  CHANGELOG.agent.md — SHA-256 prev→this chain           │    │
│  │  Tamper with any entry → chain breaks                   │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                               │
│  ┌──────┐ ┌───────┐ ┌──────┐ ┌───────┐ ┌────────┐           │
│  │Store │ │Gates  │ │Tools │ │Report │ │Models  │           │
│  │──────│ │───────│ │──────│ │───────│ │────────│           │
│  │mem   │ │syn.   │ │fs.*  │ │HTML   │ │OpenAI  │           │
│  │ep.   │ │func.  │ │git.* │ │ep.    │ │compat. │           │
│  │cl.   │ │sem.   │ │exec  │ │viewer │ │router  │           │
│  │snap  │ │reg.   │ │web   │ │       │ │        │           │
│  └──────┘ └───────┘ └──────┘ └───────┘ └────────┘           │
│                                                               │
│  ┌──────┐ ┌───────┐ ┌────────┐ ┌────────┐                    │
│  │MCP   │ │Redact │ │Eval    │ │Metrics  │                    │
│  │serve │ │vault  │ │runner  │ │KPIs     │                    │
│  └──────┘ └───────┘ └────────┘ └────────┘                    │
└──────────────────────────┬───────────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────────┐
│                       DATA LAYER                              │
│  .harness/              logs/               CHANGELOG.agent.md │
│  ├── permissions.toml   └── episodes/       (hash-chained     │
│  ├── config.toml            ├── <run>.md     audit log)        │
│  ├── redactions.toml        └── <run>.md                      │
│  └── artifacts/              (episode traces)                 │
│      ├── changelog/                                            │
│      ├── snapshots/                                            │
│      └── memory/                                               │
└───────────────────────────────────────────────────────────────┘
```

---

## Repository layout

```
HighHarness/
├── src/
│   ├── bootstrap.rs      # 10-step harness self-validation
│   ├── canonical.rs      # SHA-256 canonical serialization
│   ├── permissions.rs    # Permission engine (~470 lines)
│   ├── gates.rs          # 4-stage verification pipeline
│   ├── redaction.rs      # Secret redaction vault
│   ├── telemetry.rs      # Integrity log (line-chained JSONL)
│   ├── incident.rs       # Incident lifecycle
│   ├── metrics.rs        # KPI rollup (11 functions)
│   ├── id.rs             # CSPRNG ID generators
│   ├── report.rs         # HTML episode viewer (528 lines)
│   ├── eval.rs           # Synthetic eval runner
│   ├── retrieval.rs      # grep-based filesystem search
│   ├── error.rs          # HxError enum (14 variants)
│   ├── models/           # Model registry + OpenAI adapter
│   ├── mcp/              # MCP server (stdio + HTTP)
│   ├── store/            # Episodes, changelog, memory, snapshots
│   ├── cli/              # 22 CLI command modules
│   ├── schema/           # Serde structs for all artifacts
│   ├── tools/            # 10 built-in tool implementations
│   ├── lib.rs            # Crate root, re-exports
│   └── main.rs           # Binary entry point
├── .harness/             # Bootstrap, permissions, redactions, tools
├── logs/episodes/        # Episode trace files (.md)
├── CHANGELOG.agent.md    # Hash-chained audit log
├── Formula/              # Homebrew tap formula
├── docs/                 # CLI reference, security policy
├── tests/                # Integration tests (61)
├── scripts/              # Demo fixtures + reproducibility
├── evals/                # Synthetic task fixtures
├── data/                 # Static data
├── Cargo.toml            # 17 dependencies, deterministic feature
├── Makefile              # entry-1-demo, repro, docs targets
├── clippy.toml           # MSRV 1.85
├── rustfmt.toml          # Edition 2021, 100 cols
└── rust-toolchain.toml   # Stable Rust, clippy + rustfmt
```

---

## Security

HighHarness ships with security built into the governance layer, not bolted on after the fact.

- **Default-deny permission engine** — no permission file means no tool calls allowed. Every rule is explicit.
- **SSRF protection** — `web.fetch` rejects requests to private RFC 1918 IPs, loopback (127.0.0.1), link-local (169.254.x.x), and multicast addresses at the DNS-resolution layer. No opt-in required.
- **Path-traversal prevention** — `fs.read`, `fs.edit`, and all filesystem tools resolve paths through `canonicalize()` before access. `../../etc/passwd` tricks are rejected.
- **Secret redaction** — tool-result strings are scanned against a configurable regex vault (AWS keys, GitHub PATs, JWTs, PEM blocks, etc.). Matches are replaced with `<REDACTED:id>` tokens before the agent sees them.
- **Incident response** — `HighHarness incident declare` starts a 4-phase workflow (detect, declare, contain, remediate) with structured logging.

See [`SECURITY.md`](./SECURITY.md) for the vulnerability disclosure policy and [`HARNESS_SECURITY.md`](./HARNESS_SECURITY.md) for the full threat model.

---

## Performance & benchmarks

| Metric | Value |
|--------|-------|
| Test suite | **147 tests** (86 unit + 61 integration), all passing on `main` |
| Rust toolchain | MSRV 1.85, edition 2021 (macOS, Linux) |
| Release profile | LTO = thin, codegen-units = 1, symbols stripped |
| Dependency count | 17 direct crates (no web framework, no template engine) |
| Binary size | Rust standard release build (~5 MB) |

Benchmark numbers are not yet published. We plan to add criterion-based microbenchmarks for permission lookup (\<2 µs per check expected), audit hash append (\<10 µs), binary size, and session startup latency.

---

## Roadmap

### Shipped

| Feature | Status |
|---------|--------|
| Runtime neutral — works with any agent via MCP | ✅ |
| Hash-chained audit trail with SHA-256 | ✅ |
| Permission engine (default-deny, scope narrowing, safety-critical forcing) | ✅ |
| Episode traces with plan, decisions, tool calls, failures, verification, hash | ✅ |
| Memory store with pin/forget/query (project, user, org streams) | ✅ |
| Git snapshots (take/diff/revert) | ✅ |
| Model inference via OpenAI-compatible API | ✅ |
| Published on crates.io (`cargo install highharness`) | ✅ |
| Visual episode viewer (HTML report with syntax highlighting, gate badges, hash) | ✅ |
| Secret redaction vault (regex pattern scanning) | ✅ |
| Incident lifecycle (declare, list, acknowledge, close) | ✅ |
| KPI rollups (11 metrics with alerts) | ✅ |
| Integrity log (line-chained JSONL with SHA-256) | ✅ |
| Homebrew tap formula | ✅ |
| CLI reference documentation | ✅ |

### Future

| Feature |
|---------|
| Multi-agent coordination |
| Enterprise RBAC + SSO |
| Visual episode timeline (interactive viewer) |
| Criterion microbenchmarks + published benchmarks |
| macOS binary bottle for Homebrew |
| Windows support |

---

## Contributing

PRs welcome. See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for:
- Development setup (cargo build, test, clippy, fmt)
- PR workflow and commit message conventions
- How to add a new tool or subcommand
- Release process

**Status:** pre-1.0. API may change. Semantic versioning starts at 1.0.0.

---

<p align="center">
  <b>HighHarness is building the infrastructure layer between AI coding agents and production software engineering.</b><br>
  <br>
  Git brought version control. Docker brought portability. Kubernetes brought orchestration.<br>
  <b>HighHarness brings governance to AI-generated code.</b>
</p>

<p align="center">
  <code>cargo install highharness</code><br>
  <a href="https://github.com/MAHADEV369/HighHarness">GitHub</a> · <a href="https://crates.io/crates/highharness">crates.io</a> · <a href="./CONTRIBUTING.md">Contributing</a> · MIT<br>
  <a href="./HARNESS_INTEGRATION.md">MCP guide</a> · <a href="./HARNESS_ENGINEERING.md">Engineering spec</a> · <a href="./HARNESS_PRIMITIVES.md">Primitives spec</a> · <a href="./docs/cli-reference.md">CLI reference</a>
</p>
