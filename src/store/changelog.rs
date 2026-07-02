//! Changelog store: the compare-and-append primitive + chain verify.
//!
//! Implements `HARNESS_PRIMITIVES.md` §3.5.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::canonical;
use crate::error::{HxError, HxResult};
use crate::schema::changelog::Entry;
use crate::store::locks::FileLock;
use crate::store::{changelog_lock_path, changelog_path};

/// Compare-and-append the given entry. Returns the computed `this_hash`.
///
/// Per `HARNESS_PRIMITIVES.md` §3.5, this is the canonical primitive.
/// All public writes flow through this function.
pub fn append(entry: &mut Entry, root: &Path) -> HxResult<String> {
    let lock_path = changelog_lock_path(root);
    let cpath = changelog_path(root);
    let _lock = FileLock::acquire(&lock_path, 5000)?;

    // First pass: read latest under lock.
    let prev_hash = read_latest_or_genesis_hash_under_lock(&cpath)?;
    entry.prev_hash = prev_hash.clone();
    let this_hash = canonical::entry_hash(entry);
    entry.this_hash = this_hash.clone();

    // Second pass: re-read latest under lock to detect concurrent writes.
    let prev_hash2 = read_latest_or_genesis_hash_under_lock(&cpath)?;
    if prev_hash2 != prev_hash {
        // Lost the race once. Re-hash with the updated prev_hash.
        entry.prev_hash = prev_hash2.clone();
        let this_hash2 = canonical::entry_hash(entry);
        entry.this_hash = this_hash2.clone();

        // Third pass: read latest once more.
        let prev_hash3 = read_latest_or_genesis_hash_under_lock(&cpath)?;
        if prev_hash3 != prev_hash2 {
            return Err(HxError::HarnessContention);
        }

        // Append
        let bytes = canonical::serialize_entry(entry);
        append_bytes(&cpath, &bytes)?;

        // Verify by re-reading the entry we just wrote (anchored by our prev_hash).
        let written_hash = read_block_hash(&cpath, &prev_hash2)?;
        if written_hash != this_hash2 {
            return Err(HxError::AuditForgery(format!(
                "post-write hash mismatch: expected {} got {}",
                this_hash2, written_hash
            )));
        }
        return Ok(this_hash2);
    }

    // No race: append.
    let bytes = canonical::serialize_entry(entry);
    append_bytes(&cpath, &bytes)?;

    let written_hash = read_block_hash(&cpath, &prev_hash)?;
    if written_hash != this_hash {
        return Err(HxError::AuditForgery(format!(
            "post-write hash mismatch: expected {} got {}",
            this_hash, written_hash
        )));
    }
    Ok(this_hash)
}

/// Return the latest committed entry. `Err(NotBootstrapped)` if there is no
/// GENESIS marker.
pub fn latest(root: &Path) -> HxResult<Entry> {
    let cpath = changelog_path(root);
    let txt = fs::read_to_string(&cpath).unwrap_or_default();
    let entries = parse_all_entries(&txt);
    if let Some(e) = entries.last() {
        return Ok(e.clone());
    }
    Err(HxError::NotBootstrapped)
}

/// Return the `this_hash` of the latest entry, or the GENESIS marker's
/// `this_hash` if no entries have been appended yet.
pub fn latest_or_genesis(root: &Path) -> HxResult<String> {
    let cpath = changelog_path(root);
    let txt = fs::read_to_string(&cpath).unwrap_or_default();
    // If there are entries, return the latest entry's this_hash.
    if let Some(last) = parse_all_entries(&txt).last() {
        return Ok(last.this_hash.clone());
    }
    // Otherwise return GENESIS, or NotBootstrapped if no GENESIS marker either.
    if let Some(g) = parse_genesis_hash(&txt) {
        return Ok(g);
    }
    Err(HxError::NotBootstrapped)
}

/// Verify the chain; return an empty Vec on success, otherwise indices of
/// broken links (1-based entry numbers, by spec convention).
pub fn verify_chain(root: &Path, _tail: Option<usize>) -> HxResult<Vec<usize>> {
    let cpath = changelog_path(root);
    let txt = fs::read_to_string(&cpath).unwrap_or_default();
    let entries = parse_all_entries(&txt);
    let mut prev = match parse_genesis_hash(&txt) {
        Some(h) => h,
        None => return Ok(Vec::new()), // no chain to verify
    };
    let mut broken = Vec::new();
    for e in &entries {
        if e.prev_hash != prev {
            broken.push(e.n as usize);
        }
        let computed = canonical::entry_hash(e);
        if computed != e.this_hash {
            broken.push(e.n as usize);
        }
        prev = e.this_hash.clone();
    }
    Ok(broken)
}

