<p align="center">
  <a href="https://crates.io/crates/highharness"><img src="https://img.shields.io/crates/v/highharness?style=flat&color=brightgreen" alt="crates.io"></a>
  <a href="https://github.com/MAHADEV369/HighHarness/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT"></a>
  <a href="https://github.com/MAHADEV369/HighHarness/actions"><img src="https://img.shields.io/github/actions/workflow/status/MAHADEV369/HighHarness/.github/workflows/ci.yml?branch=main" alt="CI"></a>
  <img src="https://img.shields.io/badge/rust-1.85+-orange" alt="Rust">
</p>

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

## How it works

```
Developer
    │
    │  "Fix the login timeout bug"
    ▼
AI Agent (Claude Code / Cursor / Codex / opencode)
    │
    │  Agent plans changes, calls tools
    ▼
┌─────────────────────────────────────────────────────────────┐
│                      HighHarness                             │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Permission   │  │  Episode     │  │  Verification    │  │
│  │  Engine       │──│  Recorder    │──│  Gates           │  │
│  │  allow/deny   │  │  every call  │  │  4-stage check   │  │
│  │  /ask         │  │  logged      │  │                  │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │            │
│  ┌──────▼─────────────────▼────────────────────▼─────────┐  │
│  │                  Hash Chain                           │  │
│  │  CHANGELOG.agent.md — SHA-256 prev→this chain         │  │
│  │  Tamper with any entry → chain breaks                 │  │
│  └──────────────────────┬────────────────────────────────┘  │
│                         │                                   │
│  ┌──────────────────────▼────────────────────────────────┐  │
│  │  Memory · Snapshots · Clarifications · Model Route    │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
Filesystem · Git · Shell · Network
```

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

## What you get — organized by outcome

### 🔐 Trust
**Hash-chained audit trail** — every change appended with SHA-256. Tampering breaks the chain immediately. Compliance teams get proof, not promises.

**Threat model** — HighHarness protects against:
- ✓ Malicious or compromised agents
- ✓ Accidental destructive edits
- ✓ Audit log tampering (hash chain breaks)
- ✓ Secret leakage (AWS keys, PEMs, PATs redacted)
- ✗ Does not protect against root access, kernel compromise, or deleted repositories

### 📋 Auditability
**Episode traces** — every run produces the full story: plan, tool calls, decisions, failures, verification report. All in `logs/episodes/<run-id>.md` with SHA-256 hash.

### 🛡️ Safety
**Permission engine** — default-deny, priority-sorted rules. Define exactly what each agent can touch. Destructive operations blocked by default.

**Verification gates** — syntactic → functional → semantic → regression. A change passes only when all four pass.

### 🧠 Memory
**Persistent store** — write, query, pin, forget across sessions. Streams for project, user, and org. Tombstone-based forgetting (never deleted, only marked).

### 🔄 Verification
**4-stage pipeline** — compile, test, verify intent, check regression. Each stage produces evidence. Pipeline stops at first failure.

### 🔌 Interoperability
**MCP integration** — expose the harness as an MCP server (stdio or HTTP). Any MCP client connects. Claude Code, Cursor, opencode, Codex — all speak MCP.

---

## Design principles

- **Everything is append-only.** No edits, no deletes. Reverting is a new entry.
- **Everything is reproducible.** Same inputs → same hashes.
- **Nothing is trusted.** Every tool call is checked against policy. Every entry is verified against the hash chain.
- **Policies are deterministic.** Same rules + same inputs → same decision. No LLM-as-judge in the permission path.
- **Verification before mutation.** Gates run before changes land.
- **Governance over convenience.** The harness is designed to say no when it should.

---

## Quick start

```bash
# 1. Install
cargo install highharness

# 2. Start governance
HighHarness mcp serve-http --port 8931 &

# 3. Connect your agent
opencode mcp add highharness --url http://127.0.0.1:8931
```

Your agent is now governed. Every tool call is checked, recorded, and hash-chained.

---

## Benchmarks

| Metric | Value |
|--------|-------|
| Binary size | **5.6 MB** (static, no dependencies) |
| Permission lookup | **0.3 ms** (in-process, deterministic) |
| Audit append | **0.7 ms** (SHA-256 + file append) |
| Session startup | **18 ms** (MCP server init) |
| Episode hash | **SHA-256**, canonical serialization |
| Test suite | **142 tests**, all passing |
| Rust toolchain | MSRV 1.85, edition 2021 |

---

## Internal design

| Layer | What it does |
|---|---|
| **Permission engine** | Default-deny, priority-sorted rule evaluation with path/network/env predicates |
| **Episode recorder** | Full trace capture — plan, calls, decisions, failures, verification, hash |
| **Hash chain** | SHA-256 chained changelog with canonical byte serialization per spec |
| **Memory store** | JSONL-backed persistent streams with tombstone-based forgetting |
| **Git snapshots** | Point-in-time workspace captures with diff and revert |
| **Model adapter** | OpenAI-compatible HTTP inference via `OPENAI_API_KEY` |
| **MCP server** | JSON-RPC 2.0 over stdio or HTTP — any MCP client connects |
| **Verification gates** | 4-stage pipeline: syntactic → functional → semantic → regression |
| **Secret redaction** | Regex vault scanning all tool results, episodes, and memory writes |

---

## Repository layout

```
src/
├── bootstrap.rs      # 10-step harness self-validation
├── canonical.rs      # SHA-256 canonical serialization per spec
├── permissions.rs    # Permission engine (464 lines)
├── gates.rs          # 4-stage verification pipeline
├── models/           # Model registry + OpenAI-compatible adapter
├── mcp/              # MCP server (stdio + HTTP transports)
├── store/            # Episodes, changelog, memory, snapshots, approvals
├── cli/              # 22 CLI command modules
├── schema/           # Serde structs for all artifacts
└── tools/            # 10 built-in tool implementations
```

---

## Roadmap

| Status | Feature |
|---|---|
| ✅ | Runtime neutral — works with any agent via MCP |
| ✅ | Hash-chained audit trail |
| ✅ | Permission engine (default-deny) |
| ✅ | Episode traces with SHA-256 |
| ✅ | Memory store with pin/forget/query |
| ✅ | Git snapshots (take/diff/revert) |
| ✅ | Model inference via OpenAI-compatible API |
| ✅ | Published on crates.io (`cargo install`) |
| 🛠️ | Visual episode viewer (HTML report) |
| 🛠️ | Brew tap distribution |
| 🔜 | Multi-agent coordination |
| 🔜 | Enterprise RBAC + SSO |

---

<p align="center">
  <b>HighHarness is building the infrastructure layer between AI coding agents and production software engineering.</b><br>
  <br>
  Git brought version control. Docker brought portability. Kubernetes brought orchestration.<br>
  <b>HighHarness brings governance to AI-generated code.</b>
</p>

<p align="center">
  <code>cargo install highharness</code><br>
  <a href="https://github.com/MAHADEV369/HighHarness">GitHub</a> · <a href="https://crates.io/crates/highharness">crates.io</a> · MIT<br>
  <a href="./HARNESS_INTEGRATION.md">MCP connection guide</a>
</p>
