//! Skills repository: list, get, upsert, and load from directory.

use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing;
use uuid::Uuid;

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

/// A rule that triggers automatic activation of a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRule {
    pub id: String,
    pub skill_id: String,
    /// "file_pattern" or "keyword"
    pub trigger_type: String,
    pub trigger_pattern: String,
    pub enabled: bool,
    pub created_at: String,
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

    /// Insert or update a skill by id. On conflict (same id), updates all fields
    /// except usage_count and created_at.
    pub fn upsert(&self, input: &UpsertSkill) -> ForgeResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO skills (id, name, description, category, subcategory, content, source_repo, parameters_json, examples_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                category = excluded.category,
                subcategory = excluded.subcategory,
                content = excluded.content,
                source_repo = excluded.source_repo,
                parameters_json = excluded.parameters_json,
                examples_json = excluded.examples_json",
            rusqlite::params![
                input.id,
                input.name,
                input.description,
                input.category,
                input.subcategory,
                input.content,
                input.source_repo,
                input.parameters_json,
                input.examples_json,
            ],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(())
    }

    /// Load all `.md` skill files from a directory. Each file must have YAML
    /// frontmatter delimited by `---`. Parsed fields: name, description, tags, tools.
    /// Files are upserted into the skills table.
    pub fn load_from_dir(&self, dir: &Path) -> ForgeResult<usize> {
        let entries = std::fs::read_dir(dir).map_err(ForgeError::Io)?;
        let mut count = 0;
        for entry in entries {
            let entry = entry.map_err(ForgeError::Io)?;
            let path = entry.path();
            if path.is_dir() {
                count += self.load_from_dir(&path)?;
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let raw = std::fs::read_to_string(&path).map_err(ForgeError::Io)?;
            match parse_skill_frontmatter(&raw) {
                Some(input) => {
                    self.upsert(&input)?;
                    count += 1;
                    tracing::info!(skill = %input.name, "loaded skill");
                }
                None => {
                    tracing::warn!(path = %path.display(), "skipping file: no valid frontmatter");
                }
            }
        }
        Ok(count)
    }

    // --- Skill rule methods ---

    /// Create a new activation rule for a skill.
    pub fn create_rule(
        &self,
        skill_id: &str,
        trigger_type: &str,
        trigger_pattern: &str,
    ) -> ForgeResult<SkillRule> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO skill_rules (id, skill_id, trigger_type, trigger_pattern, enabled)
             VALUES (?1, ?2, ?3, ?4, 1)",
            rusqlite::params![id, skill_id, trigger_type, trigger_pattern],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, skill_id, trigger_type, trigger_pattern, enabled, created_at
                 FROM skill_rules WHERE id = ?1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        stmt.query_row(rusqlite::params![id], row_to_skill_rule)
            .map_err(|e| ForgeError::Database(Box::new(e)))
    }

    /// List all rules for a given skill.
    pub fn list_rules(&self, skill_id: &str) -> ForgeResult<Vec<SkillRule>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, skill_id, trigger_type, trigger_pattern, enabled, created_at
                 FROM skill_rules WHERE skill_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let rules: Vec<SkillRule> = stmt
            .query_map(rusqlite::params![skill_id], row_to_skill_rule)
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(rules)
    }

    /// Delete a skill rule by id.
    pub fn delete_rule(&self, id: &str) -> ForgeResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let rows = conn
            .execute("DELETE FROM skill_rules WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if rows == 0 {
            return Err(ForgeError::Internal(format!("skill rule not found: {}", id)));
        }

        Ok(())
    }

    /// Find skills whose rules match the given working directory and/or prompt.
    ///
    /// - `file_pattern` rules: check if `Path::new(working_dir).join(pattern).exists()`
    /// - `keyword` rules: check if `prompt.to_lowercase()` contains the pattern (case-insensitive)
    ///
    /// Returns a de-duplicated list of matched skills.
    pub fn find_matching_rules(&self, working_dir: &str, prompt: &str) -> ForgeResult<Vec<Skill>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT sr.id, sr.skill_id, sr.trigger_type, sr.trigger_pattern, sr.enabled, sr.created_at,
                        s.id, s.name, s.description, s.category, s.subcategory, s.content, s.source_repo, s.parameters_json, s.examples_json, s.usage_count, s.created_at
                 FROM skill_rules sr
                 JOIN skills s ON sr.skill_id = s.id
                 WHERE sr.enabled = 1",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let prompt_lower = prompt.to_lowercase();
        let mut seen_skill_ids = HashSet::new();
        let mut matched_skills = Vec::new();

        let rows = stmt
            .query_map([], |row| {
                let trigger_type: String = row.get(2)?;
                let trigger_pattern: String = row.get(3)?;
                let skill = row_to_skill_from_offset(row, 6)?;
                Ok((trigger_type, trigger_pattern, skill))
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        for row_result in rows {
            let (trigger_type, trigger_pattern, skill) =
                row_result.map_err(|e| ForgeError::Database(Box::new(e)))?;

            let matches = match trigger_type.as_str() {
                "file_pattern" => Path::new(working_dir).join(&trigger_pattern).exists(),
                "keyword" => prompt_lower.contains(&trigger_pattern.to_lowercase()),
                _ => false,
            };

            if matches && seen_skill_ids.insert(skill.id.clone()) {
                matched_skills.push(skill);
            }
        }

        Ok(matched_skills)
    }
}

