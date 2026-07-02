# CHANGELOG.agent.md

**Append-only, structured, hash-chained log of every change any agent makes to this repository.**

This file is governed by `HARNESS_ENGINEERING.md` Section 4. Read `readharness.md` for the human-friendly explanation of what this file is and why it exists.

---

## Rules

- **Append-only.** Never edit or delete an existing entry. Reverting a change is a new entry with a new entry referencing the original in its `intent` field.
- **One entry per change.** A run that makes three changes appends three entries.
- **Hash-chained.** Each entry's `prev_hash` equals the prior entry's `this_hash`. The first entry's `prev_hash` equals the `this_hash` of the `## GENESIS` marker that the bootstrap protocol (`HARNESS_VERSIONING.md` §6.1) writes **before** any agent run begins. The marker is not an entry and is not numbered.
- **Canonical hashing.** SHA inputs are byte-exact per `HARNESS_PRIMITIVES.md` §3.5.1. The `this_hash` field is blanked (`""`) before hashing. Agents MUST read the genesis linkage from `changelog.latest_or_genesis()`; they MUST NOT compute the GENESIS hash themselves.
- **Dense and factual.** No narrative, no justification beyond `intent` and `attribution`.
- **If you cannot compute a SHA, stop and ask.** Do not fabricate a hash.

---

## Entry format

```
## ENTRY <N> — <ISO-8601 timestamp>
- agent:        <agent id / model>
- run_id:       <run id, links to logs/episodes/<run-id>.md>
- tier:         <trivial | standard | safety-critical — see HARNESS_ENGINEERING.md §16>
- files:        <paths touched, comma-separated>
- intent:       <one sentence — what this change was supposed to do>
- diff_summary: <one or two lines — what actually changed>
- evidence:     <test outputs, type check, lint results, links>
- attribution:  <if a failure was found: agent | spec | env | flaky | pre-existing | none>
- verification: <syntactic | functional | semantic | regression | full>
- status:       <added | modified | reverted | deleted>
- prev_hash:    <SHA-256 of the previous entry's canonical text; entry 1 reads the
                 GENESIS marker's this_hash via changelog.latest_or_genesis()>
- this_hash:    <SHA-256 of this entry's canonical text (computed after writing;
                 this_hash field is blanked "" before hashing — see §3.5.1)>
```

---

## Entries

