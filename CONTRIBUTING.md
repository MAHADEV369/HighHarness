# Contributing to HighHarness

> spec_version: 1
> status: stable

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

## PR Workflow

1. Create a feature branch from `main`
2. Make your changes, keeping commits atomic and messages descriptive
3. Run `cargo clippy --all-targets --all-features -- -D warnings` and fix any warnings
4. Run `cargo fmt --all -- --check` and fix formatting
5. Run `cargo test --all-features` and ensure all tests pass
6. Run `cargo doc --no-deps` and verify no missing-docs errors
7. Open a pull request against `main` with a clear description of the change

## Commit Message Conventions

Use conventional commits format:

```
<type>: <short description>

<optional body>
```

Types: `feat` (new feature), `fix` (bug fix), `docs` (documentation), `refactor` (code change without feature/fix), `test` (tests), `chore` (maintenance), `sec` (security).

Examples:
- `feat: add --dry-run flag to fs.edit`
- `fix: prevent panic on missing permissions.toml`
- `docs: add troubleshooting section to HARNESS_INTEGRATION.md`

## Code Review Expectations

- All PRs require at least one approval before merging
- Reviewers should verify: correctness, test coverage, documentation, security implications
- All CI checks must pass (clippy, fmt, test, doc)
- Prefer small, focused PRs over large, sweeping changes

## Issue Tracker

Report bugs and request features at [GitHub Issues](https://github.com/MAHADEV369/HighHarness/issues).

When filing a bug report, include:
- HighHarness version (`HighHarness --version`)
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected vs actual behavior
- Relevant log output or error messages

## Release Process

1. Update version in `Cargo.toml` per [SemVer](https://semver.org/)
2. Update `CHANGELOG.agent.md` with the release entry
3. Run full test suite: `cargo test --all-features`
4. Verify determinism: `cargo build --release --features deterministic`
5. Tag the release: `git tag vX.Y.Z && git push --tags`
6. Publish: `cargo publish`
7. Create a GitHub Release with release notes

## Testing Philosophy

- Unit tests live alongside the code in `#[cfg(test)]` modules
- Integration tests live in `tests/` and test the binary as a subprocess
- All new features should include tests
- Prefer deterministic tests (no network or time dependencies when avoidable)
- Use `TempDir` for filesystem tests, never the real working tree
