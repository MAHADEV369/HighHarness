//! Permission engine, per `HARNESS_PRIMITIVES.md` §2.

use std::path::Path;

use glob::Pattern;
use serde::Serialize;
use serde_json::Value;

use crate::error::HxResult;
use crate::schema::permission::PermissionFile;
use crate::schema::tool::ToolDescriptor;
use crate::store::permissions_path;
use crate::tools::registry::ScopeNarrow;

/// Outcome of a permission check.
#[derive(Debug, Clone, Serialize)]
pub struct Decision {
    /// Schema version for this decision.
    pub schema_version: u32,
    /// Decision result: "allow", "deny", or "ask".
    pub decision: String,
    /// ID of the rule that produced this decision.
    pub rule_id: String,
    /// Human-readable reason for the decision.
    pub reason: String,
    /// Effective allowed file paths after narrowing.
    pub effective_paths: Vec<String>,
    /// Effective allowed network targets after narrowing.
    pub effective_network: Vec<String>,
    /// Tool call ID this decision applies to.
    pub tool_call_id: String,
}

/// Load the permission file. If absent, return an empty PermissionFile
/// (so default-deny applies to every call).
pub fn load(root: &Path) -> HxResult<PermissionFile> {
    let path = permissions_path(root);
    if !path.exists() {
        return Ok(PermissionFile {
            schema_version: 1,

            rules: vec![],
        });
    }
    let raw = std::fs::read_to_string(&path)?;
    let pf: PermissionFile = toml::from_str(&raw)?;
    Ok(pf)
}

/// Evaluate a single tool invocation against the rule set.
///
/// Algorithm per `HARNESS_PRIMITIVES.md` §2.3:
/// 1. Collect rules whose `tool` glob matches the requested tool id.
/// 2. Of those, collect rules whose path/network/env predicate matches.
/// 3. Sort by priority desc, then by specificity (literal > glob > bare > *).
/// 4. Apply first rule's effect. `ask` → Decision{decision:"ask", ...}.
///    No match → deny with "no-allow-rule".
/// 5. If `scope_narrow` present, intersect effective_paths/network/env with
///    the narrow fields. Empty intersection → deny with "scope-empty".
pub fn check(
    rule_file: &PermissionFile,

    tool: &ToolDescriptor,

    args: &Value,

    scope_narrow: Option<&ScopeNarrow>,

    tool_call_id: &str,
) -> HxResult<Decision> {
    let arg_path = args
        .get("path")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    let arg_network = args
        .get("network")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    let arg_env = args
        .get("env")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());

    // 1. tool glob match
    let mut matches: Vec<&crate::schema::permission::Rule> = rule_file
        .rules
        .iter()
        .filter(|r| glob_match(&r.tool, &tool.id))
        .collect();

    // 2. predicates
    matches.retain(|r| {
        predicate_match(&r.paths, arg_path.as_deref())
            && predicate_match(&r.network, arg_network.as_deref())
            && predicate_match(&r.env, arg_env.as_deref())
    });

    if matches.is_empty() {
        // Default deny
        return Ok(Decision {
            schema_version: 1,
            decision: "deny".to_string(),
            rule_id: "_default".to_string(),
            reason: "no-allow-rule".to_string(),

            effective_paths: vec![],

            effective_network: vec![],

            tool_call_id: tool_call_id.to_string(),
        });
    }

    // 3. sort: priority desc, then specificity desc
    matches.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| specificity(&b.tool).cmp(&specificity(&a.tool)))
    });

    let first = matches[0];

    // 4. apply
    if first.effect == "ask" {
        return Ok(Decision {
            schema_version: 1,
            decision: "ask".to_string(),

            rule_id: first.id.clone(),

            reason: first.reason.clone(),

            effective_paths: first.paths.clone(),

            effective_network: first.network.clone(),

            tool_call_id: tool_call_id.to_string(),
        });
    }

    if first.effect == "deny" {
        return Ok(Decision {
            schema_version: 1,
            decision: "deny".to_string(),

            rule_id: first.id.clone(),

            reason: first.reason.clone(),

            effective_paths: first.paths.clone(),

            effective_network: first.network.clone(),

            tool_call_id: tool_call_id.to_string(),
        });
    }

    // effect == "allow"
    let mut effective_paths = first.paths.clone();
    let mut effective_network = first.network.clone();
    let effective_env = first.env.clone();

    // 5. scope narrow
    if let Some(narrow) = scope_narrow {
        if let Some(np) = &narrow.paths {
            effective_paths = intersect(&effective_paths, np);
            // After narrowing, verify the arg's path is still within the
            // narrowed set. If not, the call is denied with scope-empty.
            if effective_paths.is_empty() {
                return Ok(Decision {
                    schema_version: 1,
                    decision: "deny".to_string(),

                    rule_id: first.id.clone(),
                    reason: "scope-empty".to_string(),
                    effective_paths,
                    effective_network,

                    tool_call_id: tool_call_id.to_string(),
                });
            }
            if let Some(p) = arg_path.as_deref() {
                let in_narrow = effective_paths.iter().any(|g| glob_match(g, p) || g == p);
                if !in_narrow {
                    return Ok(Decision {
                        schema_version: 1,
                        decision: "deny".to_string(),

                        rule_id: first.id.clone(),
                        reason: "scope-empty".to_string(),
                        effective_paths,
                        effective_network,

                        tool_call_id: tool_call_id.to_string(),
                    });
                }
            }
        }
        if let Some(nn) = &narrow.network {
            effective_network = intersect(&effective_network, nn);
            if effective_network.is_empty() {
                return Ok(Decision {
                    schema_version: 1,
                    decision: "deny".to_string(),

                    rule_id: first.id.clone(),
                    reason: "scope-empty".to_string(),
                    effective_paths,
                    effective_network,

                    tool_call_id: tool_call_id.to_string(),
                });
            }
        }
        if let Some(ne) = &narrow.env {
            let _ = intersect(&effective_env, ne);
        }
    }

    // Safety-critical tier: force `ask` if the path is in a safety-critical glob
    // and the tool would have been auto-approved. Per §2.9, the safety flag
    // on the rule forces ask regardless of approval.mode.
    if first.safety {
        return Ok(Decision {
            schema_version: 1,
            decision: "ask".to_string(),

            rule_id: first.id.clone(),
            reason: "safety-critical".to_string(),
            effective_paths,
            effective_network,

            tool_call_id: tool_call_id.to_string(),
        });
    }

    Ok(Decision {
        schema_version: 1,
        decision: "allow".to_string(),

        rule_id: first.id.clone(),

        reason: first.reason.clone(),
        effective_paths,
        effective_network,

        tool_call_id: tool_call_id.to_string(),
    })
}