/// Return the entry with the given number.
pub fn get(n: u64, root: &Path) -> HxResult<Entry> {
    let cpath = changelog_path(root);
    let txt = fs::read_to_string(&cpath).unwrap_or_default();
    for e in parse_all_entries(&txt) {
        if e.n == n {
            return Ok(e);
        }
    }
    Err(HxError::Other(format!("entry {} not found", n)))
}

// -- internal helpers -------------------------------------------------------

fn read_latest_or_genesis_hash_under_lock(path: &Path) -> HxResult<String> {
    let txt = fs::read_to_string(path).unwrap_or_default();
    if let Some(g) = parse_genesis_hash(&txt) {
        // If there are also entries, return the last entry's hash.
        if let Some(last) = parse_all_entries(&txt).last() {
            return Ok(last.this_hash.clone());
        }
        return Ok(g);
    }
    Err(HxError::NotBootstrapped)
}

fn append_bytes(path: &Path, bytes: &[u8]) -> HxResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(bytes)?;
    f.sync_data()?;
    Ok(())
}

fn read_block_hash(path: &Path, prev_hash: &str) -> HxResult<String> {
    // Find the entry whose prev_hash matches what we wrote, then return
    // its this_hash. This is race-safe: if a concurrent writer appended
    // an entry, we still find OUR entry by its prev_hash anchor.
    let txt = fs::read_to_string(path)?;
    for e in parse_all_entries(&txt) {
        if e.prev_hash == prev_hash {
            return Ok(e.this_hash);
        }
    }
    Err(HxError::Other(format!(
        "no entry with prev_hash {} found after write",
        prev_hash
    )))
}

fn parse_all_entries(txt: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    // Split on lines starting with "## ENTRY "
    let mut current: Vec<String> = Vec::new();
    for line in txt.lines() {
        if line.starts_with("## ENTRY ") {
            if !current.is_empty() {
                if let Some(e) = parse_entry_block(&current) {
                    out.push(e);
                }
            }
            current.clear();
            current.push(line.to_string());
        } else if line.starts_with("## GENESIS") {
            // ignore in entry list
            current.clear();
        } else if !current.is_empty() {
            current.push(line.to_string());
        }
    }
    if !current.is_empty() {
        if let Some(e) = parse_entry_block(&current) {
            out.push(e);
        }
    }
    out
}

fn parse_entry_block(lines: &[String]) -> Option<Entry> {
    if lines.is_empty() {
        return None;
    }
    // First line: "## ENTRY <N> — <ts>"
    let header = &lines[0];
    let after_entry = header.strip_prefix("## ENTRY ")?;
    let mut parts = after_entry.splitn(2, '—');
    let n_part = parts.next()?.trim();
    let ts_part = parts.next()?.trim().to_string();
    let n: u64 = n_part.parse().ok()?;

    let mut agent = String::new();
    let mut run_id = String::new();
    let mut tier = String::new();
    let mut files_str = String::new();
    let mut intent = String::new();
    let mut diff_summary = String::new();
    let mut evidence = String::new();
    let mut attribution = String::new();
    let mut verification = String::new();
    let mut status = String::new();
    let mut prev_hash = String::new();
    let mut this_hash = String::new();

    // Continuation lines start with 16 spaces. We rebuild multi-line values.
    let mut current_field: Option<&'static str> = None;
    let mut continuation = String::new();
    for line in &lines[1..] {
        if line.starts_with("                ") || line.starts_with('\t') {
            // Continuation
            if let Some(f) = current_field {
                let stripped = line
                    .trim_start_matches(' ')
                    .trim_start_matches('\t')
                    .to_string();
                let target = match f {
                    "diff_summary" => &mut diff_summary,
                    "evidence" => &mut evidence,
                    "intent" => &mut intent,
                    "files" => &mut files_str,
                    _ => &mut continuation,
                };
                if !target.is_empty() {
                    target.push('\n');
                }
                target.push_str(&stripped);
            }
        } else if let Some(rest) = line.strip_prefix("- ") {
            // New field
            current_field = None;
            if let Some((name, value)) = rest.split_once(':') {
                let value = value.trim_start().to_string();
                match name.trim() {
                    "agent" => {
                        agent = value;
                        current_field = Some("agent");
                    }
                    "run_id" => {
                        run_id = value;
                        current_field = Some("run_id");
                    }
                    "tier" => {
                        tier = value;
                        current_field = Some("tier");
                    }
                    "files" => {
                        files_str = value;
                        current_field = Some("files");
                    }
                    "intent" => {
                        intent = value;
                        current_field = Some("intent");
                    }
                    "diff_summary" => {
                        diff_summary = value;
                        current_field = Some("diff_summary");
                    }
                    "evidence" => {
                        evidence = value;
                        current_field = Some("evidence");
                    }
                    "attribution" => {
                        attribution = value;
                        current_field = Some("attribution");
                    }
                    "verification" => {
                        verification = value;
                        current_field = Some("verification");
                    }
                    "status" => {
                        status = value;
                        current_field = Some("status");
                    }
                    "prev_hash" => {
                        prev_hash = value;
                        current_field = Some("prev_hash");
                    }
                    "this_hash" => {
                        this_hash = value;
                        current_field = Some("this_hash");
                    }
                    _ => {}
                }
            }
        }
    }

    let files: Vec<String> = files_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Some(Entry {
        n,

        ts: ts_part,
        agent,
        run_id,
        tier,
        files,
        intent,
        diff_summary,
        evidence,
        attribution,
        verification,
        status,
        prev_hash,
        this_hash,
    })
}