/// Input for upserting a skill.
pub struct UpsertSkill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub content: String,
    pub source_repo: Option<String>,
    pub parameters_json: Option<String>,
    pub examples_json: Option<String>,
}

/// Parse YAML frontmatter from a skill markdown file.
/// Expects `---` delimiters around key: value pairs.
fn parse_skill_frontmatter(raw: &str) -> Option<UpsertSkill> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_first = &trimmed[3..];
    let end_idx = after_first.find("---")?;
    let frontmatter = &after_first[..end_idx];
    let body = after_first[end_idx + 3..].trim().to_string();

    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut tools: Vec<String> = Vec::new();

    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "name" => name = Some(value.to_string()),
                "description" => description = Some(value.to_string()),
                "tags" => tags = parse_bracket_list(value),
                "tools" => tools = parse_bracket_list(value),
                _ => {}
            }
        }
    }

    let skill_name = name?;
    let category = tags.first().cloned();
    let tags_json = serde_json::to_string(&tags).ok();
    let tools_json = serde_json::to_string(&tools).ok();

    Some(UpsertSkill {
        id: skill_name.clone(),
        name: skill_name,
        description,
        category,
        subcategory: None,
        content: body,
        source_repo: Some("builtin".to_string()),
        parameters_json: tags_json,
        examples_json: tools_json,
    })
}

