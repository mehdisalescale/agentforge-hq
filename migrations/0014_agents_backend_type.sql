-- Add backend_type column to agents table.
-- Default 'claude' for backward compatibility with all existing agents.
ALTER TABLE agents ADD COLUMN backend_type TEXT NOT NULL DEFAULT 'claude';

INSERT INTO schema_version (version) VALUES (14);
