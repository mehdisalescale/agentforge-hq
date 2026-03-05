//! Analytics queries over sessions for cost tracking and projections.

use chrono::{Datelike, NaiveDate, Utc};
use forge_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize)]
pub struct AgentCostBreakdown {
    pub agent_id: String,
    pub total_cost: f64,
    pub session_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyCost {
    pub date: String,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionStats {
    pub total: i64,
    pub completed: i64,
    pub failed: i64,
    pub avg_cost: f64,
    pub p90_cost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageReport {
    pub total_cost: f64,
    pub daily_costs: Vec<DailyCost>,
    pub agent_breakdown: Vec<AgentCostBreakdown>,
    pub stats: SessionStats,
    pub projected_monthly_cost: f64,
}

pub struct AnalyticsRepo {
    conn: Arc<Mutex<Connection>>,
}

impl AnalyticsRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn total_cost(&self, start: &str, end: &str) -> ForgeResult<f64> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let cost: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(cost_usd), 0.0) FROM sessions WHERE created_at BETWEEN ?1 AND ?2",
                rusqlite::params![start, end],
                |row| row.get(0),
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(cost)
    }

    pub fn cost_by_agent(&self, start: &str, end: &str) -> ForgeResult<Vec<AgentCostBreakdown>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT agent_id, COALESCE(SUM(cost_usd), 0.0), COUNT(*)
                 FROM sessions WHERE created_at BETWEEN ?1 AND ?2
                 GROUP BY agent_id ORDER BY SUM(cost_usd) DESC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(rusqlite::params![start, end], |row| {
                Ok(AgentCostBreakdown {
                    agent_id: row.get(0)?,
                    total_cost: row.get(1)?,
                    session_count: row.get(2)?,
                })
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(rows)
    }

    pub fn daily_costs(&self, start: &str, end: &str) -> ForgeResult<Vec<DailyCost>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT date(created_at) AS day, COALESCE(SUM(cost_usd), 0.0)
                 FROM sessions WHERE created_at BETWEEN ?1 AND ?2
                 GROUP BY day ORDER BY day ASC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(rusqlite::params![start, end], |row| {
                Ok(DailyCost {
                    date: row.get(0)?,
                    cost: row.get(1)?,
                })
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(rows)
    }

    pub fn session_stats(&self, start: &str, end: &str) -> ForgeResult<SessionStats> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let (total, completed, failed, avg_cost): (i64, i64, i64, f64) = conn
            .query_row(
                "SELECT COUNT(*),
                    COALESCE(SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0),
                    COALESCE(AVG(cost_usd), 0.0)
                 FROM sessions WHERE created_at BETWEEN ?1 AND ?2",
                rusqlite::params![start, end],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let p90_cost: f64 = conn
            .query_row(
                "SELECT cost_usd FROM sessions
                 WHERE created_at BETWEEN ?1 AND ?2 AND cost_usd > 0
                 ORDER BY cost_usd ASC
                 LIMIT 1 OFFSET (
                     SELECT CAST(COUNT(*) * 0.9 AS INTEGER)
                     FROM sessions WHERE created_at BETWEEN ?3 AND ?4 AND cost_usd > 0
                 )",
                rusqlite::params![start, end, start, end],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        Ok(SessionStats { total, completed, failed, avg_cost, p90_cost })
    }

    pub fn projected_monthly_cost(&self) -> ForgeResult<f64> {
        let now = Utc::now();
        let first = format!("{}-{:02}-01T00:00:00", now.year(), now.month());
        let end = now.format("%Y-%m-%dT%H:%M:%S").to_string();
        let cost_so_far = self.total_cost(&first, &end)?;
        let days_elapsed = now.day() as f64;
        if days_elapsed < 1.0 {
            return Ok(0.0);
        }
        let dim = days_in_month(now.year(), now.month()) as f64;
        Ok((cost_so_far / days_elapsed) * dim)
    }

    pub fn usage_report(&self, start: &str, end: &str) -> ForgeResult<UsageReport> {
        Ok(UsageReport {
            total_cost: self.total_cost(start, end)?,
            daily_costs: self.daily_costs(start, end)?,
            agent_breakdown: self.cost_by_agent(start, end)?,
            stats: self.session_stats(start, end)?,
            projected_monthly_cost: self.projected_monthly_cost()?,
        })
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next.unwrap()
        .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL REFERENCES agents(id),
                claude_session_id TEXT,
                directory TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'created',
                cost_usd REAL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    fn insert_agent(conn: &Arc<Mutex<Connection>>, id: &str) {
        let db = conn.lock().unwrap();
        db.execute(
            "INSERT OR IGNORE INTO agents (id, name) VALUES (?1, ?2)",
            rusqlite::params![id, format!("agent-{id}")],
        )
        .unwrap();
    }

    fn insert_session(conn: &Arc<Mutex<Connection>>, agent_id: &str, status: &str, cost: f64, created_at: &str) {
        let db = conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        db.execute(
            "INSERT INTO sessions (id, agent_id, directory, status, cost_usd, created_at, updated_at)
             VALUES (?1, ?2, '/tmp', ?3, ?4, ?5, ?5)",
            rusqlite::params![id, agent_id, status, cost, created_at],
        )
        .unwrap();
    }

    #[test]
    fn total_cost_sums_correctly() {
        let conn = setup_db();
        insert_agent(&conn, "a1");
        insert_session(&conn, "a1", "completed", 1.50, "2026-03-01T10:00:00");
        insert_session(&conn, "a1", "completed", 2.50, "2026-03-02T10:00:00");
        insert_session(&conn, "a1", "completed", 3.00, "2026-03-03T10:00:00");
        let repo = AnalyticsRepo::new(conn);
        let total = repo.total_cost("2026-03-01T00:00:00", "2026-03-31T23:59:59").unwrap();
        assert!((total - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cost_by_agent_groups_correctly() {
        let conn = setup_db();
        insert_agent(&conn, "a1");
        insert_agent(&conn, "a2");
        insert_session(&conn, "a1", "completed", 1.00, "2026-03-01T10:00:00");
        insert_session(&conn, "a1", "completed", 2.00, "2026-03-02T10:00:00");
        insert_session(&conn, "a2", "completed", 5.00, "2026-03-01T10:00:00");
        let repo = AnalyticsRepo::new(conn);
        let bd = repo.cost_by_agent("2026-03-01T00:00:00", "2026-03-31T23:59:59").unwrap();
        assert_eq!(bd.len(), 2);
        assert_eq!(bd[0].agent_id, "a2");
        assert!((bd[0].total_cost - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn daily_costs_groups_by_date() {
        let conn = setup_db();
        insert_agent(&conn, "a1");
        insert_session(&conn, "a1", "completed", 1.00, "2026-03-01T08:00:00");
        insert_session(&conn, "a1", "completed", 2.00, "2026-03-01T16:00:00");
        insert_session(&conn, "a1", "completed", 3.00, "2026-03-02T10:00:00");
        let repo = AnalyticsRepo::new(conn);
        let daily = repo.daily_costs("2026-03-01T00:00:00", "2026-03-31T23:59:59").unwrap();
        assert_eq!(daily.len(), 2);
        assert_eq!(daily[0].date, "2026-03-01");
        assert!((daily[0].cost - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn session_stats_counts_statuses() {
        let conn = setup_db();
        insert_agent(&conn, "a1");
        insert_session(&conn, "a1", "completed", 2.00, "2026-03-01T10:00:00");
        insert_session(&conn, "a1", "completed", 4.00, "2026-03-02T10:00:00");
        insert_session(&conn, "a1", "failed", 1.00, "2026-03-03T10:00:00");
        insert_session(&conn, "a1", "running", 0.50, "2026-03-04T10:00:00");
        let repo = AnalyticsRepo::new(conn);
        let stats = repo.session_stats("2026-03-01T00:00:00", "2026-03-31T23:59:59").unwrap();
        assert_eq!(stats.total, 4);
        assert_eq!(stats.completed, 2);
        assert_eq!(stats.failed, 1);
    }

    #[test]
    fn p90_cost_calculates() {
        let conn = setup_db();
        insert_agent(&conn, "a1");
        for i in 1..=10 {
            insert_session(&conn, "a1", "completed", i as f64, &format!("2026-03-{:02}T10:00:00", i));
        }
        let repo = AnalyticsRepo::new(conn);
        let stats = repo.session_stats("2026-03-01T00:00:00", "2026-03-31T23:59:59").unwrap();
        assert!((stats.p90_cost - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_range_returns_zeros() {
        let conn = setup_db();
        let repo = AnalyticsRepo::new(conn);
        let total = repo.total_cost("2099-01-01T00:00:00", "2099-12-31T23:59:59").unwrap();
        assert!((total).abs() < f64::EPSILON);
        let stats = repo.session_stats("2099-01-01T00:00:00", "2099-12-31T23:59:59").unwrap();
        assert_eq!(stats.total, 0);
    }
}
