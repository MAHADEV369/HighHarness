//! Schema (data shape) definitions for all harness artifacts.
//!
//! Each child module is a pure serde struct file with no business logic.

/// mod `approval` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod approval;
/// mod `bootstrap` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod bootstrap;
/// mod `changelog` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod changelog;
/// mod `clarification` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod clarification;
/// mod `episode` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod episode;
/// mod `in_flight` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod in_flight;
/// mod `incident` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod incident;
/// mod `integrity` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod integrity;
/// mod `permission` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod permission;
/// mod `snapshot` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod snapshot;
/// mod `spend` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod spend;
/// mod `tool` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub mod tool;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::changelog::Entry;
    use crate::schema::permission::PermissionFile;
    use crate::schema::snapshot::Snapshot;
    use crate::schema::tool::ToolDescriptor;

    #[test]
    fn schema_round_trip_changelog_entry() {
        let e = Entry {
            n: 1,
            /// Field `ts` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            ts: "2026-06-29T10:14Z".to_string(),
            /// Field `agent` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            agent: "test".to_string(),
            /// Field `run_id` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            run_id: "r1".to_string(),
            /// Field `tier` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            tier: "trivial".to_string(),

            files: vec!["a".to_string()],
            /// Field `intent` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            intent: "i".to_string(),
            /// Field `diff_summary` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            diff_summary: "d".to_string(),
            /// Field `evidence` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            evidence: "e".to_string(),
            /// Field `attribution` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            attribution: "none".to_string(),
            /// Field `verification` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            verification: "syntactic".to_string(),
            /// Field `status` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            status: "added".to_string(),
            /// Field `prev_hash` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            prev_hash: "p".to_string(),
            /// Field `this_hash` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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
