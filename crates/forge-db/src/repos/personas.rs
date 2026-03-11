use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use forge_persona::model::{Persona, PersonaDivision, PersonaDivisionId, PersonaId};

pub struct PersonaRepo {
    conn: Arc<Mutex<Connection>>,
}

impl PersonaRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn upsert_divisions(&self, divisions: &[PersonaDivision]) -> ForgeResult<()> {
        let mut conn = self.conn.lock().expect("db mutex poisoned");
        let tx = conn
            .transaction()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        for d in divisions {
            tx.execute(
                "INSERT INTO persona_divisions (id, slug, name, description, agent_count, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(slug) DO UPDATE SET
                   name = excluded.name,
                   description = excluded.description,
                   agent_count = excluded.agent_count,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    d.id.0.to_string(),
                    d.slug,
                    d.name,
                    d.description,
                    d.agent_count as i64,
                    d.created_at.to_rfc3339(),
                    d.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        }

        tx.commit()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn upsert_personas(&self, personas: &[Persona]) -> ForgeResult<()> {
        let mut conn = self.conn.lock().expect("db mutex poisoned");
        let tx = conn
            .transaction()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        for p in personas {
            let tags_json = serde_json::to_string(&p.tags).map_err(|e| ForgeError::Validation(e.to_string()))?;
            tx.execute(
                "INSERT INTO personas (id, division_slug, slug, name, short_description, personality, deliverables, success_metrics, workflow, tags_json, source_file, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                 ON CONFLICT(source_file) DO UPDATE SET
                   division_slug = excluded.division_slug,
                   slug = excluded.slug,
                   name = excluded.name,
                   short_description = excluded.short_description,
                   personality = excluded.personality,
                   deliverables = excluded.deliverables,
                   success_metrics = excluded.success_metrics,
                   workflow = excluded.workflow,
                   tags_json = excluded.tags_json,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    p.id.0.to_string(),
                    p.division_slug,
                    p.slug,
                    p.name,
                    p.short_description,
                    p.personality,
                    p.deliverables,
                    p.success_metrics,
                    p.workflow,
                    tags_json,
                    p.source_file,
                    p.created_at.to_rfc3339(),
                    p.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        }

        tx.commit()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn list(&self, division_slug: Option<&str>, search: Option<&str>) -> ForgeResult<Vec<Persona>> {
        let conn = self.conn.lock().expect("db mutex poisoned");

        let mut sql = String::from(
            "SELECT id, division_slug, slug, name, short_description, personality, deliverables, success_metrics, workflow, tags_json, source_file, created_at, updated_at
             FROM personas",
        );
        let mut params: Vec<String> = Vec::new();

        if division_slug.is_some() || search.is_some() {
            sql.push_str(" WHERE 1=1");
        }

        if let Some(div) = division_slug {
            sql.push_str(" AND division_slug = ?");
            params.push(div.to_string());
        }

        if let Some(q) = search {
            sql.push_str(
                " AND (LOWER(name) LIKE ? OR LOWER(short_description) LIKE ? OR LOWER(tags_json) LIKE ?)",
            );
            let pattern = format!("%{}%", q.to_lowercase());
            params.push(pattern.clone());
            params.push(pattern.clone());
            params.push(pattern);
        }

        sql.push_str(" ORDER BY name ASC");

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let mapped = stmt
            .query_map(
                rusqlite::params_from_iter(params.iter()),
                |row| row_to_persona(row),
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let mut personas = Vec::new();
        for res in mapped {
            let p = res.map_err(|e| ForgeError::Database(Box::new(e)))?;
            personas.push(p);
        }

        Ok(personas)
    }

    pub fn get(&self, id: &PersonaId) -> ForgeResult<Persona> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, division_slug, slug, name, short_description, personality, deliverables, success_metrics, workflow, tags_json, source_file, created_at, updated_at
                 FROM personas WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        stmt.query_row(rusqlite::params![id.0.to_string()], |row| row_to_persona(row))
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    ForgeError::Validation(format!("persona not found: {}", id.0))
                }
                other => ForgeError::Database(Box::new(other)),
            })
    }
}

fn row_to_persona(row: &rusqlite::Row<'_>) -> Result<Persona, rusqlite::Error> {
    let id_str: String = row.get(0)?;
    let id = PersonaId(
        id_str
            .parse()
            .map_err(|_| rusqlite::Error::InvalidQuery)?,
    );
    let division_slug: String = row.get(1)?;
    let slug: String = row.get(2)?;
    let name: String = row.get(3)?;
    let short_description: String = row.get(4)?;
    let personality: Option<String> = row.get(5)?;
    let deliverables: Option<String> = row.get(6)?;
    let success_metrics: Option<String> = row.get(7)?;
    let workflow: Option<String> = row.get(8)?;
    let tags_json: String = row.get(9)?;
    let source_file: String = row.get(10)?;
    let created_at_str: String = row.get(11)?;
    let updated_at_str: String = row.get(12)?;

    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|_| rusqlite::Error::InvalidQuery)?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|_| rusqlite::Error::InvalidQuery)?
        .with_timezone(&Utc);

    Ok(Persona {
        id,
        division_slug,
        slug,
        name,
        short_description,
        personality,
        deliverables,
        success_metrics,
        workflow,
        tags,
        source_file,
        created_at,
        updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn setup_conn() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = crate::migrations::Migrator::new(&conn);
        migrator.apply_pending().unwrap();
        Arc::new(Mutex::new(conn))
    }

    fn sample_division(slug: &str) -> PersonaDivision {
        let now = Utc::now();
        PersonaDivision {
            id: PersonaDivisionId(Uuid::new_v4()),
            slug: slug.to_string(),
            name: slug.to_string(),
            description: None,
            agent_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    fn sample_persona(division_slug: &str, name: &str, source: &str) -> Persona {
        let now = Utc::now();
        Persona {
            id: PersonaId(Uuid::new_v4()),
            division_slug: division_slug.to_string(),
            slug: name.to_lowercase().replace(' ', "-"),
            name: name.to_string(),
            short_description: "short".into(),
            personality: None,
            deliverables: None,
            success_metrics: None,
            workflow: None,
            tags: vec!["tag1".into()],
            source_file: source.into(),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn upsert_and_list_personas_is_idempotent() {
        let conn = setup_conn();
        let repo = PersonaRepo::new(Arc::clone(&conn));

        let div = sample_division("marketing");
        repo.upsert_divisions(&[div]).unwrap();

        let p1 = sample_persona("marketing", "Marketing Growth Hacker", "marketing/growth.md");
        repo.upsert_personas(&[p1.clone()]).unwrap();
        repo.upsert_personas(&[p1]).unwrap();

        let list = repo.list(None, None).unwrap();
        assert_eq!(list.len(), 1);
    }
}

