//! Schema migration runner.

use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use tracing::info;

const MIGRATION_001: &str = include_str!("../../../migrations/0001_init.sql");
const MIGRATION_002: &str = include_str!("../../../migrations/0002_add_cost.sql");
const MIGRATION_003: &str = include_str!("../../../migrations/0003_add_memory.sql");
const MIGRATION_004: &str = include_str!("../../../migrations/0004_add_hooks.sql");
const MIGRATION_005: &str = include_str!("../../../migrations/0005_scheduler_analytics.sql");

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
            applied += 1;
            info!("migration 0005 applied, now at version 5");
        }

        if applied == 0 {
            info!(version = current, "schema already at latest version");
        }

        Ok(applied)
    }
}
