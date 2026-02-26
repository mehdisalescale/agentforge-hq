-- Add cost tracking to sessions (from Claude CLI result cost_usd).
ALTER TABLE sessions ADD COLUMN cost_usd REAL DEFAULT 0.0;

INSERT INTO schema_version (version) VALUES (2);
