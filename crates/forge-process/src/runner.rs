//! Process runner: maps stream-json events to ForgeEvent and emits to EventBus.

use chrono::Utc;
use forge_core::event_bus::EventBus;
use forge_core::events::{ForgeEvent, OutputKind};
use forge_core::ids::{AgentId, SessionId};
use forge_core::ForgeResult;
use std::sync::Arc;
use tracing::debug;

use crate::stream_event::{ContentBlock, StreamJsonEvent as ParsedEvent};

/// Kind of event from stream-json (or stub). Agent A produces these; we map to ForgeEvent.
#[derive(Debug, Clone)]
pub enum StreamJsonKind {
    Started,
    Output {
        kind: OutputKind,
        content: String,
    },
    Completed {
        exit_code: i32,
    },
    Failed {
        error: String,
    },
}

/// Parsed event from stream-json output. Runner maps each to ForgeEvent and emits.
#[derive(Debug, Clone)]
pub struct StreamJsonEvent {
    pub kind: StreamJsonKind,
}

impl StreamJsonEvent {
    pub fn started() -> Self {
        Self {
            kind: StreamJsonKind::Started,
        }
    }

    pub fn output(kind: OutputKind, content: impl Into<String>) -> Self {
        Self {
            kind: StreamJsonKind::Output {
                kind,
                content: content.into(),
            },
        }
    }

    pub fn completed(exit_code: i32) -> Self {
        Self {
            kind: StreamJsonKind::Completed { exit_code },
        }
    }

    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            kind: StreamJsonKind::Failed {
                error: error.into(),
            },
        }
    }
}

/// Runs a process (or stub) and emits process lifecycle events to the EventBus.
pub struct ProcessRunner {
    event_bus: Arc<EventBus>,
}

impl ProcessRunner {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }

    /// Emit a single ForgeEvent to the bus.
    pub fn emit(&self, event: ForgeEvent) -> ForgeResult<()> {
        self.event_bus.emit(event)
    }

    /// Run a stub sequence: ProcessStarted, two ProcessOutput, ProcessCompleted.
    /// Used when Agent A's spawn/parse is not yet available; tests can assert on received events.
    pub fn emit_stub_run(
        &self,
        session_id: SessionId,
        agent_id: AgentId,
    ) -> ForgeResult<()> {
        let now = Utc::now();
        self.event_bus.emit(ForgeEvent::ProcessStarted {
            session_id: session_id.clone(),
            agent_id: agent_id.clone(),
            timestamp: now,
        })?;
        self.event_bus.emit(ForgeEvent::ProcessOutput {
            session_id: session_id.clone(),
            kind: OutputKind::Assistant,
            content: "Stub output line 1.".into(),
            timestamp: Utc::now(),
        })?;
        self.event_bus.emit(ForgeEvent::ProcessOutput {
            session_id: session_id.clone(),
            kind: OutputKind::Assistant,
            content: "Stub output line 2.".into(),
            timestamp: Utc::now(),
        })?;
        self.event_bus.emit(ForgeEvent::ProcessCompleted {
            session_id,
            exit_code: 0,
            timestamp: Utc::now(),
        })?;
        debug!("stub run emitted ProcessStarted, 2x ProcessOutput, ProcessCompleted");
        Ok(())
    }

    /// Map a stream-json event to ForgeEvent and emit. Used when Agent A's parser produces events.
    pub fn emit_stream_event(
        &self,
        session_id: &SessionId,
        agent_id: &AgentId,
        event: &StreamJsonEvent,
    ) -> ForgeResult<()> {
        let now = Utc::now();
        let forge_event = match &event.kind {
            StreamJsonKind::Started => ForgeEvent::ProcessStarted {
                session_id: session_id.clone(),
                agent_id: agent_id.clone(),
                timestamp: now,
            },
            StreamJsonKind::Output { kind, content } => ForgeEvent::ProcessOutput {
                session_id: session_id.clone(),
                kind: kind.clone(),
                content: content.clone(),
                timestamp: now,
            },
            StreamJsonKind::Completed { exit_code } => ForgeEvent::ProcessCompleted {
                session_id: session_id.clone(),
                exit_code: *exit_code,
                timestamp: now,
            },
            StreamJsonKind::Failed { error } => ForgeEvent::ProcessFailed {
                session_id: session_id.clone(),
                error: error.clone(),
                timestamp: now,
            },
        };
        self.event_bus.emit(forge_event)
    }

    /// Emit a sequence of stream-json events (e.g. from Agent A's parser).
    pub fn emit_from_stream(
        &self,
        session_id: SessionId,
        agent_id: AgentId,
        events: impl IntoIterator<Item = StreamJsonEvent>,
    ) -> ForgeResult<()> {
        for event in events {
            self.emit_stream_event(&session_id, &agent_id, &event)?;
        }
        Ok(())
    }

    /// Map a parsed stream-json event (from Agent A's parser) to ForgeEvent and emit.
    /// Emits one ForgeEvent per content block, preserving ToolUse/ToolResult/Thinking kinds.
    pub fn emit_parsed_event(
        &self,
        session_id: &SessionId,
        _agent_id: &AgentId,
        event: &ParsedEvent,
    ) -> ForgeResult<()> {
        match event {
            ParsedEvent::System(_) => {
                // First system can be treated as process started; we emit only once per run elsewhere
                Ok(())
            }
            ParsedEvent::Assistant(p) => {
                if let Some(ref msg) = p.message {
                    for block in &msg.content {
                        if let Some((kind, content)) = content_block_output(block) {
                            self.event_bus.emit(ForgeEvent::ProcessOutput {
                                session_id: session_id.clone(),
                                kind,
                                content,
                                timestamp: Utc::now(),
                            })?;
                        }
                    }
                }
                Ok(())
            }
            ParsedEvent::User(_) => Ok(()),
            ParsedEvent::Result(_) => self.event_bus.emit(ForgeEvent::ProcessCompleted {
                session_id: session_id.clone(),
                exit_code: 0,
                timestamp: Utc::now(),
            }),
            ParsedEvent::Error(p) => self.event_bus.emit(ForgeEvent::ProcessFailed {
                session_id: session_id.clone(),
                error: p.message.clone().unwrap_or_else(|| "unknown error".into()),
                timestamp: Utc::now(),
            }),
        }
    }
}

