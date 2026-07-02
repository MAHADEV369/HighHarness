//! Chain tests for `store::changelog` (Step 4 done-when).
//!
//! Verifies the GENESIS marker → Entry 1 → Entry 2 chain, including tamper
//! detection.

use std::path::Path;

use highharness::canonical;
use highharness::id;
use highharness::schema::changelog::Entry;
use highharness::store::changelog;
use tempfile::TempDir;

fn write_genesis(dir: &Path) -> String {
    let ts = id::now_compact();
    let hash = canonical::genesis_hash(&ts);
    let body = format!(
        "## GENESIS — {}\n- prev_hash: null\n- this_hash: {}\n- bootstrap_human: admin\n- bootstrap_commit: 0\n- spec_versions: {{}}\n",
        ts, hash
    );
    std::fs::write(highharness::store::changelog_path(dir), body).unwrap();
    hash
}

fn make_entry(n: u64, intent: &str) -> Entry {
    Entry {
        n,
        ts: id::now_iso(),
        agent: "test-agent".to_string(),
        run_id: id::run_id("chain", "tst"),
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
    let gh = write_genesis(dir.path());
    let mut e = make_entry(1, "first");
    e.prev_hash = changelog::latest_or_genesis(dir.path()).unwrap();
    assert_eq!(
        e.prev_hash, gh,
        "Entry 1 prev_hash must equal GENESIS this_hash"
    );
    let h = changelog::append(&mut e, dir.path()).unwrap();
    assert_eq!(h, e.this_hash);
    let broken = changelog::verify_chain(dir.path(), None).unwrap();
    assert!(broken.is_empty(), "chain must be healthy: {:?}", broken);
}

#[test]
fn multi_entry_chain() {
    let dir = TempDir::new().unwrap();
    write_genesis(dir.path());
    for n in 1..=5u64 {
        let mut e = make_entry(n, &format!("entry {}", n));
        e.prev_hash = changelog::latest_or_genesis(dir.path()).unwrap();
        changelog::append(&mut e, dir.path()).unwrap();
    }
    let broken = changelog::verify_chain(dir.path(), None).unwrap();
    assert!(broken.is_empty());
}

#[test]
fn verify_chain_detects_field_tamper() {
    let dir = TempDir::new().unwrap();
    write_genesis(dir.path());
    for n in 1..=3u64 {
        let mut e = make_entry(n, &format!("entry {}", n));
        e.prev_hash = changelog::latest_or_genesis(dir.path()).unwrap();
        changelog::append(&mut e, dir.path()).unwrap();
    }
    // Tamper: change entry 2's intent in the file.
    let p = highharness::store::changelog_path(dir.path());
    let mut txt = std::fs::read_to_string(&p).unwrap();
    txt = txt.replace("entry 2", "entry 2 TAMPERED");
    std::fs::write(&p, txt).unwrap();
    let broken = changelog::verify_chain(dir.path(), None).unwrap();
    assert!(broken.contains(&2));
}
