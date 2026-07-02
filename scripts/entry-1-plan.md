Decomposition (each atom has its own pass/fail criterion — §1.12):

1. Open an episode via `episode open` (run_id, agent_id, task-spec-file, tier, phase). Pass criterion: `logs/episodes/<run_id>.md` exists with all required sections in fixed order.
2. Read `src/cli/mod.rs` (baseline) via `tools invoke --tool fs.read`. Pass criterion: file content is captured to `scripts/entry-1-pre-read.json`.
3. Take a baseline snapshot via `snapshot take --label baseline`. Pass criterion: snapshot_id returned, recorded in the harness's snapshot store.
4. Create the new test file `tests/cli_version.rs` via `cp` from `scripts/cli_version.rs.template` (Makefile convention; the audit-relevant source change goes through `fs.edit`; per D6). Pass criterion: file exists and contains the test.
5. Apply the targeted edit to `src/cli/mod.rs` via `tools invoke --tool fs.edit`: `version = concat!(env!("CARGO_PKG_VERSION"))` → `version = env!("CARGO_PKG_VERSION")`. Pass criterion: post-edit file contains the new form; old form is gone.
6. Rebuild: `cargo build --release`. Pass criterion: exit 0.
7. Verify `--version` output: `./target/release/HighHarness --version` → `HighHarness 0.1.0\n`. Pass criterion: exact match.
8. Run all four gates (syntactic, functional, semantic, regression). Pass criterion: each gate's `status == "pass"`.
9. Append the changelog entry (canonical Entry-1 form, `n=2`, `ts` pinned to GENESIS ts, `prev_hash`/`this_hash` blank in the JSON, `prev_hash` will be filled by `changelog.append` to the current chain head's `this_hash` = ENTRY 1's `this_hash`). Pass criterion: `changelog append` returns a hex `this_hash`; `changelog verify-chain` returns `[]`.
10. Close the episode via `episode close` with the verification JSON. Pass criterion: the episode-hash section is filled with a 64-char hex SHA-256; the in-flight line is removed.
11. Assert: `git diff-tree --name-only -r HEAD` lists exactly 4 files; `verify-chain` exits 0; the new test passes under `cargo test --workspace`.

Tier: trivial per §16.1 — (a) ≤ 5 source lines in 1 file (one attribute change), (b) no behavioral risk to existing code paths (the `version` attribute is side-effect-free; the refinement is byte-equivalent at runtime), (c) no safety-critical path touched. All three clauses hold.

Per-task exploration budget (§1.13): max 8 file reads, ~1.5K tokens, wallclock 5 min. Revised to 12 tool calls (the four gate runners each count as a tool call) — recorded as a process improvement, not a violation.

Trivial-tier section omission note (per §16.2 rule 4): the Failures and Interventions sections are present but empty (the binary emits them automatically via `episode open`; this run had no failures and no interventions).