/// Parse a simple bracket list like `[a, b, c]` into a Vec<String>.
fn parse_bracket_list(s: &str) -> Vec<String> {
    let s = s.trim();
    let s = s.strip_prefix('[').unwrap_or(s);
    let s = s.strip_suffix(']').unwrap_or(s);
    s.split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn row_to_skill(row: &rusqlite::Row<'_>) -> Result<Skill, rusqlite::Error> {
    row_to_skill_from_offset(row, 0)
}

/// Parse a Skill from a row starting at the given column offset.
fn row_to_skill_from_offset(
    row: &rusqlite::Row<'_>,
    offset: usize,
) -> Result<Skill, rusqlite::Error> {
    let id: String = row.get(offset)?;
    let name: String = row.get(offset + 1)?;
    let description: Option<String> = row.get(offset + 2)?;
    let category: Option<String> = row.get(offset + 3)?;
    let subcategory: Option<String> = row.get(offset + 4)?;
    let content: String = row.get(offset + 5)?;
    let source_repo: Option<String> = row.get(offset + 6)?;
    let parameters_json: Option<String> = row.get(offset + 7)?;
    let examples_json: Option<String> = row.get(offset + 8)?;
    let usage_count: i32 = row.get(offset + 9)?;
    let created_at: String = row.get(offset + 10)?;
    let created_at = parse_sqlite_datetime(&created_at)?;
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

fn row_to_skill_rule(row: &rusqlite::Row<'_>) -> Result<SkillRule, rusqlite::Error> {
    Ok(SkillRule {
        id: row.get(0)?,
        skill_id: row.get(1)?,
        trigger_type: row.get(2)?,
        trigger_pattern: row.get(3)?,
        enabled: row.get::<_, i32>(4)? != 0,
        created_at: row.get(5)?,
    })
}

/// Parse a datetime string in either RFC3339 or SQLite `datetime('now')` format.
fn parse_sqlite_datetime(s: &str) -> Result<DateTime<Utc>, rusqlite::Error> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map(|ndt| ndt.and_utc())
        .map_err(|_| rusqlite::Error::InvalidParameterName(s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DbPool, Migrator};

    fn setup_test_db() -> Arc<Mutex<Connection>> {
        let db = DbPool::in_memory().unwrap();
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending().unwrap();
        drop(conn);
        db.conn_arc()
    }

    #[test]
    fn upsert_inserts_new_skill() {
        let conn = setup_test_db();
        let repo = SkillRepo::new(Arc::clone(&conn));

        let input = UpsertSkill {
            id: "test-skill".into(),
            name: "test-skill".into(),
            description: Some("A test skill".into()),
            category: Some("testing".into()),
            subcategory: None,
            content: "# Test\nBody content".into(),
            source_repo: Some("builtin".into()),
            parameters_json: Some(r#"["testing","quality"]"#.into()),
            examples_json: Some(r#"["Read","Grep"]"#.into()),
        };
        repo.upsert(&input).unwrap();

        let skill = repo.get("test-skill").unwrap();
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description.as_deref(), Some("A test skill"));
        assert_eq!(skill.category.as_deref(), Some("testing"));
        assert_eq!(skill.content, "# Test\nBody content");
        assert_eq!(skill.source_repo.as_deref(), Some("builtin"));
        assert_eq!(skill.usage_count, 0);
    }

    #[test]
    fn upsert_updates_existing_skill() {
        let conn = setup_test_db();
        let repo = SkillRepo::new(Arc::clone(&conn));

        let input = UpsertSkill {
            id: "update-me".into(),
            name: "update-me".into(),
            description: Some("Original".into()),
            category: None,
            subcategory: None,
            content: "original body".into(),
            source_repo: None,
            parameters_json: None,
            examples_json: None,
        };
        repo.upsert(&input).unwrap();

        let updated = UpsertSkill {
            id: "update-me".into(),
            name: "update-me".into(),
            description: Some("Updated".into()),
            category: Some("new-cat".into()),
            subcategory: None,
            content: "updated body".into(),
            source_repo: Some("builtin".into()),
            parameters_json: None,
            examples_json: None,
        };
        repo.upsert(&updated).unwrap();

        let skill = repo.get("update-me").unwrap();
        assert_eq!(skill.description.as_deref(), Some("Updated"));
        assert_eq!(skill.category.as_deref(), Some("new-cat"));
        assert_eq!(skill.content, "updated body");
    }

    #[test]
    fn upsert_preserves_usage_count() {
        let conn = setup_test_db();
        let repo = SkillRepo::new(Arc::clone(&conn));

        let input = UpsertSkill {
            id: "counter-test".into(),
            name: "counter-test".into(),
            description: None,
            category: None,
            subcategory: None,
            content: "body".into(),
            source_repo: None,
            parameters_json: None,
            examples_json: None,
        };
        repo.upsert(&input).unwrap();

        // Manually bump usage_count
        {
            let c = conn.lock().unwrap();
            c.execute(
                "UPDATE skills SET usage_count = 5 WHERE id = 'counter-test'",
                [],
            )
            .unwrap();
        }

        // Upsert again
        repo.upsert(&input).unwrap();

        let skill = repo.get("counter-test").unwrap();
        assert_eq!(skill.usage_count, 5);
    }

    #[test]
    fn parse_frontmatter_valid() {
        let raw = r#"---
name: code-review
description: Thorough code review methodology
tags: [review, quality, code]
tools: [Read, Grep, Glob]
---

# Code Review

Body content here.
"#;
        let result = parse_skill_frontmatter(raw).unwrap();
        assert_eq!(result.id, "code-review");
        assert_eq!(result.name, "code-review");
        assert_eq!(
            result.description.as_deref(),
            Some("Thorough code review methodology")
        );
        assert_eq!(result.category.as_deref(), Some("review"));
        assert_eq!(result.source_repo.as_deref(), Some("builtin"));
        assert!(result.content.contains("# Code Review"));
        assert!(result.content.contains("Body content here."));

        let tags: Vec<String> = serde_json::from_str(result.parameters_json.as_deref().unwrap()).unwrap();
        assert_eq!(tags, vec!["review", "quality", "code"]);

        let tools: Vec<String> = serde_json::from_str(result.examples_json.as_deref().unwrap()).unwrap();
        assert_eq!(tools, vec!["Read", "Grep", "Glob"]);
    }

    #[test]
    fn parse_frontmatter_no_delimiters() {
        assert!(parse_skill_frontmatter("no frontmatter here").is_none());
    }

    #[test]
    fn parse_frontmatter_no_name() {
        let raw = "---\ndescription: No name field\n---\nBody";
        assert!(parse_skill_frontmatter(raw).is_none());
    }

    #[test]
    fn parse_bracket_list_works() {
        assert_eq!(
            parse_bracket_list("[a, b, c]"),
            vec!["a", "b", "c"]
        );
        assert_eq!(
            parse_bracket_list("[single]"),
            vec!["single"]
        );
        assert!(parse_bracket_list("[]").is_empty());
    }

    fn make_temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("forge-skill-test-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn load_from_dir_loads_md_files() {
        let conn = setup_test_db();
        let repo = SkillRepo::new(Arc::clone(&conn));

        let dir = make_temp_dir("load");
        std::fs::write(
            dir.join("test-skill.md"),
            "---\nname: test-skill\ndescription: A test\ntags: [test]\ntools: [Read]\n---\n\n# Test\n\nBody.",
        )
        .unwrap();
        std::fs::write(
            dir.join("another.md"),
            "---\nname: another\ndescription: Another skill\ntags: [other]\ntools: []\n---\n\nContent.",
        )
        .unwrap();
        // Non-md file should be skipped
        std::fs::write(dir.join("ignore.txt"), "not a skill").unwrap();

        let count = repo.load_from_dir(&dir).unwrap();
        assert_eq!(count, 2);

        let skills = repo.list().unwrap();
        assert_eq!(skills.len(), 2);

        let skill = repo.get("test-skill").unwrap();
        assert_eq!(skill.description.as_deref(), Some("A test"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_from_dir_skips_invalid_frontmatter() {
        let conn = setup_test_db();
        let repo = SkillRepo::new(Arc::clone(&conn));

        let dir = make_temp_dir("skip");
        std::fs::write(
            dir.join("valid.md"),
            "---\nname: valid\ndescription: OK\ntags: []\ntools: []\n---\nBody",
        )
        .unwrap();
        std::fs::write(dir.join("invalid.md"), "no frontmatter").unwrap();

        let count = repo.load_from_dir(&dir).unwrap();
        assert_eq!(count, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- Skill rule tests ---

    /// Setup DB with skill_rules table for rule tests.
    fn setup_test_db_with_rules() -> Arc<Mutex<Connection>> {
        let db = DbPool::in_memory().unwrap();
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending().unwrap();
        // Create the skill_rules table (migration 0008 may not be wired yet)
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS skill_rules (
                id TEXT PRIMARY KEY,
                skill_id TEXT NOT NULL,
                trigger_type TEXT NOT NULL,
                trigger_pattern TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_skill_rules_skill ON skill_rules(skill_id);",
        )
        .unwrap();
        drop(conn);
        db.conn_arc()
    }

    fn insert_test_skill(conn: &Arc<Mutex<Connection>>, id: &str, name: &str) {
        let repo = SkillRepo::new(Arc::clone(conn));
        repo.upsert(&UpsertSkill {
            id: id.into(),
            name: name.into(),
            description: Some("test skill".into()),
            category: Some("testing".into()),
            subcategory: None,
            content: "# Test\nBody".into(),
            source_repo: Some("builtin".into()),
            parameters_json: None,
            examples_json: None,
        })
        .unwrap();
    }

    #[test]
    fn skill_rule_crud() {
        let conn = setup_test_db_with_rules();
        insert_test_skill(&conn, "sk-1", "test-skill");
        let repo = SkillRepo::new(Arc::clone(&conn));

        // Create rule
        let rule = repo.create_rule("sk-1", "keyword", "rust").unwrap();
        assert_eq!(rule.skill_id, "sk-1");
        assert_eq!(rule.trigger_type, "keyword");
        assert_eq!(rule.trigger_pattern, "rust");
        assert!(rule.enabled);

        // List rules
        let rules = repo.list_rules("sk-1").unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, rule.id);

        // Delete rule
        repo.delete_rule(&rule.id).unwrap();
        let rules = repo.list_rules("sk-1").unwrap();
        assert!(rules.is_empty());
    }

    #[test]
    fn skill_rule_matching_keyword() {
        let conn = setup_test_db_with_rules();
        insert_test_skill(&conn, "sk-rust", "rust-skill");
        let repo = SkillRepo::new(Arc::clone(&conn));

        repo.create_rule("sk-rust", "keyword", "rust").unwrap();

        let matched = repo.find_matching_rules("/tmp", "write rust code").unwrap();
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].id, "sk-rust");
    }

    #[test]
    fn auto_activation_no_match_returns_empty() {
        let conn = setup_test_db_with_rules();
        insert_test_skill(&conn, "sk-python", "python-skill");
        let repo = SkillRepo::new(Arc::clone(&conn));

        repo.create_rule("sk-python", "keyword", "python").unwrap();

        let matched = repo.find_matching_rules("/tmp", "write rust code").unwrap();
        assert!(matched.is_empty());
    }
}
