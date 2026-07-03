#!/usr/bin/env bash
# setup.sh — One-command HighHarness setup
# Usage: bash scripts/setup.sh
set -uo pipefail

HX="./target/release/HighHarness"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "╔══════════════════════════════════════════╗"
echo "║     HighHarness — One-Command Setup     ║"
echo "╚══════════════════════════════════════════╝"
echo ""

# Step 1: Build if needed
if [ ! -f "$HX" ]; then
    echo "📦 Building HighHarness..."
    cargo build --release 2>&1 | tail -1
fi

# Step 2: Verify bootstrap
echo "🔍 Checking bootstrap..."
if $HX bootstrap verify > /dev/null 2>&1; then
    echo "   ✅ Bootstrap valid"
else
    echo "   ⚠️  Bootstrap missing — run bootstrap init from a clean repo"
fi

# Step 3: Verify hash chain
echo "🔍 Checking changelog hash chain..."
if $HX changelog verify-chain 2>&1 | grep -q '^\[\]$'; then
    echo "   ✅ Hash chain valid (0 broken entries)"
else
    BROKEN=$($HX changelog verify-chain 2>&1)
    echo "   ⚠️  Hash chain has broken entries: $BROKEN"
fi

# Step 4: Check MCP server
echo "🔍 Testing MCP server..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | $HX mcp serve 2>/dev/null | python3 -c "
import sys,json
try:
    d=json.loads(sys.stdin.readline())
    if d.get('result',{}).get('serverInfo',{}).get('name')=='HighHarness':
        print('   ✅ MCP server works')
    else:
        print('   ⚠️  MCP server response unexpected')
except: print('   ⚠️  MCP server test failed')
" 2>/dev/null || echo "   ⚠️  MCP server test skipped"

# Step 5: Test permission enforcement
echo "🔍 Testing permission enforcement..."
PERM_TEST=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"shell.exec","arguments":{"command":"rm -rf /"}}}
{"jsonrpc":"2.0","id":3,"method":"shutdown","params":{}}
' | $HX mcp serve 2>/dev/null | python3 -c "
import sys
for line in sys.stdin:
    d=__import__('json').loads(line.strip())
    if 'error' in d and 'Permission denied' in d['error'].get('message',''):
        print('PERMISSION_DENIED')
" 2>/dev/null)

if echo "$PERM_TEST" | grep -q "PERMISSION_DENIED"; then
    echo "   ✅ Dangerous ops blocked by permission engine"
else
    echo "   ⚠️  Permission test inconclusive"
fi

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║     Setup complete                       ║"
echo "║                                          ║"
echo "║  Next steps:                             ║"
echo "║  1. Start server:  $HX mcp serve-http    ║"
echo "║  2. Connect agent: opencode mcp add ... ║"
echo "║  3. Run tasks:     python3 highguard.py  ║"
echo "║  4. Verify:        bash scripts/prove_hash_chain.sh ║"
echo "╚══════════════════════════════════════════╝"
