/// Secret redaction vault per HARNESS_SECURITY.md §5.
///
/// Scans content for secret patterns (AWS keys, PEM blocks, GitHub PATs,
/// JWTs, GCP keys) and replaces matches with `<REDACTED:id>` tokens.
/// The original values are held in-memory only (never persisted) and wiped
/// on `wipe()` or process exit.
use crate::error::HxResult;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

/// A redaction pattern loaded from `.harness/redactions.toml`.
#[derive(Debug, Clone, Serialize)]
pub struct PatternSpec {
    /// Unique pattern identifier (e.g. "aws-access-key", "pem-block").
    pub id: String,
    /// Regex pattern string used to detect secrets.
    pub regex_str: String,
    /// Severity level: "low", "medium", "high", or "critical".
    pub severity: String,
    #[serde(skip)]
    /// Compiled regex (None if pattern failed to compile).
    pub compiled: Option<Regex>,
}

/// A single redaction applied to content: the byte range, reason, and replacement token.
#[derive(Debug, Clone, Serialize)]
pub struct TextRedaction {
    /// Byte range `[start, end)` in the original content.
    pub range: [usize; 2],
    /// Pattern id that matched (e.g. "aws-access-key").
    pub reason: String,
    /// Replacement token inserted into content (e.g. "<REDACTED:aws-access-key>").
    pub replacement: String,
}

/// The redaction engine: holds patterns and an in-memory map of
/// (replacement_token → original_secret) for the current process.
pub struct Redactions {
    /// Compiled redaction patterns loaded from config.
    pub patterns: Vec<PatternSpec>,
    /// In-memory map of (replacement_token → original_value). Process-local only.
    pub in_memory_map: Mutex<Vec<(String, String)>>,
}

impl Redactions {
    /// Load redactions from `.harness/redactions.toml`.
    pub fn load(root: &Path) -> HxResult<Self> {
        let path = root.join(".harness").join("redactions.toml");
        if !path.exists() {
            return Ok(Redactions {
                patterns: Vec::new(),
                in_memory_map: Mutex::new(Vec::new()),
            });
        }
        let raw = fs::read_to_string(&path)?;
        let toml_value: toml::Value = toml::from_str(&raw)?;
        let mut patterns = Vec::new();
        if let Some(arr) = toml_value.get("patterns").and_then(|v| v.as_array()) {
            for item in arr {
                let id = item
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let regex_str = item
                    .get("regex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let severity = item
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("medium")
                    .to_string();
                let compiled = Regex::new(&regex_str).ok();
                patterns.push(PatternSpec {
                    id,
                    regex_str,
                    severity,
                    compiled,
                });
            }
        }
        Ok(Redactions {
            patterns,
            in_memory_map: Mutex::new(Vec::new()),
        })
    }

    /// Scan content for matching patterns, returning a list of redactions.
    pub fn scan(&self, content: &str) -> Vec<TextRedaction> {
        let mut results = Vec::new();
        for p in &self.patterns {
            if let Some(ref re) = p.compiled {
                for m in re.find_iter(content) {
                    results.push(TextRedaction {
                        range: [m.start(), m.end()],
                        reason: p.id.clone(),
                        replacement: format!("<REDACTED:{}>", p.id),
                    });
                }
            }
        }
        results
    }

    /// Apply redactions to content in-place, replacing matches with tokens.
    /// Returns the list of redactions applied.
    pub fn apply(&self, content: &mut String) -> Vec<TextRedaction> {
        let redactions = self.scan(content);
        if redactions.is_empty() {
            return redactions;
        }
        let mut map = self.in_memory_map.lock().unwrap();
        for r in redactions.iter().rev() {
            let replacement = r.replacement.clone();
            map.push((
                replacement.clone(),
                content[r.range[0]..r.range[1]].to_string(),
            ));
            content.replace_range(r.range[0]..r.range[1], &replacement);
        }
        redactions
    }

    /// Look up the original secret value for a replacement token.
    ///
    /// `replacement_token` is the `<REDACTED:id>` string inserted by `apply()`.
    /// Returns the original secret text if found in the in-memory map.
    pub fn lookup(&self, replacement_token: &str) -> Option<String> {
        let map = self.in_memory_map.lock().unwrap();
        for (token, secret) in map.iter() {
            if token == replacement_token {
                return Some(secret.clone());
            }
        }
        None
    }

    /// Wipe the in-memory redaction map. Called on process exit or explicitly.
    pub fn wipe(&self) {
        let mut map = self.in_memory_map.lock().unwrap();
        map.clear();
    }
}
