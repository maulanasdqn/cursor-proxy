use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct AnthropicRequest {
    pub model: Option<String>,
    pub messages: Vec<Value>,
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stream: bool,
    pub system: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: &'static str,
    pub role: &'static str,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: &'static str,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Serialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub type_field: &'static str,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct MessageStartEvent {
    #[serde(rename = "type")]
    pub type_field: &'static str,
    pub message: MessageStartPayload,
}

#[derive(Debug, Serialize)]
pub struct MessageStartPayload {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: &'static str,
    pub role: &'static str,
    pub content: Vec<()>,
    pub model: String,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Serialize)]
pub struct ContentBlockStart {
    #[serde(rename = "type")]
    pub type_field: &'static str,
    pub index: u32,
    pub content_block: ContentBlock,
}

#[derive(Debug, Serialize)]
pub struct ContentBlockDelta {
    #[serde(rename = "type")]
    pub type_field: &'static str,
    pub index: u32,
    pub delta: TextDelta,
}

#[derive(Debug, Serialize)]
pub struct TextDelta {
    #[serde(rename = "type")]
    pub type_field: &'static str,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct MessageDelta {
    #[serde(rename = "type")]
    pub type_field: &'static str,
    pub delta: MessageDeltaPayload,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Serialize)]
pub struct MessageDeltaPayload {
    pub stop_reason: &'static str,
}
