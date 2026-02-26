//! Workflows repository: list and get by id.

use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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
                rusqlite::Error::QueryReturnedNoRows => ForgeError::WorkflowNotFound(id.to_string()),
                other => ForgeError::Database(Box::new(other)),
            })
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
