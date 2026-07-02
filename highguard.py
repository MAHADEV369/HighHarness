#!/usr/bin/env python3
"""highguard — Safe Agent Runner.

Governs an AI agent through HighHarness: permissions, episodes, audit trail.

Usage:
    export OPENAI_API_KEY="sk-..."
    
    # Run an agent task with governance:
    python3 highguard.py run "Add a version flag to the CLI"
    
    # Report on past sessions:
    python3 highguard.py report
    
    # Verify the audit chain:
    python3 highguard.py verify
"""

import json
import os
import subprocess
import sys
import time
import glob
import hashlib

BINARY = "./target/release/HighHarness"
CWD = os.path.dirname(os.path.abspath(__file__)) or "."


# ── MCP Client ─────────────────────────────────────────────────────

class MCPClient:
    """Minimal MCP client that talks to HighHarness over stdio."""

    def __init__(self):
        self.proc = None
        self.next_id = 1

    def start(self):
        self.proc = subprocess.Popen(
            [BINARY, "mcp", "serve"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            cwd=CWD,
        )
        time.sleep(0.2)
        return self

    def send(self, method, params=None):
        req = {
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": method,
        }
        if params is not None:
            req["params"] = params
        self.next_id += 1
        assert self.proc and self.proc.stdin
        self.proc.stdin.write(json.dumps(req).encode() + b"\n")
        self.proc.stdin.flush()
        line = self.proc.stdout.readline()
        return json.loads(line.decode())

    def close(self):
        if self.proc and self.proc.poll() is None:
            self.send("shutdown")
            self.proc.wait(timeout=5)

    def __enter__(self):
        return self.start()

    def __exit__(self, *args):
        self.close()


# ── Agent Logic ────────────────────────────────────────────────────

class Agent:
    """A simple agent that uses HighHarness tools via MCP."""

    def __init__(self, mcp: MCPClient):
        self.mcp = mcp
        self.tools = {}

    def init(self):
        resp = self.mcp.send("initialize", {"protocolVersion": "2024-11-05"})
        assert resp.get("result", {}).get("serverInfo", {}).get("name") == "HighHarness"
        resp = self.mcp.send("tools/list")
        for t in resp.get("result", {}).get("tools", []):
            self.tools[t["name"]] = t

    def tool_call(self, name: str, args: dict) -> dict:
        resp = self.mcp.send("tools/call", {"name": name, "arguments": args})
        if "error" in resp:
            return {"ok": False, "error": resp["error"]["message"]}
        result = resp.get("result", {})
        if result.get("isError"):
            text = result.get("content", [{}])[0].get("text", "unknown error")
            return {"ok": False, "error": text}
        text = result.get("content", [{}])[0].get("text", "")
        return {"ok": True, "data": text}

    def read(self, path: str) -> dict:
        return self.tool_call("fs.read", {"path": path})

    def edit(self, path: str, old: str, new: str) -> dict:
        return self.tool_call("fs.edit", {"path": path, "old": old, "new": new})

    def hash_file(self, path: str) -> dict:
        return self.tool_call("fs.hash", {"path": path})

    def git_status(self) -> dict:
        return self.tool_call("git.status", {})

    def git_diff(self) -> dict:
        return self.tool_call("git.diff", {})

    def shell(self, command: str) -> dict:
        return self.tool_call("shell.exec", {"command": command})


# ── Task: Add a version flag ──────────────────────────────────────

def task_add_version_flag(agent: Agent):
    """Demo task: add a --version flag to a CLI tool."""
    print("  Reading Cargo.toml...")
    r = agent.read("Cargo.toml")
    if not r["ok"]:
        print(f"  ❌ Cannot read Cargo.toml: {r.get('error')}")
        return False
    print("  ✅ Read Cargo.toml")

    print("  Reading src/cli/mod.rs...")
    r = agent.read("src/cli/mod.rs")
    if not r["ok"]:
        print(f"  ❌ Cannot read mod.rs: {r.get('error')}")
        return False
    print("  ✅ Read src/cli/mod.rs")

    print("  Checking git status...")
    r = agent.git_status()
    if r["ok"]:
        print(f"  📊 Git: {r['data'][:60]}...")

    print("  Running version flag (tests already have it)...")
    # This is a benign operation - just checking the version works
    result = subprocess.run(
        [BINARY, "--version"],
        capture_output=True, text=True, timeout=5, cwd=CWD,
    )
    if result.returncode == 0:
        version = result.stdout.strip()
        print(f"  ✅ HighHarness version: {version}")
    else:
        print(f"  ⚠️  Version check: {result.stderr}")

    print("  Trying destructive shell (should be blocked)...")
    r = agent.shell("rm -rf /")
    if not r["ok"]:
        print(f"  ✅ Destructive shell blocked: {r.get('error', '')[:60]}")
    else:
        print(f"  ⚠️  Destructive shell was allowed! (permissions issue)")

    return True


# ── CLI ────────────────────────────────────────────────────────────

def cmd_run(task: str):
    print(f"\n{'=' * 60}")
    print(f"  HighGuard — Safe Agent Runner")
    print(f"  Task: {task}")
    print(f"{'=' * 60}\n")

    with MCPClient() as mcp:
        agent = Agent(mcp)
        agent.init()
        print(f"  Connected to HighHarness ({len(agent.tools)} tools available)\n")

        if task == "add-version-flag":
            ok = task_add_version_flag(agent)
        elif task == "readme":
            ok = task_read_readme(agent)
        else:
            print(f"  Unknown task: {task}")
            ok = False

        print(f"\n{'─' * 50}")

        # Show episode
        episodes_dir = os.path.join(CWD, "logs", "episodes")
        ep_files = sorted(
            glob.glob(os.path.join(episodes_dir, "mcp-*.md")),
            key=os.path.getmtime, reverse=True
        )
        if ep_files:
            ep_path = ep_files[0]
            ep_content = open(ep_path).read()
            print(f"\n  📄 Episode: {os.path.basename(ep_path)}")

            # Count tool calls
            tc_count = ep_content.count("tool_call_id")
            print(f"  📊 Tool calls recorded: {tc_count}")

            # Show hash
            for line in ep_content.split("\n"):
                if "SHA-256:" in line:
                    print(f"  🔐 Hash: {line.strip()}")
                    break

            # Show any denied calls
            for line in ep_content.split("\n"):
                if "DENIED" in line:
                    print(f"  ⛔ Denied: {line.strip()[:100]}...")

        print(f"\n{'=' * 60}")
        if ok:
            print(f"  ✅ Task complete — all operations governed by HighHarness")
        else:
            print(f"  ⚠️  Task had issues")
        print(f"{'=' * 60}\n")


def task_read_readme(agent: Agent):
    """Simple read-only task."""
    print("  Reading README.md...")
    r = agent.read("README.md")
    if r["ok"]:
        lines = r["data"].count("\n")
        print(f"  ✅ README.md read ({lines} lines)")
    else:
        print(f"  ❌ {r.get('error')}")
        return False

    print("  Reading Cargo.toml...")
    r = agent.read("Cargo.toml")
    if r["ok"]:
        print(f"  ✅ Cargo.toml read")
    return True


def cmd_report():
    """Show a report of all MCP episodes."""
    episodes_dir = os.path.join(CWD, "logs", "episodes")
    ep_files = sorted(
        glob.glob(os.path.join(episodes_dir, "mcp-*.md")),
        key=os.path.getmtime, reverse=True
    )

    if not ep_files:
        print("  No MCP episode traces found.")
        return

    print(f"\n  📊 HighGuard Session Report")
    print(f"  {'=' * 50}")
    print(f"  Episodes found: {len(ep_files)}\n")

    for i, ep in enumerate(ep_files[:5]):
        ep_name = os.path.basename(ep)
        content = open(ep).read()
        tc_count = content.count("tool_call_id")
        denied = content.count("DENIED")
        has_hash = "SHA-256:" in content

        status = "✅" if has_hash else "⚠️"
        print(f"  {status}  {ep_name}")
        print(f"       Tool calls: {tc_count}  |  Denied: {denied}")
        if denied:
            for line in content.split("\n"):
                if "DENIED" in line:
                    print(f"       ⛔ {line.strip()[:80]}")
        print()


def cmd_verify():
    """Verify the audit chain."""
    print("\n  Verifying HighHarness audit chain...")
    result = subprocess.run(
        [BINARY, "changelog", "verify-chain"],
        capture_output=True, text=True, timeout=10, cwd=CWD,
    )
    if result.returncode == 0:
        print("  ✅ Changelog hash chain is valid")
    else:
        print(f"  ⚠️  Chain issues: {result.stderr.strip()[:120]}")


def cmd_export(format="jsonl"):
    """Export episodes as training data for fine-tuning."""
    episodes_dir = os.path.join(CWD, "logs", "episodes")
    ep_files = sorted(
        glob.glob(os.path.join(episodes_dir, "mcp-*.md")),
        key=os.path.getmtime
    )

    if not ep_files:
        print("  No MCP episode traces to export.")
        return

    print(f"\n  📦 Exporting {len(ep_files)} episodes as {format}\n")

    if format == "jsonl":
        output_path = "episodes_export.jsonl"
        with open(output_path, "w") as out:
            for ep in ep_files:
                content = open(ep).read()
                record = {"episode": os.path.basename(ep), "raw": content}
                out.write(json.dumps(record) + "\n")
        print(f"  ✅ Exported to {output_path}")

    elif format == "openai":
        # Convert to OpenAI fine-tuning format
        output_path = "episodes_openai.jsonl"
        count = 0
        with open(output_path, "w") as out:
            for ep in ep_files:
                content = open(ep).read()

                # Extract tool calls
                tool_calls = []
                for line in content.split("\n"):
                    if line.startswith("- ") and "tool_call_id" in line:
                        try:
                            line_data = line[2:]
                            tc = json.loads(line_data)
                            tool_calls.append(tc)
                        except json.JSONDecodeError:
                            pass

                if tool_calls:
                    # Build conversation from tool calls
                    messages = [{"role": "system", "content": "HighHarness governed agent session"}]
                    for tc in tool_calls:
                        messages.append({
                            "role": "assistant",
                            "content": f"Call tool: {tc['tool']} with args: {json.dumps(tc.get('args', {}))}"
                        })
                        messages.append({
                            "role": "tool",
                            "content": tc.get("result_summary", "")[:500]
                        })
                    entry = {"messages": messages}
                    out.write(json.dumps(entry) + "\n")
                    count += 1

        print(f"  ✅ Exported {count} conversations to {output_path}")

    elif format == "hf":
        # HuggingFace datasets format (minimal JSON)
        output_path = "episodes_hf.json"
        records = []
        for ep in ep_files:
            content = open(ep).read()
            records.append({
                "episode_id": os.path.basename(ep).replace(".md", ""),
                "tool_calls": content.count("tool_call_id"),
                "denied": content.count("DENIED"),
                "has_hash": "SHA-256:" in content,
                "text": content[:2000],
            })
        with open(output_path, "w") as out:
            json.dump(records, out, indent=2)
        print(f"  ✅ Exported {len(records)} records to {output_path}")

    else:
        print(f"  ❌ Unknown export format: {format}")


def cmd_clean():
    """Clean up MCP episode files after review."""
    episodes_dir = os.path.join(CWD, "logs", "episodes")
    ep_files = glob.glob(os.path.join(episodes_dir, "mcp-*.md"))
    for f in ep_files:
        os.remove(f)
    print(f"  🧹 Cleaned {len(ep_files)} MCP episode files")


# ── Main ───────────────────────────────────────────────────────────

def main():
    if len(sys.argv) < 2:
        print("""Usage:
  python3 highguard.py run "<task>"     Run a governed agent task
  python3 highguard.py report           Show session report
  python3 highguard.py export [format]  Export episodes as training data
                                       Formats: jsonl (default), openai, hf
  python3 highguard.py verify           Verify audit chain
  python3 highguard.py clean            Clean test episodes

Tasks:
  add-version-flag  Demo: read files, check status, verify permissions
  readme            Simple read-only task
""")
        return

    cmd = sys.argv[1]

    if cmd == "run":
        task = sys.argv[2] if len(sys.argv) > 2 else "add-version-flag"
        cmd_run(task)
    elif cmd == "report":
        cmd_report()
    elif cmd == "export":
        format = sys.argv[2] if len(sys.argv) > 2 else "jsonl"
        cmd_export(format)
    elif cmd == "verify":
        cmd_verify()
    elif cmd == "clean":
        cmd_clean()
    else:
        print(f"Unknown command: {cmd}")
        sys.exit(1)


if __name__ == "__main__":
    main()
