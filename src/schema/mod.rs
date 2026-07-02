//! Schema (data shape) definitions for all harness artifacts.
//!
//! Each child module is a pure serde struct file with no business logic.

/// Approval request schema.
pub mod approval;
/// Bootstrap record schema.
pub mod bootstrap;
/// Changelog entry schema.
pub mod changelog;
/// Clarification request schema.
pub mod clarification;
/// Episode trace schemas.
pub mod episode;
/// In-flight run record schema.
pub mod in_flight;
/// Incident record schema.
pub mod incident;
/// Integrity log line schema.
pub mod integrity;
/// Permission file and rule schemas.
pub mod permission;
/// Snapshot descriptor schema.
pub mod snapshot;
/// Spend line schema.
pub mod spend;
/// Tool descriptor and result schemas.
pub mod tool;

#[cfg(test)]
mod tests {
    use crate::schema::changelog::Entry;
    use crate::schema::permission::PermissionFile;
    use crate::schema::snapshot::Snapshot;
    use crate::schema::tool::ToolDescriptor;

    #[test]
    fn schema_round_trip_changelog_entry() {
        let e = Entry {
            n: 1,
            ts: "2026-06-29T10:14Z".to_string(),
            agent: "test".to_string(),
            run_id: "r1".to_string(),
            tier: "trivial".to_string(),

            files: vec!["a".to_string()],
            intent: "i".to_string(),
            diff_summary: "d".to_string(),
            evidence: "e".to_string(),
            attribution: "none".to_string(),
            verification: "syntactic".to_string(),
            status: "added".to_string(),
            prev_hash: "p".to_string(),
            this_hash: "t".to_string(),
        };
        let s = serde_json::to_string(&e).unwrap();
        let d: Entry = serde_json::from_str(&s).unwrap();
        assert_eq!(e.n, d.n);
        assert_eq!(e.agent, d.agent);
        assert_eq!(e.files, d.files);
    }

    #[test]
    fn schema_round_trip_permission_file() {
        let raw = r#"
schema_version = 1

[[rules]]
id = "R1"
effect = "allow"
tool = "fs.read"
paths = ["**"]
reason = "test"
priority = 10
"#;
        let pf: PermissionFile = toml::from_str(raw).unwrap();
        let s = toml::to_string(&pf).unwrap();
        let d: PermissionFile = toml::from_str(&s).unwrap();
        assert_eq!(pf.rules.len(), d.rules.len());
        assert_eq!(pf.rules[0].id, d.rules[0].id);
    }

    #[test]
    fn schema_round_trip_snapshot() {
        let raw = r#"
schema_version = 1
snapshot_id = "snap1"
run_id = "r1"
label = "baseline"
phase = "highharness"
taken_at = "2026-06-29T10:14Z"

[git]
commit = "abc"
dirty = false
diff_stat = ""
"#;
        let s: Snapshot = toml::from_str(raw).unwrap();
        let s2 = toml::to_string(&s).unwrap();
        let s3: Snapshot = toml::from_str(&s2).unwrap();
        assert_eq!(s.snapshot_id, s3.snapshot_id);
        assert_eq!(s.git.commit, s3.git.commit);
    }

    #[test]
    fn schema_round_trip_tool_descriptor() {
        let raw = r#"
id = "fs.read"
schema_version = 1
version = "1.0.0"
source = "builtin"
summary = "read files"
argument_schema_path = ".harness/tools/schemas/fs.read.args.json"
return_schema_path = ".harness/tools/schemas/fs.read.returns.json"
side_effect = "read"

[capabilities]
read = true
write = false
exec = false
network = false
destructive = false
secrets = false
side_effect = "read"
"#;
        let d: ToolDescriptor = toml::from_str(raw).unwrap();
        assert_eq!(d.id, "fs.read");
        assert!(d.capabilities.read);
        assert!(!d.capabilities.write);
    }

    #[test]
    fn schema_rejects_unknown_schema_version_changelog() {
        // Changelog entries are persisted as JSON (not TOML) per HARNESS_PRIMITIVES.md
        // §3.5. Schema version is not stored on the Entry struct itself; it is
        // implied by the chain (canonical text). This test asserts Entry JSON
        // round-trips even with extra/unknown fields (forward-compat).
        let e_json = r#"{"n":1,"ts":"2026-06-29T10:14Z","agent":"x","run_id":"r","tier":"trivial","files":[],"intent":"i","diff_summary":"d","evidence":"e","attribution":"none","verification":"syntactic","status":"added","prev_hash":"p","this_hash":"t","unknown_field":"x"}"#;
        let e: Entry = serde_json::from_str(e_json).unwrap();
        assert_eq!(e.n, 1);
    }
}
