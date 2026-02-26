//! Structured events parsed from Claude CLI stream-json output (one JSON object per line).
//! Agent B maps these to ForgeEvent (ProcessOutput, ProcessCompleted, etc.).

use serde::{Deserialize, Serialize};

/// One stream-json event. Variants match CLI output: system, assistant, user, result, error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamJsonEvent {
    #[serde(rename = "system")]
    System(#[serde(default)] SystemPayload),

    #[serde(rename = "assistant")]
    Assistant(#[serde(default)] AssistantPayload),

    #[serde(rename = "user")]
    User(#[serde(default)] UserPayload),

    #[serde(rename = "result")]
    Result(#[serde(default)] ResultPayload),

    #[serde(rename = "error")]
    Error(#[serde(default)] ErrorPayload),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemPayload {
    pub subtype: Option<String>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub tools: Vec<serde_json::Value>,
    #[serde(default)]
    pub mcp_servers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssistantPayload {
    pub message: Option<MessagePayload>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPayload {
    pub message: Option<MessagePayload>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessagePayload {
    pub id: Option<String>,
    pub role: Option<String>,
    #[serde(default)]
    pub content: Vec<ContentBlock>,
    pub usage: Option<serde_json::Value>,
    pub stop_reason: Option<String>,
}

/// Content block: text, tool_use, or tool_result (for mapping to OutputKind).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text {
        #[serde(default)]
        text: String,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: Option<String>,
        name: Option<String>,
        input: Option<serde_json::Value>,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: Option<String>,
        #[serde(default)]
        content: String,
        is_error: Option<bool>,
    },
    #[serde(rename = "thinking")]
    Thinking {
        #[serde(default)]
        thinking: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResultPayload {
    pub subtype: Option<String>,
    pub result: Option<String>,
    pub cost_usd: Option<f64>,
    pub duration_ms: Option<u64>,
    pub session_id: Option<String>,
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub message: Option<String>,
    pub code: Option<String>,
}
