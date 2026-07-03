<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT">
  <img src="https://img.shields.io/badge/rust-1.85+-orange" alt="Rust">
  <img src="https://img.shields.io/badge/status-beta-yellow" alt="Beta">
</p>

<h1 align="center">HighHarness</h1>
<p align="center"><b>Governance for AI coding agents.</b><br>
Every agent action is permissioned, recorded, and tamper-evident.</p>

<p align="center">
  <code>cargo install highharness</code><br>
  <code>git clone https://github.com/MAHADEV369/HighHarness.git && cd HighHarness && cargo build --release</code>
</p>

<p align="center">
  <i>Works with Claude Code · Cursor · opencode · any MCP client</i>
</p>

---

## Why

AI coding agents write code fast. They also `rm -rf /`, delete migrations, and edit files they shouldn't. HighHarness sits between your agent and your codebase and says **"no"** when it should.

## Quick start

```bash
# 1. Install
brew install MAHADEV369/tap/highharness

# 2. Start the governance server (in background)
HighHarness mcp serve-http --port 8931 &

# 3. Connect your agent
opencode mcp add highharness --url http://127.0.0.1:8931
# or add to Claude Code / Cursor MCP config (see HARNESS_INTEGRATION.md)

# 4. Your agent is now governed. Try it:
python3 highguard.py run add-version-flag

# 5. See what happened:
python3 highguard.py report
python3 highguard.py verify
```

## What it does

**Permission engine** — define rules in `.harness/permissions.toml`:
```toml
[[rules]]
effect = "deny"
tool = "shell.exec"
reason = "Shell commands blocked by default"
```

**Episode traces** — every session recorded in `logs/episodes/` with SHA-256 hash:
```
## Tool calls
- fs.read Cargo.toml → allowed
- shell.exec rm -rf / → DENIED (Destructive shell blocked)
## Episode hash
SHA-256: c06a2a2541b39ee161afa0252d12bb2bce4b2be4f64771acc636361c4e1ec314
```

**Hash-chained changelog** — tamper with any entry, the chain breaks immediately:
```bash
HighHarness changelog verify-chain
# → [] if valid
# → [3] if entry 3 was tampered
```

**The hash chain is not a promise. It's a mathematical proof.** Every entry's `this_hash` is `SHA-256(canonical_entry_bytes)`. Change one byte → hash changes → chain breaks → `verify-chain` catches it. Anyone can recompute every hash independently using the canonical serializer.

## Install

| Method | Command | Status |
|--------|---------|--------|
| **cargo** | `cargo install highharness` | ✅ Works now |
| **From source** | `git clone` + `cargo build --release` | Works now |
| **Script** | `curl -fsSL https://raw.githubusercontent.com/MAHADEV369/HighHarness/main/scripts/install.sh \| bash` | Works now |
| **Homebrew tap** | `brew install MAHADEV369/tap/highharness` | Pending — requires `github.com/MAHADEV369/homebrew-tap` repo |

## Connect your agent

| Agent | How |
|-------|-----|
| **opencode** | `opencode mcp add highharness --url http://127.0.0.1:8931` |
| **Claude Code** | Add to `~/.claude/claude_desktop_config.json` (see docs) |
| **Cursor** | Settings → MCP Servers → Add: `HighHarness mcp serve` |
| **Any MCP client** | Connect over stdio or HTTP (see `HARNESS_INTEGRATION.md`) |

## Demo: tamper-proof audit

```bash
# Run this to see the hash chain in action:
bash scripts/prove_hash_chain.sh

# It shows:
#   ✅ Chain valid → ✏️ Tamper → 🚨 Chain broken! → ✅ Restored
```

## Commands

| Command | What |
|---------|------|
| `mcp serve` | Start MCP server (stdio, for local agents) |
| `mcp serve-http` | Start MCP server (HTTP, for opencode/remote) |
| `changelog verify-chain` | Validate the hash chain |
| `bootstrap verify` | Check harness integrity |
| `models complete` | Call OpenAI-compatible models |
| `memory write/query/forget` | Persistent agent memory |
| `snapshot take/diff/revert` | Git snapshots |

## Under the hood

```
Agent ──MCP──► HighHarness ──check──► Permission Engine
                   │
                   └── record ──► Episode Trace (SHA-256)
                   │
                   └── append ──► CHANGELOG.agent.md (hash chain)
                   │
                   └── store ──► Memory · Snapshots · Clarifications
```

A single 5.6MB Rust binary. No Python, no Docker, no Postgres.

## Homebrew tap setup

For `brew install MAHADEV369/tap/highharness`, create a tap repository:

```bash
# One-time: create the tap repo
gh repo create MAHADEV369/homebrew-tap --public --clone
cd homebrew-tap
mkdir Formula
cp /path/to/HighHarness/Formula/highharness.rb Formula/
git add -A && git commit -m "Add highharness formula"
git push origin main

# Then anyone can install:
brew install MAHADEV369/tap/highharness
```

For `brew install highharness` (no prefix), submit a PR to [Homebrew/homebrew-core](https://github.com/Homebrew/homebrew-core) once the project has a stable release tag.

## License

MIT
