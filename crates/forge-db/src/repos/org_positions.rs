use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrgPosition {
    pub id: String,
    pub company_id: String,
    pub department_id: Option<String>,
    pub agent_id: Option<String>,
    pub reports_to: Option<String>,
    pub role: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewOrgPosition {
    pub company_id: String,
    pub department_id: Option<String>,
    pub agent_id: Option<String>,
    pub reports_to: Option<String>,
    pub role: String,
    pub title: Option<String>,
}

pub struct OrgPositionRepo {
    conn: Arc<Mutex<Connection>>,
}

impl OrgPositionRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewOrgPosition) -> ForgeResult<OrgPosition> {
        if input.role.trim().is_empty() {
            return Err(ForgeError::Validation("org position role is required".into()));
        }

        let conn = crate::pool::lock_conn(&self.conn)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO org_positions (
                id, company_id, department_id, agent_id, reports_to, role, title, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                id,
                input.company_id,
                input.department_id,
                input.agent_id,
                input.reports_to,
                input.role,
                input.title,
                now,
                now,
            ],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        drop(conn);
        self.get(&id)
    }

    pub fn get(&self, id: &str) -> ForgeResult<OrgPosition> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, company_id, department_id, agent_id, reports_to, role, title, created_at, updated_at
                 FROM org_positions WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        stmt.query_row(rusqlite::params![id], row_to_org_position)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    ForgeError::Validation(format!("org position not found: {id}"))
                }
                other => ForgeError::Database(Box::new(other)),
            })
    }

    pub fn list_by_company(&self, company_id: &str) -> ForgeResult<Vec<OrgPosition>> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, company_id, department_id, agent_id, reports_to, role, title, created_at, updated_at
                 FROM org_positions WHERE company_id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let rows = stmt
            .query_map(rusqlite::params![company_id], row_to_org_position)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(rows)
    }
}

fn row_to_org_position(row: &rusqlite::Row<'_>) -> Result<OrgPosition, rusqlite::Error> {
    let id: String = row.get(0)?;
    let company_id: String = row.get(1)?;
    let department_id: Option<String> = row.get(2)?;
    let agent_id: Option<String> = row.get(3)?;
    let reports_to: Option<String> = row.get(4)?;
    let role: String = row.get(5)?;
    let title: Option<String> = row.get(6)?;
    let created_at_str: String = row.get(7)?;
    let updated_at_str: String = row.get(8)?;

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(created_at_str.clone()))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(updated_at_str.clone()))?
        .with_timezone(&Utc);

    Ok(OrgPosition {
        id,
        company_id,
        department_id,
        agent_id,
        reports_to,
        role,
        title,
        created_at,
        updated_at,
    })
}

