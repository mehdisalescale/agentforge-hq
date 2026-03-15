//! Workflows repository: list, get, create, update, delete.

use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub definition_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct WorkflowRepo {
    conn: Arc<Mutex<Connection>>,
}

impl WorkflowRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> ForgeResult<Vec<Workflow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, definition_json, created_at, updated_at
                 FROM workflows ORDER BY created_at DESC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let workflows: Vec<Workflow> = stmt
            .query_map([], row_to_workflow)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(workflows)
    }

    pub fn get(&self, id: &str) -> ForgeResult<Workflow> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, definition_json, created_at, updated_at
                 FROM workflows WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        stmt.query_row(rusqlite::params![id], row_to_workflow)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => ForgeError::NotFound { entity: "workflow", id: id.to_string() },
                other => ForgeError::Database(Box::new(other)),
            })
    }

    /// Create a new workflow. Returns the created `Workflow`.
    pub fn create(
        &self,
        name: &str,
        description: Option<&str>,
        definition_json: &str,
    ) -> ForgeResult<Workflow> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO workflows (id, name, description, definition_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, name, description, definition_json, now, now],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(&id)
    }

    /// Update an existing workflow. Only non-`None` fields are changed.
    /// Returns the updated `Workflow`.
    pub fn update(
        &self,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        definition_json: Option<&str>,
    ) -> ForgeResult<Workflow> {
        // Verify workflow exists first
        let _ = self.get(id)?;

        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().expect("db mutex poisoned");

        let mut sets = vec!["updated_at = ?1".to_string()];
        let mut param_idx = 2u32;
        // Collect dynamic params as trait objects
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

        if let Some(n) = name {
            sets.push(format!("name = ?{}", param_idx));
            params.push(Box::new(n.to_string()));
            param_idx += 1;
        }
        if let Some(d) = description {
            sets.push(format!("description = ?{}", param_idx));
            params.push(Box::new(d.to_string()));
            param_idx += 1;
        }
        if let Some(dj) = definition_json {
            sets.push(format!("definition_json = ?{}", param_idx));
            params.push(Box::new(dj.to_string()));
            param_idx += 1;
        }
        // Always need the id as the last param for WHERE clause
        let id_param_idx = param_idx;
        params.push(Box::new(id.to_string()));

        let sql = format!(
            "UPDATE workflows SET {} WHERE id = ?{}",
            sets.join(", "),
            id_param_idx,
        );

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, param_refs.as_slice())
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(id)
    }

    /// Delete a workflow by id.
    pub fn delete(&self, id: &str) -> ForgeResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let rows = conn
            .execute("DELETE FROM workflows WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        if rows == 0 {
            return Err(ForgeError::NotFound { entity: "workflow", id: id.to_string() });
        }
        Ok(())
    }
}

fn row_to_workflow(row: &rusqlite::Row<'_>) -> Result<Workflow, rusqlite::Error> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let description: Option<String> = row.get(2)?;
    let definition_json: String = row.get(3)?;
    let created_at: String = row.get(4)?;
    let updated_at: String = row.get(5)?;
    let created_at = parse_sqlite_datetime(&created_at)?;
    let updated_at = parse_sqlite_datetime(&updated_at)?;
    Ok(Workflow {
        id,
        name,
        description,
        definition_json,
        created_at,
        updated_at,
    })
}

/// Parse datetime from SQLite: RFC3339 or "YYYY-MM-DD HH:MM:SS" (datetime('now')).
fn parse_sqlite_datetime(s: &str) -> Result<DateTime<Utc>, rusqlite::Error> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc())
                .map_err(|_| rusqlite::Error::InvalidParameterName(s.to_string()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_repo() -> WorkflowRepo {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                definition_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .expect("create workflows table");
        WorkflowRepo::new(Arc::new(Mutex::new(conn)))
    }

    #[test]
    fn workflow_create_and_get() {
        let repo = setup_repo();
        let wf = repo
            .create("My Workflow", Some("A test workflow"), r#"{"steps":[]}"#)
            .expect("create workflow");
        assert_eq!(wf.name, "My Workflow");
        assert_eq!(wf.description.as_deref(), Some("A test workflow"));
        assert_eq!(wf.definition_json, r#"{"steps":[]}"#);
        assert!(!wf.id.is_empty());

        let fetched = repo.get(&wf.id).expect("get workflow");
        assert_eq!(fetched.id, wf.id);
        assert_eq!(fetched.name, "My Workflow");
        assert_eq!(fetched.description.as_deref(), Some("A test workflow"));
    }

    #[test]
    fn workflow_update_name() {
        let repo = setup_repo();
        let wf = repo
            .create("Original", None, r#"{"steps":[]}"#)
            .expect("create workflow");

        let updated = repo
            .update(&wf.id, Some("Renamed"), None, None)
            .expect("update workflow");
        assert_eq!(updated.name, "Renamed");
        // description and definition_json should be preserved
        assert_eq!(updated.description, None);
        assert_eq!(updated.definition_json, r#"{"steps":[]}"#);
        // updated_at should be >= created_at
        assert!(updated.updated_at >= updated.created_at);
    }

    #[test]
    fn workflow_delete() {
        let repo = setup_repo();
        let wf = repo
            .create("ToDelete", None, "{}")
            .expect("create workflow");

        repo.delete(&wf.id).expect("delete workflow");

        let result = repo.get(&wf.id);
        assert!(result.is_err());
        match result.unwrap_err() {
            ForgeError::NotFound { entity, id } => {
                assert_eq!(entity, "workflow");
                assert_eq!(id, wf.id);
            }
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn workflow_delete_nonexistent_returns_not_found() {
        let repo = setup_repo();
        let result = repo.delete("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn workflow_list_returns_all() {
        let repo = setup_repo();
        repo.create("W1", None, "{}").expect("create w1");
        repo.create("W2", Some("desc"), "{}").expect("create w2");

        let all = repo.list().expect("list workflows");
        assert_eq!(all.len(), 2);
    }
}
