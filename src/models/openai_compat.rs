//! OpenAI-chat-completions-compatible provider adapter per HARNESS_PRIMITIVES.md §6.

use crate::error::{HxError, HxResult};
use crate::redaction::Redactions;
use serde::Serialize;
use std::path::Path;

/// struct `CompleteRequest` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
#[derive(Debug, Serialize)]
pub struct CompleteRequest {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub model_id: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub messages: Vec<Message>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tools: Option<Vec<serde_json::Value>>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub system: Option<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub max_tokens: Option<u32>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub temperature: Option<f32>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub reasoning_effort: Option<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub prefill: Option<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub stream: bool,
}

/// struct `Message` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
#[derive(Debug, Serialize)]
pub struct Message {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub role: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub content: String,
}

/// struct `ModelEvent` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
#[derive(Debug, Serialize)]
pub struct ModelEvent {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub kind: String,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub delta: Option<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub tool_call: Option<serde_json::Value>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub usage: Option<Usage>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub cost: Option<Cost>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub finish_reason: Option<String>,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub error: Option<String>,
}

/// struct `Usage` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
#[derive(Debug, Serialize)]
pub struct Usage {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub input_tokens: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub output_tokens: u32,
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub reasoning_tokens: Option<u32>,
}

/// struct `Cost` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
#[derive(Debug, Serialize)]
pub struct Cost {
    /// item `?` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
    pub usd: f64,
}

/// fn `complete` — Implements HARNESS_PRIMITIVES.md / HARNESS_ENGINEERING.md.
pub fn complete(
    req: &CompleteRequest,
    redactions: &Redactions,
    root: &Path,
) -> HxResult<Vec<ModelEvent>> {
    let _ = redactions;
    let _ = root;

    // For now, this is a stub that returns a single error event
    // Real implementation would make HTTP calls to the provider
    // See BUILD_PHASE_2_5.md W4 for the full design

    Ok(vec![ModelEvent {
        kind: "error".to_string(),
        delta: None,
        tool_call: None,
        usage: None,
        cost: None,
        finish_reason: None,
        error: Some("provider adapter not yet connected — this is a stub for Phase 2.5. Wire a real model endpoint to use this.".to_string()),
    }])
}
