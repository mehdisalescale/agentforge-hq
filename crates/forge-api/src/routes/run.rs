//! Run endpoint: POST /api/v1/run — start a process for an agent + prompt; optional session_id for resume.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use forge_core::ids::{AgentId, SessionId};
use forge_db::NewSession;
use serde::Deserialize;
use std::sync::Arc;

use crate::error::{api_error, parse_uuid, rate_limit_exceeded};
use crate::configurator::AgentConfigurator;
use crate::middleware::{
    CircuitBreakerMiddleware, CostCheckMiddleware, GovernanceMiddleware, MiddlewareChain,
    MiddlewareError, PersistMiddleware, RateLimitMiddleware, RunContext, SecurityScanMiddleware,
    SpawnMiddleware,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub agent_id: String,
    pub prompt: String,
    pub session_id: Option<String>,
    /// Optional working directory for the run (used when creating a new session and as spawn cwd).
    pub directory: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct RunResponse {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/run", post(run_handler))
}

async fn run_handler(
    State(state): State<AppState>,
    Json(body): Json<RunRequest>,
) -> Result<impl IntoResponse, axum::response::Response> {
    // 1. Parse & validate
    let agent_id = AgentId(parse_uuid(&body.agent_id)?);
    state.agent_repo.get(&agent_id).map_err(api_error)?;

    let session = if let Some(ref sid) = body.session_id {
        let id = SessionId(parse_uuid(sid)?);
        state.session_repo.get(&id).map_err(api_error)?
    } else {
        let directory = body.directory.as_deref().unwrap_or(".").to_string();
        state
            .session_repo
            .create(&NewSession {
                agent_id: agent_id.clone(),
                directory: directory.clone(),
                claude_session_id: None,
            })
            .map_err(api_error)?
    };
    let session_id = session.id.clone();

    // 2. Build RunContext
    let mut ctx = RunContext {
        agent_id: body.agent_id.clone(),
        prompt: body.prompt.clone(),
        session_id: session_id.0.to_string(),
        working_dir: body.directory.clone(),
        metadata: Default::default(),
        agent_id_typed: agent_id,
        session_id_typed: session_id.clone(),
        resume_session_id: body.session_id.clone(),
        directory: session.directory.clone(),
    };

    // 3. Build middleware chain
    let mut chain = MiddlewareChain::new();
    chain.add(RateLimitMiddleware {
        rate_limiter: Arc::clone(&state.safety.rate_limiter),
    });
    chain.add(CircuitBreakerMiddleware {
        circuit_breaker: Arc::clone(&state.safety.circuit_breaker),
    });
    chain.add(CostCheckMiddleware {
        cost_tracker: Arc::clone(&state.safety.cost_tracker),
        session_repo: Arc::clone(&state.session_repo),
    });
    chain.add(GovernanceMiddleware {
        company_repo: Arc::clone(&state.company_repo),
        org_position_repo: Arc::clone(&state.org_position_repo),
        goal_repo: Arc::clone(&state.goal_repo),
        approval_repo: Arc::clone(&state.approval_repo),
    });
    chain.add(SecurityScanMiddleware {
        event_bus: Arc::clone(&state.event_bus),
    });
    chain.add(PersistMiddleware {
        session_repo: Arc::clone(&state.session_repo),
        event_bus: Arc::clone(&state.event_bus),
    });

    // AgentConfigurator replaces SkillInjection + TaskTypeDetection middlewares
    let configurator = Arc::new(AgentConfigurator {
        skill_repo: Arc::clone(&state.skill_repo),
        company_repo: Arc::clone(&state.company_repo),
        org_position_repo: Arc::clone(&state.org_position_repo),
        goal_repo: Arc::clone(&state.goal_repo),
        persona_repo: Arc::clone(&state.persona_repo),
        agent_repo: Arc::clone(&state.agent_repo),
    });

    chain.add(SpawnMiddleware {
        event_bus: Arc::clone(&state.event_bus),
        session_repo: Arc::clone(&state.session_repo),
        circuit_breaker: Arc::clone(&state.safety.circuit_breaker),
        cost_tracker: Arc::clone(&state.safety.cost_tracker),
        configurator,
    });

    // 4. Execute and map errors to HTTP responses
    let chain_result: Result<crate::middleware::RunResponse, MiddlewareError> =
        chain.execute(&mut ctx).await;
    chain_result.map_err(|e| match e {
        MiddlewareError::RateLimited => rate_limit_exceeded(),
        MiddlewareError::CircuitOpen => {
            api_error(forge_core::error::ForgeError::Internal("circuit breaker open".into()))
        }
        MiddlewareError::BudgetExceeded { cost, limit } => api_error(
            forge_core::error::ForgeError::Internal(format!(
                "budget exceeded: ${:.2} >= ${:.2}",
                cost, limit
            )),
        ),
        MiddlewareError::ExitGateTriggered(reason) => api_error(
            forge_core::error::ForgeError::Internal(format!("exit gate: {}", reason)),
        ),
        MiddlewareError::QualityGateFailed { score, threshold } => api_error(
            forge_core::error::ForgeError::Internal(format!(
                "quality gate failed: score {:.1} < threshold {:.1}",
                score, threshold
            )),
        ),
        MiddlewareError::SpawnFailed(msg) => {
            api_error(forge_core::error::ForgeError::Internal(msg))
        }
        MiddlewareError::Internal(msg) => {
            api_error(forge_core::error::ForgeError::Internal(msg))
        }
    })?;

    // 5. Return 202 Accepted
    Ok((
        StatusCode::ACCEPTED,
        Json(RunResponse {
            session_id: session_id.0.to_string(),
            message: Some("Started.".into()),
        }),
    ))
}
