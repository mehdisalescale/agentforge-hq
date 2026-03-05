//! Usage analytics: GET /api/v1/analytics/usage?start=&end=

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;

use crate::error::api_error;
use crate::state::AppState;

use forge_db::repos::analytics::UsageReport;

#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    pub start: Option<String>,
    pub end: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/analytics/usage", get(usage_report))
}

async fn usage_report(
    State(state): State<AppState>,
    Query(query): Query<UsageQuery>,
) -> Result<Json<UsageReport>, axum::response::Response> {
    let now = Utc::now();
    let start = query.start.unwrap_or_else(|| {
        (now - chrono::Duration::days(30))
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string()
    });
    let end = query
        .end
        .unwrap_or_else(|| now.format("%Y-%m-%dT%H:%M:%S").to_string());

    let report = state
        .analytics_repo
        .usage_report(&start, &end)
        .map_err(api_error)?;
    Ok(Json(report))
}
