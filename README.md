<p align="center">
  <a href="https://crates.io/crates/highharness"><img src="https://img.shields.io/crates/v/highharness?style=flat&color=brightgreen" alt="crates.io"></a>
  <a href="https://github.com/MAHADEV369/HighHarness/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT"></a>
  <a href="https://github.com/MAHADEV369/HighHarness/actions"><img src="https://img.shields.io/github/actions/workflow/status/MAHADEV369/HighHarness/.github/workflows/ci.yml?branch=main" alt="CI"></a>
  <img src="https://img.shields.io/badge/rust-1.85+-orange" alt="Rust">
</p>

<h1 align="center">HighHarness</h1>
<p align="center"><b>HighHarness turns LLM-generated code edits into deterministic, auditable, attributable software changes.</b></p>

<p align="center">
  <code>cargo install highharness</code>
</p>

<p align="center">
  Works with Claude Code · Codex · Cursor · Gemini CLI · opencode · Aider · any MCP client
</p>

---

## Problem

Modern AI coding tools can:

❌ Silently edit files without oversight

❌ Overwrite unrelated code

❌ Hallucinate changes that don't match intent

❌ Lose attribution — no record of who or what changed what

❌ Skip verification before modifying the repo

❌ Produce unauditable commits that compliance cannot accept

## Solution

HighHarness sits between any AI agent and your codebase. Every tool call is **permissioned, recorded, and hash-chained**. Nothing happens without a trace.

> AI should never directly modify software.
> AI should propose. The harness should verify. Humans should approve. Repositories should remember.

---

## Why not just use Cursor / Claude Code / Codex directly?

| | Agent | IDE/CLI | Audit trail | Permission engine | Runtime neutral |
|---|---|---|---|---|---|
| **Cursor** | ✓ | IDE only | Partial | ❌ | ❌ |
| **Claude Code** | ✓ | CLI | Limited | ❌ | ❌ |
| **Codex CLI** | ✓ | CLI | Limited | ❌ | ❌ |
| **Gemini CLI** | ✓ | CLI | Limited | ❌ | ❌ |
| **HighHarness** | ✓ | Any (MCP) | **Full — hash-chained** | ✓ default-deny | ✓ |

Every agent gives you the model. HighHarness gives you the **governance layer** — and works with ALL of them.

---

## The tamper-proof audit trail

This is what makes HighHarness different. Every action is recorded in a **SHA-256 hash chain**. Tamper with any entry — the chain breaks immediately.

```
$ HighHarness changelog verify-chain
[]                                                  ← empty = chain valid

    ↓ edit CHANGELOG.agent.md (tamper with entry 3) ↓

$ HighHarness changelog verify-chain
[3]                                                 ← entry 3 broken
```

**Not a promise. SHA-256.** Each entry's `this_hash` = SHA-256(canonical entry bytes with `this_hash` blanked). Change one byte → hash changes → next entry's `prev_hash` won't match → `verify-chain` catches it. Anyone can recompute every hash independently.

Run the proof yourself:

```bash
bash scripts/prove_hash_chain.sh
# ✅ valid → ✏️ tamper → 🚨 detected → ✅ restored
```

---

## What it does

| Capability | What HighHarness enforces |
|---|---|
| **🔗 Hash-chained audit trail** | Every change appended with SHA-256 chaining. Tampering breaks the chain visibly. |
| **🛡️ Permission engine** | Default-deny, priority-sorted rules. Destructive operations blocked by default. |
| **📜 Episode traces** | Every run produces the full story: plan, tool calls, decisions, failures, verification. |
| **🧠 Memory** | Persistent key-value store with streams, pin/forget, query across sessions. |
| **🔍 Verification gates** | Syntactic → Functional → Semantic → Regression. |
| **🤖 Model inference** | Call OpenAI-compatible models via `OPENAI_API_KEY`. |
| **🔌 MCP integration** | Expose the harness as an MCP server (stdio or HTTP). Any MCP client connects. |
| **🔄 Git snapshots** | Take, diff, and revert point-in-time snapshots for safe rollback. |
| **📋 Clarifications** | Request, list, and resolve persistent clarification requests. |
| **✂️ Secret redaction** | Regex vault catches AWS keys, PEMs, GitHub PATs, JWTs, GCP keys before they leak. |

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

## Architecture

```
Developer prompt
     │
     ▼
Agent (Claude Code / Cursor / opencode)
     │
     │  MCP (JSON-RPC 2.0 over stdio or HTTP)
     ▼
┌─────────────────────────────────────────────────────┐
│                  HighHarness                         │
│                                                     │
│  ┌──────────────┐  ┌───────────────────────────┐    │
│  │ Permission   │  │  Episode Recorder         │    │
│  │ Engine       │  │  ─ every call logged      │    │
│  │ allow/deny   │  │  ─ SHA-256 hash computed  │    │
│  │ /ask         │  │  ─ full trace preserved   │    │
│  └──────┬───────┘  └───────────┬───────────────┘    │
│         │                      │                    │
│  ┌──────▼──────────────────────▼───────────────┐    │
│  │         Hash Chain Append                   │    │
│  │  CHANGELOG.agent.md — prev→this chain       │    │
│  └──────────────────┬──────────────────────────┘    │
│                     │                               │
│  ┌──────────────────▼──────────────────────────┐    │
│  │ Memory · Snapshots · Clarifications · Model │    │
│  └─────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
                      │
                      ▼
           Filesystem · Git · Shell · Network
```

