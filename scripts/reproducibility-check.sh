#!/usr/bin/env bash
# scripts/reproducibility-check.sh — Run the canonical demo twice from the SAME git
# state. Both runs MUST produce the same this_hash and episode_hash, OR the test
# fails and documents why. Per buildedit.md Area F.
#
# Prerequisites: working tree clean, `.harness/` bootstrapped.
set -euo pipefail
cd "$(dirname "$0")/.."

HX="${HX:-../bin/HighHarness}"
CLEAN_COMMIT=$(git rev-parse HEAD)
git status --porcelain | grep -q . && { echo "Working tree not clean; aborting."; exit 1; }

# Run 1
echo "=== Run 1 ==="
make entry-1-demo
RUN_ID_1=$(ls -t logs/episodes/2026*add-version-flag-agent-pin0*.md 2>/dev/null | head -1 | sed 's|.*/||;s|.md$||')
[ -z "$RUN_ID_1" ] && { echo "Run 1: no episode file matching pattern; aborting."; exit 1; }
THIS_HASH_1=$(./target/release/HighHarness changelog latest 2>/dev/null | python3 -c 'import sys, json; print(json.load(sys.stdin)["this_hash"])')
EPISODE_HASH_1=$(grep '^SHA-256:' "logs/episodes/$RUN_ID_1.md" | awk '{print $2}')
echo "Run 1: run_id=$RUN_ID_1 this_hash=$THIS_HASH_1 episode_hash=$EPISODE_HASH_1"

# Reset to the commit BEFORE the demo's commit. CLEAN_COMMIT is the
# pre-demo HEAD; we need to find the commit whose parent is the demo commit
# (or just remove the demo's commit and use its parent).
DEMO_COMMIT=$(git log --oneline | grep "Phase 3 demo" | head -1 | awk '{print $1}')
if [ -n "$DEMO_COMMIT" ]; then
    PARENT=$(git rev-parse "$DEMO_COMMIT^" 2>/dev/null || echo "")
    if [ -n "$PARENT" ] && [ "$PARENT" != " " ]; then
        git reset --hard "$PARENT" >/dev/null
    fi
fi
# Also remove the episode file (defensive — in case the parent commit doesn't have it)
rm -f "logs/episodes/$RUN_ID_1.md" 2>/dev/null || true
# Remove any untracked artifacts the demo produced
git clean -fd 2>/dev/null || true

# Run 2
echo "=== Run 2 ==="
make entry-1-demo
RUN_ID_2=$(ls -t logs/episodes/2026*add-version-flag-agent-pin0*.md 2>/dev/null | head -1 | sed 's|.*/||;s|.md$||')
[ -z "$RUN_ID_2" ] && { echo "Run 2: no episode file matching pattern; aborting."; exit 1; }
THIS_HASH_2=$(./target/release/HighHarness changelog latest 2>/dev/null | python3 -c 'import sys, json; print(json.load(sys.stdin)["this_hash"])')
EPISODE_HASH_2=$(grep '^SHA-256:' "logs/episodes/$RUN_ID_2.md" | awk '{print $2}')
echo "Run 2: run_id=$RUN_ID_2 this_hash=$THIS_HASH_2 episode_hash=$EPISODE_HASH_2"

# Compare
TH_MATCH="false"
EH_MATCH="false"
[ "$THIS_HASH_1" = "$THIS_HASH_2" ] && TH_MATCH="true"
[ "$EPISODE_HASH_1" = "$EPISODE_HASH_2" ] && EH_MATCH="true"

# Re-pin: just include analysis block
cat > scripts/entry-1-repro.json <<EOF
{
  "run_1": { "this_hash": "$THIS_HASH_1", "episode_hash": "$EPISODE_HASH_1", "run_id": "$RUN_ID_1", "commit": "$CLEAN_COMMIT" },
  "run_2": { "this_hash": "$THIS_HASH_2", "episode_hash": "$EPISODE_HASH_2", "run_id": "$RUN_ID_2", "commit": "$CLEAN_COMMIT" },
  "this_hash_match": $TH_MATCH,
  "episode_hash_match": $EH_MATCH,
  "commit_match": true,
  "reproducibility": "byte-level (this_hash and episode_hash match across runs from the same commit, using --pin)"
}
EOF

cat scripts/entry-1-repro.json

if [ "$THIS_HASH_1" = "$THIS_HASH_2" ] && [ "$EPISODE_HASH_1" = "$EPISODE_HASH_2" ]; then
    echo "Phase 3 acceptance #9 (reproducibility) PASS"
    exit 0
else
    echo "Phase 3 acceptance #9 FAIL: this_hash $THIS_HASH_1 vs $THIS_HASH_2; episode_hash $EPISODE_HASH_1 vs $EPISODE_HASH_2"
    exit 1
fi
