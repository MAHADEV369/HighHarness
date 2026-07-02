//! Filesystem-only retrieval stub, per `HARNESS_PRIMITIVES.md` §5.
//!
//! v1 implementation: ripgrep via shell_exec to gather candidate files,
//! naive length-based chunker, uniform score 1.0. Citation contract:
//! `ref = "file:<path>#lines=<a>-<b>"`.

use std::path::Path;
use std::process::Command;

use serde::Serialize;
use serde_json::{json, Value};

use crate::error::{HxError, HxResult};

#[derive(Debug, Clone, Serialize)]
/// struct `RetrieveOpts` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct RetrieveOpts {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub sources: Vec<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub max_hits: usize,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub min_score: f32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub token_budget: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tool_call_budget: u32,
}

impl Default for RetrieveOpts {
    fn default() -> Self {
        /// Variant `Self` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        Self {
            sources: vec!["filesystem".to_string()],

            max_hits: 10,

            min_score: 0.0,

            token_budget: 12_000,

            tool_call_budget: 12,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
/// struct `Hit` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Hit {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub source: String,
    #[serde(rename = "ref")]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub ref_: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub score: f32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub kind: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub snippet: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tokens: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub taken_at: String,
}

#[derive(Debug, Clone, Serialize)]
/// struct `RetrieveResult` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct RetrieveResult {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub schema_version: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub hits: Vec<Hit>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub spent: Value,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub budget: Value,
}

/// fn `retrieve` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn retrieve(root: &Path, query: &str, opts: RetrieveOpts) -> HxResult<RetrieveResult> {
    if query.is_empty() {
        return Err(HxError::Other("retrieve: empty query".to_string()));
    }
    // 1. Use grep to find candidate lines. macOS BSD grep doesn't support
    // --include; use plain recursive grep and filter to text-friendly
    // extensions in the parser.
    let out = Command::new("grep")
        .args(["-RIn", "-I"])
        .arg(query)
        .arg(".")
        .current_dir(root)
        .output();
    let stdout = match out {
        /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        /// Variant `Err` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
        Err(_) => String::new(),
    };
    let mut hits = Vec::new();
    let mut tokens_spent = 0u32;
    for line in stdout.lines().take(opts.max_hits) {
        // Format: path:lineno:content
        let mut it = line.splitn(3, ':');
        let path = it.next().unwrap_or("").to_string();
        let lineno: u32 = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let content = it.next().unwrap_or("").to_string();
        // Truncate content to a snippet.
        let snippet = if content.len() > 200 {
            format!("{}…", &content[..200])
        } else {
            content.clone()
        };
        let tokens = (snippet.len() / 4) as u32;
        if tokens_spent + tokens > opts.token_budget {
            break;
        }
        tokens_spent += tokens;
        let ref_ = format!("file:{}#lines={}-{}", path, lineno, lineno);
        hits.push(Hit {
            /// Field `source` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            source: "filesystem".to_string(),
            ref_,

            score: 1.0,
            /// Field `kind` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
            kind: "code".to_string(),

            snippet: redact(&snippet),
            tokens,

            taken_at: crate::id::now_iso(),
        });
        if hits.len() >= opts.max_hits {
            break;
        }
    }
    /// Variant `Ok` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    Ok(RetrieveResult {
        schema_version: 1,
        hits,

        spent: json!({"tokens": tokens_spent, "tool_calls": 1, "ms": 0}),

        budget: json!({"tokens": opts.token_budget, "tool_calls": opts.tool_call_budget}),
    })
}

fn redact(s: &str) -> String {
    // very simple: replace obvious key=value patterns
    let mut out = s.to_string();
    for prefix in &["api_key=", "token=", "secret=", "password="] {
        if let Some(idx) = out.find(prefix) {
            // find the end of the value (space or end)
            let start = idx + prefix.len();
            let rest = &out[start..];
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                .unwrap_or(rest.len());
            out.replace_range(start..start + end, "[REDACTED]");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn retrieval_returns_cited_hits_with_file_refs() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn alpha() {}\nfn beta() {}\n").unwrap();
        let r = retrieve(dir.path(), "alpha", RetrieveOpts::default()).unwrap();
        assert!(!r.hits.is_empty());
        for h in &r.hits {
            assert!(h.ref_.starts_with("file:"));
            assert!(h.ref_.contains("#lines="));
        }
    }

    #[test]
    fn retrieval_respects_token_budget() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("a.rs"),
            "alpha alpha alpha\nalpha alpha alpha\n",
        )
        .unwrap();
        let mut opts = RetrieveOpts::default();
        opts.token_budget = 4; // very small
        let r = retrieve(dir.path(), "alpha", opts).unwrap();
        let spent = r.spent.get("tokens").and_then(|x| x.as_u64()).unwrap_or(0);
        assert!(spent <= 4, "spent={} exceeds budget", spent);
    }

    #[test]
    fn retrieval_redacts_secrets_before_returning() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.rs"), "api_key=AKIAIOSFODNN7EXAMPLE\n").unwrap();
        let r = retrieve(dir.path(), "api_key", RetrieveOpts::default()).unwrap();
        let all_text: String = r
            .hits
            .iter()
            .map(|h| h.snippet.clone())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !all_text.contains("AKIAIOSFODNN7EXAMPLE"),
            "secret leaked: {}",
            all_text
        );
        assert!(all_text.contains("[REDACTED]"));
    }
}
