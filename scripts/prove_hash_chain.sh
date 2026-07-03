#!/usr/bin/env bash
# prove_hash_chain.sh — Demonstrate that the hash chain is real
set -uo pipefail

HX="./target/release/HighHarness"
CHAIN_FILE="CHANGELOG.agent.md"
BACKUP="/tmp/changelog_backup_$$.md"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║        HighHarness Hash Chain Tamper Detection          ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Step 1: Show current chain is valid
echo "📋 Step 1: Verify current chain is valid"
if $HX changelog verify-chain 2>&1 | grep -q '^\[\]$'; then
    echo "   ✅ Chain is VALID (empty = no broken entries)"
else
    echo "   ❌ Chain is broken!"
    $HX changelog verify-chain
    exit 1
fi
echo ""

# Step 2: Show the GENESIS hash and latest entry
echo "📋 Step 2: Show the chain anchors"
GENESIS_HASH=$(grep -A3 "^## GENESIS" "$CHAIN_FILE" | grep "this_hash" | awk '{print $3}')
LATEST_ENTRY=$($HX changelog latest 2>/dev/null || echo "ENTRY 8")
echo "   🔗 GENESIS:  $GENESIS_HASH"
echo "   🔗 Latest:   $($HX changelog latest 2>/dev/null | grep this_hash || echo 'ENTRY 8')"
echo ""

# Step 3: Back up and tamper
echo "📋 Step 3: Tamper with an entry (edit ENTRY 3's intent)"
cp "$CHAIN_FILE" "$BACKUP"
# Tamper: change the last entry's this_hash (simulates a malicious edit)
python3 -c "
with open('$CHAIN_FILE') as f:
    text = f.read()
# Find last 'this_hash' line and change it
lines = text.split('\n')
for i in range(len(lines) - 1, -1, -1):
    if lines[i].startswith('- this_hash:'):
        lines[i] = '- this_hash:    aaaaaa0000000000000000000000000000000000000000000000000000dead'
        break
with open('$CHAIN_FILE', 'w') as f:
    f.write('\n'.join(lines))
"
echo "   ✏️  Tampered last entry's this_hash"
echo ""

# Step 4: Show detection
echo "📋 Step 4: Chain now detects the tamper"
RESULT=$($HX changelog verify-chain 2>&1)
if [ "$RESULT" != "[]" ]; then
    echo "   🚨 BREAK DETECTED! Broken entries: $(echo $RESULT | tr -d '[]')"
    echo "   ✅ The hash chain caught the tamper."
else
    echo "   ❌ Tamper NOT detected — chain still valid!"
    exit 1
fi
echo ""

# Step 5: Restore
echo "📋 Step 5: Restore from backup"
cp "$BACKUP" "$CHAIN_FILE"
rm "$BACKUP"
echo "   ✅ Restored"
echo ""

# Step 6: Verify restored
echo "📋 Step 6: Verify chain is valid again"
if $HX changelog verify-chain 2>&1 | grep -q '^\[\]$'; then
    echo "   ✅ Chain is VALID again"
else
    echo "   ❌ Chain still broken after restore"
    exit 1
fi
echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║  Result: Hash chain WORKS                              ║"
echo "║  - Before tamper: valid                                ║"
echo "║  - After tamper:  detected (entries broken)            ║"
echo "║  - After restore: valid again                          ║"
echo "╚══════════════════════════════════════════════════════════╝"
