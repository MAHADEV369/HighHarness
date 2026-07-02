#!/usr/bin/env bash
# scripts/prune-stale-artifacts.sh — Remove episodes-work + snapshots dirs older than HH_PRUNE_HOURS
# (default 24h) AND not present in .harness/artifacts/in-flight.jsonl. Never touches
# CHANGELOG.agent.md, harness.log, tool-calls.jsonl, approvals/, interventions/,
# memory/, spend/ — those are part of the harness contract.
#
# Per buildedit.md Area E (D1 fix).
set -euo pipefail
HOURS="${HH_PRUNE_HOURS:-24}"
now=$(date +%s)
cutoff=$(( now - HOURS * 3600 ))

# Collect in-flight run_ids
in_flight=""
if [ -f .harness/artifacts/in-flight.jsonl ]; then
    in_flight="$(cat .harness/artifacts/in-flight.jsonl 2>/dev/null | tr -d '\n' || echo "")"
fi

prune_dir() {
    local base="$1"
    [ -d "$base" ] || return 0
    for d in "$base"/*; do
        [ -e "$d" ] || continue
        name=$(basename "$d")
        [ "$name" = "test" ] && continue
        # In-flight check: skip if this run_id is still active
        if [ -n "$in_flight" ] && echo "$in_flight" | grep -q "\"run_id\"[^a-zA-Z0-9]*\"$name\"" 2>/dev/null; then
            continue
        fi
        # mtime (macOS: stat -f %m; Linux: stat -c %Y)
        mtime=$(stat -f %m "$d" 2>/dev/null || stat -c %Y "$d" 2>/dev/null || echo 0)
        if [ "$mtime" -lt "$cutoff" ]; then
            rm -rf "$d"
            echo "pruned $d"
        fi
    done
}

prune_dir .harness/artifacts/episodes-work
prune_dir .harness/artifacts/snapshots
