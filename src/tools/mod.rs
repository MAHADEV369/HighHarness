//! Tool registry and built-in tool implementations.
//!
//! Built-in tools are described in `.harness/tools/<id>.toml` per
//! `HARNESS_PRIMITIVES.md` §1.1. This module loads the registry, dispatches
//! invocations through the permission gate, and writes tool-call ledger lines.

pub mod fs_edit;
pub mod fs_hash;
pub mod fs_read;
pub mod git_blame;
pub mod git_diff;
pub mod git_status;
pub mod lint_run;
pub mod registry;
pub mod shell_exec;
pub mod test_run;
pub mod web_fetch;

use std::path::{Component, Path, PathBuf};
use serde_json::Value;

/// Read a config value from `.harness/config.toml` by key.
pub fn read_tool_cmd(root: &Path, key: &str) -> Option<String> {
    let path = crate::store::config_path(root);
    let raw = std::fs::read_to_string(&path).ok()?;
    let v: Value = toml::from_str(&raw).ok()?;
    v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string())
}

/// Resolve a user-supplied path relative to root, ensuring it does NOT
/// escape via absolute paths, `..` traversal, or symlinks.
///
/// Returns the canonicalized path on success.  For paths that do not yet
/// exist the parent directory is canonicalized and the file component is
/// appended, still enforcing the root boundary.
pub fn resolve_safe_path(root: &Path, path: &str) -> Result<PathBuf, crate::error::HxError> {
    let p = Path::new(path);
    if p.is_absolute() {
        return Err(crate::error::HxError::Other(format!(
            "path traversal denied: absolute path '{}'",
            path
        )));
    }
    for comp in p.components() {
        if comp == Component::ParentDir {
            return Err(crate::error::HxError::Other(format!(
                "path traversal denied: '..' in path '{}'",
                path
            )));
        }
    }
    let root_canon = root
        .canonicalize()
        .map_err(|_| crate::error::HxError::Other("cannot canonicalize workspace root".into()))?;
    let full = root.join(p);
    let full_canon = if full.exists() {
        full.canonicalize().map_err(|_| {
            crate::error::HxError::Other(format!("cannot resolve path '{}'", path))
        })?
    } else {
        let parent = full.parent().ok_or_else(|| {
            crate::error::HxError::Other(format!("invalid path '{}'", path))
        })?;
        let parent_canon = parent.canonicalize().map_err(|_| {
            crate::error::HxError::Other(format!(
                "cannot resolve parent of '{}'",
                path
            ))
        })?;
        parent_canon.join(
            full.file_name()
                .ok_or_else(|| crate::error::HxError::Other("empty filename".into()))?,
        )
    };
    if cfg!(unix) && !full_canon.starts_with(&root_canon) {
        return Err(crate::error::HxError::Other(format!(
            "path traversal denied: '{}' escapes workspace root",
            path
        )));
    }
    Ok(full_canon)
}
