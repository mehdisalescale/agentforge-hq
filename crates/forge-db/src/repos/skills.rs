//! Skills repository: list and get by id.

use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub content: String,
    pub source_repo: Option<String>,
    pub parameters_json: Option<String>,
    pub examples_json: Option<String>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
}

pub struct SkillRepo {
    conn: Arc<Mutex<Connection>>,
}

impl SkillRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> ForgeResult<Vec<Skill>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, category, subcategory, content, source_repo, parameters_json, examples_json, usage_count, created_at
                 FROM skills ORDER BY created_at DESC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let skills: Vec<Skill> = stmt
            .query_map([], row_to_skill)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(skills)
    }

    pub fn get(&self, id: &str) -> ForgeResult<Skill> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, category, subcategory, content, source_repo, parameters_json, examples_json, usage_count, created_at
                 FROM skills WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        stmt.query_row(rusqlite::params![id], row_to_skill)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => ForgeError::SkillNotFound(id.to_string()),
                other => ForgeError::Database(Box::new(other)),
            })
    }
}

fn row_to_skill(row: &rusqlite::Row<'_>) -> Result<Skill, rusqlite::Error> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let description: Option<String> = row.get(2)?;
    let category: Option<String> = row.get(3)?;
    let subcategory: Option<String> = row.get(4)?;
    let content: String = row.get(5)?;
    let source_repo: Option<String> = row.get(6)?;
    let parameters_json: Option<String> = row.get(7)?;
    let examples_json: Option<String> = row.get(8)?;
    let usage_count: i32 = row.get(9)?;
    let created_at: String = row.get(10)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .map_err(|_| rusqlite::Error::InvalidParameterName(created_at.clone()))?
        .with_timezone(&Utc);
    Ok(Skill {
        id,
        name,
        description,
        category,
        subcategory,
        content,
        source_repo,
        parameters_json,
        examples_json,
        usage_count,
        created_at,
    })
}
