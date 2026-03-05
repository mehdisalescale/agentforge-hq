//! Schedule CRUD repository with cron expression parsing.

use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub name: String,
    pub cron_expr: String,
    pub agent_id: String,
    pub prompt: String,
    pub directory: String,
    pub enabled: bool,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub run_count: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewSchedule {
    pub name: String,
    pub cron_expr: String,
    pub agent_id: String,
    pub prompt: String,
    #[serde(default = "default_directory")]
    pub directory: String,
}

fn default_directory() -> String {
    ".".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateSchedule {
    pub name: Option<String>,
    pub cron_expr: Option<String>,
    pub prompt: Option<String>,
    pub directory: Option<String>,
    pub enabled: Option<bool>,
}

pub struct ScheduleRepo {
    conn: Arc<Mutex<Connection>>,
}

impl ScheduleRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewSchedule) -> ForgeResult<Schedule> {
        if input.name.trim().is_empty() {
            return Err(ForgeError::Validation("schedule name cannot be empty".into()));
        }
        if input.cron_expr.trim().is_empty() {
            return Err(ForgeError::Validation("cron expression cannot be empty".into()));
        }
        validate_cron(&input.cron_expr)?;

        let next = next_occurrence(&input.cron_expr)?;
        let next_str = next.format("%Y-%m-%dT%H:%M:%S").to_string();

        let conn = self.conn.lock().expect("db mutex poisoned");
        let id = uuid::Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO schedules (id, name, cron_expr, job_type, job_config_json, agent_id, prompt, directory, enabled, next_run_at)
             VALUES (?1, ?2, ?3, 'agent', '{}', ?4, ?5, ?6, 1, ?7)",
            rusqlite::params![id, input.name, input.cron_expr, input.agent_id, input.prompt, input.directory, next_str],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        drop(conn);
        self.get(&id)?
            .ok_or_else(|| ForgeError::Internal("schedule not found after insert".into()))
    }

    pub fn get(&self, id: &str) -> ForgeResult<Option<Schedule>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, name, cron_expr, agent_id, prompt, directory, enabled, last_run_at, next_run_at, run_count, created_at
                 FROM schedules WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let schedule = stmt
            .query_row(rusqlite::params![id], row_to_schedule)
            .optional()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(schedule)
    }

    pub fn list(&self) -> ForgeResult<Vec<Schedule>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, name, cron_expr, agent_id, prompt, directory, enabled, last_run_at, next_run_at, run_count, created_at
                 FROM schedules ORDER BY created_at DESC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let rows = stmt
            .query_map([], row_to_schedule)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(rows)
    }

    pub fn list_enabled(&self) -> ForgeResult<Vec<Schedule>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, name, cron_expr, agent_id, prompt, directory, enabled, last_run_at, next_run_at, run_count, created_at
                 FROM schedules WHERE enabled = 1 ORDER BY next_run_at ASC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let rows = stmt
            .query_map([], row_to_schedule)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(rows)
    }

    pub fn update(&self, id: &str, input: &UpdateSchedule) -> ForgeResult<Schedule> {
        let existing = self
            .get(id)?
            .ok_or_else(|| ForgeError::Internal(format!("schedule not found: {}", id)))?;

        let name = input.name.as_deref().unwrap_or(&existing.name);
        let cron_expr = input.cron_expr.as_deref().unwrap_or(&existing.cron_expr);
        let prompt = input.prompt.as_deref().unwrap_or(&existing.prompt);
        let directory = input.directory.as_deref().unwrap_or(&existing.directory);
        let enabled = input.enabled.unwrap_or(existing.enabled);

        if input.cron_expr.is_some() {
            validate_cron(cron_expr)?;
        }

        let next = next_occurrence(cron_expr)?;
        let next_str = next.format("%Y-%m-%dT%H:%M:%S").to_string();

        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "UPDATE schedules SET name = ?1, cron_expr = ?2, prompt = ?3, directory = ?4, enabled = ?5, next_run_at = ?6 WHERE id = ?7",
            rusqlite::params![name, cron_expr, prompt, directory, enabled, next_str, id],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        drop(conn);
        self.get(id)?
            .ok_or_else(|| ForgeError::Internal("schedule not found after update".into()))
    }

    pub fn delete(&self, id: &str) -> ForgeResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let rows = conn
            .execute("DELETE FROM schedules WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if rows == 0 {
            return Err(ForgeError::Internal(format!("schedule not found: {}", id)));
        }
        Ok(())
    }

    pub fn update_last_run(&self, id: &str) -> ForgeResult<Schedule> {
        let existing = self
            .get(id)?
            .ok_or_else(|| ForgeError::Internal(format!("schedule not found: {}", id)))?;

        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let next = next_occurrence(&existing.cron_expr)?;
        let next_str = next.format("%Y-%m-%dT%H:%M:%S").to_string();

        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "UPDATE schedules SET last_run_at = ?1, next_run_at = ?2, run_count = run_count + 1 WHERE id = ?3",
            rusqlite::params![now, next_str, id],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        drop(conn);
        self.get(id)?
            .ok_or_else(|| ForgeError::Internal("schedule not found after update_last_run".into()))
    }
}

