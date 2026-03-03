//! Hook CRUD repository and shell runner.

use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// A hook that runs a shell command before or after an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub id: String,
    pub name: String,
    pub event_type: String,
    pub timing: String, // "pre" | "post"
    pub command: String,
    pub enabled: bool,
    pub created_at: String,
}

/// Input for creating a new hook.
#[derive(Debug, Deserialize)]
pub struct NewHook {
    pub name: String,
    pub event_type: String,
    pub timing: String,
    pub command: String,
}

/// Input for updating an existing hook.
#[derive(Debug, Deserialize)]
pub struct UpdateHook {
    pub name: Option<String>,
    pub command: Option<String>,
    pub enabled: Option<bool>,
}

/// Result of executing a single hook.
#[derive(Debug, Clone, Serialize)]
pub struct HookResult {
    pub hook_id: String,
    pub hook_name: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

pub struct HookRepo {
    conn: Arc<Mutex<Connection>>,
}

impl HookRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewHook) -> ForgeResult<Hook> {
        if input.name.trim().is_empty() {
            return Err(ForgeError::Validation("hook name cannot be empty".into()));
        }
        if input.command.trim().is_empty() {
            return Err(ForgeError::Validation(
                "hook command cannot be empty".into(),
            ));
        }
        if input.timing != "pre" && input.timing != "post" {
            return Err(ForgeError::Validation(
                "hook timing must be 'pre' or 'post'".into(),
            ));
        }

        let conn = self.conn.lock().expect("db mutex poisoned");
        let id = uuid::Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO hooks (id, name, event_type, timing, command, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            rusqlite::params![
                id,
                input.name,
                input.event_type,
                input.timing,
                input.command
            ],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        drop(conn);
        self.get(&id)?
            .ok_or_else(|| ForgeError::Internal("hook not found after insert".into()))
    }

    pub fn get(&self, id: &str) -> ForgeResult<Option<Hook>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, name, event_type, timing, command, enabled, created_at
                 FROM hooks WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let hook = stmt
            .query_row(rusqlite::params![id], |row| row_to_hook(row))
            .optional()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(hook)
    }

    pub fn list(&self) -> ForgeResult<Vec<Hook>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, name, event_type, timing, command, enabled, created_at
                 FROM hooks ORDER BY created_at DESC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let hooks = stmt
            .query_map([], |row| row_to_hook(row))
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(hooks)
    }

    pub fn update(&self, id: &str, input: &UpdateHook) -> ForgeResult<Hook> {
        let existing = self
            .get(id)?
            .ok_or_else(|| ForgeError::Internal(format!("hook not found: {}", id)))?;

        let name = input.name.as_deref().unwrap_or(&existing.name);
        let command = input.command.as_deref().unwrap_or(&existing.command);
        let enabled = input.enabled.unwrap_or(existing.enabled);

        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "UPDATE hooks SET name = ?1, command = ?2, enabled = ?3 WHERE id = ?4",
            rusqlite::params![name, command, enabled, id],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        drop(conn);
        self.get(id)?
            .ok_or_else(|| ForgeError::Internal("hook not found after update".into()))
    }

    pub fn delete(&self, id: &str) -> ForgeResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let rows = conn
            .execute("DELETE FROM hooks WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if rows == 0 {
            return Err(ForgeError::Internal(format!("hook not found: {}", id)));
        }

        Ok(())
    }

    pub fn find_by_event(&self, event_type: &str, timing: &str) -> ForgeResult<Vec<Hook>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, name, event_type, timing, command, enabled, created_at
                 FROM hooks
                 WHERE event_type = ?1 AND timing = ?2 AND enabled = 1
                 ORDER BY created_at ASC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let hooks = stmt
            .query_map(rusqlite::params![event_type, timing], |row| {
                row_to_hook(row)
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(hooks)
    }
}

fn row_to_hook(row: &rusqlite::Row<'_>) -> rusqlite::Result<Hook> {
    Ok(Hook {
        id: row.get(0)?,
        name: row.get(1)?,
        event_type: row.get(2)?,
        timing: row.get(3)?,
        command: row.get(4)?,
        enabled: row.get(5)?,
        created_at: row.get(6)?,
    })
}

/// Trait for optional query results (mirrors rusqlite pattern).
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

/// Runs hook commands as shell subprocesses.
pub struct HookRunner;

impl HookRunner {
    /// Execute a list of hooks sequentially, returning results for each.
    pub async fn run_hooks(hooks: &[Hook]) -> Vec<HookResult> {
        let mut results = Vec::with_capacity(hooks.len());
        for hook in hooks {
            let result = Self::run_one(hook).await;
            results.push(result);
        }
        results
    }

