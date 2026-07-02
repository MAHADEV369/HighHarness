//! Deterministic ID generators for the HighHarness binary.
//!
//! Implements the ID formats mandated by `HARNESS_PRIMITIVES.md` §4.1.
//! All randomness comes from a CSPRNG ([`rand::rngs::OsRng`]); there is
//! no non-cryptographic path.

use rand::{rngs::OsRng, RngCore};
use std::fmt;

/// Deterministic `run_id` derived from a fixed seed (the GENESIS bootstrap timestamp).
/// Used ONLY by the canonical demo Makefile via `id-run --pin`. Normal agent runs
/// MUST NOT use this — see HARNESS_SECURITY.md §2 (run-ids should not be predictable).
pub fn run_id_pinned(slug: &str, agent_short: &str, genesis_ts: &str) -> String {
    // Deterministic format: <genesis_ts in compact form>-<slug>-<agent-short>-pin0
    // The trailing 4 hex chars are fixed to "pin0" under --pin, signaling pinnness.
    let ts_compact = genesis_ts.replace('-', "").replace(":", "");
    format!("{}-{}-{}-pin0", ts_compact, slug, agent_short)
}

/// Deterministic `agent_id` derived from a fixed seed (the GENESIS bootstrap timestamp).
/// Used ONLY by the canonical demo Makefile via `id-agent --pin`.
pub fn agent_id_pinned(genesis_ts: &str) -> String {
    let ts_compact = genesis_ts.replace('-', "");
    format!("agent_pinned_{}", ts_compact)
}

/// Allocate a stable per-process agent id.
///
/// Format: `agent_<random8hex>_<iso8601-second>`.
pub fn agent_id() -> String {
    let mut b = [0u8; 4];
    OsRng.fill_bytes(&mut b);
    let rand8: u32 =
        ((b[0] as u32) << 24) | ((b[1] as u32) << 16) | ((b[2] as u32) << 8) | (b[3] as u32);
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    format!("agent_{:08x}_{}", rand8, ts)
}

/// Allocate a top-level run id.
///
/// Format: `<iso8601>-<slug>-<agent_short>-<rand4>`.
pub fn run_id(slug: &str, agent_short: &str) -> String {
    let mut b = [0u8; 2];
    OsRng.fill_bytes(&mut b);
    let rand4 = format!("{:02x}{:02x}", b[0], b[1]);
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H%M%SZ");
    format!("{}-{}-{}-{}", ts, slug, agent_short, rand4)
}

/// Allocate a sub-run id (for child agents spawned under a parent run).
pub fn sub_run_id(run_id: &str, n: u32) -> String {
    format!("{}.{}", run_id, n)
}

/// Allocate a monotonic tool-call id within a run.
pub fn tool_call_id(run_id: &str, seq: u32) -> String {
    format!("tc_{}_{}", run_id, seq)
}

/// Allocate an approval request id.
pub fn approval_id(seq: u32) -> String {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    format!("appr_{}_{:06}", ts, seq)
}

/// Allocate a clarification request id.
pub fn clarification_id(seq: u32) -> String {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    format!("clr_{}_{:06}", ts, seq)
}

/// Allocate an intervention record id.
pub fn intervention_id(seq: u32) -> String {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    format!("int_{}_{:06}", ts, seq)
}

/// Allocate a snapshot id.
pub fn snapshot_id(run_id: &str, label: &str) -> String {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    format!("snap_{}_{}_{}", ts, run_id, label)
}

/// Allocate an incident id.
pub fn incident_id() -> String {
    let mut b = [0u8; 4];
    OsRng.fill_bytes(&mut b);
    let rand8: u32 =
        ((b[0] as u32) << 24) | ((b[1] as u32) << 16) | ((b[2] as u32) << 8) | (b[3] as u32);
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    format!("inc_{}_{:08x}", ts, rand8)
}

/// Format an ISO-8601 second-precision timestamp.
pub fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Format an ISO-8601 second-precision timestamp (compact form used in IDs).
pub fn now_compact() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H%M%SZ").to_string()
}

