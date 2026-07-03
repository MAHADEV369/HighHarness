<p align="center">
  <a href="https://crates.io/crates/highharness"><img src="https://img.shields.io/crates/v/highharness?style=flat&color=brightgreen" alt="crates.io"></a>
  <a href="https://github.com/MAHADEV369/HighHarness/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT"></a>
  <img src="https://img.shields.io/badge/rust-1.85+-orange" alt="Rust">
  <img src="https://img.shields.io/badge/status-beta-yellow" alt="Beta">
</p>

<h1 align="center">HighHarness</h1>
<p align="center"><b>Governance for AI coding agents.</b><br>
Every agent action is permissioned, recorded, and tamper-evident.</p>

<p align="center">
  <code>cargo install highharness</code>
</p>

<p align="center">
  Works with Claude Code · Cursor · opencode · any MCP client
</p>

> The fix for unreliable AI agents is almost never a bigger model. It is a better harness.

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
| **🔄 Git snapshots** | Take, diff, and revert point-in-time snapshots. |
| **📋 Clarifications** | Request, list, and resolve persistent clarification requests. |
| **✂️ Secret redaction** | Regex vault catches AWS keys, PEMs, GitHub PATs, JWTs, GCP keys. |

---

## The tamper-proof audit trail

This is the one thing that makes HighHarness different. Every action is recorded in a **hash chain**. Tamper with any entry — the chain breaks immediately.

```
$ HighHarness changelog verify-chain
[]                                                  ← empty = chain valid

    ↓ edit CHANGELOG.agent.md (tamper with entry 3) ↓

$ HighHarness changelog verify-chain
[3]                                                 ← entry 3 broken
```

**Not a promise. SHA-256.** Each entry's `this_hash` is computed over the canonical entry bytes with `this_hash` blanked. Change one byte — the hash changes. The next entry's `prev_hash` won't match. `verify-chain` catches it.

Run the proof yourself:

```bash
bash scripts/prove_hash_chain.sh
# ✅ valid → ✏️ tamper → 🚨 detected → ✅ restored
```

---

## Quick start — install, start, connect

```bash
# 1. Install
cargo install highharness

# 2. Start governance (in background)
HighHarness mcp serve-http --port 8931 &

# 3. Connect your agent
opencode mcp add highharness --url http://127.0.0.1:8931
```

Your agent is now governed. Every tool call is checked, recorded, and hash-chained.

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

If you change any byte in ENTRY 7:
- ENTRY 7's `this_hash` changes
- ENTRY 8's `prev_hash` no longer matches
- `HighHarness changelog verify-chain` reports ENTRY 8 as broken

The hash chain is a **mathematical invariant**, not a policy. It cannot be overridden, bypassed, or ignored.

---

## Architecture

```
Agent (Claude Code / Cursor / opencode)
    │
    │  MCP (JSON-RPC 2.0 over stdio or HTTP)
    ▼
HighHarness
    │
    ├── Permission engine ──► allow / deny / ask
    ├── Episode recording ──► logs/episodes/<id>.md + SHA-256
    ├── Hash chain append ──► CHANGELOG.agent.md
    ├── Memory store ──────► .harness/artifacts/memory/
    ├── Model inference ───► OpenAI-compatible APIs
    └── Git snapshots ─────► take, diff, revert
         │
         ▼
    Filesystem · Git · Shell · Network
```

A single **5.6MB Rust binary**. No Python, no Docker, no Postgres.

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

---

<p align="center">
  <code>cargo install highharness</code><br>
  <a href="https://github.com/MAHADEV369/HighHarness">GitHub</a> · <a href="https://crates.io/crates/highharness">crates.io</a> · MIT<br>
  <a href="./HARNESS_INTEGRATION.md">MCP connection guide</a>
</p>
