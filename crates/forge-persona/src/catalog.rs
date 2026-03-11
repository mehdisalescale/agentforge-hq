use crate::model::Persona;

pub struct PersonaCatalog {
    personas: Vec<Persona>,
}

impl PersonaCatalog {
    pub fn from_personas(personas: Vec<Persona>) -> Self {
        Self { personas }
    }

    pub fn all(&self) -> &[Persona] {
        &self.personas
    }

    pub fn by_division(&self, division_slug: &str) -> Vec<&Persona> {
        self.personas
            .iter()
            .filter(|p| p.division_slug == division_slug)
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Persona> {
        let q = query.to_lowercase();
        self.personas
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&q)
                    || p.short_description.to_lowercase().contains(&q)
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Persona, PersonaId};
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_persona(name: &str, division: &str, tags: &[&str]) -> Persona {
        let now = Utc::now();
        Persona {
            id: PersonaId(Uuid::new_v4()),
            division_slug: division.to_string(),
            slug: name.to_lowercase().replace(' ', "-"),
            name: name.to_string(),
            short_description: "short".into(),
            personality: None,
            deliverables: None,
            success_metrics: None,
            workflow: None,
            tags: tags.iter().map(|t| t.to_string()).collect(),
            source_file: format!("{division}/{name}.md"),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn filters_by_division_and_search() {
        let p1 = sample_persona("Marketing Growth Hacker", "marketing", &["growth"]);
        let p2 = sample_persona("Senior Backend Engineer", "engineering", &["backend"]);
        let catalog = PersonaCatalog::from_personas(vec![p1, p2]);

        let marketing = catalog.by_division("marketing");
        assert_eq!(marketing.len(), 1);

        let hits = catalog.search("backend");
        assert_eq!(hits.len(), 1);
    }
}

