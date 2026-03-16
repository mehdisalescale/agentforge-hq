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

/// Normalize any backend's raw stream-json events into ForgeEvent variants.
///
/// Each backend may produce different raw event formats, but they all get
/// normalized here before emission to EventBus. This ensures the UI,
/// BatchWriter, and analytics work identically regardless of backend.
///
/// Returns `None` for events that don't map to a ForgeEvent (e.g. System, User).
pub fn normalize_to_forge_event(
    _backend: &str,
    raw: &StreamJsonEvent,
    session_id: &forge_core::ids::SessionId,
    _agent_id: &forge_core::ids::AgentId,
) -> Vec<forge_core::events::ForgeEvent> {
    use chrono::Utc;
    use forge_core::events::ForgeEvent;

    match raw {
        StreamJsonEvent::System(_) => {
            // System events are handled by PersistMiddleware (ProcessStarted)
            vec![]
        }
        StreamJsonEvent::Assistant(p) => {
            let mut events = Vec::new();
            if let Some(ref msg) = p.message {
                for block in &msg.content {
                    if let Some((kind, content)) = content_block_to_output(block) {
                        events.push(ForgeEvent::ProcessOutput {
                            session_id: session_id.clone(),
                            kind,
                            content,
                            timestamp: Utc::now(),
                        });
                    }
                }
            }
            events
        }
        StreamJsonEvent::User(_) => vec![],
        StreamJsonEvent::Result(_) => {
            vec![ForgeEvent::ProcessCompleted {
                session_id: session_id.clone(),
                exit_code: 0,
                timestamp: Utc::now(),
            }]
        }
        StreamJsonEvent::Error(p) => {
            vec![ForgeEvent::ProcessFailed {
                session_id: session_id.clone(),
                error: p
                    .message
                    .clone()
                    .unwrap_or_else(|| "unknown error".into()),
                timestamp: Utc::now(),
            }]
        }
    }
}

/// Map a ContentBlock to (OutputKind, content string). Returns None for empty blocks.
fn content_block_to_output(
    block: &ContentBlock,
) -> Option<(forge_core::events::OutputKind, String)> {
    use forge_core::events::OutputKind;

    match block {
        ContentBlock::Text { text } if !text.is_empty() => {
            Some((OutputKind::Assistant, text.clone()))
        }
        ContentBlock::Thinking { thinking } if !thinking.is_empty() => {
            Some((OutputKind::Thinking, thinking.clone()))
        }
        ContentBlock::ToolUse { name, input, .. } => {
            let tool_name = name.as_deref().unwrap_or("unknown");
            let input_str = input
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap_or_default())
                .unwrap_or_default();
            Some((OutputKind::ToolUse, format!("{}({})", tool_name, input_str)))
        }
        ContentBlock::ToolResult {
            content, is_error, ..
        } => {
            let prefix = if *is_error == Some(true) {
                "[error] "
            } else {
                ""
            };
            Some((
                OutputKind::ToolResult,
                format!("{}{}", prefix, content),
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_core::ids::{AgentId, SessionId};

    #[test]
    fn normalize_assistant_text_produces_process_output() {
        let event = StreamJsonEvent::Assistant(AssistantPayload {
            message: Some(MessagePayload {
                id: None,
                role: Some("assistant".into()),
                content: vec![ContentBlock::Text {
                    text: "hello world".into(),
                }],
                usage: None,
                stop_reason: None,
            }),
        });
        let sid = SessionId::new();
        let aid = AgentId::new();
        let results = normalize_to_forge_event("claude", &event, &sid, &aid);
        assert_eq!(results.len(), 1);
        match &results[0] {
            forge_core::events::ForgeEvent::ProcessOutput { content, .. } => {
                assert_eq!(content, "hello world");
            }
            _ => panic!("expected ProcessOutput"),
        }
    }

    #[test]
    fn normalize_result_produces_process_completed() {
        let event = StreamJsonEvent::Result(ResultPayload {
            subtype: None,
            result: Some("done".into()),
            cost_usd: Some(0.01),
            duration_ms: Some(100),
            session_id: None,
            usage: None,
        });
        let sid = SessionId::new();
        let aid = AgentId::new();
        let results = normalize_to_forge_event("claude", &event, &sid, &aid);
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0],
            forge_core::events::ForgeEvent::ProcessCompleted { .. }
        ));
    }

    #[test]
    fn normalize_error_produces_process_failed() {
        let event = StreamJsonEvent::Error(ErrorPayload {
            message: Some("something broke".into()),
            code: None,
        });
        let sid = SessionId::new();
        let aid = AgentId::new();
        let results = normalize_to_forge_event("claude", &event, &sid, &aid);
        assert_eq!(results.len(), 1);
        match &results[0] {
            forge_core::events::ForgeEvent::ProcessFailed { error, .. } => {
                assert_eq!(error, "something broke");
            }
            _ => panic!("expected ProcessFailed"),
        }
    }

    #[test]
    fn normalize_system_and_user_produce_nothing() {
        let sid = SessionId::new();
        let aid = AgentId::new();

        let system = StreamJsonEvent::System(SystemPayload::default());
        assert!(normalize_to_forge_event("claude", &system, &sid, &aid).is_empty());

        let user = StreamJsonEvent::User(UserPayload::default());
        assert!(normalize_to_forge_event("claude", &user, &sid, &aid).is_empty());
    }

    #[test]
    fn normalize_multiple_content_blocks() {
        let event = StreamJsonEvent::Assistant(AssistantPayload {
            message: Some(MessagePayload {
                id: None,
                role: Some("assistant".into()),
                content: vec![
                    ContentBlock::Text {
                        text: "first".into(),
                    },
                    ContentBlock::ToolUse {
                        id: Some("t1".into()),
                        name: Some("Read".into()),
                        input: None,
                    },
                    ContentBlock::Text {
                        text: String::new(),
                    }, // empty — skipped
                ],
                usage: None,
                stop_reason: None,
            }),
        });
        let sid = SessionId::new();
        let aid = AgentId::new();
        let results = normalize_to_forge_event("claude", &event, &sid, &aid);
        assert_eq!(results.len(), 2); // text + tool_use, empty text skipped
    }
}