## GENESIS — 2026-06-29T095108Z
- prev_hash: null
- this_hash: 97e9f4376a26ed09c89c0544d1ae2ed24756ad6217846601dc2ee473cf8d705b
- bootstrap_human: admin
- bootstrap_commit: 0
- spec_versions: { engineering: 1, primitives: 1, security: 1, metrics: 1, versioning: 1 }
## ENTRY 1 — 2026-06-29T09:51:08Z
- agent:        highharness/bootstrap
- run_id:       2026-06-29T095108Z-bootstrap-eval-hx-dc11
- tier:         trivial
- files:        evals/bootstrap-readme/notes.md
- intent:       bootstrap eval: verify compare-and-append against GENESIS
- diff_summary: appended 'verified by HighHarness' to evals/bootstrap-readme/notes.md
- evidence:     changelog verify_chain returns empty
- attribution:  none
- verification: syntactic
- status:       added
- prev_hash:    97e9f4376a26ed09c89c0544d1ae2ed24756ad6217846601dc2ee473cf8d705b
- this_hash:    18598b7d949d41cb784b2336918c4f28aef092c1f0515f83c46e2a1b6625a98f
## ENTRY 2 — 2026-06-29T09:51:08Z
- agent:        trident_phase3_demo
- run_id:       20260629T095108Z-add-version-flag-agent-pin0
- tier:         trivial
- files:        src/cli/mod.rs, tests/cli_version.rs, CHANGELOG.agent.md, logs/episodes/20260629T095108Z-add-version-flag-agent-pin0.md
- intent:       Add a --version flag to the HighHarness CLI that prints "HighHarness 0.1.0".
- diff_summary: Refined src/cli/mod.rs clap Command derive to use version = env!("CARGO_PKG_VERSION").
- evidence:     gate-syntactic.log = cargo check --all-targets exit 0; gate-functional.log = cargo test --workspace --no-run exit 0; gate-semantic.log = typed-AST diff limited to one Command attribute clause; gate-regression.log = cargo test --workspace exit 0; ./target/release/HighHarness --version prints 'HighHarness 0.1.0'.
- attribution:  none
- verification: full
- status:       modified
- prev_hash:    18598b7d949d41cb784b2336918c4f28aef092c1f0515f83c46e2a1b6625a98f
- this_hash:    4d2bef01f300fb1a87573ab80628bf64bcb41d4d636bee8d55efc792201d5c3a
## ENTRY 3 — 2026-06-29T12:08:30Z
- agent:        agent_8f94d832_20260629T120551Z
- run_id:       2026-06-29T120551Z-buildedit-phase-1.5-agent-80c9
- tier:         safety-critical
- files:        src/cli/tools.rs, src/cli/gates.rs, src/cli/changelog.rs, src/cli/util.rs, src/cli/mod.rs, Makefile, tests/cli_tools_args_inline.rs, tests/cli_gates_run_subcommand.rs, tests/cli_changelog_append_flag.rs
- intent:       buildedit A: align CLI shapes with the spec (inline-JSON args, gates run subcommand, --entry flag).
- diff_summary: src/cli/tools.rs: --args + --scope_narrow switched from PathBuf to String with inline-JSON heuristic; src/cli/util.rs: new shared read_json_or_path helper; src/cli/gates.rs: added `run` subcommand; src/cli/changelog.rs: `append` requires --entry flag; 3 new test files added.
- evidence:     cargo build --release exit 0; cargo test --test cli_tools_args_inline (3 pass) + cli_gates_run_subcommand (2 pass) + cli_changelog_append_flag (2 pass)
- attribution:  none
- verification: full
- status:       modified
- prev_hash:    4d2bef01f300fb1a87573ab80628bf64bcb41d4d636bee8d55efc792201d5c3a
- this_hash:    8d21581983534d3e2c29eaccf1ea2dda9f1d00ab097b0d27e1bd5e754118773a
## ENTRY 4 — 2026-06-29T12:30:00Z
- agent:        buildedit-area-b
- run_id:       buildedit-area-b
- tier:         safety-critical
- files:        src/gates.rs, src/cli/gates.rs, src/bootstrap.rs, Makefile, scripts/entry-1-semantic-verification.json, tests/gate_semantic.rs
- intent:       Make the semantic gate a real §7.3 gate (parses verification JSON + orthogonality check) instead of the Phase 1 no-op `true`.
- diff_summary: Added `gates::run_semantic` that parses §7.3 mapping JSON and rejects any evidence that overlaps the functional-gate log. Added 4 tests.
- evidence:     cargo test --test gate_semantic = 4 passed; orthogonality violation detected; CLI rejects without --verification
- attribution:  none
- verification: full
- status:       modified
- prev_hash:    8d21581983534d3e2c29eaccf1ea2dda9f1d00ab097b0d27e1bd5e754118773a
- this_hash:    0f49942a834ca007672cc22e5a889da9411a35a9149b09a62d81104fff01a20c
## ENTRY 5 — 2026-06-29T13:30:00Z
- agent:        buildedit-area-d
- run_id:       buildedit-area-d
- tier:         safety-critical
- files:        tests/cli_tools_args_inline.rs
- intent:       Add CLI-level deny test for inline-JSON args (defense-in-depth; bypasses reg.invoke_raw).
- diff_summary: Added `tools_invoke_deny_returns_exit_3_on_harness_path_via_inline_json` which calls the binary end-to-end with inline-JSON args editing `.harness/x` and asserts exit code 3 + R-DENY-HARNESS in output.
- evidence:     cargo test --test cli_tools_args_inline = 3 passed (positive-inline, positive-path, deny-via-inline)
- attribution:  none
- verification: full
- status:       modified
- prev_hash:    0f49942a834ca007672cc22e5a889da9411a35a9149b09a62d81104fff01a20c
- this_hash:    6d8b805c72fc5df9bec1032a0133d5eb4aca04261a7af5d26b1623cdd7a3c344
## ENTRY 6 — 2026-06-29T14:00:00Z
- agent:        buildedit-area-e
- run_id:       buildedit-area-e
- tier:         trivial
- files:        Makefile, scripts/prune-stale-artifacts.sh
- intent:       Make `entry-1-demo-clean` actually clean (D1): remove only the demo run's own episodes-work + snapshots, never the changelog or harness log.
- diff_summary: Extracted prune-stale-artifacts.sh (bash, HH_PRUNE_HOURS=24 default). Replaced entry-1-demo-clean in the Makefile to call it.
- evidence:     HH_PRUNE_HOURS=0 bash scripts/prune-stale-artifacts.sh removed 15 episodes-work + 15 snapshots; CHANGELOG.agent.md and harness.log untouched
- attribution:  none
- verification: syntactic
- status:       modified
- prev_hash:    6d8b805c72fc5df9bec1032a0133d5eb4aca04261a7af5d26b1623cdd7a3c344
- this_hash:    52653a7da8f9be91b64992f5d11e297e838f7dd8fb228577ab0db6f021feec64
## ENTRY 7 — 2026-06-29T14:30:00Z
- agent:        buildedit-area-f
- run_id:       buildedit-area-f
- tier:         safety-critical
- files:        src/id.rs, src/cli/id_cmd.rs, Makefile, scripts/reproducibility-check.sh
- intent:       Make the canonical demo byte-reproducible via --pin flag on id-run/id-agent (B1 + B2).
- diff_summary: Added `id::run_id_pinned` and `id::agent_id_pinned` helpers seeded by the GENESIS bootstrap timestamp. Makefile uses --pin on both. scripts/reproducibility-check.sh runs the demo twice from the same commit and verifies this_hash + episode_hash match.
- evidence:     make repro: this_hash_match=true, episode_hash_match=true, commit_match=true across 2 runs; entry-1-repro.json contains the run hashes.
- attribution:  none
- verification: full
- status:       modified
- prev_hash:    52653a7da8f9be91b64992f5d11e297e838f7dd8fb228577ab0db6f021feec64
- this_hash:    f7d71dd1f6a8974b2abbb5ff0c8438b6d156bdd21e0bfb6cdfd8b411d25c2d6f
## ENTRY 8 — 2026-06-29T15:00:00Z
- agent:        buildedit-area-g
- run_id:       buildedit-area-g
- tier:         trivial
- files:        logs/episodes/_EXAMPLE.md, readharness.md, README.md, CHANGELOG.agent.md
- intent:       Reconcile spec/doc pointers: _EXAMPLE.md superseded header points at pinned run, readharness.md and README.md point at the canonical Entry 1.
- diff_summary: Updated _EXAMPLE.md header to point at the latest pinned run_id. Updated readharness.md file-map row to point at the canonical Entry 1. Updated README.md Repo contents table to point at the same.
- evidence:     self-check: head -3 _EXAMPLE.md | grep -c = 1; grep -c readharness.md = 1; grep -c README.md = 1; all match buildedit.md §6 G.6 expectations.
- attribution:  none
- verification: syntactic
- status:       modified
- prev_hash:    f7d71dd1f6a8974b2abbb5ff0c8438b6d156bdd21e0bfb6cdfd8b411d25c2d6f
- this_hash:    c2652e5d03d71c14d5e42128b1a6425bc1827df7cc0ea550e316a51f0b1f0261
