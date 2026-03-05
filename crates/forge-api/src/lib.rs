//! Forge HTTP API: health, agent CRUD, WebSocket event stream, embedded frontend.

pub mod error;
pub mod middleware;
pub mod routes;
pub mod state;

pub use state::AppState;
pub use routes::router;

use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
};
use axum::Router;
use http::{header::AUTHORIZATION, header::CONTENT_TYPE, Method, StatusCode};
use mime_guess::from_path;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

/// Embedded frontend assets (SvelteKit adapter-static output).
/// Path is relative to `crates/forge-api/Cargo.toml`.
/// Allow missing so the crate builds when frontend has not been built yet (e.g. CI).
#[derive(rust_embed::RustEmbed)]
#[folder = "../../frontend/build"]
#[allow_missing = true]
struct FrontendAssets;

/// Build the application router with CORS, API routes, and embedded frontend fallback.
/// CORS origin: set `FORGE_CORS_ORIGIN` to a specific origin (e.g. `https://app.example.com`)
/// or leave unset for `*` (permissive, suitable for local dev).
/// GET requests not matched by `/api/v1/*` are served from embedded frontend (SPA fallback to index.html).
pub fn app(state: AppState) -> Router {
    let cors_origin = env::var("FORGE_CORS_ORIGIN").unwrap_or_else(|_| "*".into());
    let methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ];
    let headers = [CONTENT_TYPE, AUTHORIZATION];

    let cors = if cors_origin == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(methods)
            .allow_headers(headers)
    } else {
        let origin = cors_origin
            .parse()
            .expect("FORGE_CORS_ORIGIN must be a valid HTTP header value");
        CorsLayer::new()
            .allow_origin(AllowOrigin::exact(origin))
            .allow_methods(methods)
            .allow_headers(headers)
    };

    Router::new()
        .nest("/api/v1", routes::router())
        .fallback(serve_embedded_fallback)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Serves embedded frontend files. Tries path, path/index.html, path.html; then SPA fallback to index.html.
/// Non-GET returns 405.
async fn serve_embedded_fallback(request: Request) -> Response {
    if request.method() != Method::GET {
        return (StatusCode::METHOD_NOT_ALLOWED, Body::empty()).into_response();
    }

    let path = request.uri().path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    let candidates = [
        path.to_string(),
        format!("{}/index.html", path),
        format!("{}.html", path),
        "index.html".to_string(),
    ];

    for candidate in &candidates {
        if let Some(file) = FrontendAssets::get(candidate.as_str()) {
            let mime = from_path(candidate.as_str()).first_or_octet_stream();
            let value = match http::HeaderValue::try_from(mime.as_ref()) {
                Ok(v) => v,
                Err(_) => continue,
            };
            return ([(CONTENT_TYPE, value)], file.data.to_vec()).into_response();
        }
    }

    // SPA fallback: serve index.html for client-side routing
    if let Some(index) = FrontendAssets::get("index.html") {
        let value = http::HeaderValue::from_static("text/html");
        return ([(CONTENT_TYPE, value)], index.data.to_vec()).into_response();
    }

    (StatusCode::NOT_FOUND, Body::empty()).into_response()
}

/// Run the server on the given address. Blocks until the server is shut down.
pub async fn serve(addr: SocketAddr, state: AppState) -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve_with_listener(listener, state).await
}

/// Run the server until the given shutdown future completes (e.g. Ctrl+C).
/// Use this for graceful shutdown so in-flight requests can finish and BatchWriter can flush.
pub async fn serve_until_signal<F>(
    addr: SocketAddr,
    state: AppState,
    shutdown: F,
) -> Result<(), std::io::Error>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    info!(%local_addr, "listening");
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown)
        .await
}

