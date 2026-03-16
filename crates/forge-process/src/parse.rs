//! Parse stream-json lines (one JSON object per line) into StreamJsonEvent.

use crate::stream_event::StreamJsonEvent;
use forge_core::ForgeError;

/// Parse a single line of stream-json output into a StreamJsonEvent.
/// Returns Ok(None) for blank lines; Err for unknown type or invalid JSON.
pub fn parse_line(line: &str) -> Result<Option<StreamJsonEvent>, ForgeError> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_str(line)
        .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?;
    let typ = value
        .get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| ForgeError::Validation("missing type field in stream event".into()))?;
    match typ {
        "system" => Ok(Some(serde_json::from_value(value)
            .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?)),
        "assistant" => Ok(Some(serde_json::from_value(value)
            .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?)),
        "user" => Ok(Some(serde_json::from_value(value)
            .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?)),
        "result" => Ok(Some(serde_json::from_value(value)
            .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?)),
        "error" => Ok(Some(serde_json::from_value(value)
            .map_err(|e| ForgeError::Process(format!("JSON parse error: {e}")))?)),
        other => Err(ForgeError::Validation(format!("unknown event type: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream_event::StreamJsonEvent;
    use forge_core::ForgeError;

    #[test]
    fn parse_empty_line_returns_none() {
        assert!(matches!(parse_line(""), Ok(None)));
        assert!(matches!(parse_line("  \n  "), Ok(None)));
    }

    #[test]
    fn parse_result_line() {
        let line = r#"{"type":"result","subtype":"success","result":"Done.","session_id":"sess_1"}"#;
        let out = parse_line(line).unwrap();
        let ev = out.expect("some event");
        match &ev {
            StreamJsonEvent::Result(p) => {
                assert_eq!(p.subtype.as_deref(), Some("success"));
                assert_eq!(p.result.as_deref(), Some("Done."));
            }
            _ => panic!("expected Result"),
        }
    }

    #[test]
    fn parse_assistant_line() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hello"}]}}"#;
        let out = parse_line(line).unwrap();
        let ev = out.expect("some event");
        match &ev {
            StreamJsonEvent::Assistant(p) => {
                let msg = p.message.as_ref().unwrap();
                assert_eq!(msg.content.len(), 1);
                match &msg.content[0] {
                    crate::stream_event::ContentBlock::Text { text } => assert_eq!(text, "Hello"),
                    _ => panic!("expected text block"),
                }
            }
            _ => panic!("expected Assistant"),
        }
    }

    #[test]
    fn parse_system_line() {
        let line = r#"{"type":"system","subtype":"init","session_id":"sess_abc","model":"claude-sonnet-4"}"#;
        let out = parse_line(line).unwrap();
        let ev = out.expect("some event");
        match &ev {
            StreamJsonEvent::System(p) => {
                assert_eq!(p.session_id.as_deref(), Some("sess_abc"));
                assert_eq!(p.model.as_deref(), Some("claude-sonnet-4"));
            }
            _ => panic!("expected System"),
        }
    }

    #[test]
    fn parse_error_line() {
        let line = r#"{"type":"error","message":"Something failed","code":"ERR"}"#;
        let out = parse_line(line).unwrap();
        let ev = out.expect("some event");
        match &ev {
            StreamJsonEvent::Error(p) => {
                assert_eq!(p.message.as_deref(), Some("Something failed"));
            }
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn parse_unknown_type_fails() {
        let line = r#"{"type":"unknown_kind"}"#;
        let err = parse_line(line).unwrap_err();
        assert!(matches!(err, ForgeError::Validation(_)));
    }

    #[test]
    fn parse_invalid_json_fails() {
        let line = r#"not json"#;
        let err = parse_line(line).unwrap_err();
        assert!(matches!(err, ForgeError::Process(_)));
    }
}
