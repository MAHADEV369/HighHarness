//! Concurrency test: two threads appending simultaneously.
//!
//! Per `HARNESS_PRIMITIVES.md` §3.5: the chain is tamper-evident under serial
//! writers. The compare-and-append primitive retries exactly once on a lost
//! race; a second lost race surfaces as `HarnessContention`.

use std::path::Path;
use std::sync::{Arc, Barrier};
use std::thread;

use highharness::canonical;
use highharness::id;
use highharness::schema::changelog::Entry;
use highharness::store::changelog;
use tempfile::TempDir;

fn write_genesis(dir: &Path) {
    let ts = id::now_compact();
    let hash = canonical::genesis_hash(&ts);
    let body = format!(
        "## GENESIS — {}\n- prev_hash: null\n- this_hash: {}\n- bootstrap_human: admin\n- bootstrap_commit: 0\n- spec_versions: {{}}\n",
        ts, hash
    );
    std::fs::write(highharness::store::changelog_path(dir), body).unwrap();
}

fn make_entry(n: u64, intent: &str) -> Entry {
    Entry {
        n,
        ts: id::now_iso(),
        agent: "concurrency-test".to_string(),
        run_id: id::run_id("concurrency", "tst"),
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
fn compare_and_append_serial_after_contention() {
    // 2 threads; each computes the same prev_hash, then races to append.
    // Per spec: at most one retry, then a second lost race surfaces as
    // HarnessContention (acceptable outcome). The chain MUST still verify
    // healthy after contention resolves.
    let dir = TempDir::new().unwrap();
    write_genesis(dir.path());

    let barrier = Arc::new(Barrier::new(2));
    let d1 = dir.path().to_path_buf();
    let d2 = dir.path().to_path_buf();
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    let h1 = thread::spawn(move || {
        b1.wait();
        let mut e = make_entry(1, "from thread 1");
        e.prev_hash = changelog::latest_or_genesis(&d1).unwrap();
        changelog::append(&mut e, &d1)
    });
    let h2 = thread::spawn(move || {
        b2.wait();
        let mut e = make_entry(1, "from thread 2");
        e.prev_hash = changelog::latest_or_genesis(&d2).unwrap();
        changelog::append(&mut e, &d2)
    });

    let r1 = h1.join().unwrap();
    let r2 = h2.join().unwrap();

    let outcomes: [&Result<String, highharness::error::HxError>; 2] = [&r1, &r2];
    let successes: Vec<_> = outcomes.iter().filter(|r| r.is_ok()).collect();
    let contentions: Vec<_> = outcomes
        .iter()
        .filter(|r| matches!(r, Err(highharness::error::HxError::HarnessContention)))
        .collect();

    // At least one succeeded; the other may have succeeded via retry or
    // surfaced HarnessContention.
    assert!(
        !successes.is_empty(),
        "at least one thread must succeed: {:?} / {:?}",
        r1,
        r2
    );
    assert!(
        successes.len() + contentions.len() == 2,
        "outcomes must be success or HarnessContention: {:?} / {:?}",
        r1,
        r2
    );

    // Chain verifies healthy.
    let broken = changelog::verify_chain(dir.path(), None).unwrap();
    assert!(broken.is_empty(), "broken after contention: {:?}", broken);
}
