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

/// Options controlling retrieval behavior.
#[derive(Debug, Clone, Serialize)]
pub struct RetrieveOpts {
    /// Sources to search (e.g., "filesystem").
    pub sources: Vec<String>,
    /// Maximum number of hits to return.
    pub max_hits: usize,
    /// Minimum score threshold for inclusion.
    pub min_score: f32,
    /// Maximum tokens to spend on retrieval.
    pub token_budget: u32,
    /// Maximum tool calls allowed for retrieval.
    pub tool_call_budget: u32,
}

impl Default for RetrieveOpts {
    fn default() -> Self {
        Self {
            sources: vec!["filesystem".to_string()],

            max_hits: 10,

            min_score: 0.0,

            token_budget: 12_000,

            tool_call_budget: 12,
        }
    }
}

/// A single retrieval hit with a cited reference.
#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    /// Source type that produced this hit.
    pub source: String,
    /// Citation reference (e.g., "file:path#lines=1-5").
    #[serde(rename = "ref")]
    pub ref_: String,
    /// Relevance score for this hit.
    pub score: f32,
    /// Content type of the hit (e.g., "code").
    pub kind: String,
    /// Snippet of matched content.
    pub snippet: String,
    /// Estimated token count of the snippet.
    pub tokens: u32,
    /// ISO-8601 timestamp when the hit was captured.
    pub taken_at: String,
}

/// Result of a retrieval query.
#[derive(Debug, Clone, Serialize)]
pub struct RetrieveResult {
    /// Schema version for this result.
    pub schema_version: u32,
    /// List of retrieved hits.
    pub hits: Vec<Hit>,
    /// Resources spent during retrieval (tokens, tool calls, ms).
    pub spent: Value,
    /// Budget limits applied to this retrieval.
    pub budget: Value,
}

/// Retrieve relevant file snippets matching a query string.
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
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
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
            source: "filesystem".to_string(),
            ref_,

            score: 1.0,
            kind: "code".to_string(),

            snippet: redact(&snippet),
            tokens,

            taken_at: crate::id::now_iso(),
        });
        if hits.len() >= opts.max_hits {
            break;
        }
    }
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
        let opts = RetrieveOpts {
            token_budget: 4,
            ..Default::default()
        };
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
