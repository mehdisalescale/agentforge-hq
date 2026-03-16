//! Forge binary: DB, migrations, EventBus, BatchWriter, AgentRepo, API server (FORGE_HOST:FORGE_PORT).
//! Graceful shutdown on Ctrl+C: server stops accepting, then BatchWriter flushes and exits.

use forge_api::state::SafetyState;
use forge_api::{serve_until_signal, AppState};
use forge_core::EventBus;
use forge_db::{BatchWriter, DbPool, Migrator, UnitOfWork};

mod scheduler;
use forge_agent::model::NewAgent;
use forge_agent::preset::AgentPreset;
use forge_db::{
    AgentRepo, ApprovalRepo, CompanyRepo, DepartmentRepo, GoalRepo, OrgPositionRepo,
};
use forge_persona::parser::PersonaParser;
use forge_persona::model::{Persona, PersonaDivision, PersonaDivisionId};
use forge_process::{BackendRegistry, ClaudeBackend};
use forge_safety::{CircuitBreaker, CostTracker, RateLimiter};
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    info!("shutdown signal received");
}

fn default_db_path() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let new_path = format!("{}/.agentforge/forge.db", home);
    let legacy_path = format!("{}/.claude-forge/forge.db", home);

    // Graceful migration: use legacy path if it exists and new path doesn't
    if !std::path::Path::new(&new_path).exists() && std::path::Path::new(&legacy_path).exists() {
        eprintln!("Note: Found database at legacy path ~/.claude-forge/forge.db");
        eprintln!("      Consider moving to ~/.agentforge/forge.db");
        return legacy_path;
    }
    new_path
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    if cfg!(not(debug_assertions)) {
        // Production: JSON structured logging for machine parsing
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .init();
    } else {
        // Development: human-readable format
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    let db_path = env::var("FORGE_DB_PATH").unwrap_or_else(|_| default_db_path());
    let path = Path::new(&db_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    info!(path = %db_path, "opening database");
    let db = DbPool::new(path)?;
    {
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        let applied = migrator.apply_pending()?;
        if applied > 0 {
            info!(count = applied, "migrations applied");
        }
    }

    let db = Arc::new(db);
    let uow = Arc::new(UnitOfWork::new(Arc::clone(&db)));

    let (event_bus, persist_rx) = EventBus::new(1024, 256);

    // Load seed skills from the skills/ directory.
    if let Err(e) = uow.skill_repo.load_from_dir(std::path::Path::new("skills")) {
        tracing::warn!("skill loading failed: {}", e);
    }

    // Seed persona catalog from personas/ directory.
    seed_personas(&uow.persona_repo);

    // Seed demo data on first launch so pages aren't empty.
    seed_demo_data(
        &uow.company_repo,
        &uow.department_repo,
        &uow.agent_repo,
        &uow.org_position_repo,
        &uow.goal_repo,
        &uow.approval_repo,
    );

    // Persistence channel: guaranteed delivery via mpsc (no Lagged errors possible)
    let conn_arc = db.conn_arc();
    let batch_writer = Arc::new(BatchWriter::spawn(Arc::clone(&conn_arc)));
    let bw = Arc::clone(&batch_writer);
    let mut persist_rx = persist_rx;
    tokio::spawn(async move {
        while let Some(event) = persist_rx.recv().await {
            if let Err(e) = bw.write(event) {
                tracing::warn!(error = %e, "batch writer: failed to queue event");
            }
        }
        info!("persistence channel closed, stopping event persistence");
    });
    info!("event persistence wired (BatchWriter <- mpsc guaranteed channel)");

    let rate_limit_max: u32 = env::var("FORGE_RATE_LIMIT_MAX")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let rate_limit_refill_ms: u64 = env::var("FORGE_RATE_LIMIT_REFILL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    let rate_limiter = Arc::new(RateLimiter::new(
        rate_limit_max,
        std::time::Duration::from_millis(rate_limit_refill_ms),
    ));
    let budget_warn = env::var("FORGE_BUDGET_WARN").ok().and_then(|s| s.parse().ok());
    let budget_limit = env::var("FORGE_BUDGET_LIMIT").ok().and_then(|s| s.parse().ok());
    let circuit_breaker = Arc::new(CircuitBreaker::default());

    // Load persisted safety state
    if let Ok(Some(cb_json)) = uow.safety_repo.get("circuit_breaker") {
        if let Ok(saved) = serde_json::from_str::<forge_safety::CircuitBreakerState>(&cb_json) {
            circuit_breaker.restore_state(&saved);
            info!("restored circuit breaker state: {}", saved.state);
        }
    }

    let circuit_breaker_for_shutdown = Arc::clone(&circuit_breaker);
    let safety_repo_for_shutdown = Arc::clone(&uow.safety_repo);
    let safety = SafetyState {
        circuit_breaker,
        rate_limiter,
        cost_tracker: Arc::new(CostTracker::new(budget_warn, budget_limit)),
    };

    let mut backend_registry = BackendRegistry::new("claude");
    backend_registry.register(Box::new(ClaudeBackend::new()));
    let backend_registry = Arc::new(backend_registry);

    let schedule_repo = Arc::clone(&uow.schedule_repo);
    let state = AppState::new(Arc::clone(&uow), Arc::new(event_bus), safety, backend_registry);

    // Spawn background scheduler.
    let cancel = tokio_util::sync::CancellationToken::new();
    let _scheduler_handle = scheduler::spawn(schedule_repo, cancel.clone());

    let host = env::var("FORGE_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("FORGE_PORT").unwrap_or_else(|_| "4173".into());
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    info!(%addr, "starting forge server");
    serve_until_signal(addr, state, shutdown_signal()).await?;

    // Persist safety state before shutdown
    let cb_state = circuit_breaker_for_shutdown.export_state();
    if let Ok(json) = serde_json::to_string(&cb_state) {
        if let Err(e) = safety_repo_for_shutdown.set("circuit_breaker", &json) {
            tracing::warn!("failed to persist circuit breaker state: {}", e);
        }
    }

    // Cancel the scheduler.
    cancel.cancel();

    // Shut down BatchWriter so it flushes remaining events. If another ref exists, drop and let thread exit on channel close.
    match Arc::try_unwrap(batch_writer) {
        Ok(bw) => {
            if let Err(e) = bw.shutdown() {
                tracing::warn!(error = %e, "batch writer shutdown error");
            }
        }
        Err(arc) => {
            tracing::warn!(
                "batch writer: extra refs held, dropping (thread will flush on channel close)"
            );
            drop(arc);
        }
    }
    Ok(())
}

fn seed_personas(persona_repo: &forge_db::PersonaRepo) {
    let personas_dir = std::path::Path::new("personas");
    if !personas_dir.is_dir() {
        info!("no personas/ directory found, skipping persona seeding");
        return;
    }

    let parser = PersonaParser::new(personas_dir);
    let parsed = match parser.parse_all() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("persona parsing failed: {:?}", e);
            return;
        }
    };

    if parsed.is_empty() {
        info!("no persona files found in personas/");
        return;
    }

    // Build divisions from discovered division slugs.
    let mut seen_divisions = std::collections::HashMap::<String, usize>::new();
    for p in &parsed {
        *seen_divisions.entry(p.division_slug.clone()).or_insert(0) += 1;
    }

    let now = chrono::Utc::now();
    let divisions: Vec<PersonaDivision> = seen_divisions
        .iter()
        .map(|(slug, count)| {
            let name = slug
                .split('-')
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().to_string() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            PersonaDivision {
                id: PersonaDivisionId::new(),
                slug: slug.clone(),
                name,
                description: None,
                agent_count: *count as u32,
                created_at: now,
                updated_at: now,
            }
        })
        .collect();

    if let Err(e) = persona_repo.upsert_divisions(&divisions) {
        tracing::warn!("persona division seeding failed: {}", e);
        return;
    }

    let personas: Vec<Persona> = parsed.into_iter().map(Persona::from).collect();
    let count = personas.len();
    if let Err(e) = persona_repo.upsert_personas(&personas) {
        tracing::warn!("persona seeding failed: {}", e);
        return;
    }

    info!(
        count,
        divisions = divisions.len(),
        "persona catalog seeded"
    );
}

