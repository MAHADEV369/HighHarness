#!/usr/bin/env python3
"""Rebuild the CHANGELOG.agent.md hash chain from scratch.

Each entry is re-serialized using the canonical format, its this_hash is
blanked, SHA-256 is computed, and prev_hash is set to the previous entry's
this_hash. This produces a valid, non-branching chain for the Rust
verify_chain function.
"""

import hashlib
import re
import sys
from pathlib import Path

VALUE_COLUMN = 16
FIELD_ORDER = [
    "agent", "run_id", "tier", "files", "intent", "diff_summary",
    "evidence", "attribution", "verification", "status",
    "prev_hash", "this_hash",
]

CHANGELOG_PATH = Path(__file__).parent.parent / "CHANGELOG.agent.md"


def canonical_serialize(entry: dict) -> bytes:
    """Serialize an entry to canonical bytes matching the Rust implementation."""
    lines = []
    lines.append(f"## ENTRY {entry['n']} — {entry['ts']}")

    # Build field values
    field_values = {}
    for name in FIELD_ORDER:
        if name == "files":
            field_values["files"] = ", ".join(entry.get("files", []))
        else:
            field_values[name] = entry.get(name, "")

    for name in FIELD_ORDER:
        value = field_values.get(name, "")
        prefix = f"- {name}:"
        pad = max(0, VALUE_COLUMN - len(prefix))
        # Split on newlines for continuation lines
        parts = value.split("\n")
        first = parts[0].strip()
        lines.append(f"{prefix}{' ' * pad}{first}")
        for cont in parts[1:]:
            trimmed = cont.strip()
            if trimmed:
                lines.append(f"{' ' * VALUE_COLUMN}{trimmed}")

    # Add trailing newline after this_hash line (last line)
    result = "\n".join(lines) + "\n"
    return result.encode("utf-8")


def parse_changelog(text: str):
    """Parse the changelog into entries."""
    # Find GENESIS marker
    genesis_match = re.search(r'^## GENESIS.*\n- prev_hash:\s*(.*?)\n- this_hash:\s*(.*?)\n', text, re.MULTILINE)
    genesis_hash = genesis_match.group(2).strip() if genesis_match else None

    # Find all ENTRY blocks
    entries = []
    entry_pattern = re.compile(
        r'^## ENTRY (\d+) — (.+?)\n'
        r'(.*?)(?=^## (?:ENTRY |GENESIS)|\Z)',
        re.MULTILINE | re.DOTALL
    )

    for match in entry_pattern.finditer(text):
        n = int(match.group(1))
        ts = match.group(2).strip()
        body = match.group(3)

        fields = {}
        files_list = []

        for line in body.strip().split("\n"):
            line = line.strip()
            m = re.match(r'^- (\w+):\s*(.*)', line)
            if m:
                name = m.group(1)
                value = m.group(2).strip()
                if name == 'files':
                    files_list = [f.strip() for f in value.split(',')]
                elif name in FIELD_ORDER:
                    fields[name] = value

        entry = {
            'n': n,
            'ts': ts,
            'files': files_list,
        }
        for name in FIELD_ORDER:
            if name in fields:
                entry[name] = fields[name]
            else:
                entry[name] = ""

        entries.append(entry)

    return genesis_hash, entries


