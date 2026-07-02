//! `HighHarness id-run` and `HighHarness id-agent` subcommands.
//!
//! Thin Phase 2 wrappers around `crate::id::run_id` and `crate::id::agent_id`.
//! They print exactly the id on stdout, no other output.
//!
//! `--pin` (buildedit.md Area F) reads the GENESIS bootstrap timestamp from
//! `.harness/artifacts/bootstrap/bootstrap.json` and uses it as the seed for
//! deterministic id generation. Used ONLY by the canonical demo Makefile;
//! normal agent runs MUST NOT use `--pin` per `HARNESS_SECURITY.md` §2.

use std::path::Path;

use clap::Parser;

use crate::error::HxResult;

/// `HighHarness id-run` — allocate a top-level run id.
///
/// Format: `<iso8601>-<slug>-<agent_short>-<rand4>`.
#[derive(Parser, Debug)]
/// struct `IdRunCmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct IdRunCmd {
    /// Short slug for the run.
    #[clap(long, default_value = "run")]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub slug: String,

    /// Short identifier of the calling agent (lowercase alnum).
    #[clap(long, default_value = "agent")]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub agent: String,

    /// Pin the run_id to the GENESIS bootstrap timestamp for reproducible
    /// canonical demos. Normal runs MUST NOT use --pin.
    #[clap(long)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub pin: bool,
}

/// `HighHarness id-agent` — allocate a stable per-process agent id.
#[derive(Parser, Debug)]
/// struct `IdAgentCmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct IdAgentCmd {
    /// (Reserved) path to a state dir for sticky agent ids. Currently unused.
    #[clap(long, default_value = ".")]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub state_dir: String,

    /// Pin the agent_id to the GENESIS bootstrap timestamp for reproducible
    /// canonical demos. Normal runs MUST NOT use --pin.
    #[clap(long)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub pin: bool,
}

/// fn `run_id_cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run_id_cmd(cmd: IdRunCmd) -> HxResult<i32> {
    let id = if cmd.pin {
        // Read GENESIS bootstrap timestamp from bootstrap.json.
        let bs_path = crate::store::bootstrap_path(Path::new("."));
        let raw = std::fs::read_to_string(&bs_path).map_err(|e| {
            crate::error::HxError::Other(format!(
                "id-run --pin: cannot read {}: {}",
                bs_path.display(),
                e
            ))
        })?;
        let bs: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
            crate::error::HxError::Other(format!("id-run --pin: malformed bootstrap.json: {}", e))
        })?;
        let ts = bs
            .get("bootstrapped_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::HxError::Other(
                    "id-run --pin: bootstrap.json missing bootstrapped_at".into(),
                )
            })?
            .to_string();
        crate::id::run_id_pinned(&cmd.slug, &cmd.agent, &ts)
    } else {
        crate::id::run_id(&cmd.slug, &cmd.agent)
    };
    println!("{}", id);
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(0)
}

/// fn `run_id_agent_cmd` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn run_id_agent_cmd(cmd: IdAgentCmd) -> HxResult<i32> {
    let _ = cmd.state_dir;
    let id = if cmd.pin {
        let bs_path = crate::store::bootstrap_path(Path::new("."));
        let raw = std::fs::read_to_string(&bs_path).map_err(|e| {
            crate::error::HxError::Other(format!(
                "id-agent --pin: cannot read {}: {}",
                bs_path.display(),
                e
            ))
        })?;
        let bs: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
            crate::error::HxError::Other(format!("id-agent --pin: malformed bootstrap.json: {}", e))
        })?;
        let ts = bs
            .get("bootstrapped_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::HxError::Other(
                    "id-agent --pin: bootstrap.json missing bootstrapped_at".into(),
                )
            })?
            .to_string();
        crate::id::agent_id_pinned(&ts)
    } else {
        crate::id::agent_id()
    };
    println!("{}", id);
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(0)
}