/// Seed demo data on first launch so every page has something to show.
/// Only runs if no companies exist yet.
fn seed_demo_data(
    company_repo: &CompanyRepo,
    department_repo: &DepartmentRepo,
    agent_repo: &AgentRepo,
    org_position_repo: &OrgPositionRepo,
    goal_repo: &GoalRepo,
    approval_repo: &ApprovalRepo,
) {
    // Skip if companies already exist.
    match company_repo.list() {
        Ok(companies) if !companies.is_empty() => return,
        Err(_) => return,
        _ => {}
    }

    info!("seeding demo data for first launch");

    // 1. Create a demo company.
    let company = match company_repo.create(&forge_db::NewCompany {
        name: "Acme AI Corp".into(),
        mission: Some("Ship reliable AI-powered products with autonomous agent teams".into()),
        budget_limit: Some(500.0),
    }) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("demo seed: failed to create company: {}", e);
            return;
        }
    };

    // 2. Create departments.
    let eng_dept = department_repo.create(&forge_db::NewDepartment {
        company_id: company.id.clone(),
        name: "Engineering".into(),
        description: Some("Software development and architecture".into()),
    });
    let product_dept = department_repo.create(&forge_db::NewDepartment {
        company_id: company.id.clone(),
        name: "Product".into(),
        description: Some("Product management and strategy".into()),
    });

    let eng_id = eng_dept.as_ref().ok().map(|d| d.id.clone());
    let product_id = product_dept.as_ref().ok().map(|d| d.id.clone());

    // 3. Create sample agents with org positions.
    let agents_to_create: Vec<(&str, AgentPreset, Option<String>, &str)> = vec![
        ("Lead-Architect", AgentPreset::Architect, eng_id.clone(), "Chief Architect"),
        ("Code-Writer", AgentPreset::CodeWriter, eng_id.clone(), "Senior Engineer"),
        ("Code-Reviewer", AgentPreset::Reviewer, eng_id.clone(), "Staff Engineer"),
        ("Product-Manager", AgentPreset::Coordinator, product_id.clone(), "Head of Product"),
    ];

    let mut lead_position_id: Option<String> = None;

    for (i, (name, preset, dept_id, title)) in agents_to_create.into_iter().enumerate() {
        let agent = match agent_repo.create(&NewAgent {
            name: name.into(),
            model: None,
            system_prompt: Some(format!("You are {}, a specialist AI agent.", title)),
            allowed_tools: None,
            max_turns: None,
            use_max: None,
            preset: Some(preset),
            config: None,
            backend_type: None,
        }) {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!("demo seed: agent {}: {}", name, e);
                continue;
            }
        };

        let reports_to = if i > 0 { lead_position_id.clone() } else { None };

        let pos = org_position_repo.create(&forge_db::NewOrgPosition {
            company_id: company.id.clone(),
            department_id: dept_id,
            agent_id: Some(agent.id.0.to_string()),
            reports_to,
            role: name.into(),
            title: Some(title.into()),
        });

        if i == 0 {
            lead_position_id = pos.ok().map(|p| p.id);
        }
    }

    // 4. Create sample goals.
    let parent_goal = goal_repo.create(&forge_db::NewGoal {
        company_id: company.id.clone(),
        parent_id: None,
        title: "Launch v1.0 product".into(),
        description: Some("Ship the first production-ready release with core features".into()),
    });

    if let Ok(pg) = &parent_goal {
        let _ = goal_repo.create(&forge_db::NewGoal {
            company_id: company.id.clone(),
            parent_id: Some(pg.id.clone()),
            title: "Complete API integration tests".into(),
            description: Some("Ensure all API endpoints have test coverage above 80%".into()),
        });
        let _ = goal_repo.create(&forge_db::NewGoal {
            company_id: company.id.clone(),
            parent_id: Some(pg.id.clone()),
            title: "Security audit pass".into(),
            description: Some("Run OWASP scan and resolve all critical findings".into()),
        });
    }

    // 5. Create a sample approval.
    let _ = approval_repo.create(&forge_db::NewApproval {
        company_id: company.id.clone(),
        approval_type: "budget_increase".into(),
        requester: "Lead-Architect".into(),
        data_json: serde_json::json!({
            "title": "Increase compute budget for load testing",
            "current_budget": 500,
            "requested_budget": 750,
            "reason": "Need additional capacity for pre-launch stress tests"
        })
        .to_string(),
    });

    info!("demo data seeded: 1 company, 2 departments, 4 agents, 3 goals, 1 approval");
}
