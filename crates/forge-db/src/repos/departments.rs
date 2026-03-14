use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Department {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewDepartment {
    pub company_id: String,
    pub name: String,
    pub description: Option<String>,
}

pub struct DepartmentRepo {
    conn: Arc<Mutex<Connection>>,
}

impl DepartmentRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewDepartment) -> ForgeResult<Department> {
        if input.name.trim().is_empty() {
            return Err(ForgeError::Validation("department name is required".into()));
        }

        let conn = self.conn.lock().expect("db mutex poisoned");
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let result = conn.execute(
            "INSERT INTO departments (id, company_id, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, input.company_id, input.name, input.description, now, now],
        );

        match result {
            Ok(_) => drop(conn),
            Err(e) => {
                return Err(ForgeError::Database(Box::new(e)));
            }
        }

        self.get(&id)
    }

    pub fn get(&self, id: &str) -> ForgeResult<Department> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, company_id, name, description, created_at, updated_at
                 FROM departments WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        stmt.query_row(rusqlite::params![id], row_to_department)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    ForgeError::Validation(format!("department not found: {id}"))
                }
                other => ForgeError::Database(Box::new(other)),
            })
    }

    pub fn update(
        &self,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> ForgeResult<Department> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let now = Utc::now().to_rfc3339();

        let mut sets = vec!["updated_at = ?1".to_string()];
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

        if let Some(n) = name {
            params.push(Box::new(n.to_string()));
            sets.push(format!("name = ?{}", params.len()));
        }
        if let Some(d) = description {
            params.push(Box::new(d.to_string()));
            sets.push(format!("description = ?{}", params.len()));
        }

        params.push(Box::new(id.to_string()));
        let id_param = params.len();

        let sql = format!(
            "UPDATE departments SET {} WHERE id = ?{}",
            sets.join(", "),
            id_param
        );

        let affected = conn
            .execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if affected == 0 {
            return Err(ForgeError::Validation(format!("department not found: {id}")));
        }

        drop(conn);
        self.get(id)
    }

    pub fn delete(&self, id: &str) -> ForgeResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let affected = conn
            .execute("DELETE FROM departments WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if affected == 0 {
            return Err(ForgeError::Validation(format!("department not found: {id}")));
        }
        Ok(())
    }

    pub fn list_by_company(&self, company_id: &str) -> ForgeResult<Vec<Department>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, company_id, name, description, created_at, updated_at
                 FROM departments WHERE company_id = ?1 ORDER BY name ASC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let rows = stmt
            .query_map(rusqlite::params![company_id], row_to_department)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(rows)
    }
}

fn row_to_department(row: &rusqlite::Row<'_>) -> Result<Department, rusqlite::Error> {
    let id: String = row.get(0)?;
    let company_id: String = row.get(1)?;
    let name: String = row.get(2)?;
    let description: Option<String> = row.get(3)?;
    let created_at_str: String = row.get(4)?;
    let updated_at_str: String = row.get(5)?;

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(created_at_str.clone()))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|_| rusqlite::Error::InvalidParameterName(updated_at_str.clone()))?
        .with_timezone(&Utc);

    Ok(Department {
        id,
        company_id,
        name,
        description,
        created_at,
        updated_at,
    })
}

