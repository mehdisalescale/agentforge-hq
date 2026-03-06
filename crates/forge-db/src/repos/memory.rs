//! Memory CRUD repository — cross-session memory facts.

use chrono::{DateTime, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// A persisted memory fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub category: String,
    pub content: String,
    pub confidence: f64,
    pub source_session_id: Option<String>,
    pub memory_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new memory.
#[derive(Debug, Deserialize)]
pub struct NewMemory {
    pub category: Option<String>,
    pub content: String,
    pub confidence: Option<f64>,
    pub source_session_id: Option<String>,
}

/// Input for updating an existing memory.
#[derive(Debug, Deserialize)]
pub struct UpdateMemory {
    pub content: Option<String>,
    pub category: Option<String>,
    pub confidence: Option<f64>,
}

/// A fact extracted from a session transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFact {
    pub category: String,
    pub content: String,
    pub confidence: f64,
}

/// Stopwords filtered out during keyword extraction.
const STOPWORDS: &[&str] = &[
    "the", "a", "an", "is", "are", "was", "were", "to", "in", "for", "of", "and", "or", "it",
    "this", "that", "with",
];

/// Classify an extracted fact into a memory type based on content keywords.
/// Returns one of: "personal", "tool", or "task".
pub fn classify_fact(fact: &ExtractedFact) -> &'static str {
    let content_lower = fact.content.to_lowercase();
    if content_lower.contains("prefers")
        || content_lower.contains("always")
        || content_lower.contains("style")
    {
        "personal"
    } else if content_lower.contains("tool")
        || content_lower.contains("command")
        || content_lower.contains("cli")
        || content_lower.contains("api")
    {
        "tool"
    } else {
        "task"
    }
}

pub struct MemoryRepo {
    conn: Arc<Mutex<Connection>>,
}

