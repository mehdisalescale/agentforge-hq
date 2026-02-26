//! Schema migration runner.

use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use tracing::info;

const MIGRATION_SQL: &str = include_str!("../../../migrations/0001_init.sql");

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
        let current = self.current_version()?;

        if current >= 1 {
            info!(version = current, "schema already at latest version");
            return Ok(0);
        }

        info!("applying migration 0001_init.sql");
        self.conn
            .execute_batch(MIGRATION_SQL)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        info!("migration applied, now at version 1");

        Ok(1)
    }
}
