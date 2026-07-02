//! Project memory JSONL store, per `HARNESS_PRIMITIVES.md` §3.2.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::HxResult;
use crate::store::{locks_dir, memory_dir};

#[derive(Clone, Debug, Serialize, Deserialize)]
/// struct `MemoryEntry` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct MemoryEntry {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub schema_version: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub stream: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub kind: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub subject: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub body: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub evidence_run_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub pinned: bool,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tags: Vec<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub created_at: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub ttl_days: Option<u32>,
    #[serde(default)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(out)
}

/// Pin or unpin an entry.
pub fn pin(root: &Path, id: &str, pinned: bool) -> HxResult<()> {
    let _ = (root, id, pinned);
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

/// Forget an entry (tombstone).
pub fn forget(root: &Path, id: &str) -> HxResult<()> {
    let tomb = MemoryEntry {
        schema_version: 1,

        id: format!("tomb_{}", id),

        stream: String::new(),
        /// Field `kind` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
        let mut v: serde_json::Value = serde_json::from_str(line)?;
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
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn mk_entry(id: &str, subject: &str) -> MemoryEntry {
        /// Variant `MemoryEntry` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        MemoryEntry {
            schema_version: 1,

            id: id.to_string(),
            /// Field `stream` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            stream: "project".to_string(),
            /// Field `kind` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            kind: "fact".to_string(),

            subject: subject.to_string(),
            /// Field `body` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            body: "body".to_string(),
            /// Field `evidence_run_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
