//! Usage analytics: GET /api/v1/analytics/usage?start=&end=&company_id=

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
    pub company_id: Option<String>,
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

    let mut report = state
        .uow.analytics_repo
        .usage_report(&start, &end)
        .map_err(api_error)?;

    // Filter agent_breakdown to only agents belonging to the selected company
    if let Some(ref company_id) = query.company_id {
        let positions = state
            .uow.org_position_repo
            .list_by_company(company_id)
            .map_err(api_error)?;
        let company_agent_ids: std::collections::HashSet<String> = positions
            .iter()
            .filter_map(|p| p.agent_id.clone())
            .collect();
        report.agent_breakdown.retain(|ab| company_agent_ids.contains(&ab.agent_id));
        report.total_cost = report.agent_breakdown.iter().map(|ab| ab.total_cost).sum();
    }

    Ok(Json(report))
}
