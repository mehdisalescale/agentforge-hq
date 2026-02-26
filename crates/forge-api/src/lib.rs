//! Forge HTTP API: health, agent CRUD, WebSocket event stream.

pub mod error;
pub mod routes;
pub mod state;

pub use state::AppState;
pub use routes::router;

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// Build the application router with CORS and API routes.
pub fn app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1", routes::router())
        .layer(cors)
        .with_state(state)
}

/// Run the server on the given address. Blocks until the server is shut down.
pub async fn serve(addr: SocketAddr, state: AppState) -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve_with_listener(listener, state).await
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
    use forge_db::{AgentRepo, EventRepo, Migrator, DbPool, SessionRepo};
    use http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

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
        let event_bus = EventBus::new(16);
        let state = AppState::new(
            Arc::new(agent_repo),
            Arc::new(session_repo),
            Arc::new(event_repo),
            Arc::new(event_bus),
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
        use forge_db::{AgentRepo, EventRepo, Migrator, DbPool, SessionRepo};
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
        let event_bus = EventBus::new(16);
        let state = AppState::new(
            Arc::new(agent_repo),
            Arc::new(session_repo),
            Arc::new(event_repo),
            Arc::new(event_bus),
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
    async fn run_returns_202_and_session_id() {
        use axum::body::Body;
        use forge_core::EventBus;
        use forge_agent::model::NewAgent;
        use forge_db::{AgentRepo, EventRepo, Migrator, DbPool, SessionRepo};
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
        let event_bus = EventBus::new(16);
        let state = AppState::new(
            Arc::new(agent_repo),
            Arc::new(session_repo),
            Arc::new(event_repo),
            Arc::new(event_bus),
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
