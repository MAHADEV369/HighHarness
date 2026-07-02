//! Secret redaction vault per HARNESS_SECURITY.md §5.

use crate::error::{HxError, HxResult};
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
/// struct `PatternSpec` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct PatternSpec {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub regex_str: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub severity: String,
    #[serde(skip)]
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub compiled: Option<Regex>,
}

#[derive(Debug, Clone, Serialize)]
/// struct `TextRedaction` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct TextRedaction {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub range: [usize; 2],
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub reason: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub replacement: String,
}

/// struct `Redactions` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub struct Redactions {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub patterns: Vec<PatternSpec>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
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

    /// Look up the original secret for a redaction token.
    pub fn lookup(&self, id: &str) -> Option<String> {
        let map = self.in_memory_map.lock().unwrap();
        for (token, secret) in map.iter() {
            if token == id {
                return Some(secret.clone());
            }
        }
        None
    }

    /// Wipe the in-memory redaction map.
    pub fn wipe(&self) {
        let mut map = self.in_memory_map.lock().unwrap();
        map.clear();
    }
}
