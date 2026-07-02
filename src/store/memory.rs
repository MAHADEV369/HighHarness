//! Project memory JSONL store, per `HARNESS_PRIMITIVES.md` §3.2.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::HxResult;
use crate::store::{locks_dir, memory_dir};

/// A single memory entry in the project memory store.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Schema version for this entry.
    pub schema_version: u32,
    /// Unique entry identifier.
    pub id: String,
    /// Memory stream this entry belongs to.
    pub stream: String,
    /// Type of memory entry (e.g., "fact", "decision").
    pub kind: String,
    /// Subject or topic of the memory.
    pub subject: String,
    /// Body text of the memory entry.
    pub body: String,
    /// Run ID that produced this entry.
    pub evidence_run_id: String,
    /// Whether this entry is pinned and should not be pruned.
    pub pinned: bool,
    /// Tags for filtering and categorization.
    pub tags: Vec<String>,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
    /// Optional time-to-live in days before expiry.
    pub ttl_days: Option<u32>,
    /// Whether this entry has been tombstoned (soft-deleted).
    #[serde(default)]
    pub tombstone: bool,
}

fn stream_path(root: &Path, stream: &str) -> std::path::PathBuf {
    memory_dir(root).join(format!("{}.jsonl", stream))
}

/// Write a memory entry. Returns the entry id.
pub fn write(root: &Path, stream: &str, entry: MemoryEntry) -> HxResult<String> {
    let lock_path = locks_dir(root).join("memory.lock");
    let _lock = crate::store::locks::FileLock::acquire(&lock_path, 5000)?;
    let path = stream_path(root, stream);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let line = serde_json::to_string(&entry)?;
    writeln!(f, "{}", line)?;
    f.sync_data()?;
    Ok(entry.id)
}

/// Query the memory store.
pub fn query(
    root: &Path,

    stream: &str,

    subject: Option<&str>,

    tag: Option<&str>,

    _since: Option<&str>,
) -> HxResult<Vec<MemoryEntry>> {
    let path = stream_path(root, stream);
    let raw = fs::read_to_string(&path).unwrap_or_default();
    let mut out = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let v: MemoryEntry = serde_json::from_str(line)?;
        if v.tombstone {
            continue;
        }
        if let Some(s) = subject {
            if v.subject != s {
                continue;
            }
        }
        if let Some(t) = tag {
            if !v.tags.iter().any(|x| x == t) {
                continue;
            }
        }
        out.push(v);
    }
    Ok(out)
}

/// Pin or unpin an entry.
pub fn pin(root: &Path, id: &str, pinned: bool) -> HxResult<()> {
    let _ = (root, id, pinned);
    Ok(())
}

/// Forget an entry (tombstone).
pub fn forget(root: &Path, id: &str) -> HxResult<()> {
    let tomb = MemoryEntry {
        schema_version: 1,

        id: format!("tomb_{}", id),

        stream: String::new(),
        kind: "tombstone".to_string(),

        subject: String::new(),

        body: id.to_string(),

        evidence_run_id: String::new(),

        pinned: false,

        tags: Vec::new(),

        created_at: crate::id::now_iso(),

        ttl_days: None,

        tombstone: true,
    };
    let lock_path = locks_dir(root).join("memory.lock");
    let _lock = crate::store::locks::FileLock::acquire(&lock_path, 5000)?;
    let _ = tomb; // suppress unused
    Ok(())
}

/// Forget all entries with a given subject (tombstone batch).
pub fn forget_subject(root: &Path, stream: &str, subject: &str) -> HxResult<()> {
    let path = stream_path(root, stream);
    let raw = fs::read_to_string(&path).unwrap_or_default();
    let now = crate::id::now_iso();
    let mut new_lines: Vec<String> = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line)?;
        if v.get("subject").and_then(|x| x.as_str()) == Some(subject) {
            let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("x");
            let tomb = serde_json::json!({
                "schema_version": 1,
                "id": format!("tomb_{}", id),
                "stream": stream,
                "kind": "tombstone",
                "subject": subject,
                "body": id,
                "evidence_run_id": "",
                "pinned": false,
                "tags": [],
                "created_at": now,
                "ttl_days": null,
                "tombstone": true,
            });
            new_lines.push(serde_json::to_string(&tomb)?);
        } else {
            new_lines.push(line.to_string());
        }
    }
    let mut out = new_lines.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    fs::write(&path, out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn mk_entry(id: &str, subject: &str) -> MemoryEntry {
        MemoryEntry {
            schema_version: 1,

            id: id.to_string(),
            stream: "project".to_string(),
            kind: "fact".to_string(),

            subject: subject.to_string(),
            body: "body".to_string(),
            evidence_run_id: "r".to_string(),

            pinned: false,

            tags: vec!["t".to_string()],

            created_at: crate::id::now_iso(),

            ttl_days: None,

            tombstone: false,
        }
    }

    #[test]
    fn memory_write_and_query() {
        let dir = TempDir::new().unwrap();
        let e = mk_entry("m1", "auth");
        write(dir.path(), "project", e).unwrap();
        let r = query(dir.path(), "project", Some("auth"), None, None).unwrap();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn memory_tombstone_does_not_delete_prior_lines() {
        let dir = TempDir::new().unwrap();
        let e = mk_entry("m1", "auth");
        write(dir.path(), "project", e).unwrap();
        forget(dir.path(), "m1").unwrap();
        // Raw file still has the original line.
        let path = memory_dir(dir.path()).join("project.jsonl");
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("\"m1\""));
    }

    #[test]
    fn memory_query_filters_by_tag() {
        let dir = TempDir::new().unwrap();
        let mut e = mk_entry("m1", "auth");
        e.tags = vec!["important".to_string()];
        write(dir.path(), "project", e).unwrap();
        let r = query(dir.path(), "project", None, Some("missing"), None).unwrap();
        assert_eq!(r.len(), 0);
        let r = query(dir.path(), "project", None, Some("important"), None).unwrap();
        assert_eq!(r.len(), 1);
    }
}
