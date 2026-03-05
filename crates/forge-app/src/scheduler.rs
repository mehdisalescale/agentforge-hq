//! Background scheduler: checks enabled schedules every 60s, fires due ones.

use chrono::Utc;
use forge_db::repos::schedules::ScheduleRepo;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

/// Spawn the scheduler background task.
/// Returns a JoinHandle that can be awaited for clean shutdown.
pub fn spawn(
    schedule_repo: Arc<ScheduleRepo>,
    cancel: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        info!("scheduler started");
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    info!("scheduler shutting down");
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                    tick(&schedule_repo).await;
                }
            }
        }
    })
}

async fn tick(repo: &ScheduleRepo) {
    let schedules = match repo.list_enabled() {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "scheduler: failed to list enabled schedules");
            return;
        }
    };

    let now = Utc::now();
    let now_str = now.format("%Y-%m-%dT%H:%M:%S").to_string();

    for schedule in &schedules {
        let is_due = schedule
            .next_run_at
            .as_deref()
            .map(|next| next <= now_str.as_str())
            .unwrap_or(false);

        if !is_due {
            continue;
        }

        info!(
            schedule_id = %schedule.id,
            name = %schedule.name,
            "scheduler: triggering schedule"
        );

        // Update last_run and advance next_run.
        if let Err(e) = repo.update_last_run(&schedule.id) {
            warn!(
                schedule_id = %schedule.id,
                error = %e,
                "scheduler: failed to update_last_run"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_db::repos::schedules::{NewSchedule, ScheduleRepo};
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn setup_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS schedules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                cron_expr TEXT NOT NULL,
                job_type TEXT NOT NULL DEFAULT 'agent',
                job_config_json TEXT NOT NULL DEFAULT '{}',
                agent_id TEXT REFERENCES agents(id),
                prompt TEXT,
                directory TEXT DEFAULT '.',
                enabled BOOLEAN NOT NULL DEFAULT 1,
                last_run_at TEXT,
                next_run_at TEXT,
                run_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO agents (id, name) VALUES ('agent-1', 'test-agent');",
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[tokio::test]
    async fn tick_triggers_due_schedule() {
        let conn = setup_db();
        let repo = Arc::new(ScheduleRepo::new(Arc::clone(&conn)));
        let s = repo
            .create(&NewSchedule {
                name: "due-now".into(),
                cron_expr: "* * * * * * *".into(), // every second
                agent_id: "agent-1".into(),
                prompt: "test".into(),
                directory: ".".into(),
            })
            .unwrap();
        assert_eq!(s.run_count, 0);

        // Set next_run_at to the past so tick sees it as due.
        {
            let db = conn.lock().unwrap();
            db.execute(
                "UPDATE schedules SET next_run_at = '2020-01-01T00:00:00' WHERE id = ?1",
                rusqlite::params![s.id],
            )
            .unwrap();
        }

        tick(&repo).await;

        let updated = repo.get(&s.id).unwrap().unwrap();
        assert_eq!(updated.run_count, 1);
    }

    #[tokio::test]
    async fn tick_skips_disabled() {
        let conn = setup_db();
        let repo = Arc::new(ScheduleRepo::new(Arc::clone(&conn)));
        let s = repo
            .create(&NewSchedule {
                name: "disabled".into(),
                cron_expr: "* * * * * * *".into(),
                agent_id: "agent-1".into(),
                prompt: "test".into(),
                directory: ".".into(),
            })
            .unwrap();
        repo.update(
            &s.id,
            &forge_db::repos::schedules::UpdateSchedule {
                name: None,
                cron_expr: None,
                prompt: None,
                directory: None,
                enabled: Some(false),
            },
        )
        .unwrap();

        tick(&repo).await;

        let after = repo.get(&s.id).unwrap().unwrap();
        assert_eq!(after.run_count, 0);
    }

    #[tokio::test]
    async fn tick_skips_future_schedule() {
        let conn = setup_db();
        let repo = Arc::new(ScheduleRepo::new(Arc::clone(&conn)));
        // Schedule for far future (year 2099 at 9am)
        let s = repo
            .create(&NewSchedule {
                name: "future".into(),
                cron_expr: "0 0 9 1 1 * 2099".into(),
                agent_id: "agent-1".into(),
                prompt: "test".into(),
                directory: ".".into(),
            })
            .unwrap();

        tick(&repo).await;

        let after = repo.get(&s.id).unwrap().unwrap();
        assert_eq!(after.run_count, 0);
    }
}
