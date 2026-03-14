//! Forge HTTP API: health, agent CRUD, WebSocket event stream, embedded frontend.

pub mod error;
pub mod middleware;
pub mod openapi;
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
        .merge(openapi::openapi_routes())
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
    use forge_db::{
        AgentRepo, AnalyticsRepo, ApprovalRepo, CompanyRepo, CompactionRepo, DbPool, DepartmentRepo,
        EventRepo, GoalRepo, HookRepo, MemoryRepo, Migrator, OrgPositionRepo, PersonaRepo,
        ScheduleRepo, SessionRepo, SkillRepo, WorkflowRepo,
    };
    use crate::state::SafetyState;
    use forge_safety::{CircuitBreaker, RateLimiter};
    use std::time::Duration;
    use http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let db = DbPool::in_memory().unwrap();
        let conn_arc = db.conn_arc();
        {
            let conn = conn_arc.lock().unwrap();
            let migrator = Migrator::new(&conn);
            migrator.apply_pending().unwrap();
        }
        AppState::new(
            Arc::new(AgentRepo::new(Arc::clone(&conn_arc))),
            Arc::new(SessionRepo::new(Arc::clone(&conn_arc))),
            Arc::new(EventRepo::new(Arc::clone(&conn_arc))),
            Arc::new(EventBus::new(16)),
            Arc::new(SkillRepo::new(Arc::clone(&conn_arc))),
            Arc::new(WorkflowRepo::new(Arc::clone(&conn_arc))),
            Arc::new(MemoryRepo::new(Arc::clone(&conn_arc))),
            Arc::new(HookRepo::new(Arc::clone(&conn_arc))),
            Arc::new(ScheduleRepo::new(Arc::clone(&conn_arc))),
            Arc::new(AnalyticsRepo::new(Arc::clone(&conn_arc))),
            Arc::new(CompactionRepo::new(Arc::clone(&conn_arc))),
            Arc::new(CompanyRepo::new(Arc::clone(&conn_arc))),
            Arc::new(DepartmentRepo::new(Arc::clone(&conn_arc))),
            Arc::new(OrgPositionRepo::new(Arc::clone(&conn_arc))),
            Arc::new(GoalRepo::new(Arc::clone(&conn_arc))),
            Arc::new(ApprovalRepo::new(Arc::clone(&conn_arc))),
            Arc::new(PersonaRepo::new(Arc::clone(&conn_arc))),
            SafetyState {
                circuit_breaker: Arc::new(CircuitBreaker::default()),
                rate_limiter: Arc::new(RateLimiter::new(100, Duration::from_secs(1))),
                cost_tracker: Arc::new(forge_safety::CostTracker::default()),
            },
        )
    }

    #[tokio::test]
    async fn skills_list_returns_200_and_empty_array() {
        let state = test_state();
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
        let state = test_state();
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
        let state = test_state();
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
        use forge_agent::model::NewAgent;

        let state = test_state();
        let agent = state.agent_repo
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
        use forge_agent::model::NewAgent;

        let state = test_state();
        let agent = state.agent_repo
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

    // --- Epic 1 integration tests ---

    async fn json_post(app: &Router, uri: &str, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .method("POST")
            .uri(format!("http://localhost{uri}"))
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    async fn json_get(app: &Router, uri: &str) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .uri(format!("http://localhost{uri}"))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    async fn json_patch(app: &Router, uri: &str, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .method("PATCH")
            .uri(format!("http://localhost{uri}"))
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    async fn json_delete(app: &Router, uri: &str) -> StatusCode {
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("http://localhost{uri}"))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        res.status()
    }

    #[tokio::test]
    async fn epic1_company_crud() {
        let app = app(test_state());

        // List initially empty
        let (status, json) = json_get(&app, "/api/v1/companies").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json.as_array().unwrap().len(), 0);

        // Create
        let (status, company) = json_post(&app, "/api/v1/companies", serde_json::json!({
            "name": "Acme Corp",
            "mission": "Build great things",
            "budget_limit": 10000.0
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(company["name"], "Acme Corp");
        assert!(company["id"].as_str().is_some());

        // List now has one
        let (_, json) = json_get(&app, "/api/v1/companies").await;
        assert_eq!(json.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn epic1_departments_and_org_positions() {
        let app = app(test_state());

        let (_, company) = json_post(&app, "/api/v1/companies", serde_json::json!({
            "name": "TestOrg"
        })).await;
        let cid = company["id"].as_str().unwrap();

        // Create department
        let (status, dept) = json_post(&app, "/api/v1/departments", serde_json::json!({
            "company_id": cid,
            "name": "Engineering",
            "description": "Build stuff"
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(dept["name"], "Engineering");

        // List departments
        let (_, depts) = json_get(&app, &format!("/api/v1/departments?company_id={cid}")).await;
        assert_eq!(depts.as_array().unwrap().len(), 1);

        // Create org position
        let (status, pos) = json_post(&app, "/api/v1/org-positions", serde_json::json!({
            "company_id": cid,
            "department_id": dept["id"].as_str().unwrap(),
            "role": "lead",
            "title": "Tech Lead"
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(pos["role"], "lead");

        // List positions
        let (_, positions) = json_get(&app, &format!("/api/v1/org-positions?company_id={cid}")).await;
        assert_eq!(positions.as_array().unwrap().len(), 1);

        // Org chart
        let (status, chart) = json_get(&app, &format!("/api/v1/org-chart?company_id={cid}")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(chart["company"]["name"], "TestOrg");
        assert_eq!(chart["roots"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn epic1_goal_lifecycle() {
        let app = app(test_state());

        let (_, company) = json_post(&app, "/api/v1/companies", serde_json::json!({
            "name": "GoalCo"
        })).await;
        let cid = company["id"].as_str().unwrap();

        // Create goal
        let (status, goal) = json_post(&app, "/api/v1/goals", serde_json::json!({
            "company_id": cid,
            "title": "Ship v1",
            "description": "Launch the product"
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(goal["status"], "planned");
        let gid = goal["id"].as_str().unwrap();

        // List goals
        let (_, goals) = json_get(&app, &format!("/api/v1/goals?company_id={cid}")).await;
        assert_eq!(goals.as_array().unwrap().len(), 1);

        // Update status to in_progress
        let (status, updated) = json_patch(&app, &format!("/api/v1/goals/{gid}/status"), serde_json::json!({
            "status": "in_progress"
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["status"], "in_progress");

        // Update status to completed
        let (status, updated) = json_patch(&app, &format!("/api/v1/goals/{gid}/status"), serde_json::json!({
            "status": "completed"
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["status"], "completed");

        // Invalid status rejected
        let (status, _) = json_patch(&app, &format!("/api/v1/goals/{gid}/status"), serde_json::json!({
            "status": "bogus"
        })).await;
        assert!(status.is_client_error() || status.is_server_error());
    }

    #[tokio::test]
    async fn epic1_approval_lifecycle() {
        let app = app(test_state());

        let (_, company) = json_post(&app, "/api/v1/companies", serde_json::json!({
            "name": "ApprovalCo"
        })).await;
        let cid = company["id"].as_str().unwrap();

        // Create approval
        let (status, approval) = json_post(&app, "/api/v1/approvals", serde_json::json!({
            "company_id": cid,
            "approval_type": "budget_increase",
            "requester": "alice",
            "data_json": { "amount": 5000 }
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(approval["status"], "pending");
        assert_eq!(approval["requester"], "alice");
        let aid = approval["id"].as_str().unwrap();

        // List approvals
        let (_, approvals) = json_get(&app, &format!("/api/v1/approvals?company_id={cid}")).await;
        assert_eq!(approvals.as_array().unwrap().len(), 1);

        // Filter by status
        let (_, filtered) = json_get(&app, &format!("/api/v1/approvals?company_id={cid}&status=approved")).await;
        assert_eq!(filtered.as_array().unwrap().len(), 0);

        // Approve it
        let (status, updated) = json_patch(&app, &format!("/api/v1/approvals/{aid}"), serde_json::json!({
            "status": "approved",
            "approver": "bob"
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["status"], "approved");
        assert_eq!(updated["approver"], "bob");

        // Now filter returns it
        let (_, filtered) = json_get(&app, &format!("/api/v1/approvals?company_id={cid}&status=approved")).await;
        assert_eq!(filtered.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn epic1_personas_list() {
        let app = app(test_state());

        // Persona catalog starts empty (no seed data in test DB)
        let (status, json) = json_get(&app, "/api/v1/personas").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json.is_array());
    }

    #[tokio::test]
    async fn epic1_company_detail_and_update() {
        let app = app(test_state());

        // Create company
        let (_, company) = json_post(&app, "/api/v1/companies", serde_json::json!({
            "name": "DetailCo",
            "mission": "Original mission",
            "budget_limit": 5000.0
        })).await;
        let id = company["id"].as_str().unwrap();

        // Get single
        let (status, detail) = json_get(&app, &format!("/api/v1/companies/{id}")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(detail["name"], "DetailCo");

        // Update mission only
        let (status, updated) = json_patch(&app, &format!("/api/v1/companies/{id}"), serde_json::json!({
            "mission": "New mission"
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["mission"], "New mission");
        assert_eq!(updated["name"], "DetailCo"); // name unchanged

        // Update name and budget
        let (status, updated) = json_patch(&app, &format!("/api/v1/companies/{id}"), serde_json::json!({
            "name": "RenamedCo",
            "budget_limit": 9999.0
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["name"], "RenamedCo");
        assert_eq!(updated["budget_limit"], 9999.0);
    }

    #[tokio::test]
    async fn epic1_company_delete() {
        let app = app(test_state());

        let (_, company) = json_post(&app, "/api/v1/companies", serde_json::json!({
            "name": "DeleteMe"
        })).await;
        let id = company["id"].as_str().unwrap();

        let status = json_delete(&app, &format!("/api/v1/companies/{id}")).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        // List should be empty
        let (_, list) = json_get(&app, "/api/v1/companies").await;
        assert_eq!(list.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn epic1_department_detail_update_delete() {
        let app = app(test_state());

        let (_, company) = json_post(&app, "/api/v1/companies", serde_json::json!({
            "name": "DeptTestCo"
        })).await;
        let cid = company["id"].as_str().unwrap();

        let (_, dept) = json_post(&app, "/api/v1/departments", serde_json::json!({
            "company_id": cid,
            "name": "Sales",
            "description": "Sell things"
        })).await;
        let did = dept["id"].as_str().unwrap();

        // Get single department
        let (status, detail) = json_get(&app, &format!("/api/v1/departments/{did}")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(detail["name"], "Sales");

        // Update
        let (status, updated) = json_patch(&app, &format!("/api/v1/departments/{did}"), serde_json::json!({
            "name": "Revenue",
            "description": "Drive revenue"
        })).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["name"], "Revenue");
        assert_eq!(updated["description"], "Drive revenue");

        // Delete
        let status = json_delete(&app, &format!("/api/v1/departments/{did}")).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        // List should be empty
        let (_, depts) = json_get(&app, &format!("/api/v1/departments?company_id={cid}")).await;
        assert_eq!(depts.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn epic1_persona_divisions_list() {
        let app = app(test_state());

        // Divisions list returns 200 (empty in test DB)
        let (status, json) = json_get(&app, "/api/v1/personas/divisions").await;
        assert_eq!(status, StatusCode::OK);
        assert!(json.is_array());
    }
}
