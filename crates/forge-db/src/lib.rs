//! Forge database layer: pool, migrations, batch writer, and repositories.
#![forbid(unsafe_code)]

pub mod batch_writer;
pub mod migrations;
pub mod pool;
pub mod repos;

pub use batch_writer::BatchWriter;
pub use migrations::Migrator;
pub use pool::DbPool;
pub use repos::agents::AgentRepo;
pub use repos::events::{EventRepo, StoredEvent};
pub use repos::sessions::{NewSession, Session, SessionRepo};
pub use repos::skills::{Skill, SkillRepo};
pub use repos::workflows::{Workflow, WorkflowRepo};

#[cfg(test)]
mod tests {
    use super::*;
    use forge_agent::model::NewAgent;
    use forge_core::events::ForgeEvent;
    use std::sync::Arc;
    use std::time::Duration;

    fn defaults_new_agent() -> NewAgent {
        NewAgent {
            name: "TestAgent".into(),
            model: None,
            system_prompt: None,
            allowed_tools: None,
            max_turns: None,
            use_max: None,
            preset: None,
            config: None,
        }
    }

    fn setup_test_db() -> Arc<std::sync::Mutex<rusqlite::Connection>> {
        let db = DbPool::in_memory().unwrap();
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending().unwrap();
        drop(conn);
        db.conn_arc()
    }

    #[test]
    fn migration_applies_cleanly() {
        let db = DbPool::in_memory().unwrap();
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        assert!(migrator.apply_pending().unwrap() >= 1);
    }

    #[test]
    fn migration_is_idempotent() {
        let db = DbPool::in_memory().unwrap();
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending().unwrap();
        assert_eq!(migrator.apply_pending().unwrap(), 0);
    }

    #[test]
    fn migration_version_tracked() {
        let db = DbPool::in_memory().unwrap();
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending().unwrap();
        assert!(migrator.current_version().unwrap() >= 1);
    }

    #[test]
    fn agent_crud_roundtrip() {
        let conn = setup_test_db();
        let repo = AgentRepo::new(Arc::clone(&conn));
        let input = NewAgent {
            name: "TestAgent".into(),
            ..defaults_new_agent()
        };
        let created = repo.create(&input).unwrap();
        assert_eq!(created.name, "TestAgent");

        let fetched = repo.get(&created.id).unwrap();
        assert_eq!(fetched.name, "TestAgent");

        let agents = repo.list().unwrap();
        assert!(agents.len() >= 1);

        repo.delete(&created.id).unwrap();
        assert!(repo.get(&created.id).is_err());
    }

    #[test]
    fn agent_name_unique() {
        let conn = setup_test_db();
        let repo = AgentRepo::new(Arc::clone(&conn));
        let input = NewAgent {
            name: "Unique".into(),
            ..defaults_new_agent()
        };
        repo.create(&input).unwrap();
        assert!(repo.create(&input).is_err());
    }

    #[test]
    fn session_crud_roundtrip() {
        use crate::{NewSession, SessionRepo};

        let conn = setup_test_db();
        let agent_repo = AgentRepo::new(Arc::clone(&conn));
        let agent = agent_repo.create(&NewAgent {
            name: "SessionTestAgent".into(),
            ..defaults_new_agent()
        }).unwrap();

        let session_repo = SessionRepo::new(Arc::clone(&conn));
        let input = NewSession {
            agent_id: agent.id.clone(),
            directory: "/tmp/forge-test".into(),
            claude_session_id: None,
        };
        let created = session_repo.create(&input).unwrap();
        assert_eq!(created.directory, "/tmp/forge-test");
        assert_eq!(created.agent_id.0, agent.id.0);

        let fetched = session_repo.get(&created.id).unwrap();
        assert_eq!(fetched.id.0, created.id.0);

        let sessions = session_repo.list().unwrap();
        assert!(sessions.len() >= 1);

        session_repo.delete(&created.id).unwrap();
        assert!(session_repo.get(&created.id).is_err());
    }

    #[test]
    fn session_update_status() {
        use crate::{NewSession, SessionRepo};

        let conn = setup_test_db();
        let agent_repo = AgentRepo::new(Arc::clone(&conn));
        let agent = agent_repo.create(&NewAgent {
            name: "StatusTestAgent".into(),
            ..defaults_new_agent()
        }).unwrap();

        let session_repo = SessionRepo::new(Arc::clone(&conn));
        let session = session_repo.create(&NewSession {
            agent_id: agent.id.clone(),
            directory: "/tmp/status-test".into(),
            claude_session_id: None,
        }).unwrap();
        assert_eq!(session.status, "created");

        let updated = session_repo.update_status(&session.id, "running").unwrap();
        assert_eq!(updated.status, "running");
        assert!(updated.updated_at > session.updated_at);

        let updated2 = session_repo.update_claude_session_id(&session.id, "claude-abc-123").unwrap();
        assert_eq!(updated2.claude_session_id.as_deref(), Some("claude-abc-123"));
    }

    #[test]
    fn skill_repo_list_empty_after_migration() {
        use crate::SkillRepo;

        let conn = setup_test_db();
        let repo = SkillRepo::new(Arc::clone(&conn));
        let list = repo.list().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn workflow_repo_list_empty() {
        use crate::WorkflowRepo;

        let conn = setup_test_db();
        let repo = WorkflowRepo::new(Arc::clone(&conn));
        let list = repo.list().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn fts5_tables_exist() {
        let db = DbPool::in_memory().unwrap();
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name LIKE '%_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count >= 3);
    }

    #[test]
    fn batch_writer_flushes_at_50() {
        let conn = setup_test_db();
        let writer = BatchWriter::spawn(Arc::clone(&conn));

        for _ in 0..50 {
            writer
                .write(ForgeEvent::Heartbeat {
                    timestamp: chrono::Utc::now(),
                })
                .unwrap();
        }

        std::thread::sleep(Duration::from_millis(3500));

        let count: i32 = conn
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 50);

        writer.shutdown().unwrap();
    }

    #[test]
    fn batch_writer_flushes_at_2s() {
        let conn = setup_test_db();
        let writer = BatchWriter::spawn(Arc::clone(&conn));

        for _ in 0..5 {
            writer
                .write(ForgeEvent::Heartbeat {
                    timestamp: chrono::Utc::now(),
                })
                .unwrap();
        }

        std::thread::sleep(Duration::from_millis(2500));

        let count: i32 = conn
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 5);

        writer.shutdown().unwrap();
    }

    #[test]
    fn batch_writer_shutdown_flushes() {
        let conn = setup_test_db();
        let writer = BatchWriter::spawn(Arc::clone(&conn));

        for _ in 0..10 {
            writer
                .write(ForgeEvent::Heartbeat {
                    timestamp: chrono::Utc::now(),
                })
                .unwrap();
        }

        writer.shutdown().unwrap();

        let count: i32 = conn
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 10);
    }

    #[test]
    fn event_persisted_with_correct_type() {
        let conn = setup_test_db();
        let writer = BatchWriter::spawn(Arc::clone(&conn));

        writer
            .write(ForgeEvent::Heartbeat {
                timestamp: chrono::Utc::now(),
            })
            .unwrap();

        std::thread::sleep(Duration::from_millis(100));
        writer.shutdown().unwrap();

        let event_type: String = conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT event_type FROM events ORDER BY timestamp DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(event_type, "Heartbeat");
    }
}
