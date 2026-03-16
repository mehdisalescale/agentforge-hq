//! Schema migration runner.

use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use tracing::info;

const MIGRATION_001: &str = include_str!("../../../migrations/0001_init.sql");
const MIGRATION_002: &str = include_str!("../../../migrations/0002_add_cost.sql");
const MIGRATION_003: &str = include_str!("../../../migrations/0003_add_memory.sql");
const MIGRATION_004: &str = include_str!("../../../migrations/0004_add_hooks.sql");
const MIGRATION_005: &str = include_str!("../../../migrations/0005_scheduler_analytics.sql");
const MIGRATION_006: &str = include_str!("../../../migrations/0006_add_compactions.sql");
const MIGRATION_007: &str = include_str!("../../../migrations/0007_add_workflow_columns.sql");
const MIGRATION_008: &str = include_str!("../../../migrations/0008_memory_types_and_skill_rules.sql");
const MIGRATION_009: &str = include_str!("../../../migrations/0009_personas.sql");
const MIGRATION_011: &str = include_str!("../../../migrations/0011_org_charts.sql");
const MIGRATION_012: &str = include_str!("../../../migrations/0012_agents_persona_id.sql");
const MIGRATION_013: &str = include_str!("../../../migrations/0013_safety_state.sql");
const MIGRATION_014: &str = include_str!("../../../migrations/0014_agents_backend_type.sql");

pub struct Migrator<'a> {
    conn: &'a Connection,
}

impl<'a> Migrator<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn current_version(&self) -> ForgeResult<u32> {
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if !exists {
            return Ok(0);
        }

        let version: u32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(version)
    }

    pub fn apply_pending(&self) -> ForgeResult<u32> {
        let mut current = self.current_version()?;
        let mut applied = 0;

        if current < 1 {
            info!("applying migration 0001_init.sql");
            self.conn
                .execute_batch(MIGRATION_001)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 1;
            applied += 1;
            info!("migration 0001 applied, now at version 1");
        }

        if current < 2 {
            info!("applying migration 0002_add_cost.sql");
            self.conn
                .execute_batch(MIGRATION_002)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 2;
            applied += 1;
            info!("migration 0002 applied, now at version 2");
        }

        if current < 3 {
            info!("applying migration 0003_add_memory.sql");
            self.conn
                .execute_batch(MIGRATION_003)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 3;
            applied += 1;
            info!("migration 0003 applied, now at version 3");
        }

        if current < 4 {
            info!("applying migration 0004_add_hooks.sql");
            self.conn
                .execute_batch(MIGRATION_004)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 4;
            applied += 1;
            info!("migration 0004 applied, now at version 4");
        }

        if current < 5 {
            info!("applying migration 0005_scheduler_analytics.sql");
            self.conn
                .execute_batch(MIGRATION_005)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 5;
            applied += 1;
            info!("migration 0005 applied, now at version 5");
        }

        if current < 6 {
            info!("applying migration 0006_add_compactions.sql");
            self.conn
                .execute_batch(MIGRATION_006)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 6;
            applied += 1;
            info!("migration 0006 applied, now at version 6");
        }

        if current < 7 {
            info!("applying migration 0007_add_workflow_columns.sql");
            self.conn
                .execute_batch(MIGRATION_007)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 7;
            applied += 1;
            info!("migration 0007 applied, now at version 7");
        }

        if current < 8 {
            info!("applying migration 0008_memory_types_and_skill_rules.sql");
            self.conn
                .execute_batch(MIGRATION_008)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 8;
            applied += 1;
            info!("migration 0008 applied, now at version 8");
        }

        if current < 9 {
            info!("applying migration 0009_personas.sql");
            self.conn
                .execute_batch(MIGRATION_009)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 9;
            applied += 1;
            info!("migration 0009 applied, now at version 9");
        }

        if current < 11 {
            info!("applying migration 0011_org_charts.sql");
            self.conn
                .execute_batch(MIGRATION_011)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 11;
            applied += 1;
            info!("migration 0011 applied, now at version 11");
        }

        if current < 12 {
            info!("applying migration 0012_agents_persona_id.sql");
            self.conn
                .execute_batch(MIGRATION_012)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 12;
            applied += 1;
            info!("migration 0012 applied, now at version 12");
        }

        if current < 13 {
            info!("applying migration 0013_safety_state.sql");
            self.conn
                .execute_batch(MIGRATION_013)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 13;
            applied += 1;
            info!("migration 0013 applied, now at version 13");
        }

        if current < 14 {
            info!("applying migration 0014_agents_backend_type.sql");
            self.conn
                .execute_batch(MIGRATION_014)
                .map_err(|e| ForgeError::Database(Box::new(e)))?;
            current = 14;
            applied += 1;
            info!("migration 0014 applied, now at version 14");
        }

        if applied == 0 {
            info!(version = current, "schema already at latest version");
        }

        Ok(applied)
    }
}