    async fn run_one(hook: &Hook) -> HookResult {
        let start = std::time::Instant::now();
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&hook.command)
            .output()
            .await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match output {
            Ok(output) => HookResult {
                hook_id: hook.id.clone(),
                hook_name: hook.name.clone(),
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration_ms,
            },
            Err(e) => HookResult {
                hook_id: hook.id.clone(),
                hook_name: hook.name.clone(),
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: format!("failed to execute hook: {}", e),
                duration_ms,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS hooks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                event_type TEXT NOT NULL,
                timing TEXT NOT NULL CHECK (timing IN ('pre', 'post')),
                command TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_hooks_event_timing ON hooks(event_type, timing);",
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn create_and_get_hook() {
        let conn = setup_db();
        let repo = HookRepo::new(conn);

        let hook = repo
            .create(&NewHook {
                name: "lint".into(),
                event_type: "session.complete".into(),
                timing: "post".into(),
                command: "echo lint".into(),
            })
            .unwrap();

        assert_eq!(hook.name, "lint");
        assert_eq!(hook.event_type, "session.complete");
        assert_eq!(hook.timing, "post");
        assert!(hook.enabled);

        let fetched = repo.get(&hook.id).unwrap().unwrap();
        assert_eq!(fetched.id, hook.id);
    }

    #[test]
    fn list_hooks() {
        let conn = setup_db();
        let repo = HookRepo::new(conn);

        repo.create(&NewHook {
            name: "hook-a".into(),
            event_type: "agent.run".into(),
            timing: "pre".into(),
            command: "echo a".into(),
        })
        .unwrap();
        repo.create(&NewHook {
            name: "hook-b".into(),
            event_type: "agent.run".into(),
            timing: "post".into(),
            command: "echo b".into(),
        })
        .unwrap();

        let hooks = repo.list().unwrap();
        assert_eq!(hooks.len(), 2);
    }

    #[test]
    fn update_hook() {
        let conn = setup_db();
        let repo = HookRepo::new(conn);

        let hook = repo
            .create(&NewHook {
                name: "old-name".into(),
                event_type: "session.start".into(),
                timing: "pre".into(),
                command: "echo old".into(),
            })
            .unwrap();

        let updated = repo
            .update(
                &hook.id,
                &UpdateHook {
                    name: Some("new-name".into()),
                    command: Some("echo new".into()),
                    enabled: Some(false),
                },
            )
            .unwrap();

        assert_eq!(updated.name, "new-name");
        assert_eq!(updated.command, "echo new");
        assert!(!updated.enabled);
    }

    #[test]
    fn delete_hook() {
        let conn = setup_db();
        let repo = HookRepo::new(conn);

        let hook = repo
            .create(&NewHook {
                name: "to-delete".into(),
                event_type: "agent.run".into(),
                timing: "pre".into(),
                command: "echo x".into(),
            })
            .unwrap();

        repo.delete(&hook.id).unwrap();
        let result = repo.get(&hook.id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn delete_nonexistent_hook_fails() {
        let conn = setup_db();
        let repo = HookRepo::new(conn);

        let result = repo.delete("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn find_by_event() {
        let conn = setup_db();
        let repo = HookRepo::new(conn);

        repo.create(&NewHook {
            name: "pre-run".into(),
            event_type: "agent.run".into(),
            timing: "pre".into(),
            command: "echo pre".into(),
        })
        .unwrap();
        repo.create(&NewHook {
            name: "post-run".into(),
            event_type: "agent.run".into(),
            timing: "post".into(),
            command: "echo post".into(),
        })
        .unwrap();
        repo.create(&NewHook {
            name: "pre-other".into(),
            event_type: "session.start".into(),
            timing: "pre".into(),
            command: "echo other".into(),
        })
        .unwrap();

        let pre_run = repo.find_by_event("agent.run", "pre").unwrap();
        assert_eq!(pre_run.len(), 1);
        assert_eq!(pre_run[0].name, "pre-run");

        let post_run = repo.find_by_event("agent.run", "post").unwrap();
        assert_eq!(post_run.len(), 1);
        assert_eq!(post_run[0].name, "post-run");
    }

    #[test]
    fn find_by_event_excludes_disabled() {
        let conn = setup_db();
        let repo = HookRepo::new(conn);

        let hook = repo
            .create(&NewHook {
                name: "disabled-hook".into(),
                event_type: "agent.run".into(),
                timing: "pre".into(),
                command: "echo disabled".into(),
            })
            .unwrap();
        repo.update(
            &hook.id,
            &UpdateHook {
                name: None,
                command: None,
                enabled: Some(false),
            },
        )
        .unwrap();

        let hooks = repo.find_by_event("agent.run", "pre").unwrap();
        assert!(hooks.is_empty());
    }

    #[test]
    fn validation_rejects_empty_name() {
        let conn = setup_db();
        let repo = HookRepo::new(conn);

        let result = repo.create(&NewHook {
            name: "".into(),
            event_type: "agent.run".into(),
            timing: "pre".into(),
            command: "echo x".into(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn validation_rejects_invalid_timing() {
        let conn = setup_db();
        let repo = HookRepo::new(conn);

        let result = repo.create(&NewHook {
            name: "bad".into(),
            event_type: "agent.run".into(),
            timing: "during".into(),
            command: "echo x".into(),
        });
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn hook_runner_executes_command() {
        let hook = Hook {
            id: "test-id".into(),
            name: "echo-test".into(),
            event_type: "test".into(),
            timing: "post".into(),
            command: "echo hello".into(),
            enabled: true,
            created_at: "2026-01-01T00:00:00".into(),
        };

        let results = HookRunner::run_hooks(&[hook]).await;
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(results[0].exit_code, Some(0));
        assert!(results[0].stdout.contains("hello"));
    }

    #[tokio::test]
    async fn hook_runner_captures_failure() {
        let hook = Hook {
            id: "fail-id".into(),
            name: "fail-test".into(),
            event_type: "test".into(),
            timing: "post".into(),
            command: "exit 1".into(),
            enabled: true,
            created_at: "2026-01-01T00:00:00".into(),
        };

        let results = HookRunner::run_hooks(&[hook]).await;
        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
        assert_eq!(results[0].exit_code, Some(1));
    }
}