A **5.6MB static Rust binary**. No Python, no Docker, no Postgres. One binary that sits between any agent and your infrastructure.

---

## For the skeptic — the hash chain math

```
ENTRY 7:
  prev_hash: 52653a7da8f9be91b64992f5d11e297e838f7dd8fb228577ab0db6f021feec64
  this_hash: f7d71dd1f6a8974b2abbb5ff0c8438b6d156bdd21e0bfb6cdfd8b411d25c2d6f

ENTRY 8:
  prev_hash: f7d71dd1f6a8974b2abbb5ff0c8438b6d156bdd21e0bfb6cdfd8b411d25c2d6f
  this_hash: c2652e5d03d71c14d5e42128b1a6425bc1827df7cc0ea550e316a51f0b1f0261
                                  ↑
                        ENTRY 8's prev_hash
                        MUST equal ENTRY 7's this_hash
```

Change any byte in ENTRY 7 → ENTRY 7's `this_hash` changes → ENTRY 8's `prev_hash` doesn't match → `verify-chain` reports ENTRY 8 as broken.

The hash chain is a **mathematical invariant**, not a policy. It cannot be overridden, bypassed, or ignored.

---

## Internal design

| Layer | Responsibility |
|---|---|
| **Permission engine** | Default-deny rule evaluation with predicate matching |
| **Episode recorder** | Full trace capture — plan, calls, decisions, failures, verification |
| **Hash chain** | SHA-256 chained changelog with canonical byte serialization |
| **Memory store** | JSONL-backed persistent streams with tombstone-based forgetting |
| **Git snapshots** | Point-in-time workspace captures with diff and revert |
| **Model adapter** | OpenAI-compatible HTTP inference via `OPENAI_API_KEY` |
| **MCP server** | JSON-RPC 2.0 over stdio or HTTP — any client can connect |
| **Secret redaction** | Regex-based vault scanning all tool results and episodes |

---

## Repository layout

```
src/
├── bootstrap.rs      # 10-step harness self-validation
├── canonical.rs      # SHA-256 canonical serialization
├── permissions.rs    # Permission engine
├── gates.rs          # 4-stage verification pipeline
├── models/           # Model registry + OpenAI-compatible adapter
├── mcp/              # MCP server (stdio + HTTP)
├── store/            # Episodes, changelog, memory, snapshots
├── cli/              # 22 CLI command modules
├── schema/           # Serde structs for all artifacts
└── tools/            # 10 built-in tool implementations
```

---

## Why this exists

Modern AI agents can write code. They cannot yet guarantee:

- **Attribution** — who or what made this change?
- **Auditability** — can we prove nothing was tampered with?
- **Reproducibility** — can we replay the exact session?
- **Deterministic verification** — does the change actually work?

HighHarness exists to solve that layer. It is the infrastructure between AI coding agents and production software engineering.

---

## Install

| Method | Command |
|--------|---------|
| **cargo** | `cargo install highharness` |
| **From source** | `git clone https://github.com/MAHADEV369/HighHarness.git && cd HighHarness && cargo build --release` |
| **Script** | `curl -fsSL https://raw.githubusercontent.com/MAHADEV369/HighHarness/main/scripts/install.sh \| bash` |

## Connect your agent

| Agent | How |
|-------|------|
| **opencode** | `opencode mcp add highharness --url http://127.0.0.1:8931` |
| **Claude Code** | Add to `~/.claude/claude_desktop_config.json` (see [docs](./HARNESS_INTEGRATION.md)) |
| **Cursor** | Settings → Features → MCP Servers → Add: `HighHarness mcp serve` |
| **Codex CLI** | Connect via MCP stdio: `HighHarness mcp serve` |
| **Any MCP client** | `HighHarness mcp serve` (stdio) or `HighHarness mcp serve-http` (HTTP) |

## Key commands

| Command | What |
|---------|------|
| `mcp serve` | Start MCP server (stdio, for local agents) |
| `mcp serve-http --port 8931` | Start MCP server (HTTP, for opencode/remote) |
| `changelog verify-chain` | Validate the hash chain |
| `bootstrap verify` | Check harness integrity |
| `models complete` | Call OpenAI-compatible models via `OPENAI_API_KEY` |
| `memory write / query / forget` | Persistent agent memory |
| `snapshot take / diff / revert` | Git snapshots |
| `clarification request / list / resolve` | Persistent clarifications |

## Roadmap

| Status | Feature |
|---|---|
| ✅ | Runtime neutral — works with any agent via MCP |
| ✅ | Hash-chained audit trail |
| ✅ | Permission engine (default-deny) |
| ✅ | Episode traces with SHA-256 |
| ✅ | Memory store |
| ✅ | Git snapshots |
| 🛠️ | Multi-agent coordination |
| 🔜 | Visual episode viewer (HTML report) |
| 🔜 | Brew tap distribution |
| 🔜 | Enterprise RBAC + SSO |

---

<p align="center">
  <b>HighHarness is building the infrastructure layer between AI coding agents and production software engineering.</b><br>
  If you're interested in trustworthy AI software development, contributions and discussions are welcome.
</p>

<p align="center">
  <code>cargo install highharness</code><br>
  <a href="https://github.com/MAHADEV369/HighHarness">GitHub</a> · <a href="https://crates.io/crates/highharness">crates.io</a> · MIT<br>
  <a href="./HARNESS_INTEGRATION.md">MCP connection guide</a>
</p>