fn parse_genesis_hash(txt: &str) -> Option<String> {
    // The GENESIS marker:
    //   ## GENESIS — <ts>
    //   - prev_hash: null
    //   - this_hash: <hex>
    let mut in_genesis = false;
    for line in txt.lines() {
        if line.starts_with("## GENESIS") {
            in_genesis = true;
            continue;
        }
        if in_genesis {
            if line.starts_with("## ") {
                break;
            }
            if let Some(rest) = line.strip_prefix("- this_hash:") {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id;
    use tempfile::TempDir;

    fn write_genesis(dir: &Path) {
        let ts = id::now_iso();
        let hash = canonical::genesis_hash(&ts);
        let body = format!(
            "## GENESIS — {}\n- prev_hash: null\n- this_hash: {}\n- bootstrap_human: admin\n- bootstrap_commit: 0\n- spec_versions: {{}}\n",
            ts, hash
        );
        std::fs::write(changelog_path(dir), body).unwrap();
    }

    fn make_entry(n: u64, intent: &str) -> Entry {
        Entry {
            n,

            ts: id::now_iso(),
            agent: "test-agent".to_string(),

            run_id: id::run_id("test", "tst"),
            tier: "trivial".to_string(),

            files: vec!["a.txt".to_string()],

            intent: intent.to_string(),
            diff_summary: "n/a".to_string(),
            evidence: "n/a".to_string(),
            attribution: "none".to_string(),
            verification: "syntactic".to_string(),
            status: "added".to_string(),

            prev_hash: String::new(),

            this_hash: String::new(),
        }
    }

    #[test]
    fn genesis_linkage_v1() {
        let dir = TempDir::new().unwrap();
        write_genesis(dir.path());
        let mut e = make_entry(1, "first");
        e.prev_hash = latest_or_genesis(dir.path()).unwrap();
        let h = append(&mut e, dir.path()).unwrap();
        assert!(!h.is_empty());
        assert_eq!(verify_chain(dir.path(), None).unwrap(), Vec::<usize>::new());
    }

    #[test]
    fn compare_and_append_serial() {
        let dir = TempDir::new().unwrap();
        write_genesis(dir.path());
        for n in 1..=20u64 {
            let mut e = make_entry(n, &format!("entry {}", n));
            e.prev_hash = latest_or_genesis(dir.path()).unwrap();
            append(&mut e, dir.path()).unwrap();
        }
        let broken = verify_chain(dir.path(), None).unwrap();
        assert!(broken.is_empty(), "broken: {:?}", broken);
    }

    #[test]
    fn verify_chain_detects_tamper() {
        let dir = TempDir::new().unwrap();
        write_genesis(dir.path());
        for n in 1..=3u64 {
            let mut e = make_entry(n, &format!("entry {}", n));
            e.prev_hash = latest_or_genesis(dir.path()).unwrap();
            append(&mut e, dir.path()).unwrap();
        }
        // Tamper with entry 2's intent directly in the file.
        let p = changelog_path(dir.path());
        let mut txt = std::fs::read_to_string(&p).unwrap();
        txt = txt.replace("entry 2", "entry 2 TAMPERED");
        std::fs::write(&p, txt).unwrap();
        let broken = verify_chain(dir.path(), None).unwrap();
        assert!(
            broken.contains(&2),
            "should detect tamper at entry 2: {:?}",
            broken
        );
    }

    #[test]
    fn latest_or_genesis_returns_genesis_when_no_entries() {
        let dir = TempDir::new().unwrap();
        write_genesis(dir.path());
        let h = latest_or_genesis(dir.path()).unwrap();
        assert_eq!(h.len(), 64);
    }
}
