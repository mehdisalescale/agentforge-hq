use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Approval {
    pub id: String,
    pub company_id: String,
    pub approval_type: String,
    pub status: String,
    pub requester: String,
    pub approver: Option<String>,
    pub data_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewApproval {
    pub company_id: String,
    pub approval_type: String,
    pub requester: String,
    pub data_json: String,
}

pub struct ApprovalRepo {
    conn: Arc<Mutex<Connection>>,
}

impl ApprovalRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewApproval) -> ForgeResult<Approval> {
        if input.approval_type.trim().is_empty() {
            return Err(ForgeError::Validation("approval_type is required".into()));
        }

        let conn = crate::pool::lock_conn(&self.conn)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO approvals (
                id, company_id, approval_type, status, requester, approver, data_json, created_at, updated_at
             ) VALUES (?1, ?2, ?3, 'pending', ?4, NULL, ?5, ?6, ?7)",
            rusqlite::params![
                id,
                input.company_id,
                input.approval_type,
                input.requester,
                input.data_json,
                now,
                now,
            ],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(&id)
    }

    pub fn get(&self, id: &str) -> ForgeResult<Approval> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, company_id, approval_type, status, requester, approver, data_json, created_at, updated_at
                 FROM approvals WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        stmt.query_row(rusqlite::params![id], row_to_approval)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    ForgeError::Validation(format!("approval not found: {id}"))
                }
                other => ForgeError::Database(Box::new(other)),
            })
    }

    pub fn update_status(
        &self,
        id: &str,
        status: &str,
        approver: Option<&str>,
    ) -> ForgeResult<Approval> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let now = Utc::now().to_rfc3339();
        let rows = conn
            .execute(
                "UPDATE approvals SET status = ?1, approver = ?2, updated_at = ?3 WHERE id = ?4",
                rusqlite::params![status, approver, now, id],
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        if rows == 0 {
            return Err(ForgeError::Validation(format!("approval not found: {id}")));
        }
        drop(conn);
        self.get(id)
    }

    pub fn list_by_company(&self, company_id: &str) -> ForgeResult<Vec<Approval>> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, company_id, approval_type, status, requester, approver, data_json, created_at, updated_at
                 FROM approvals WHERE company_id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(rusqlite::params![company_id], row_to_approval)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(rows)
    }
}

fn row_to_approval(row: &rusqlite::Row<'_>) -> Result<Approval, rusqlite::Error> {
    let id: String = row.get(0)?;
    let company_id: String = row.get(1)?;
    let approval_type: String = row.get(2)?;
    let status: String = row.get(3)?;
    let requester: String = row.get(4)?;
    let approver: Option<String> = row.get(5)?;
    let data_json: String = row.get(6)?;
    let created_at_str: String = row.get(7)?;
    let updated_at_str: String = row.get(8)?;

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(created_at_str.clone()))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(updated_at_str.clone()))?
        .with_timezone(&Utc);

    Ok(Approval {
        id,
        company_id,
        approval_type,
        status,
        requester,
        approver,
        data_json,
        created_at,
        updated_at,
    })
}

