# HighHarness MCP Integration

HighHarness exposes all its tools via the Model Context Protocol (MCP) over stdio or HTTP.
Any agent that supports MCP clients can connect to it as a tool provider, gaining
permission enforcement, episode recording, and hash-chained audit trails.

## Transport modes

| Mode | Command | Best for |
|------|---------|----------|
| **stdio** | `HighHarness mcp serve` | Claude Code, Cursor, local tools |
| **HTTP** | `HighHarness mcp serve-http --port 8931` | opencode, remote clients |

## Connecting agents

### opencode

```bash
# 1. Start HighHarness in HTTP mode (default port 8931)
/path/to/HighHarness mcp serve-http --port 8931 &

# 2. Register with opencode
opencode mcp add highharness --url http://127.0.0.1:8931

# 3. Verify connection
opencode mcp list
# → highharness ✓ connected
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

1. Open Cursor Settings → Features → MCP Servers
2. Add a new MCP server:
   - Name: `highharness`
   - Type: `command`
   - Command: `/path/to/HighHarness mcp serve`

### Python (any MCP client)

```python
import subprocess, json

proc = subprocess.Popen(["HighHarness", "mcp", "serve"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE)

def send(method, params=None):
    req = {"jsonrpc": "2.0", "id": 1, "method": method}
    if params: req["params"] = params
    proc.stdin.write(json.dumps(req).encode() + b"\n")
    proc.stdin.flush()
    return json.loads(proc.stdout.readline().decode())

# Initialize
send("initialize", {"protocolVersion": "2024-11-05"})

# List tools
tools = send("tools/list")["result"]["tools"]

# Call tool (permission enforced)
result = send("tools/call", {
    "name": "fs.read",
    "arguments": {"path": "Cargo.toml"}
})
```

## Permissions

By default, HighHarness allows reads and denies dangerous operations.
Configure permissions in `.harness/permissions.toml`:

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

## Available tools

| Tool | Description | Permission |
|------|-------------|------------|
| `fs.read` | Read a file | auto (allowed) |
| `fs.hash` | SHA-256 hash a file | auto (allowed) |
| `fs.edit` | Edit a file | ask (requires approval) |
| `shell.exec` | Execute shell command | ask (requires approval) |
| `git.status` | Git status | auto (allowed) |
| `git.diff` | Git diff | auto (allowed) |
| `git.blame` | Git blame | auto (allowed) |
| `test.run` | Run tests | auto (allowed) |
| `lint.run` | Run linter | auto (allowed) |
| `web.fetch` | Fetch URL | ask (requires approval) |

## Episode traces

Every session produces an episode trace at `logs/episodes/<run-id>.md` with:
- All tool calls (allowed and denied)
- Decisions and reasoning
- Failures and interventions
- Verification report
- Tamper-evident SHA-256 hash

View the latest episode:

```bash
ls -t logs/episodes/ | head -1 | xargs -I{} cat logs/episodes/{}
```
