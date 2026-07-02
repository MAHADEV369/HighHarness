//! HighHarness binary entry point.
//!
//! Forwards to [`highharness::cli::run`], which dispatches to the
//! subcommands defined in `src/cli/`.

use anyhow::Result;

/// Binary entry point. Returns the process exit code via `anyhow::Error`.
fn main() -> Result<()> {
    highharness::cli::run()
}