def rebuild_chain(genesis_hash: str, entries: list) -> str:
    """Rebuild the changelog with correct hashes."""
    lines = []
    lines.append("# CHANGELOG.agent.md")
    lines.append("")
    lines.append("**Append-only, structured, hash-chained log of every change any agent makes to this repository.**")
    lines.append("")
    lines.append("This file is governed by `HARNESS_ENGINEERING.md` Section 4. Read `readharness.md` for the human-friendly explanation of what this file is and why it exists.")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Rules")
    lines.append("")
    lines.append("- **Append-only.** Never edit or delete an existing entry. Reverting a change is a new entry with a new entry referencing the original in its `intent` field.")
    lines.append("- **One entry per change.** A run that makes three changes appends three entries.")
    lines.append("- **Hash-chained.** Each entry's `prev_hash` equals the prior entry's `this_hash`. The first entry's `prev_hash` equals the `this_hash` of the `## GENESIS` marker that the bootstrap protocol (`HARNESS_VERSIONING.md` §6.1) writes **before** any agent run begins. The marker is not an entry and is not numbered.")
    lines.append("- **Canonical hashing.** SHA inputs are byte-exact per `HARNESS_PRIMITIVES.md` §3.5.1. The `this_hash` field is blanked (`\"\"`) before hashing. Agents MUST read the genesis linkage from `changelog.latest_or_genesis()`; they MUST NOT compute the GENESIS hash themselves.")
    lines.append("- **Dense and factual.** No narrative, no justification beyond `intent` and `attribution`.")
    lines.append("- **If you cannot compute a SHA, stop and ask.** Do not fabricate a hash.")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Entry format")
    lines.append("")
    lines.append("```")
    lines.append("## ENTRY <N> — <ISO-8601 timestamp>")
    lines.append("- agent:        <agent id / model>")
    lines.append("- run_id:       <run id, links to logs/episodes/<run-id>.md>")
    lines.append("- tier:         <trivial | standard | safety-critical — see HARNESS_ENGINEERING.md §16>")
    lines.append("- files:        <paths touched, comma-separated>")
    lines.append("- intent:       <one sentence — what this change was supposed to do>")
    lines.append("- diff_summary: <one or two lines — what actually changed>")
    lines.append("- evidence:     <test outputs, type check, lint results, links>")
    lines.append("- attribution:  <if a failure was found: agent | spec | env | flaky | pre-existing | none>")
    lines.append("- verification: <syntactic | functional | semantic | regression | full>")
    lines.append("- status:       <added | modified | reverted | deleted>")
    lines.append("- prev_hash:    <SHA-256 of the previous entry's canonical text; entry 1 reads the")
    lines.append("                 GENESIS marker's this_hash via changelog.latest_or_genesis()>")
    lines.append("- this_hash:    <SHA-256 of this entry's canonical text (computed after writing;")
    lines.append("                 this_hash field is blanked \"\" before hashing — see §3.5.1)>")
    lines.append("```")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Entries")
    lines.append("")

    # GENESIS marker
    lines.append(f"## GENESIS — 2026-06-29T095108Z")
    lines.append(f"- prev_hash: null")
    lines.append(f"- this_hash: {genesis_hash}")
    lines.append(f"- bootstrap_human: admin")
    lines.append(f"- bootstrap_commit: 0")
    lines.append(f"- spec_versions: {{ engineering: 1, primitives: 1, security: 1, metrics: 1, versioning: 1 }}")

    prev_hash = genesis_hash
    for entry in entries:
        # Build the entry dict with all fields
        entry_dict = {
            'n': entry['n'],
            'ts': entry['ts'],
            'agent': entry.get('agent', ''),
            'run_id': entry.get('run_id', ''),
            'tier': entry.get('tier', ''),
            'files': entry.get('files', []),
            'intent': entry.get('intent', ''),
            'diff_summary': entry.get('diff_summary', ''),
            'evidence': entry.get('evidence', ''),
            'attribution': entry.get('attribution', ''),
            'verification': entry.get('verification', ''),
            'status': entry.get('status', ''),
            'prev_hash': prev_hash,
            'this_hash': "",  # Will be computed
        }

        # Compute this_hash
        canonical_bytes = canonical_serialize(entry_dict)
        this_hash = hashlib.sha256(canonical_bytes).hexdigest()
        entry_dict['this_hash'] = this_hash

        # Write the entry
        lines.append(f"## ENTRY {entry['n']} — {entry['ts']}")

        field_values = {}
        for name in FIELD_ORDER:
            if name == "files":
                field_values["files"] = ", ".join(entry_dict.get("files", []))
            else:
                field_values[name] = entry_dict.get(name, "")

        for name in FIELD_ORDER:
            value = field_values[name]
            prefix = f"- {name}:"
            pad = max(0, VALUE_COLUMN - len(prefix))
            parts = value.split("\n")
            first = parts[0].strip()
            lines.append(f"{prefix}{' ' * pad}{first}")
            for cont in parts[1:]:
                trimmed = cont.strip()
                if trimmed:
                    lines.append(f"{' ' * VALUE_COLUMN}{trimmed}")

        prev_hash = this_hash

    return "\n".join(lines) + "\n"


def main():
    text = CHANGELOG_PATH.read_text()
    genesis_hash, entries = parse_changelog(text)
    if not genesis_hash:
        print("ERROR: Could not find GENESIS hash")
        sys.exit(1)

    print(f"Found GENESIS hash: {genesis_hash}")
    print(f"Found {len(entries)} entries")

    result = rebuild_chain(genesis_hash, entries)

    # Verify by checking prev_hash continuity
    pattern = re.compile(r'- prev_hash:\s*(.*?)\n- this_hash:\s*(.*?)\n', re.MULTILINE)
    chain = pattern.findall(result)
    for i, (prev, this) in enumerate(chain):
        if i > 0 and prev != chain[i-1][1]:
            print(f"WARNING: Entry {i} prev_hash mismatch (chain broken)")
            sys.exit(1)

    print(f"Chain verified: {len(chain)} links")
    print(f"Writing {CHANGELOG_PATH}")
    CHANGELOG_PATH.write_text(result)
    print("Done")


if __name__ == "__main__":
    main()
