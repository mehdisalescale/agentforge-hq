use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Goal {
    pub id: String,
    pub company_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewGoal {
    pub company_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
}

pub struct GoalRepo {
    conn: Arc<Mutex<Connection>>,
}

impl GoalRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewGoal) -> ForgeResult<Goal> {
        if input.title.trim().is_empty() {
            return Err(ForgeError::Validation("goal title is required".into()));
        }

        let conn = self.conn.lock().expect("db mutex poisoned");
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO goals (id, company_id, parent_id, title, description, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'planned', ?6, ?7)",
            rusqlite::params![
                id,
                input.company_id,
                input.parent_id,
                input.title,
                input.description,
                now,
                now,
            ],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(&id)
    }

    pub fn get(&self, id: &str) -> ForgeResult<Goal> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, company_id, parent_id, title, description, status, created_at, updated_at
                 FROM goals WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        stmt.query_row(rusqlite::params![id], row_to_goal)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    ForgeError::Validation(format!("goal not found: {id}"))
                }
                other => ForgeError::Database(Box::new(other)),
            })
    }

    pub fn update_status(&self, id: &str, status: &str) -> ForgeResult<Goal> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let now = Utc::now().to_rfc3339();
        let rows = conn
            .execute(
                "UPDATE goals SET status = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![status, now, id],
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        if rows == 0 {
            return Err(ForgeError::Validation(format!("goal not found: {id}")));
        }
        drop(conn);
        self.get(id)
    }

    pub fn list_by_company(&self, company_id: &str) -> ForgeResult<Vec<Goal>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, company_id, parent_id, title, description, status, created_at, updated_at
                 FROM goals WHERE company_id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(rusqlite::params![company_id], row_to_goal)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(rows)
    }
}

fn row_to_goal(row: &rusqlite::Row<'_>) -> Result<Goal, rusqlite::Error> {
    let id: String = row.get(0)?;
    let company_id: String = row.get(1)?;
    let parent_id: Option<String> = row.get(2)?;
    let title: String = row.get(3)?;
    let description: Option<String> = row.get(4)?;
    let status: String = row.get(5)?;
    let created_at_str: String = row.get(6)?;
    let updated_at_str: String = row.get(7)?;

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(created_at_str.clone()))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(updated_at_str.clone()))?
        .with_timezone(&Utc);

    Ok(Goal {
        id,
        company_id,
        parent_id,
        title,
        description,
        status,
        created_at,
        updated_at,
    })
}

