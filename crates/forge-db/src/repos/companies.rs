use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub mission: Option<String>,
    pub budget_limit: Option<f64>,
    pub budget_used: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewCompany {
    pub name: String,
    pub mission: Option<String>,
    pub budget_limit: Option<f64>,
}

pub struct CompanyRepo {
    conn: Arc<Mutex<Connection>>,
}

impl CompanyRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewCompany) -> ForgeResult<Company> {
        if input.name.trim().is_empty() {
            return Err(ForgeError::Validation("company name is required".into()));
        }

        let conn = crate::pool::lock_conn(&self.conn)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let result = conn.execute(
            "INSERT INTO companies (id, name, mission, budget_limit, budget_used, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 0.0, ?5, ?6)",
            rusqlite::params![
                id,
                input.name,
                input.mission,
                input.budget_limit,
                now,
                now,
            ],
        );

        match result {
            Ok(_) => drop(conn),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("UNIQUE") && msg.contains("name") {
                    return Err(ForgeError::Validation(format!(
                        "company with name '{}' already exists",
                        input.name
                    )));
                }
                return Err(ForgeError::Database(Box::new(e)));
            }
        }

        self.get(&id)
    }

    pub fn get(&self, id: &str) -> ForgeResult<Company> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, mission, budget_limit, budget_used, created_at, updated_at
                 FROM companies WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        stmt.query_row(rusqlite::params![id], row_to_company)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    ForgeError::Validation(format!("company not found: {id}"))
                }
                other => ForgeError::Database(Box::new(other)),
            })
    }

    pub fn update(
        &self,
        id: &str,
        name: Option<&str>,
        mission: Option<&str>,
        budget_limit: Option<f64>,
    ) -> ForgeResult<Company> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let now = Utc::now().to_rfc3339();

        let mut sets = vec!["updated_at = ?1".to_string()];
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

        if let Some(n) = name {
            params.push(Box::new(n.to_string()));
            sets.push(format!("name = ?{}", params.len()));
        }
        if let Some(m) = mission {
            params.push(Box::new(m.to_string()));
            sets.push(format!("mission = ?{}", params.len()));
        }
        if let Some(b) = budget_limit {
            params.push(Box::new(b));
            sets.push(format!("budget_limit = ?{}", params.len()));
        }

        params.push(Box::new(id.to_string()));
        let id_param = params.len();

        let sql = format!(
            "UPDATE companies SET {} WHERE id = ?{}",
            sets.join(", "),
            id_param
        );

        let affected = conn
            .execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if affected == 0 {
            return Err(ForgeError::Validation(format!("company not found: {id}")));
        }

        drop(conn);
        self.get(id)
    }

    pub fn delete(&self, id: &str) -> ForgeResult<()> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let affected = conn
            .execute("DELETE FROM companies WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if affected == 0 {
            return Err(ForgeError::Validation(format!("company not found: {id}")));
        }
        Ok(())
    }

    pub fn list(&self) -> ForgeResult<Vec<Company>> {
        let conn = crate::pool::lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, mission, budget_limit, budget_used, created_at, updated_at
                 FROM companies ORDER BY created_at DESC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let rows = stmt
            .query_map([], row_to_company)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(rows)
    }
}

fn row_to_company(row: &rusqlite::Row<'_>) -> Result<Company, rusqlite::Error> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let mission: Option<String> = row.get(2)?;
    let budget_limit: Option<f64> = row.get(3)?;
    let budget_used: f64 = row.get(4)?;
    let created_at_str: String = row.get(5)?;
    let updated_at_str: String = row.get(6)?;

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(created_at_str.clone()))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(updated_at_str.clone()))?
        .with_timezone(&Utc);

    Ok(Company {
        id,
        name,
        mission,
        budget_limit,
        budget_used,
        created_at,
        updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_conn() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE companies (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                mission TEXT,
                budget_limit REAL,
                budget_used REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn create_and_list_company() {
        let repo = CompanyRepo::new(setup_conn());
        let created = repo
            .create(&NewCompany {
                name: "TestCo".into(),
                mission: Some("Test mission".into()),
                budget_limit: Some(100.0),
            })
            .unwrap();
        assert_eq!(created.name, "TestCo");
        assert_eq!(created.budget_used, 0.0);

        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);
    }
}

