//! Forge core types: IDs, errors, events, and event bus.
//!
//! This crate is the contract for all other crates. Do not duplicate these types elsewhere.

pub mod error;
pub mod event_bus;
pub mod events;
pub mod ids;

pub use error::{ForgeError, ForgeResult};
pub use event_bus::{EventBus, EventSink};
pub use events::{ForgeEvent, OutputKind};
pub use ids::{AgentId, EventId, SessionId, SkillId, WorkflowId};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn emit_event_received_by_subscriber() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        let event = ForgeEvent::Heartbeat {
            timestamp: Utc::now(),
        };
        bus.emit(event.clone()).unwrap();
        let received = rx.recv().await.unwrap();
        match (&received, &event) {
            (ForgeEvent::Heartbeat { timestamp: t1 }, ForgeEvent::Heartbeat { timestamp: t2 }) => {
                assert_eq!(t1, t2);
            }
            _ => panic!("event variant mismatch"),
        }
    }

    #[tokio::test]
    async fn multiple_subscribers_all_receive() {
        let bus = EventBus::new(16);
        let mut r1 = bus.subscribe();
        let mut r2 = bus.subscribe();
        let mut r3 = bus.subscribe();
        let event = ForgeEvent::AgentCreated {
            agent_id: AgentId::new(),
            name: "Test".into(),
            timestamp: Utc::now(),
        };
        bus.emit(event.clone()).unwrap();
        let _ = r1.recv().await.unwrap();
        let _ = r2.recv().await.unwrap();
        let _ = r3.recv().await.unwrap();
    }

    #[test]
    fn event_serializes_to_json_roundtrip() {
        let event = ForgeEvent::AgentCreated {
            agent_id: AgentId::new(),
            name: "My Agent".to_string(),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: ForgeEvent = serde_json::from_str(&json).unwrap();
        match (&event, &back) {
            (
                ForgeEvent::AgentCreated { name: n1, .. },
                ForgeEvent::AgentCreated { name: n2, .. },
            ) => assert_eq!(n1, n2),
            _ => panic!("variant mismatch"),
        }
    }

    #[test]
    fn all_event_variants_serialize() {
        let agent_id = AgentId::new();
        let session_id = SessionId::new();
        let workflow_id = WorkflowId::new();
        let ts = Utc::now();
        let variants: Vec<ForgeEvent> = vec![
            ForgeEvent::SystemStarted {
                version: "0.1.0".into(),
                timestamp: ts,
            },
            ForgeEvent::SystemStopped { timestamp: ts },
            ForgeEvent::Heartbeat { timestamp: ts },
            ForgeEvent::AgentCreated {
                agent_id: agent_id.clone(),
                name: "A".into(),
                timestamp: ts,
            },
            ForgeEvent::AgentUpdated {
                agent_id: agent_id.clone(),
                name: "B".into(),
                timestamp: ts,
            },
            ForgeEvent::AgentDeleted {
                agent_id: agent_id.clone(),
                timestamp: ts,
            },
            ForgeEvent::ProcessStarted {
                session_id: session_id.clone(),
                agent_id: agent_id.clone(),
                timestamp: ts,
            },
            ForgeEvent::ProcessOutput {
                session_id: session_id.clone(),
                kind: OutputKind::Assistant,
                content: "x".into(),
                timestamp: ts,
            },
            ForgeEvent::ProcessCompleted {
                session_id: session_id.clone(),
                exit_code: 0,
                timestamp: ts,
            },
            ForgeEvent::ProcessFailed {
                session_id: session_id.clone(),
                error: "e".into(),
                timestamp: ts,
            },
            ForgeEvent::SessionCreated {
                session_id: session_id.clone(),
                agent_id: agent_id.clone(),
                directory: "/tmp".into(),
                timestamp: ts,
            },
            ForgeEvent::SessionResumed {
                session_id: session_id.clone(),
                timestamp: ts,
            },
            ForgeEvent::WorkflowStarted {
                workflow_id: workflow_id.clone(),
                timestamp: ts,
            },
            ForgeEvent::WorkflowStepCompleted {
                workflow_id: workflow_id.clone(),
                step: 1,
                timestamp: ts,
            },
            ForgeEvent::WorkflowCompleted {
                workflow_id: workflow_id.clone(),
                timestamp: ts,
            },
            ForgeEvent::WorkflowFailed {
                workflow_id: workflow_id.clone(),
                error: "err".into(),
                timestamp: ts,
            },
            ForgeEvent::CircuitBreakerTripped {
                agent_id: agent_id.clone(),
                reason: "r".into(),
                timestamp: ts,
            },
            ForgeEvent::BudgetWarning {
                current_cost: 1.0,
                limit: 10.0,
                timestamp: ts,
            },
            ForgeEvent::BudgetExceeded {
                current_cost: 11.0,
                limit: 10.0,
                timestamp: ts,
            },
            ForgeEvent::HookStarted {
                hook_id: "h1".into(),
                hook_name: "lint".into(),
                event_type: "session.complete".into(),
                timestamp: ts,
            },
            ForgeEvent::HookCompleted {
                hook_id: "h1".into(),
                hook_name: "lint".into(),
                duration_ms: 42,
                timestamp: ts,
            },
            ForgeEvent::HookFailed {
                hook_id: "h1".into(),
                hook_name: "lint".into(),
                error: "exit 1".into(),
                timestamp: ts,
            },
            ForgeEvent::SubAgentRequested {
                parent_session_id: session_id.clone(),
                sub_agent_id: agent_id.clone(),
                prompt: "do stuff".into(),
                timestamp: ts,
            },
            ForgeEvent::SubAgentStarted {
                parent_session_id: session_id.clone(),
                sub_agent_id: agent_id.clone(),
                session_id: SessionId::new(),
                timestamp: ts,
            },
            ForgeEvent::SubAgentCompleted {
                parent_session_id: session_id.clone(),
                sub_agent_id: agent_id.clone(),
                session_id: SessionId::new(),
                timestamp: ts,
            },
            ForgeEvent::SubAgentFailed {
                parent_session_id: session_id.clone(),
                sub_agent_id: agent_id.clone(),
                error: "oops".into(),
                timestamp: ts,
            },
            ForgeEvent::Error {
                message: "m".into(),
                context: None,
                timestamp: ts,
            },
        ];
        for v in variants {
            let _ = serde_json::to_string(&v).expect("serialize");
        }
    }

    #[test]
    fn serde_tag_format_correct() {
        let event = ForgeEvent::AgentCreated {
            agent_id: AgentId::new(),
            name: "My Agent".into(),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val.get("type").and_then(|v| v.as_str()), Some("AgentCreated"));
        assert!(val.get("data").is_some());
    }

    #[test]
    fn subscriber_count_tracks_active_subscribers() {
        let bus = EventBus::new(16);
        assert_eq!(bus.subscriber_count(), 0);
        let _r1 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);
        let _r2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);
        drop(_r1);
        assert_eq!(bus.subscriber_count(), 1);
        drop(_r2);
        assert_eq!(bus.subscriber_count(), 0);
    }
}
