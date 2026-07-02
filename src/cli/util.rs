//! Shared JSON-or-path reader for CLI args that accept inline JSON or a file path.
//!
//! Implements the heuristic mandated by `buildedit.md` Area A.1: if the trimmed
//! input starts with `{` or `[`, treat as inline JSON; otherwise treat as a
//! filesystem path and read its contents.

use crate::error::HxResult;

/// Read JSON content from either an inline JSON literal or a file path.
///
/// Heuristic: if the trimmed input starts with `{` or `[`, treat as inline;
/// else treat as a path and read its contents.
pub fn read_json_or_path(input: &str) -> HxResult<String> {
    let trimmed = input.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        Ok(input.to_string())
    } else {
        Ok(std::fs::read_to_string(input)?)
    }
}