fn row_to_schedule(row: &rusqlite::Row<'_>) -> rusqlite::Result<Schedule> {
    Ok(Schedule {
        id: row.get(0)?,
        name: row.get(1)?,
        cron_expr: row.get(2)?,
        agent_id: row.get(3)?,
        prompt: row.get(4)?,
        directory: row.get(5)?,
        enabled: row.get(6)?,
        last_run_at: row.get(7)?,
        next_run_at: row.get(8)?,
        run_count: row.get(9)?,
        created_at: row.get(10)?,
    })
}

/// Trait for optional query results.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Validate a cron expression.
pub fn validate_cron(expr: &str) -> ForgeResult<()> {
    CronSchedule::from_str(expr)
        .map_err(|e| ForgeError::Validation(format!("invalid cron expression: {}", e)))?;
    Ok(())
}

/// Calculate next occurrence from a cron expression.
pub fn next_occurrence(expr: &str) -> ForgeResult<DateTime<Utc>> {
    let schedule = CronSchedule::from_str(expr)
        .map_err(|e| ForgeError::Validation(format!("invalid cron expression: {}", e)))?;
    schedule
        .upcoming(Utc)
        .next()
        .ok_or_else(|| ForgeError::Internal("no upcoming occurrence for cron expression".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS schedules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                cron_expr TEXT NOT NULL,
                job_type TEXT NOT NULL DEFAULT 'agent',
                job_config_json TEXT NOT NULL DEFAULT '{}',
                agent_id TEXT REFERENCES agents(id),
                prompt TEXT,
                directory TEXT DEFAULT '.',
                enabled BOOLEAN NOT NULL DEFAULT 1,
                last_run_at TEXT,
                next_run_at TEXT,
                run_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO agents (id, name) VALUES ('agent-1', 'test-agent');",
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn create_and_get_schedule() {
        let conn = setup_db();
        let repo = ScheduleRepo::new(conn);
        let schedule = repo
            .create(&NewSchedule {
                name: "daily-task".into(),
                cron_expr: "0 0 9 * * * *".into(),
                agent_id: "agent-1".into(),
                prompt: "Run tests".into(),
                directory: "/tmp".into(),
            })
            .unwrap();

        assert_eq!(schedule.name, "daily-task");
        assert!(schedule.enabled);
        assert!(schedule.next_run_at.is_some());

        let fetched = repo.get(&schedule.id).unwrap().unwrap();
        assert_eq!(fetched.id, schedule.id);
    }

    #[test]
    fn list_schedules() {
        let conn = setup_db();
        let repo = ScheduleRepo::new(conn);
        repo.create(&NewSchedule {
            name: "sched-a".into(),
            cron_expr: "0 0 9 * * * *".into(),
            agent_id: "agent-1".into(),
            prompt: "task a".into(),
            directory: ".".into(),
        })
        .unwrap();
        repo.create(&NewSchedule {
            name: "sched-b".into(),
            cron_expr: "0 0 12 * * * *".into(),
            agent_id: "agent-1".into(),
            prompt: "task b".into(),
            directory: ".".into(),
        })
        .unwrap();

        let all = repo.list().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn list_enabled_excludes_disabled() {
        let conn = setup_db();
        let repo = ScheduleRepo::new(conn);
        let s = repo
            .create(&NewSchedule {
                name: "will-disable".into(),
                cron_expr: "0 0 9 * * * *".into(),
                agent_id: "agent-1".into(),
                prompt: "task".into(),
                directory: ".".into(),
            })
            .unwrap();
        repo.update(&s.id, &UpdateSchedule {
            name: None,
            cron_expr: None,
            prompt: None,
            directory: None,
            enabled: Some(false),
        })
        .unwrap();

        let enabled = repo.list_enabled().unwrap();
        assert!(enabled.is_empty());
    }

    #[test]
    fn update_schedule() {
        let conn = setup_db();
        let repo = ScheduleRepo::new(conn);
        let s = repo
            .create(&NewSchedule {
                name: "original".into(),
                cron_expr: "0 0 9 * * * *".into(),
                agent_id: "agent-1".into(),
                prompt: "old prompt".into(),
                directory: ".".into(),
            })
            .unwrap();

        let updated = repo
            .update(
                &s.id,
                &UpdateSchedule {
                    name: Some("renamed".into()),
                    cron_expr: None,
                    prompt: Some("new prompt".into()),
                    directory: None,
                    enabled: Some(false),
                },
            )
            .unwrap();

        assert_eq!(updated.name, "renamed");
        assert_eq!(updated.prompt, "new prompt");
        assert!(!updated.enabled);
    }

    #[test]
    fn delete_schedule() {
        let conn = setup_db();
        let repo = ScheduleRepo::new(conn);
        let s = repo
            .create(&NewSchedule {
                name: "to-delete".into(),
                cron_expr: "0 0 9 * * * *".into(),
                agent_id: "agent-1".into(),
                prompt: "task".into(),
                directory: ".".into(),
            })
            .unwrap();

        repo.delete(&s.id).unwrap();
        assert!(repo.get(&s.id).unwrap().is_none());
    }

    #[test]
    fn delete_nonexistent_fails() {
        let conn = setup_db();
        let repo = ScheduleRepo::new(conn);
        assert!(repo.delete("nonexistent").is_err());
    }

    #[test]
    fn update_last_run_increments_count() {
        let conn = setup_db();
        let repo = ScheduleRepo::new(conn);
        let s = repo
            .create(&NewSchedule {
                name: "runner".into(),
                cron_expr: "0 0 9 * * * *".into(),
                agent_id: "agent-1".into(),
                prompt: "run".into(),
                directory: ".".into(),
            })
            .unwrap();
        assert_eq!(s.run_count, 0);

        let updated = repo.update_last_run(&s.id).unwrap();
        assert_eq!(updated.run_count, 1);
        assert!(updated.last_run_at.is_some());
    }

    #[test]
    fn invalid_cron_rejected() {
        let conn = setup_db();
        let repo = ScheduleRepo::new(conn);
        let result = repo.create(&NewSchedule {
            name: "bad".into(),
            cron_expr: "not a cron".into(),
            agent_id: "agent-1".into(),
            prompt: "task".into(),
            directory: ".".into(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn next_occurrence_is_future() {
        let next = next_occurrence("0 0 9 * * * *").unwrap();
        assert!(next > Utc::now());
    }

    #[test]
    fn validation_rejects_empty_name() {
        let conn = setup_db();
        let repo = ScheduleRepo::new(conn);
        let result = repo.create(&NewSchedule {
            name: "".into(),
            cron_expr: "0 0 9 * * * *".into(),
            agent_id: "agent-1".into(),
            prompt: "task".into(),
            directory: ".".into(),
        });
        assert!(result.is_err());
    }
}
