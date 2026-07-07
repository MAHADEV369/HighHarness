use crate::error::{HxError, HxResult};
use crate::redaction::Redactions;
use serde::{Deserialize, Serialize};
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
    /// Reasoning effort level.
    pub reasoning_effort: Option<String>,
    /// Optional prefill text.
    pub prefill: Option<String>,
    /// Whether to stream the response.
    pub stream: bool,
}

/// A single message in a conversation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    /// Message role ("system", "user", "assistant").
    pub role: String,
    /// Message content text.
    pub content: String,
}

/// An event emitted during model response processing.
#[derive(Debug, Serialize)]
pub struct ModelEvent {
    /// Event kind ("text", "tool_call", "usage", "error").
    pub kind: String,
    /// Incremental text delta.
    pub delta: Option<String>,
    /// Tool call data.
    pub tool_call: Option<serde_json::Value>,
    /// Token usage information.
    pub usage: Option<Usage>,
    /// Cost information in USD.
    pub cost: Option<Cost>,
    /// Reason the model finished generating.
    pub finish_reason: Option<String>,
    /// Error message if this event represents an error.
    pub error: Option<String>,
}

/// Token usage statistics for a completion.
#[derive(Debug, Serialize)]
pub struct Usage {
    /// Number of input tokens consumed.
    pub input_tokens: u32,
    /// Number of output tokens generated.
    pub output_tokens: u32,
    /// Number of reasoning tokens used.
    pub reasoning_tokens: Option<u32>,
}

/// Cost information for a completion.
#[derive(Debug, Serialize)]
pub struct Cost {
    /// Cost in US dollars.
    pub usd: f64,
}

/// Send a completion request to the model provider.
///
/// Uses `OPENAI_API_KEY` env var for authentication and `MODEL_BASE_URL`
/// for the API endpoint (defaults to `https://api.openai.com/v1`).
/// Non-streaming only. Returns a list of events: text deltas, tool calls,
/// and usage information.
pub fn complete(
    req: &CompleteRequest,
    _redactions: &Redactions,
    _root: &Path,
) -> HxResult<Vec<ModelEvent>> {
    let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
        HxError::Other(
            "OPENAI_API_KEY not set. Set this env var to use model inference.".to_string(),
        )
    })?;

    let base_url =
        std::env::var("MODEL_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let mut messages: Vec<serde_json::Value> = req
        .messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content
            })
        })
        .collect();

    if let Some(ref system) = req.system {
        messages.insert(
            0,
            serde_json::json!({
                "role": "system",
                "content": system
            }),
        );
    }

    let mut body = serde_json::json!({
        "model": req.model_id,
        "messages": messages,
        "stream": false,
    });

    if let Some(max_tokens) = req.max_tokens {
        body["max_tokens"] = serde_json::json!(max_tokens);
    }
    if let Some(temp) = req.temperature {
        body["temperature"] = serde_json::json!(temp);
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| HxError::Other(format!("Failed to create HTTP client: {}", e)))?;

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| HxError::Other(format!("HTTP request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Ok(vec![ModelEvent {
            kind: "error".to_string(),
            delta: None,
            tool_call: None,
            usage: None,
            cost: None,
            finish_reason: None,
            error: Some(format!("API error {}: {}", status, text)),
        }]);
    }

    let data: serde_json::Value = resp
        .json()
        .map_err(|e| HxError::Other(format!("Failed to parse API response: {}", e)))?;

    let mut events = Vec::new();

    if let Some(choices) = data["choices"].as_array() {
        for choice in choices {
            if let Some(content) = choice["message"]["content"].as_str() {
                if !content.is_empty() {
                    events.push(ModelEvent {
                        kind: "text".to_string(),
                        delta: Some(content.to_string()),
                        tool_call: None,
                        usage: None,
                        cost: None,
                        finish_reason: choice["finish_reason"].as_str().map(|s| s.to_string()),
                        error: None,
                    });
                }
            }
            if let Some(tool_calls) = choice["message"]["tool_calls"].as_array() {
                for tc in tool_calls {
                    events.push(ModelEvent {
                        kind: "tool_call".to_string(),
                        delta: None,
                        tool_call: Some(tc.clone()),
                        usage: None,
                        cost: None,
                        finish_reason: None,
                        error: None,
                    });
                }
            }
        }
    }

    if let Some(usage) = data["usage"].as_object() {
        let input = usage
            .get("prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let output = usage
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        events.push(ModelEvent {
            kind: "usage".to_string(),
            delta: None,
            tool_call: None,
            usage: Some(Usage {
                input_tokens: input,
                output_tokens: output,
                reasoning_tokens: None,
            }),
            cost: Some(Cost { usd: 0.0 }),
            finish_reason: None,
            error: None,
        });
    }

    if events.is_empty() {
        events.push(ModelEvent {
            kind: "error".to_string(),
            delta: None,
            tool_call: None,
            usage: None,
            cost: None,
            finish_reason: None,
            error: Some("Empty response from API".to_string()),
        });
    }

    Ok(events)
}
