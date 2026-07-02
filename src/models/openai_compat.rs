//! OpenAI-chat-completions-compatible provider adapter per HARNESS_PRIMITIVES.md §6.

use crate::error::HxResult;
use crate::redaction::Redactions;
use serde::Serialize;
use std::path::Path;

/// A chat completion request in OpenAI-compatible format.
#[derive(Debug, Serialize)]
pub struct CompleteRequest {
    /// Model ID to use for completion.
    pub model_id: String,
    /// Conversation messages.
    pub messages: Vec<Message>,
    /// Optional tool definitions.
    pub tools: Option<Vec<serde_json::Value>>,
    /// Optional system prompt.
    pub system: Option<String>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Reasoning effort level ("low", "medium", "high").
    pub reasoning_effort: Option<String>,
    /// Optional prefill text to continue from.
    pub prefill: Option<String>,
    /// Whether to stream the response.
    pub stream: bool,
}

/// A single message in a conversation.
#[derive(Debug, Serialize)]
pub struct Message {
    /// Message role ("system", "user", "assistant").
    pub role: String,
    /// Message content text.
    pub content: String,
}

/// An event emitted during model response streaming.
#[derive(Debug, Serialize)]
pub struct ModelEvent {
    /// Event kind ("text", "tool_call", "usage", "error", etc.).
    pub kind: String,
    /// Incremental text delta for streaming.
    pub delta: Option<String>,
    /// Tool call data if the model requests a tool invocation.
    pub tool_call: Option<serde_json::Value>,
    /// Token usage information.
    pub usage: Option<Usage>,
    /// Cost information in USD.
    pub cost: Option<Cost>,
    /// Reason the model finished generating.
    pub finish_reason: Option<String>,
    /// Error message if the event represents an error.
    pub error: Option<String>,
}

/// Token usage statistics for a completion.
#[derive(Debug, Serialize)]
pub struct Usage {
    /// Number of input tokens consumed.
    pub input_tokens: u32,
    /// Number of output tokens generated.
    pub output_tokens: u32,
    /// Number of reasoning tokens used (chain-of-thought).
    pub reasoning_tokens: Option<u32>,
}

/// Cost information for a completion.
#[derive(Debug, Serialize)]
pub struct Cost {
    /// Cost in US dollars.
    pub usd: f64,
}

/// Send a completion request to the model provider (stub implementation).
pub fn complete(
    _req: &CompleteRequest,
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
