//! Model registry and router (config-only; no provider HTTP).
//!
//! Per `HARNESS_PRIMITIVES.md` §6.2–6.3.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use toml::Value as TomlValue;

use crate::error::{HxError, HxResult};
use crate::store::{models_path, routing_path};

/// OpenAI-chat-completions-compatible provider adapter.
pub mod openai_compat;

/// Descriptor for a registered model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    /// Unique model identifier.
    pub id: String,
    /// Provider name (e.g., "openai", "anthropic").
    pub provider: String,
    /// Maximum context window size in tokens.
    pub context_window: u64,
    /// Model capabilities (JSON).
    pub capabilities: Value,
    /// Pricing information (JSON).
    pub pricing: Value,
    /// Privacy level classification (JSON).
    pub privacy: Value,
    /// Authentication method required.
    pub auth: String,
    /// Model tier (e.g., "standard", "premium").
    pub tier: String,
}

/// A routing rule mapping a feature to a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    /// Feature name (e.g., "chat", "completion").
    pub feature: String,
    /// Primary model ID for this feature.
    pub primary: String,
    /// Fallback model IDs if primary is unavailable.
    #[serde(default)]
    pub fallback: Vec<String>,
    /// Routing mode ("primary-only", "failover", "manual").
    pub mode: String,
}

/// Resolved routing decision for a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    /// Feature name.
    pub feature: String,
    /// Selected primary model ID.
    pub primary: String,
    /// Available fallback model IDs.
    pub fallback: Vec<String>,
    /// Routing mode.
    pub mode: String,
}

/// Load model descriptors from the models TOML file.
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
    Ok(out)
}

/// Load routing entries from the routing TOML file.
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
    Ok(out)
}

/// Resolve the routing decision for a feature.
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
    Err(HxError::NotYetEnforced {
        what: format!("no route for feature {}", feature),
    })
}

#[allow(dead_code)]
/// Return an empty model descriptor JSON object.
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
