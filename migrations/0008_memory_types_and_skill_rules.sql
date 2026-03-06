-- Migration 0008: Add memory_type column and skill_rules table

-- Add memory_type column (default 'personal' for existing rows)
ALTER TABLE memory ADD COLUMN memory_type TEXT NOT NULL DEFAULT 'personal';

-- Skill activation rules
CREATE TABLE IF NOT EXISTS skill_rules (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    trigger_pattern TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_skill_rules_skill ON skill_rules(skill_id);

INSERT INTO schema_version (version) VALUES (8);
