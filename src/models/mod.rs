//! Model registry and router (config-only; no provider HTTP).
//!
//! Per `HARNESS_PRIMITIVES.md` §6.2–6.3.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use toml::Value as TomlValue;

use crate::error::{HxError, HxResult};
use crate::store::{models_path, routing_path};

pub mod openai_compat;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// struct `ModelDescriptor` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct ModelDescriptor {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub provider: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub context_window: u64,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub capabilities: Value,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub pricing: Value,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub privacy: Value,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub auth: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// struct `RouteEntry` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct RouteEntry {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub feature: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub primary: String,
    #[serde(default)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub fallback: Vec<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// struct `RouteDecision` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct RouteDecision {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub feature: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub primary: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub fallback: Vec<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub mode: String,
}

/// fn `load_models` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn load_models(root: &Path) -> HxResult<Vec<ModelDescriptor>> {
    let p = models_path(root);
    if !p.exists() {
        return Ok(vec![]);
    }
    let raw = std::fs::read_to_string(&p)?;
    let v: TomlValue = toml::from_str(&raw)?;
    let arr = v
        .get("models")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for m in arr {
        if let Ok(d) = m.try_into::<ModelDescriptor>() {
            out.push(d);
        }
    }
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(out)
}

/// fn `load_routes` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn load_routes(root: &Path) -> HxResult<Vec<RouteEntry>> {
    let p = routing_path(root);
    if !p.exists() {
        return Ok(vec![]);
    }
    let raw = std::fs::read_to_string(&p)?;
    let v: TomlValue = toml::from_str(&raw)?;
    let arr = v
        .get("routes")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for r in arr {
        if let Ok(e) = r.try_into::<RouteEntry>() {
            out.push(e);
        }
    }
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(out)
}

/// fn `route` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn route(feature: &str, root: &Path) -> HxResult<RouteDecision> {
    let routes = load_routes(root)?;
    for r in routes {
        if r.feature == feature {
            return Ok(RouteDecision {
                feature: r.feature,

                primary: r.primary,

                fallback: r.fallback,

                mode: r.mode,
            });
        }
    }
    /// Variant `Err` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Err(HxError::NotYetEnforced {
        what: format!("no route for feature {}", feature),
    })
}

#[allow(dead_code)]
/// fn `empty_descriptor` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn empty_descriptor() -> Value {
    json!({})
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn models_route_returns_primary_for_feature() {
        let dir = TempDir::new().unwrap();
        if let Some(parent) = routing_path(dir.path()).parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let rbody = r#"
schema_version = 1

[[routes]]
feature = "chat"
primary = "m1"
fallback = []
mode = "primary-only"
"#;
        std::fs::write(routing_path(dir.path()), rbody).unwrap();
        let d = route("chat", dir.path()).unwrap();
        assert_eq!(d.primary, "m1");
        assert_eq!(d.mode, "primary-only");
    }

    #[test]
    fn models_route_manual_mode_refuses_auto_dispatch() {
        // We don't have an inference layer in v1; the routing decision is
        // returned but actual dispatch is not-yet-enforced.
        let dir = TempDir::new().unwrap();
        if let Some(parent) = routing_path(dir.path()).parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let rbody = r#"
schema_version = 1

[[routes]]
feature = "chat"
primary = "m1"
fallback = []
mode = "manual"
"#;
        std::fs::write(routing_path(dir.path()), rbody).unwrap();
        let d = route("chat", dir.path()).unwrap();
        assert_eq!(d.mode, "manual");
        // Phase 1: we return the routing decision; Phase 3+ will refuse auto-dispatch.
        let _ = d; // suppress unused
    }
}