/// Map a ContentBlock to (OutputKind, content string). Returns None for empty blocks.
fn content_block_output(block: &ContentBlock) -> Option<(OutputKind, String)> {
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
        ContentBlock::ToolResult { content, is_error, .. } => {
            let prefix = if *is_error == Some(true) { "[error] " } else { "" };
            Some((OutputKind::ToolResult, format!("{}{}", prefix, content)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_core::events::OutputKind;
    use std::sync::Arc;

    #[tokio::test]
    async fn stub_run_emits_process_events_in_order() {
        let bus = Arc::new(EventBus::new(32));
        let mut rx = bus.subscribe();
        let runner = ProcessRunner::new(Arc::clone(&bus));

        let session_id = SessionId::new();
        let agent_id = AgentId::new();

        runner.emit_stub_run(session_id.clone(), agent_id.clone()).unwrap();

        let e1 = rx.recv().await.unwrap();
        assert!(matches!(e1, ForgeEvent::ProcessStarted { .. }));

        let e2 = rx.recv().await.unwrap();
        assert!(matches!(e2, ForgeEvent::ProcessOutput { .. }));

        let e3 = rx.recv().await.unwrap();
        assert!(matches!(e3, ForgeEvent::ProcessOutput { .. }));

        let e4 = rx.recv().await.unwrap();
        assert!(matches!(e4, ForgeEvent::ProcessCompleted { .. }));
    }

    #[tokio::test]
    async fn emit_from_stream_emits_mapped_events() {
        let bus = Arc::new(EventBus::new(32));
        let mut rx = bus.subscribe();
        let runner = ProcessRunner::new(Arc::clone(&bus));

        let session_id = SessionId::new();
        let agent_id = AgentId::new();

        let events = [
            StreamJsonEvent::started(),
            StreamJsonEvent::output(OutputKind::Assistant, "hello"),
            StreamJsonEvent::completed(0),
        ];
        runner
            .emit_from_stream(session_id, agent_id, events)
            .unwrap();

        let e1 = rx.recv().await.unwrap();
        assert!(matches!(e1, ForgeEvent::ProcessStarted { .. }));

        let e2 = rx.recv().await.unwrap();
        match &e2 {
            ForgeEvent::ProcessOutput { content, .. } => assert_eq!(content, "hello"),
            _ => panic!("expected ProcessOutput"),
        }

        let e3 = rx.recv().await.unwrap();
        assert!(matches!(e3, ForgeEvent::ProcessCompleted { .. }));
    }

    #[tokio::test]
    async fn emit_stream_event_failed_emits_process_failed() {
        let bus = Arc::new(EventBus::new(32));
        let mut rx = bus.subscribe();
        let runner = ProcessRunner::new(Arc::clone(&bus));

        let session_id = SessionId::new();
        let agent_id = AgentId::new();

        runner
            .emit_stream_event(
                &session_id,
                &agent_id,
                &StreamJsonEvent::failed("something broke"),
            )
            .unwrap();

        let e = rx.recv().await.unwrap();
        match &e {
            ForgeEvent::ProcessFailed { error, .. } => assert_eq!(error, "something broke"),
            _ => panic!("expected ProcessFailed"),
        }
    }
}
