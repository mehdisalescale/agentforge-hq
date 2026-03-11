use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonaId(pub Uuid);

impl PersonaId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PersonaId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonaDivisionId(pub Uuid);

impl PersonaDivisionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PersonaDivisionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaDivision {
    pub id: PersonaDivisionId,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub agent_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub id: PersonaId,
    pub division_slug: String,
    pub slug: String,
    pub name: String,
    pub short_description: String,
    pub personality: Option<String>,
    pub deliverables: Option<String>,
    pub success_metrics: Option<String>,
    pub workflow: Option<String>,
    pub tags: Vec<String>,
    pub source_file: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

