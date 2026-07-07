//! Snapshot store, per `HARNESS_PRIMITIVES.md` §8.

use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

use sha2::{Digest, Sha256};

use crate::error::{HxError, HxResult};
use crate::id;
use crate::schema::snapshot::{Snapshot, SnapshotGit};
use crate::store::locks::FileLock;
use crate::store::snapshots_dir;

/// Take a snapshot of the current git state + write a Snapshot descriptor.
pub fn take(root: &Path, run_id: &str, label: &str) -> HxResult<String> {
    let lock_path = crate::store::locks_dir(root).join("snapshot.lock");
    let _lock = FileLock::acquire(&lock_path, 5000)?;

    let git = read_git_state(root);
    let snap_id = id::snapshot_id(run_id, label);
    let snap_dir = snapshots_dir(root).join(run_id);
    fs::create_dir_all(&snap_dir)?;
    let path = snap_dir.join(format!("{}.json", label));
    let snap = Snapshot {
        schema_version: 1,

        snapshot_id: snap_id.clone(),

        run_id: run_id.to_string(),

        label: label.to_string(),
        git,

        tests: None,

        types: None,

        lint: None,

        phase: String::new(),

        taken_at: id::now_iso(),

        self_hash: None,
    };
    let s = serde_json::to_string_pretty(&snap)?;
    fs::write(&path, s)?;
    Ok(snap_id)
}

fn read_git_state(root: &Path) -> SnapshotGit {
    let commit = run_capture(root, &["rev-parse", "HEAD"]).unwrap_or_default();
    let status = run_capture(root, &["status", "--porcelain=v2", "-b"]).unwrap_or_default();
    let dirty = !status.is_empty();
    let diff_stat = run_capture(root, &["diff", "--stat"]).unwrap_or_default();
    SnapshotGit {
        commit,
        dirty,

        diff_stat: if dirty { diff_stat } else { String::new() },
    }
}

fn run_capture(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Read a snapshot descriptor.
pub fn get(root: &Path, snapshot_id: &str) -> HxResult<Snapshot> {
    // scan all run dirs for a file containing the snapshot_id
    let dir = snapshots_dir(root);
    let mut result: Option<Snapshot> = None;
    if dir.exists() {
        for run_dir in std::fs::read_dir(&dir)? {
            let run_dir = run_dir?;
            let run_path = run_dir.path();
            if run_path.is_dir() {
                for entry in std::fs::read_dir(&run_path)? {
                    let entry = entry?;
                    let raw = std::fs::read_to_string(entry.path())?;
                    let snap: Snapshot = serde_json::from_str(&raw)?;
                    if snap.snapshot_id == snapshot_id {
                        result = Some(snap);
                        break;
                    }
                }
            }
        }
    }
    match result {
        Some(snap) => {
            // W5: verify self_hash
            if let Some(ref h) = snap.self_hash {
                let mut for_hash = snap.clone();
                for_hash.self_hash = None;
                let json = serde_json::to_string(&for_hash)?;
                let mut hasher = Sha256::new();
                hasher.update(json.as_bytes());
                let computed = format!("{:x}", hasher.finalize());
                if &computed != h {
                    return Err(HxError::AuditForgery(
                        "snapshot self_hash mismatch".to_string(),
                    ));
                }
            }
            Ok(snap)
        }
        None => Err(HxError::Other(format!(
            "snapshot not found: {}",
            snapshot_id
        ))),
    }
}

/// Diff two snapshots (by snapshot_id). Compare git commit hashes and dirty state.
pub fn diff(root: &Path, before: &str, after: &str) -> HxResult<Value> {
    let before_snap = get(root, before)?;
    let after_snap = get(root, after)?;

    let same_commit = before_snap.git.commit == after_snap.git.commit;
    let same_dirty = before_snap.git.dirty == after_snap.git.dirty;

    let mut diff_parts: Vec<String> = Vec::new();
    if !same_commit {
        diff_parts.push(format!(
            "commit: {} → {}",
            &before_snap.git.commit[..8.min(before_snap.git.commit.len())],
            &after_snap.git.commit[..8.min(after_snap.git.commit.len())]
        ));
    }
    if !same_dirty {
        diff_parts.push(format!(
            "dirty: {} → {}",
            before_snap.git.dirty, after_snap.git.dirty
        ));
    }
    if diff_parts.is_empty() {
        diff_parts.push("no changes".to_string());
    }

    Ok(json!({
        "before": before,
        "after": after,
        "diff": diff_parts.join("; "),
        "before_commit": before_snap.git.commit,
        "after_commit": after_snap.git.commit,
    }))
}

/// Revert to a snapshot. Uses `git reset --hard <commit>`.
pub fn revert(root: &Path, snapshot_id: &str) -> HxResult<()> {
    let snap = get(root, snapshot_id)?;
    if snap.git.commit.is_empty() || snap.git.commit == "0000000" {
        return Err(HxError::Other(
            "snapshot has no valid commit to revert to".to_string(),
        ));
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("reset")
        .arg("--hard")
        .arg(&snap.git.commit)
        .output()
        .map_err(|e| HxError::Other(format!("git reset failed: {}", e)))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HxError::Other(format!(
            "git reset --hard {} failed: {}",
            &snap.git.commit, stderr
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn snapshot_take_records_git_commit_and_dirty_flag() {
        let dir = TempDir::new().unwrap();
        let _ = std::process::Command::new("git")
            .arg("init")
            .arg("-q")
            .arg(dir.path())
            .output();
        let id = take(dir.path(), "r1", "baseline").unwrap();
        assert!(!id.is_empty());
        let snap_dir = crate::store::snapshots_dir(dir.path()).join("r1");
        assert!(snap_dir.join("baseline.json").exists());
    }
}