/// Helper: build a `Display` formatter for the byte/hex encoding used by IDs.
#[allow(dead_code)]
/// fn `hex` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_run_id(id: &str) -> bool {
        // Format: YYYY-MM-DDTHHMMMSSZ-<slug>-<agent_short>-<rand4>
        // Field widths: 4-2-2 T 6 Z => 4+1+2+1+2+1+6+1 = 18 chars
        // then '-' + slug + '-' + agent_short + '-' + 4 hex
        if id.len() < 18 + 4 {
            return false;
        }
        let bytes = id.as_bytes();
        // YYYY
        if !bytes[0..4].iter().all(|b| b.is_ascii_digit()) {
            return false;
        }
        if bytes[4] != b'-' {
            return false;
        }
        if !bytes[5..7].iter().all(|b| b.is_ascii_digit()) {
            return false;
        }
        if bytes[7] != b'-' {
            return false;
        }
        if !bytes[8..10].iter().all(|b| b.is_ascii_digit()) {
            return false;
        }
        if bytes[10] != b'T' {
            return false;
        }
        if !bytes[11..17].iter().all(|b| b.is_ascii_digit()) {
            return false;
        }
        if bytes[17] != b'Z' {
            return false;
        }
        if bytes[18] != b'-' {
            return false;
        }
        // tail must be ...-<agent_short>-<rand4>
        let tail = &id[19..];
        // split from end: last 4 chars are rand4 hex
        if tail.len() < 6 {
            return false;
        }
        let rand4 = &tail[tail.len() - 4..];
        if !rand4
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        {
            return false;
        }
        if tail.as_bytes()[tail.len() - 5] != b'-' {
            return false;
        }
        // agent_short is lowercase alnum; look from the end for the second-to-last '-'
        let before_rand4 = &tail[..tail.len() - 5];
        if let Some(idx) = before_rand4.rfind('-') {
            let agent_short = &before_rand4[idx + 1..];
            if !agent_short
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            {
                return false;
            }
        } else {
            return false;
        }
        true
    }

    fn check_agent_id(id: &str) -> bool {
        // agent_<rand8hex>_<iso8601compact>
        // iso8601compact = %Y%m%dT%H%M%SZ = 4+2+2+1+2+2+2+1 = 16 chars
        let parts: Vec<&str> = id.split('_').collect();
        if parts.len() != 3 {
            return false;
        }
        if parts[0] != "agent" {
            return false;
        }
        if parts[1].len() != 8
            || !parts[1]
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        {
            return false;
        }
        if parts[2].len() != 16 {
            return false;
        }
        let p2 = parts[2].as_bytes();
        if !p2[0..8].iter().all(|b| b.is_ascii_digit()) {
            return false;
        }
        if p2[8] != b'T' || !p2[9..15].iter().all(|b| b.is_ascii_digit()) {
            return false;
        }
        if p2[15] != b'Z' {
            return false;
        }
        true
    }

    #[test]
    fn run_id_format_matches_spec() {
        let id = run_id("fix-auth-leak", "abcd");
        assert!(
            check_run_id(&id),
            "run_id format wrong: {} did not match spec",
            id
        );
    }

    #[test]
    fn agent_id_format_matches_spec() {
        let id = agent_id();
        assert!(
            check_agent_id(&id),
            "agent_id format wrong: {} did not match spec",
            id
        );
    }

    #[test]
    fn rand4_is_csprng_equipped() {
        let a = run_id("x", "y");
        let b = run_id("x", "y");
        assert_ne!(
            a, b,
            "two consecutive run_ids should differ (rand4 is CSPRNG)"
        );
    }

    #[test]
    fn sub_and_tool_and_approval_ids_format() {
        assert_eq!(sub_run_id("r", 3), "r.3");
        assert_eq!(tool_call_id("r", 7), "tc_r_7");
        assert!(approval_id(42).starts_with("appr_"));
        assert!(clarification_id(1).starts_with("clr_"));
        assert!(intervention_id(1).starts_with("int_"));
        assert!(snapshot_id("r", "baseline").starts_with("snap_"));
        assert!(incident_id().starts_with("inc_"));
    }
}
