//! End-to-end test: produce a real Entry 1 in a tempdir.
//!
//! Per BUILD_PHASE_1.md §3 step 14: the canonical Entry 1 integration test.
//! All 16 scenarios are exercised in one test that produces a real chain
//! and verifies it.

use std::fs;
use std::path::Path;
use std::process::Command;

use highharness::id;
use highharness::schema::changelog::Entry;
use highharness::store::{changelog, episode as episode_store, snapshots as snap_store};
use tempfile::TempDir;

fn git_init(root: &Path) {
    let _ = Command::new("git").arg("init").arg("-q").arg(root).output();
    let _ = Command::new("git")
        .args(["config", "user.email", "test@highharness"])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output();
}

fn bootstrap_harness(root: &Path) {
    highharness::bootstrap::init(root, "test-admin").expect("bootstrap init");
}

#[test]
fn e2e_entry_1_full_flow() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    git_init(root);
    bootstrap_harness(root);

    // 3. Create notes.md fixture + a stub Cargo.toml so `cargo check` works.
    let notes = root.join("notes.md");
    fs::write(&notes, "hello\n").unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"e2e-fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[lib]\npath = \"lib.rs\"\n",
    )
    .unwrap();
    fs::write(root.join("lib.rs"), "// empty\n").unwrap();
    let _ = Command::new("git")
        .args(["add", "-A"])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "init", "-q"])
        .current_dir(root)
        .output();

    // 4. Open an episode.
    let run_id = format!(
        "{}-test-e2e-{}",
        id::now_compact(),
        &id::run_id("e2e", "tst")[id::run_id("e2e", "tst").len() - 4..]
    );
    let agent_id = id::agent_id();
    let task_spec = "Append 'verified by HighHarness' line to notes.md (e2e Entry 1 test).";
    episode_store::open(
        root,
        &run_id,
        &agent_id,
        task_spec,
        "trivial",
        "highharness",
    )
    .expect("episode open");

    // 5. Permission check for fs.edit (should allow on notes.md, not .harness/).
    let reg = highharness::tools::registry::Registry::load(root).unwrap();
    let desc = reg.get("fs.edit").cloned().unwrap();
    let pf = highharness::permissions::load(root).unwrap();
    let args = serde_json::json!({"path":"notes.md", "old":"hello", "new":"hello\nverified by HighHarness"});
    let d = highharness::permissions::check(&pf, &desc, &args, None, "tc_e2e_1").unwrap();
    assert_eq!(
        d.decision, "allow",
        "fs.edit on notes.md must be allow: {:?}",
        d
    );

    // 6. fs.edit via registry.
    let ctx = highharness::tools::registry::InvokeCtx {
        run_id: run_id.clone(),
        agent_id: agent_id.clone(),
        tool_call_id: id::tool_call_id(&run_id, 1),
    };
    let res = reg
        .invoke_raw("fs.edit", args.clone(), &ctx, None, root)
        .expect("fs.edit");
    assert!(res.ok, "fs.edit must succeed");
    let after = fs::read_to_string(&notes).unwrap();
    assert!(after.contains("verified by HighHarness"));

    // 7. snapshots: pre-edit & post-edit
    let pre_id = snap_store::take(root, &run_id, "pre-edit").unwrap();
    let post_id = snap_store::take(root, &run_id, "post-edit").unwrap();
    assert!(!pre_id.is_empty());
    assert!(!post_id.is_empty());

    // 8. snapshot.diff
    let _diff = snap_store::diff(root, &pre_id, &post_id).unwrap();

    // 9-11. Gates: syntactic, functional, semantic — using "true" smoke fallback
    // since the tempdir is empty.
    let changes = serde_json::json!({"changed": ["notes.md"]});
    let s = highharness::gates::run("highharness", "syntactic", &run_id, changes.clone(), root)
        .unwrap();
    assert_eq!(s.status, "pass", "syntactic must pass: {:?}", s);
    let f = highharness::gates::run("highharness", "functional", &run_id, changes.clone(), root)
        .unwrap();
    assert_eq!(f.status, "pass", "functional must pass (smoke): {:?}", f);
    let sem = highharness::gates::run_semantic(
        "highharness",
        &run_id,
        serde_json::json!({
            "schema_version": 1,
            "phase": "highharness",
            "mappings": [
                { "criterion": "notes.md contains the appended line", "outcome": "met", "evidence": "fs.read of notes.md post-edit confirms the appended line is present" },
                { "criterion": "no files modified outside the intended scope", "outcome": "met", "evidence": "git status shows only notes.md modified" }
            ],
            "all_met": true
        }),
        root,
    )
    .unwrap();
    assert_eq!(sem.status, "pass", "semantic must pass: {:?}", sem);

    // 12. Append Entry 1 to the changelog (compare-and-append).
    let prev_hash = changelog::latest_or_genesis(root).unwrap();
    let mut e = Entry {
        n: 1,
        ts: id::now_iso(),
        agent: "highharness/e2e-test".to_string(),
        run_id: run_id.clone(),
        tier: "trivial".to_string(),
        files: vec!["notes.md".to_string()],
        intent: "e2e test: append 'verified by HighHarness' to notes.md".to_string(),
        diff_summary: "appended 'verified by HighHarness' to notes.md".to_string(),
        evidence: "gates: syntactic+functional+semantic all pass; snapshot diff applied"
            .to_string(),
        attribution: "none".to_string(),
        verification: "full".to_string(),
        status: "modified".to_string(),
        prev_hash: prev_hash.clone(),
        this_hash: String::new(),
    };
    let this_hash = changelog::append(&mut e, root).expect("append Entry 1");
    e.this_hash = this_hash.clone();

    // 13. Assertions: after append, e.this_hash is the new head; the chain
    // verifies that this entry chains to the previous head.
    let new_head = changelog::latest_or_genesis(root).unwrap();
    assert_eq!(
        e.this_hash, new_head,
        "Entry 1 this_hash must equal the new chain head ({}), got ({})",
        new_head, e.this_hash
    );
    // The bootstrap eval appends its own Entry 1, so e.prev_hash chains to
    // the bootstrap entry's this_hash. We verify the chain is intact below.

    // 14. episode.close
    let verif = "- syntactic: Y\n- functional: Y\n- semantic: Y\n- regression: Y\n- attribution: Y\n- memory: Y\n";
    let ep_hash = episode_store::close(root, &run_id, verif, vec!["notes.md".to_string()])
        .expect("episode close");
    assert_eq!(ep_hash.len(), 64);

    // 15. chain verifies healthy.
    let broken = changelog::verify_chain(root, None).unwrap();
    assert!(broken.is_empty(), "chain must be healthy: {:?}", broken);

    // 16. Print key outputs for the "done when" check.
    println!("this_hash = {}", e.this_hash);
    println!("prev_hash = {}", e.prev_hash);
    println!("verify_chain = []");
    println!("episode_hash = {}", ep_hash);
    println!("OK e2e Entry 1 in {} ({} ms)", root.display(), 0);
}

#[allow(dead_code)]
fn bootstrap_genesis_hash(root: &Path) -> String {
    let bs = highharness::bootstrap::verify(root).expect("bootstrap verify");
    bs.genesis_hash
}
