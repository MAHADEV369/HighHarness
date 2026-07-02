//! Golden test for canonical entry hashing.
//!
//! Reads `v1_entry_1.txt` (committed fixture), parses it into an Entry,
//! runs `entry_hash`, and asserts equality with the committed sha256.

use std::path::PathBuf;

use highharness::canonical;
use highharness::store::changelog;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/canonical/fixtures")
}

#[test]
fn canonical_golden_v1_entry_hash_matches() {
    let path = fixtures_dir().join("v1_entry_1.txt");
    let expected_path = fixtures_dir().join("v1_entry_1.sha256");

    let expected = std::fs::read_to_string(&expected_path)
        .expect("missing .sha256 file")
        .trim()
        .to_string();
    assert_eq!(expected.len(), 64, "sha256 hex must be 64 chars");

    // Parse the fixture into a temp changelog and read the entry back.
    let tmp = tempfile::tempdir().unwrap();
    let cpath = tmp.path().join("CHANGELOG.agent.md");
    std::fs::write(&cpath, std::fs::read_to_string(&path).unwrap()).unwrap();
    let entry = changelog::get(1, tmp.path()).expect("parse fixture entry");
    let computed = canonical::entry_hash(&entry);
    assert_eq!(
        computed, expected,
        "computed entry_hash does not match committed sha256"
    );
    // Also confirm verify_chain finds the entry's self-hash consistent.
    let broken = changelog::verify_chain(tmp.path(), None).unwrap();
    assert!(
        broken.is_empty(),
        "no genesis → no chain → empty broken: {:?}",
        broken
    );
}
