# HighHarness

**Runtime-neutral agent governance for AI coding agents.**

HighHarness sits between any AI agent (Cursor, Claude Code, opencode) and your codebase.
It enforces permissions, records every action in tamper-evident episode traces, and
produces hash-chained audit trails — so every agent change is auditable and safe.

> The fix for unreliable AI agents is almost never a bigger model. It is a better harness.

---

## Quick start

```bash
# Install (macOS, Linux, Windows)
cargo install highharness

# Or build from source
git clone https://github.com/MAHADEV369/HighHarness.git
cd HighHarness
cargo build --release

# Start the governance MCP server (agents connect here)
./target/release/HighHarness mcp serve         # stdio mode (for Claude Code, Cursor)
./target/release/HighHarness mcp serve-http     # HTTP mode (for opencode, remote clients)

# Test it works
python3 highguard.py run add-version-flag
python3 highguard.py report
```

---

## What it does

| Capability | What HighHarness enforces |
|---|---|
| **Permission engine** | Default-deny, priority-sorted rules. Destructive operations blocked by default. Customizable via `.harness/permissions.toml`. |
| **Episode traces** | Every agent session produces `logs/episodes/<run-id>.md` — a complete recording of every tool call, decision, and denial, with SHA-256 tamper-evident hash. |
| **Hash-chained changelog** | Every change appended to `CHANGELOG.agent.md` with SHA-256 `prev_hash` → `this_hash` chaining. Tampering breaks the chain visibly. |
| **Memory** | Persistent key-value store with streams, tombstones, pin/forget/query. |
| **Snapshots** | Git-based point-in-time snapshots with diff and revert. |
| **Model inference** | `HighHarness models complete` — calls OpenAI-compatible APIs via `OPENAI_API_KEY`. |
| **Secret redaction** | Regex-based vault scans tool results and episodes for AWS keys, PEMs, GitHub PATs, JWTs, GCP keys. |
| **MCP integration** | Expose the harness as an MCP server (stdio or HTTP). Any MCP-compatible agent connects and gets governance. |

---

## Connect your agent

### opencode

```bash
# 1. Start HighHarness in HTTP mode
HighHarness mcp serve-http --port 8931

# 2. Register with opencode
opencode mcp add highharness --url http://127.0.0.1:8931

# 3. Verify
opencode mcp list
# → highharness connected
```

### Claude Code

Add to `~/.claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "highharness": {
      "command": "/path/to/HighHarness",
      "args": ["mcp", "serve"],
      "env": {}
    }
  }
}
```

### Cursor

1. Settings → Features → MCP Servers → Add
2. Name: `highharness`, Type: `command`, Command: `/path/to/HighHarness mcp serve`

### Any MCP client

```python
# See highguard.py and demo_agent.py for complete examples
import subprocess, json

proc = subprocess.Popen(["HighHarness", "mcp", "serve"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE)

# Initialize
proc.stdin.write(b'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}\n')

# List tools
proc.stdin.write(b'{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}\n')

# Call a tool (permissions are enforced)
proc.stdin.write(b'{"jsonrpc":"2.0","id":3,"method":"tools/call",'
    b'"params":{"name":"fs.read","arguments":{"path":"Cargo.toml"}}}\n')
```

---

## Usage

### Safe Agent Runner

```bash
# Run a governed agent task
python3 highguard.py run add-version-flag

# View session report
python3 highguard.py report

# Export episodes as training data (OpenAI format)
python3 highguard.py export openai

# Export as HuggingFace dataset format
python3 highguard.py export hf

# Verify audit chain
python3 highguard.py verify
```

### Permission configuration

Permissions are defined in `.harness/permissions.toml`:

```toml
[[rules]]
id = "allow-reads"
effect = "allow"
tool = "fs.read"
paths = ["**"]
reason = "Allow reading any file"
priority = 50

[[rules]]
id = "deny-infra"
effect = "deny"
tool = "fs.edit"
paths = [".harness/**", ".git/**"]
reason = "Protected infrastructure"
priority = 60

[[rules]]
id = "deny-destructive-shell"
effect = "deny"
tool = "shell.exec"
reason = "Destructive shell patterns blocked"
priority = 60
```

### Episode traces

Every MCP session produces an episode trace at `logs/episodes/<run-id>.md`:

```
# Episode mcp-2026-07-02T19:11:24Z

## Task spec
MCP governance session

## Tool calls
- {"tool":"fs.read","result_summary":"[package] name = \"highharness\"..."}
- {"tool":"shell.exec","result_summary":"DENIED: Destructive shell patterns blocked"}

## Episode hash
SHA-256: c06a2a2541b39ee161afa0252d12bb2bce4b2be4f64771acc636361c4e1ec314
```

---

## CLI commands

| Command | Purpose |
|---|---|
| `mcp serve` | Start MCP server over stdio (for Claude Code, Cursor) |
| `mcp serve-http` | Start MCP server over HTTP (for opencode) |
| `bootstrap init` | Initialize harness config |
| `bootstrap verify` | Verify bootstrap state |
| `changelog verify-chain` | Verify hash chain integrity |
| `changelog append` | Append a change entry |
| `tools invoke` | Invoke a tool with permission check |
| `tools list` | List registered tools |
| `permissions list` | List permission rules |
| `models complete` | Call a model via OpenAI-compatible API |
| `memory write` | Write a memory entry |
| `memory query` | Query memory |
| `memory forget` | Tombstone a memory entry |
| `snapshot take` | Take a git snapshot |
| `snapshot diff` | Diff two snapshots |
| `snapshot revert` | Revert to snapshot |
| `clarification request` | Request a clarification |
| `clarification list` | List clarifications |
| `clarification resolve` | Resolve a clarification |
| `episode open/append/close` | Episode lifecycle |

---

## Built-in tools

| Tool | Capabilities | Permission default |
|---|---|---|
| `fs.read` | Read files | auto (allowed) |
| `fs.hash` | SHA-256 hash files | auto (allowed) |
| `fs.edit` | Edit files | ask (requires approval) |
| `shell.exec` | Execute commands | ask (requires approval) |
| `git.status` | Git status | auto (allowed) |
| `git.diff` | Git diff | auto (allowed) |
| `git.blame` | Git blame | auto (allowed) |
| `test.run` | Run tests | auto (allowed) |
| `lint.run` | Run linter | auto (allowed) |
| `web.fetch` | Fetch URLs | ask (requires approval) |

---

## Verification

```bash
# Run all tests
cargo test --all-features

# End-to-end verification
python3 demo_agent.py      # 28 checks, all must pass
python3 highguard.py verify # Verify audit chain
```

---

## Architecture

```
Agent (Claude Code / Cursor / opencode)
    │
    │  MCP (JSON-RPC 2.0 over stdio or HTTP)
    ▼
HighHarness
    │
    ├── Permission engine (allow/deny/ask)
    ├── Episode recording (every tool call logged)
    ├── Hash-chained changelog
    ├── Memory store
    ├── Git snapshots
    └── Model inference (optional)
         │
         ▼
    Filesystem • Git • Shell • Network
```

---

## License

MIT — see [LICENSE](LICENSE).
