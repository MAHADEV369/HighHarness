Add a `--version` flag to the `HighHarness` CLI that prints `HighHarness <x.y.z>`.

Completion criteria: running `HighHarness --version` prints exactly `HighHarness 0.1.0` followed by a single newline; the implementation modifies only `src/cli/mod.rs` (or `src/main.rs`) and `Cargo.toml`; all existing tests still pass; a new test asserts the output and is exercised via `cargo test --workspace`.

Tier: trivial (per HARNESS_ENGINEERING.md §16.1; ≤ 5 source lines, no behavioral risk, no safety-critical path).