impl MemoryRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, input: &NewMemory) -> ForgeResult<Memory> {
        self.create_with_type(input, "personal")
    }

    /// Create a new memory with an explicit memory_type.
    pub fn create_with_type(&self, input: &NewMemory, memory_type: &str) -> ForgeResult<Memory> {
        if input.content.trim().is_empty() {
            return Err(ForgeError::Validation("content cannot be empty".into()));
        }
        let conn = self.conn.lock().expect("db mutex poisoned");
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let category = input.category.as_deref().unwrap_or("general");
        let confidence = input.confidence.unwrap_or(0.5);

        // Try inserting with memory_type column; fall back to without if column doesn't exist yet
        let result = conn.execute(
            "INSERT INTO memory (id, category, content, confidence, source_session_id, memory_type, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                id,
                category,
                input.content,
                confidence,
                input.source_session_id,
                memory_type,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        );

        match result {
            Ok(_) => {}
            Err(ref e) if e.to_string().contains("memory_type") => {
                // Column doesn't exist yet — insert without it
                conn.execute(
                    "INSERT INTO memory (id, category, content, confidence, source_session_id, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        id,
                        category,
                        input.content,
                        confidence,
                        input.source_session_id,
                        now.to_rfc3339(),
                        now.to_rfc3339(),
                    ],
                )
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            }
            Err(e) => return Err(ForgeError::Database(Box::new(e))),
        }

        drop(conn);
        self.get(&id)
    }

    pub fn get(&self, id: &str) -> ForgeResult<Memory> {
        let conn = self.conn.lock().expect("db mutex poisoned");

        // Try with memory_type column first; fall back without it
        let sql = if has_memory_type_column(&conn) {
            "SELECT id, category, content, confidence, source_session_id, created_at, updated_at, memory_type
             FROM memory WHERE id = ?1"
        } else {
            "SELECT id, category, content, confidence, source_session_id, created_at, updated_at
             FROM memory WHERE id = ?1"
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        stmt.query_row(rusqlite::params![id], |row| {
            row_to_memory(row)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                ForgeError::Internal(format!("memory not found: {}", id))
            }
            rusqlite::Error::InvalidParameterName(s) => ForgeError::Validation(s),
            other => ForgeError::Database(Box::new(other)),
        })
    }

    pub fn list(&self, limit: i64, offset: i64) -> ForgeResult<Vec<Memory>> {
        let conn = self.conn.lock().expect("db mutex poisoned");

        let sql = if has_memory_type_column(&conn) {
            "SELECT id, category, content, confidence, source_session_id, created_at, updated_at, memory_type
             FROM memory ORDER BY updated_at DESC LIMIT ?1 OFFSET ?2"
        } else {
            "SELECT id, category, content, confidence, source_session_id, created_at, updated_at
             FROM memory ORDER BY updated_at DESC LIMIT ?1 OFFSET ?2"
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let memories: Vec<Memory> = stmt
            .query_map(rusqlite::params![limit, offset], |row| {
                row_to_memory(row)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| match e {
                rusqlite::Error::InvalidParameterName(s) => ForgeError::Validation(s),
                other => ForgeError::Database(Box::new(other)),
            })?;

        Ok(memories)
    }

    pub fn update(&self, id: &str, input: &UpdateMemory) -> ForgeResult<Memory> {
        let existing = self.get(id)?;
        let now = Utc::now();

        let content = input.content.as_deref().unwrap_or(&existing.content);
        if content.trim().is_empty() {
            return Err(ForgeError::Validation("content cannot be empty".into()));
        }
        let category = input.category.as_deref().unwrap_or(&existing.category);
        let confidence = input.confidence.unwrap_or(existing.confidence);

        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "UPDATE memory SET content = ?1, category = ?2, confidence = ?3, updated_at = ?4 WHERE id = ?5",
            rusqlite::params![content, category, confidence, now.to_rfc3339(), id],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;

        drop(conn);
        self.get(id)
    }

    pub fn delete(&self, id: &str) -> ForgeResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let rows = conn
            .execute("DELETE FROM memory WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if rows == 0 {
            return Err(ForgeError::Internal(format!("memory not found: {}", id)));
        }

        Ok(())
    }

    /// Extract facts from a session transcript.
    /// Parses structured patterns like key decisions, error solutions,
    /// codebase patterns, and user preferences. No LLM — pattern-based only.
    pub fn extract_facts(transcript: &[String]) -> Vec<ExtractedFact> {
        let mut facts = Vec::new();

        let decision_patterns = ["decided to", "chose", "prefer", "went with", "selected"];
        let solution_patterns = ["fixed by", "solved by", "the fix was", "resolved by", "the solution was"];
        let pattern_patterns = ["pattern:", "convention:", "always", "never", "rule:"];
        let codebase_patterns = ["/src/", "/crates/", ".rs", ".ts", ".svelte"];

        for line in transcript {
            let lower = line.to_lowercase();

            if decision_patterns.iter().any(|p| lower.contains(p)) {
                facts.push(ExtractedFact {
                    category: "decisions".into(),
                    content: line.trim().to_string(),
                    confidence: 0.8,
                });
            }

            if solution_patterns.iter().any(|p| lower.contains(p)) {
                facts.push(ExtractedFact {
                    category: "solutions".into(),
                    content: line.trim().to_string(),
                    confidence: 0.9,
                });
            }

            if pattern_patterns.iter().any(|p| lower.contains(p)) {
                facts.push(ExtractedFact {
                    category: "patterns".into(),
                    content: line.trim().to_string(),
                    confidence: 0.7,
                });
            }

            if codebase_patterns.iter().any(|p| lower.contains(p)) && lower.contains(' ') {
                facts.push(ExtractedFact {
                    category: "codebase".into(),
                    content: line.trim().to_string(),
                    confidence: 0.6,
                });
            }
        }

        facts
    }

    /// Store extracted facts from a completed session.
    /// Deduplicates against existing memories by content similarity (LIKE match).
    /// Classifies each fact into a memory_type via `classify_fact()`.
    /// Returns the number of new/updated memories.
    pub fn store_extracted(&self, facts: &[ExtractedFact], session_id: &str) -> ForgeResult<usize> {
        let mut stored = 0;

        for fact in facts {
            let mem_type = classify_fact(fact);
            let existing = self.search(&fact.content)?;

            let similar = existing.iter().find(|m| {
                m.category == fact.category
                    && (m.content.contains(&fact.content) || fact.content.contains(&m.content))
            });

            match similar {
                Some(existing_mem) if existing_mem.confidence >= fact.confidence => {
                    // Existing has equal or higher confidence — skip
                }
                Some(existing_mem) => {
                    // Existing has lower confidence — update
                    self.update(
                        &existing_mem.id,
                        &UpdateMemory {
                            content: Some(fact.content.clone()),
                            category: Some(fact.category.clone()),
                            confidence: Some(fact.confidence),
                        },
                    )?;
                    stored += 1;
                }
                None => {
                    // No similar — create new with classified type
                    self.create_with_type(
                        &NewMemory {
                            category: Some(fact.category.clone()),
                            content: fact.content.clone(),
                            confidence: Some(fact.confidence),
                            source_session_id: Some(session_id.to_string()),
                        },
                        mem_type,
                    )?;
                    stored += 1;
                }
            }
        }

        Ok(stored)
    }

    /// Find memories relevant to a prompt via keyword overlap.
    /// Returns a formatted context block for prepending to a system prompt,
    /// or None if no relevant memories are found.
    ///
    /// Priority weighting by memory_type:
    /// - task memories: weight 3x (retrieve up to 3x as many)
    /// - tool memories: weight 2x
    /// - personal memories: weight 1x
    ///
    /// Total still limited by max_memories param.
    pub fn inject_context(&self, prompt: &str, max_memories: usize) -> ForgeResult<Option<String>> {
        let keywords: Vec<String> = prompt
            .split_whitespace()
            .map(|w| w.to_lowercase().replace(|c: char| !c.is_alphanumeric(), ""))
            .filter(|w| w.len() > 1 && !STOPWORDS.contains(&w.as_str()))
            .collect();

        if keywords.is_empty() {
            return Ok(None);
        }

        let mut matched: Vec<Memory> = Vec::new();
        for keyword in &keywords {
            if let Ok(results) = self.search(keyword) {
                for mem in results {
                    if !matched.iter().any(|m| m.id == mem.id) {
                        matched.push(mem);
                    }
                }
            }
        }

        if matched.is_empty() {
            return Ok(None);
        }

        // Sort by weighted score: type weight * confidence, descending.
        // task=3x, tool=2x, personal=1x
        matched.sort_by(|a, b| {
            let weight = |m: &Memory| -> f64 {
                let w = match m.memory_type.as_str() {
                    "task" => 3.0,
                    "tool" => 2.0,
                    _ => 1.0,
                };
                w * m.confidence
            };
            weight(b)
                .partial_cmp(&weight(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matched.truncate(max_memories);

        let mut block = String::from("## Relevant Context (from previous sessions)\n\n");
        for mem in &matched {
            block.push_str(&format!("- [{}] {}\n", mem.category, mem.content));
        }

        Ok(Some(block))
    }

    /// Simple LIKE-based search across content and category.
    pub fn search(&self, query: &str) -> ForgeResult<Vec<Memory>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let pattern = format!("%{}%", query);

        let sql = if has_memory_type_column(&conn) {
            "SELECT id, category, content, confidence, source_session_id, created_at, updated_at, memory_type
             FROM memory
             WHERE content LIKE ?1 OR category LIKE ?1
             ORDER BY confidence DESC, updated_at DESC"
        } else {
            "SELECT id, category, content, confidence, source_session_id, created_at, updated_at
             FROM memory
             WHERE content LIKE ?1 OR category LIKE ?1
             ORDER BY confidence DESC, updated_at DESC"
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let memories: Vec<Memory> = stmt
            .query_map(rusqlite::params![pattern], |row| {
                row_to_memory(row)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| match e {
                rusqlite::Error::InvalidParameterName(s) => ForgeError::Validation(s),
                other => ForgeError::Database(Box::new(other)),
            })?;

        Ok(memories)
    }

    /// Search memories filtered by type.
    pub fn search_by_type(&self, memory_type: &str, query: &str) -> ForgeResult<Vec<Memory>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let pattern = format!("%{}%", query);
        let mut stmt = conn
            .prepare(
                "SELECT id, category, content, confidence, source_session_id, created_at, updated_at, memory_type
                 FROM memory
                 WHERE memory_type = ?1 AND content LIKE ?2
                 ORDER BY confidence DESC
                 LIMIT 20",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let memories: Vec<Memory> = stmt
            .query_map(rusqlite::params![memory_type, pattern], |row| {
                row_to_memory(row)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| match e {
                rusqlite::Error::InvalidParameterName(s) => ForgeError::Validation(s),
                other => ForgeError::Database(Box::new(other)),
            })?;

        Ok(memories)
    }
}

/// Check if the memory table has the memory_type column.
fn has_memory_type_column(conn: &Connection) -> bool {
    conn.prepare("SELECT memory_type FROM memory LIMIT 0")
        .is_ok()
}

fn row_to_memory(row: &rusqlite::Row<'_>) -> Result<Memory, ForgeError> {
    let id: String = row.get(0).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let category: String = row.get(1).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let content: String = row.get(2).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let confidence: f64 = row.get(3).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let source_session_id: Option<String> = row.get(4).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let created_at: String = row.get(5).map_err(|e| ForgeError::Database(Box::new(e)))?;
    let updated_at: String = row.get(6).map_err(|e| ForgeError::Database(Box::new(e)))?;

    // Column 7 is memory_type if present; default to "personal"
    let memory_type: String = row.get(7).unwrap_or_else(|_| "personal".to_string());

    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .map_err(|_| ForgeError::Validation(format!("invalid timestamp: {}", created_at)))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
        .map_err(|_| ForgeError::Validation(format!("invalid timestamp: {}", updated_at)))?
        .with_timezone(&Utc);

    Ok(Memory {
        id,
        category,
        content,
        confidence,
        source_session_id,
        memory_type,
        created_at,
        updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memory (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL DEFAULT 'general',
                content TEXT NOT NULL,
                confidence REAL NOT NULL DEFAULT 0.5,
                source_session_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_memory_category ON memory(category);",
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn create_and_get_memory() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        let mem = repo
            .create(&NewMemory {
                category: Some("patterns".into()),
                content: "Always use ForgeResult for error handling".into(),
                confidence: Some(0.9),
                source_session_id: Some("sess-123".into()),
            })
            .unwrap();

        assert_eq!(mem.category, "patterns");
        assert_eq!(mem.content, "Always use ForgeResult for error handling");
        assert!((mem.confidence - 0.9).abs() < f64::EPSILON);
        assert_eq!(mem.source_session_id, Some("sess-123".into()));

        let fetched = repo.get(&mem.id).unwrap();
        assert_eq!(fetched.id, mem.id);
    }

    #[test]
    fn create_with_defaults() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        let mem = repo
            .create(&NewMemory {
                category: None,
                content: "Some fact".into(),
                confidence: None,
                source_session_id: None,
            })
            .unwrap();

        assert_eq!(mem.category, "general");
        assert!((mem.confidence - 0.5).abs() < f64::EPSILON);
        assert_eq!(mem.source_session_id, None);
    }

    #[test]
    fn create_empty_content_fails() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        let result = repo.create(&NewMemory {
            category: None,
            content: "  ".into(),
            confidence: None,
            source_session_id: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn list_with_pagination() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        for i in 0..5 {
            repo.create(&NewMemory {
                category: None,
                content: format!("fact {}", i),
                confidence: None,
                source_session_id: None,
            })
            .unwrap();
        }

        let page1 = repo.list(2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = repo.list(2, 2).unwrap();
        assert_eq!(page2.len(), 2);

        let page3 = repo.list(2, 4).unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[test]
    fn update_memory() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        let mem = repo
            .create(&NewMemory {
                category: Some("general".into()),
                content: "original".into(),
                confidence: Some(0.5),
                source_session_id: None,
            })
            .unwrap();

        let updated = repo
            .update(
                &mem.id,
                &UpdateMemory {
                    content: Some("updated content".into()),
                    category: Some("patterns".into()),
                    confidence: Some(0.95),
                },
            )
            .unwrap();

        assert_eq!(updated.content, "updated content");
        assert_eq!(updated.category, "patterns");
        assert!((updated.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn delete_memory() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        let mem = repo
            .create(&NewMemory {
                category: None,
                content: "to be deleted".into(),
                confidence: None,
                source_session_id: None,
            })
            .unwrap();

        repo.delete(&mem.id).unwrap();
        assert!(repo.get(&mem.id).is_err());
    }

    #[test]
    fn delete_nonexistent_fails() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        let result = repo.delete("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn extract_facts_finds_decisions() {
        let transcript = vec![
            "Looking at the options...".into(),
            "We decided to use Axum for the HTTP layer".into(),
            "That should work well.".into(),
        ];
        let facts = MemoryRepo::extract_facts(&transcript);
        assert!(!facts.is_empty());
        let decision = facts.iter().find(|f| f.category == "decisions").unwrap();
        assert!(decision.content.contains("Axum"));
        assert!((decision.confidence - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn extract_facts_finds_solutions() {
        let transcript = vec![
            "The test was failing because of a lifetime issue.".into(),
            "Fixed by adding an explicit lifetime parameter to the struct.".into(),
        ];
        let facts = MemoryRepo::extract_facts(&transcript);
        let solution = facts.iter().find(|f| f.category == "solutions").unwrap();
        assert!(solution.content.contains("lifetime"));
        assert!((solution.confidence - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn extract_facts_empty_transcript() {
        let facts = MemoryRepo::extract_facts(&[]);
        assert!(facts.is_empty());
    }

    #[test]
    fn store_extracted_deduplicates() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        let facts = vec![ExtractedFact {
            category: "decisions".into(),
            content: "decided to use Axum".into(),
            confidence: 0.8,
        }];

        let stored1 = repo.store_extracted(&facts, "sess-1").unwrap();
        assert_eq!(stored1, 1);

        // Same fact again — should be skipped (same confidence)
        let stored2 = repo.store_extracted(&facts, "sess-2").unwrap();
        assert_eq!(stored2, 0);

        // Only 1 memory in DB
        let all = repo.list(100, 0).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn store_extracted_updates_higher_confidence() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        let low = vec![ExtractedFact {
            category: "decisions".into(),
            content: "decided to use Axum".into(),
            confidence: 0.5,
        }];
        repo.store_extracted(&low, "sess-1").unwrap();

        let high = vec![ExtractedFact {
            category: "decisions".into(),
            content: "decided to use Axum".into(),
            confidence: 0.9,
        }];
        let updated = repo.store_extracted(&high, "sess-2").unwrap();
        assert_eq!(updated, 1);

        let all = repo.list(100, 0).unwrap();
        assert_eq!(all.len(), 1);
        assert!((all[0].confidence - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn inject_context_returns_relevant() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        repo.create(&NewMemory {
            category: Some("decisions".into()),
            content: "Use tokio for async runtime".into(),
            confidence: Some(0.8),
            source_session_id: None,
        })
        .unwrap();

        let ctx = repo.inject_context("how does tokio work", 5).unwrap();
        assert!(ctx.is_some());
        let block = ctx.unwrap();
        assert!(block.contains("Relevant Context"));
        assert!(block.contains("tokio"));
    }

    #[test]
    fn inject_context_returns_none_when_empty() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        let ctx = repo.inject_context("something random", 5).unwrap();
        assert!(ctx.is_none());
    }

    #[test]
    fn inject_context_respects_max() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        for i in 0..5 {
            repo.create(&NewMemory {
                category: Some("patterns".into()),
                content: format!("rust pattern number {}", i),
                confidence: Some(0.5 + (i as f64) * 0.1),
                source_session_id: None,
            })
            .unwrap();
        }

        let ctx = repo.inject_context("rust pattern", 2).unwrap();
        assert!(ctx.is_some());
        let block = ctx.unwrap();
        // Count bullet lines
        let bullet_count = block.lines().filter(|l| l.starts_with("- ")).count();
        assert_eq!(bullet_count, 2);
    }

    // --- Typed memory tests ---

    /// Setup a DB with the memory_type column for typed-memory tests.
    fn setup_db_with_memory_type() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memory (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL DEFAULT 'general',
                content TEXT NOT NULL,
                confidence REAL NOT NULL DEFAULT 0.5,
                source_session_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                memory_type TEXT NOT NULL DEFAULT 'personal'
            );
            CREATE INDEX IF NOT EXISTS idx_memory_category ON memory(category);",
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn classify_fact_detects_personal() {
        let fact = ExtractedFact {
            category: "patterns".into(),
            content: "user prefers dark mode".into(),
            confidence: 0.8,
        };
        assert_eq!(classify_fact(&fact), "personal");
    }

    #[test]
    fn classify_fact_detects_tool() {
        let fact = ExtractedFact {
            category: "solutions".into(),
            content: "use cargo test command".into(),
            confidence: 0.9,
        };
        assert_eq!(classify_fact(&fact), "tool");
    }

    #[test]
    fn classify_fact_detects_task() {
        let fact = ExtractedFact {
            category: "decisions".into(),
            content: "implement the login page".into(),
            confidence: 0.7,
        };
        assert_eq!(classify_fact(&fact), "task");
    }

    #[test]
    fn memory_type_filtering() {
        let conn = setup_db_with_memory_type();
        let repo = MemoryRepo::new(conn);

        // Store a personal memory
        repo.create_with_type(
            &NewMemory {
                category: Some("patterns".into()),
                content: "user prefers dark mode theme".into(),
                confidence: Some(0.8),
                source_session_id: None,
            },
            "personal",
        )
        .unwrap();

        // Store a tool memory
        repo.create_with_type(
            &NewMemory {
                category: Some("solutions".into()),
                content: "use cargo test for testing theme issues".into(),
                confidence: Some(0.9),
                source_session_id: None,
            },
            "tool",
        )
        .unwrap();

        // Store a task memory
        repo.create_with_type(
            &NewMemory {
                category: Some("decisions".into()),
                content: "implement the theme switcher".into(),
                confidence: Some(0.7),
                source_session_id: None,
            },
            "task",
        )
        .unwrap();

        // search_by_type for "personal" with "theme" query
        let personal = repo.search_by_type("personal", "theme").unwrap();
        assert_eq!(personal.len(), 1);
        assert!(personal[0].content.contains("dark mode"));

        // search_by_type for "tool" with "theme" query
        let tool = repo.search_by_type("tool", "theme").unwrap();
        assert_eq!(tool.len(), 1);
        assert!(tool[0].content.contains("cargo test"));

        // search_by_type for "task" with "theme" query
        let task = repo.search_by_type("task", "theme").unwrap();
        assert_eq!(task.len(), 1);
        assert!(task[0].content.contains("theme switcher"));

        // search_by_type with non-matching type returns empty
        let empty = repo.search_by_type("personal", "nonexistent").unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn search_memory() {
        let conn = setup_db();
        let repo = MemoryRepo::new(conn);

        repo.create(&NewMemory {
            category: Some("rust".into()),
            content: "Use tokio for async runtime".into(),
            confidence: Some(0.9),
            source_session_id: None,
        })
        .unwrap();

        repo.create(&NewMemory {
            category: Some("python".into()),
            content: "Use asyncio for async runtime".into(),
            confidence: Some(0.8),
            source_session_id: None,
        })
        .unwrap();

        repo.create(&NewMemory {
            category: Some("rust".into()),
            content: "Use serde for serialization".into(),
            confidence: Some(0.7),
            source_session_id: None,
        })
        .unwrap();

        // Search by content
        let results = repo.search("async").unwrap();
        assert_eq!(results.len(), 2);
        // Ordered by confidence DESC
        assert!((results[0].confidence - 0.9).abs() < f64::EPSILON);

        // Search by category
        let results = repo.search("rust").unwrap();
        assert_eq!(results.len(), 2);

        // No results
        let results = repo.search("javascript").unwrap();
        assert!(results.is_empty());
    }
}