/// Run the server on an existing listener. Used by tests to bind to port 0 and get the port.
pub async fn serve_with_listener(
    listener: tokio::net::TcpListener,
    state: AppState,
) -> Result<(), std::io::Error> {
    let addr = listener.local_addr()?;
    info!(%addr, "listening");
    axum::serve(listener, app(state)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use forge_core::EventBus;
    use forge_db::{AgentRepo, AnalyticsRepo, EventRepo, HookRepo, MemoryRepo, Migrator, DbPool, ScheduleRepo, SessionRepo, SkillRepo, WorkflowRepo};
    use crate::state::SafetyState;
    use forge_safety::{CircuitBreaker, RateLimiter};
    use std::time::Duration;
    use http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn skills_list_returns_200_and_empty_array() {
        let db = DbPool::in_memory().unwrap();
        let conn_arc = db.conn_arc();
        {
            let conn = conn_arc.lock().unwrap();
            let migrator = Migrator::new(&conn);
            migrator.apply_pending().unwrap();
        }
        let agent_repo = AgentRepo::new(Arc::clone(&conn_arc));
        let session_repo = SessionRepo::new(Arc::clone(&conn_arc));
        let event_repo = EventRepo::new(Arc::clone(&conn_arc));
        let skill_repo = SkillRepo::new(Arc::clone(&conn_arc));
        let workflow_repo = WorkflowRepo::new(Arc::clone(&conn_arc));
        let memory_repo = MemoryRepo::new(Arc::clone(&conn_arc));
        let hook_repo = HookRepo::new(Arc::clone(&conn_arc));
        let event_bus = EventBus::new(16);
        let state = AppState::new(
            Arc::new(agent_repo),
            Arc::new(session_repo),
            Arc::new(event_repo),
            Arc::new(event_bus),
            Arc::new(skill_repo),
            Arc::new(workflow_repo),
            Arc::new(memory_repo),
            Arc::new(hook_repo),
            Arc::new(ScheduleRepo::new(Arc::clone(&conn_arc))),
            Arc::new(AnalyticsRepo::new(Arc::clone(&conn_arc))),
            SafetyState {
                circuit_breaker: Arc::new(CircuitBreaker::default()),
                rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
                cost_tracker: Arc::new(forge_safety::CostTracker::default()),
            },
        );
        let app = app(state);
        let request = Request::builder()
            .uri("http://localhost/api/v1/skills")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn workflows_list_returns_200() {
        let db = DbPool::in_memory().unwrap();
        let conn_arc = db.conn_arc();
        {
            let conn = conn_arc.lock().unwrap();
            let migrator = Migrator::new(&conn);
            migrator.apply_pending().unwrap();
        }
        let agent_repo = AgentRepo::new(Arc::clone(&conn_arc));
        let session_repo = SessionRepo::new(Arc::clone(&conn_arc));
        let event_repo = EventRepo::new(Arc::clone(&conn_arc));
        let skill_repo = SkillRepo::new(Arc::clone(&conn_arc));
        let workflow_repo = WorkflowRepo::new(Arc::clone(&conn_arc));
        let memory_repo = MemoryRepo::new(Arc::clone(&conn_arc));
        let hook_repo = HookRepo::new(Arc::clone(&conn_arc));
        let event_bus = EventBus::new(16);
        let state = AppState::new(
            Arc::new(agent_repo),
            Arc::new(session_repo),
            Arc::new(event_repo),
            Arc::new(event_bus),
            Arc::new(skill_repo),
            Arc::new(workflow_repo),
            Arc::new(memory_repo),
            Arc::new(hook_repo),
            Arc::new(ScheduleRepo::new(Arc::clone(&conn_arc))),
            Arc::new(AnalyticsRepo::new(Arc::clone(&conn_arc))),
            SafetyState {
                circuit_breaker: Arc::new(CircuitBreaker::default()),
                rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
                cost_tracker: Arc::new(forge_safety::CostTracker::default()),
            },
        );
        let app = app(state);
        let request = Request::builder()
            .uri("http://localhost/api/v1/workflows")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
    }

    #[tokio::test]
    async fn health_check_responds_ok() {
        let db = DbPool::in_memory().unwrap();
        let conn_arc = db.conn_arc();
        {
            let conn = conn_arc.lock().unwrap();
            let migrator = Migrator::new(&conn);
            migrator.apply_pending().unwrap();
        }
        let agent_repo = AgentRepo::new(Arc::clone(&conn_arc));
        let session_repo = SessionRepo::new(Arc::clone(&conn_arc));
        let event_repo = EventRepo::new(Arc::clone(&conn_arc));
        let skill_repo = SkillRepo::new(Arc::clone(&conn_arc));
        let workflow_repo = WorkflowRepo::new(Arc::clone(&conn_arc));
        let memory_repo = MemoryRepo::new(Arc::clone(&conn_arc));
        let hook_repo = HookRepo::new(Arc::clone(&conn_arc));
        let event_bus = EventBus::new(16);
        let state = AppState::new(
            Arc::new(agent_repo),
            Arc::new(session_repo),
            Arc::new(event_repo),
            Arc::new(event_bus),
            Arc::new(skill_repo),
            Arc::new(workflow_repo),
            Arc::new(memory_repo),
            Arc::new(hook_repo),
            Arc::new(ScheduleRepo::new(Arc::clone(&conn_arc))),
            Arc::new(AnalyticsRepo::new(Arc::clone(&conn_arc))),
            SafetyState {
                circuit_breaker: Arc::new(CircuitBreaker::default()),
                rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
                cost_tracker: Arc::new(forge_safety::CostTracker::default()),
            },
        );

        let app = app(state);
        let request = Request::builder()
            .uri("http://localhost/api/v1/health")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("ok"));
    }

    #[tokio::test]
    async fn session_crud_and_export() {
        use axum::body::Body;
        use forge_core::EventBus;
        use forge_agent::model::NewAgent;
        use forge_db::{AgentRepo, AnalyticsRepo, EventRepo, HookRepo, MemoryRepo, Migrator, DbPool, ScheduleRepo, SessionRepo, SkillRepo, WorkflowRepo};
        use http::{Request, StatusCode};
        use std::sync::Arc;
        use tower::ServiceExt;

        let db = DbPool::in_memory().unwrap();
        let conn_arc = db.conn_arc();
        {
            let conn = conn_arc.lock().unwrap();
            let migrator = Migrator::new(&conn);
            migrator.apply_pending().unwrap();
        }
        let agent_repo = AgentRepo::new(Arc::clone(&conn_arc));
        let agent = agent_repo
            .create(&NewAgent {
                name: "ExportTestAgent".into(),
                model: None,
                system_prompt: None,
                allowed_tools: None,
                max_turns: None,
                use_max: None,
                preset: None,
                config: None,
            })
            .unwrap();

        let session_repo = SessionRepo::new(Arc::clone(&conn_arc));
        let event_repo = EventRepo::new(Arc::clone(&conn_arc));
        let skill_repo = SkillRepo::new(Arc::clone(&conn_arc));
        let workflow_repo = WorkflowRepo::new(Arc::clone(&conn_arc));
        let memory_repo = MemoryRepo::new(Arc::clone(&conn_arc));
        let hook_repo = HookRepo::new(Arc::clone(&conn_arc));
        let event_bus = EventBus::new(16);
        let state = AppState::new(
            Arc::new(agent_repo),
            Arc::new(session_repo),
            Arc::new(event_repo),
            Arc::new(event_bus),
            Arc::new(skill_repo),
            Arc::new(workflow_repo),
            Arc::new(memory_repo),
            Arc::new(hook_repo),
            Arc::new(ScheduleRepo::new(Arc::clone(&conn_arc))),
            Arc::new(AnalyticsRepo::new(Arc::clone(&conn_arc))),
            SafetyState {
                circuit_breaker: Arc::new(CircuitBreaker::default()),
                rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
                cost_tracker: Arc::new(forge_safety::CostTracker::default()),
            },
        );

        let app = app(state);

        // POST create session
        let body = serde_json::json!({
            "agent_id": agent.id.0.to_string(),
            "directory": "/tmp/test",
        });
        let req = Request::builder()
            .method("POST")
            .uri("http://localhost/api/v1/sessions")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let session: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = session.get("id").and_then(|v| v.as_str()).unwrap();

        // GET list sessions
        let req = Request::builder()
            .uri("http://localhost/api/v1/sessions")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // GET export json
        let req = Request::builder()
            .uri(format!("http://localhost/api/v1/sessions/{}/export?format=json", id))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let export: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(export.get("session").is_some());
        assert!(export.get("events").is_some());

        // GET export markdown
        let req = Request::builder()
            .uri(format!("http://localhost/api/v1/sessions/{}/export?format=markdown", id))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // DELETE session
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("http://localhost/api/v1/sessions/{}", id))
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    #[ignore] // Requires `claude` CLI installed — run with `cargo test -- --ignored`
    async fn run_returns_202_and_session_id() {
        use axum::body::Body;
        use forge_core::EventBus;
        use forge_agent::model::NewAgent;
        use forge_db::{AgentRepo, AnalyticsRepo, EventRepo, HookRepo, MemoryRepo, Migrator, DbPool, ScheduleRepo, SessionRepo, SkillRepo, WorkflowRepo};
        use http::{Request, StatusCode};
        use std::sync::Arc;
        use tower::ServiceExt;

        let db = DbPool::in_memory().unwrap();
        let conn_arc = db.conn_arc();
        {
            let conn = conn_arc.lock().unwrap();
            let migrator = Migrator::new(&conn);
            migrator.apply_pending().unwrap();
        }
        let agent_repo = AgentRepo::new(Arc::clone(&conn_arc));
        let agent = agent_repo
            .create(&NewAgent {
                name: "RunTestAgent".into(),
                model: None,
                system_prompt: None,
                allowed_tools: None,
                max_turns: None,
                use_max: None,
                preset: None,
                config: None,
            })
            .unwrap();

        let session_repo = SessionRepo::new(Arc::clone(&conn_arc));
        let event_repo = EventRepo::new(Arc::clone(&conn_arc));
        let skill_repo = SkillRepo::new(Arc::clone(&conn_arc));
        let workflow_repo = WorkflowRepo::new(Arc::clone(&conn_arc));
        let memory_repo = MemoryRepo::new(Arc::clone(&conn_arc));
        let hook_repo = HookRepo::new(Arc::clone(&conn_arc));
        let event_bus = EventBus::new(16);
        let state = AppState::new(
            Arc::new(agent_repo),
            Arc::new(session_repo),
            Arc::new(event_repo),
            Arc::new(event_bus),
            Arc::new(skill_repo),
            Arc::new(workflow_repo),
            Arc::new(memory_repo),
            Arc::new(hook_repo),
            Arc::new(ScheduleRepo::new(Arc::clone(&conn_arc))),
            Arc::new(AnalyticsRepo::new(Arc::clone(&conn_arc))),
            SafetyState {
                circuit_breaker: Arc::new(CircuitBreaker::default()),
                rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
                cost_tracker: Arc::new(forge_safety::CostTracker::default()),
            },
        );

        let app = app(state);
        let body = serde_json::json!({
            "agent_id": agent.id.0.to_string(),
            "prompt": "Hello, world",
        });
        let req = Request::builder()
            .method("POST")
            .uri("http://localhost/api/v1/run")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::ACCEPTED);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("session_id").and_then(|v| v.as_str()).is_some());
    }
}