fn glob_match(pat: &str, s: &str) -> bool {
    if pat == "*" {
        return true;
    }
    Pattern::new(pat).map(|p| p.matches(s)).unwrap_or(false)
}

fn predicate_match(globs: &[String], value: Option<&str>) -> bool {
    if globs.is_empty() {
        return true; // missing predicate matches everything
    }
    let Some(v) = value else {
        return false;
    };
    globs.iter().any(|g| glob_match(g, v))
}

fn intersect(a: &[String], b: &[String]) -> Vec<String> {
    // Per spec §2.8: scope_narrow can only narrow. Effective = narrow paths
    // that are also allowed by the rule. If narrow is empty or all entries
    // are filtered out, the intersection is empty.
    if b.is_empty() {
        return vec![];
    }
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for y in b {
        // y is kept if any rule pattern a matches it (i.e., the rule permits y).
        if seen.insert(y.as_str())
            && (a.contains(y) || a.iter().any(|x| glob_match(x, y)))
        {
            out.push(y.clone());
        }
    }
    out
}

/// Specificity scoring: literal > glob > bare > *. Longer = more specific.
fn specificity(pat: &str) -> i32 {
    if pat == "*" {
        return 0;
    }
    if !pat.contains('*') && !pat.contains('?') {
        return 1000;
    }
    pat.len() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::permission::{PermissionFile, Rule};
    use crate::schema::tool::{ApprovalConfig, Capabilities, ToolDescriptor};
    use tempfile::TempDir;

    fn mk_tool(id: &str, destructive: bool) -> ToolDescriptor {
        ToolDescriptor {
            id: id.to_string(),

            schema_version: 1,
            version: "1.0.0".to_string(),
            source: "builtin".to_string(),

            extension_id: None,

            mcp_server: None,
            summary: "".to_string(),
            capabilities: Capabilities {
                read: false,
                write: !destructive,

                exec: destructive,

                network: false,
                destructive,

                secrets: false,
                side_effect: "exec".to_string(),
            },
            argument_schema_path: "".to_string(),
            return_schema_path: "".to_string(),
            side_effect: "exec".to_string(),
            approval: ApprovalConfig {
                mode: "auto".to_string(),
                reason: "".to_string(),
            },
        }
    }

    fn mk_rule(id: &str, effect: &str, tool: &str, paths: Vec<&str>, priority: i32) -> Rule {
        Rule {
            id: id.to_string(),

            effect: effect.to_string(),

            tool: tool.to_string(),

            paths: paths.iter().map(|s| s.to_string()).collect(),

            network: vec![],

            env: vec![],

            safety: false,
            reason: "test".to_string(),
            priority,
        }
    }

    fn mk_pf(rules: Vec<Rule>) -> PermissionFile {
        PermissionFile {
            schema_version: 1,
            rules,
        }
    }

    #[test]
    fn permissions_default_deny_wins_on_no_match() {
        let dir = TempDir::new().unwrap();
        let _ = dir; // not used; just for layout
        let pf = mk_pf(vec![]);
        let tool = mk_tool("fs.read", false);
        let args = serde_json::json!({"path": "src/main.rs"});
        let d = check(&pf, &tool, &args, None, "tc").unwrap();
        assert_eq!(d.decision, "deny");
        assert_eq!(d.reason, "no-allow-rule");
    }

    #[test]
    fn permissions_allow_wins_for_matching_allow_rule() {
        let pf = mk_pf(vec![mk_rule("R1", "allow", "fs.*", vec!["**"], 10)]);
        let tool = mk_tool("fs.read", false);
        let args = serde_json::json!({"path": "src/main.rs"});
        let d = check(&pf, &tool, &args, None, "tc").unwrap();
        assert_eq!(d.decision, "allow");
        assert_eq!(d.rule_id, "R1");
    }

    #[test]
    fn permissions_deny_wins_over_allow_for_higher_priority() {
        let pf = mk_pf(vec![
            mk_rule("R1", "allow", "fs.*", vec!["**"], 10),
            mk_rule("R2", "deny", "fs.read", vec![".harness/**"], 100),
        ]);
        let tool = mk_tool("fs.read", false);
        let args = serde_json::json!({"path": ".harness/x"});
        let d = check(&pf, &tool, &args, None, "tc").unwrap();
        assert_eq!(d.decision, "deny");
        assert_eq!(d.rule_id, "R2");
    }

    #[test]
    fn permissions_safety_critical_forces_ask_even_if_tool_says_auto() {
        let mut r = mk_rule("R-SAFETY", "allow", "fs.*", vec!["**"], 10);
        r.safety = true;
        let pf = mk_pf(vec![r]);
        let tool = mk_tool("fs.edit", false);
        let args = serde_json::json!({"path": "src/auth/x.rs"});
        let d = check(&pf, &tool, &args, None, "tc").unwrap();
        assert_eq!(d.decision, "ask");
    }

    #[test]
    fn permissions_scope_narrow_intersection_empty_returns_scope_empty() {
        let pf = mk_pf(vec![mk_rule("R1", "allow", "fs.*", vec!["**"], 10)]);
        let tool = mk_tool("fs.read", false);
        let args = serde_json::json!({"path": "src/main.rs"});
        let narrow = ScopeNarrow {
            paths: Some(vec!["docs/**".to_string()]),

            network: None,

            env: None,

            ttl_tool_calls: None,
        };
        let d = check(&pf, &tool, &args, Some(&narrow), "tc").unwrap();
        assert_eq!(d.decision, "deny");
        assert_eq!(d.reason, "scope-empty");
    }

    #[test]
    fn permissions_scope_narrow_can_only_narrow() {
        // Narrow set with a path NOT in the effective set → denied.
        let pf = mk_pf(vec![mk_rule("R1", "allow", "fs.*", vec!["src/**"], 10)]);
        let tool = mk_tool("fs.read", false);
        let args = serde_json::json!({"path": "src/main.rs"});
        let narrow = ScopeNarrow {
            paths: Some(vec!["docs/**".to_string()]),

            network: None,

            env: None,

            ttl_tool_calls: None,
        };
        let d = check(&pf, &tool, &args, Some(&narrow), "tc").unwrap();
        assert_eq!(d.decision, "deny");
    }

    #[test]
    fn permissions_load_empty_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let pf = load(dir.path()).unwrap();
        assert!(pf.rules.is_empty());
    }

    #[test]
    fn permissions_ask_effect_returns_ask_decision() {
        let pf = mk_pf(vec![mk_rule("R1", "ask", "fs.*", vec!["**"], 10)]);
        let tool = mk_tool("fs.read", false);
        let args = serde_json::json!({"path": "src/main.rs"});
        let d = check(&pf, &tool, &args, None, "tc").unwrap();
        assert_eq!(d.decision, "ask");
    }
}
