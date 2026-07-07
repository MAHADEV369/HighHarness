# Contributing to HighHarness

## Development Setup

```bash
# Build
cargo build --release

# Run tests
cargo test --all-features

# Run lints
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## Bootstrap Protocol

Before running agent workflows, bootstrap the harness:

```bash
# First build
cargo build --release

# Initialize (creates .harness/ directory)
./target/release/HighHarness bootstrap init --human "Your Name"

# Verify
./target/release/HighHarness bootstrap verify
```

## Project Structure

- `src/` — Rust library and binary source
  - `cli/` — CLI subcommand dispatch (20 subcommands)
  - `tools/` — Built-in tool implementations (fs.read, shell.exec, etc.)
  - `store/` — Disk-backed artifact persistence
  - `schema/` — Serde struct definitions for all artifacts
  - `mcp/` — Model Context Protocol server
  - `canonical.rs` — SHA-256 canonical serialization
  - `permissions.rs` — Default-deny permission engine
  - `gates.rs` — 4-stage verification gate runner
  - `redaction.rs` — Secret pattern scanning and replacement
  - `metrics.rs` — KPI rollup computation
  - `error.rs` — Single `HxError` enum vocabulary
- `tests/` — Integration tests (18 test files)
- `scripts/` — Demo fixtures and reproducibility scripts
- `data/evals/` — Synthetic eval task fixtures

## Spec Files

The project is governed by these spec documents (read in order):

1. `readharness.md` — Quick-start orientation
2. `HARNESS_ENGINEERING.md` — Binding operating rules
3. `HARNESS_PRIMITIVES.md` — Interfaces and data formats
4. `HARNESS_SECURITY.md` — Threat model and mitigations
5. `HARNESS_METRICS.md` — KPI definitions
6. `HARNESS_VERSIONING.md` — Versioning and upgrade protocol
7. `HARNESS_INTEGRATION.md` — MCP connection guide

## Adding a New Tool

1. Create `src/tools/my_tool.rs` with a `pub fn run(args: Value, root: &Path) -> HxResult<ToolResult>`
2. Register the module in `src/tools/mod.rs`
3. Add the dispatch in `src/tools/registry.rs` `invoke_raw()`
4. Create `.harness/tools/my_tool.toml` via `materialize_builtin_tools` in `src/bootstrap.rs`
5. Add permission rules in `.harness/permissions.toml`
6. Write unit tests in the tool module and integration tests in `tests/`

## Updating Hashes After Spec Changes

If the canonical form in `src/canonical.rs` changes, regenerate golden fixtures:

```bash
# Update the golden fixture hash
make entry-1-demo
# Verify the golden test still passes
cargo test canonical_golden
```

## Testing

```bash
# All tests
cargo test --all-features

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific test
cargo test permissions_default_deny_wins_on_no_match

# With determinism feature
cargo test --features deterministic
```

## Code Conventions

- Follow existing patterns (no comments on trivial code)
- New errors go in `src/error.rs` as `HxError` variants (avoid `HxError::Other`)
- Public items must have doc comments (enforced by `#![deny(missing_docs)]`)
- Atomic writes: write-to-temp + rename, never in-place mutation
- All shared-writer paths must acquire the documented lock
