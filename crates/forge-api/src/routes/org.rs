//! Org and governance routes: companies, departments, positions, and org chart.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use serde::Deserialize;

use crate::error::api_error;
use crate::state::AppState;
use forge_db::{Company, Department, NewCompany, NewDepartment, NewOrgPosition, OrgPosition};
use forge_org::{model as org_model, service as org_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/companies", get(list_companies).post(create_company))
        .route(
            "/companies/:id",
            get(get_company).patch(update_company).delete(delete_company),
        )
        .route(
            "/departments",
            get(list_departments_by_company).post(create_department),
        )
        .route(
            "/departments/:id",
            get(get_department).patch(update_department).delete(delete_department),
        )
        .route(
            "/org-positions",
            get(list_org_positions_by_company).post(create_org_position),
        )
        .route("/org-chart", get(get_org_chart))
}

async fn list_companies(
    State(state): State<AppState>,
) -> Result<Json<Vec<Company>>, axum::response::Response> {
    let companies = state.company_repo.list().map_err(api_error)?;
    Ok(Json(companies))
}

#[derive(Debug, Deserialize)]
struct CreateCompanyBody {
    name: String,
    mission: Option<String>,
    budget_limit: Option<f64>,
}

async fn create_company(
    State(state): State<AppState>,
    Json(body): Json<CreateCompanyBody>,
) -> Result<Json<Company>, axum::response::Response> {
    let input = NewCompany {
        name: body.name,
        mission: body.mission,
        budget_limit: body.budget_limit,
    };
    let company = state.company_repo.create(&input).map_err(api_error)?;
    Ok(Json(company))
}

async fn get_company(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Company>, axum::response::Response> {
    let company = state.company_repo.get(&id).map_err(api_error)?;
    Ok(Json(company))
}

#[derive(Debug, Deserialize)]
struct UpdateCompanyBody {
    name: Option<String>,
    mission: Option<String>,
    budget_limit: Option<f64>,
}

async fn update_company(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateCompanyBody>,
) -> Result<Json<Company>, axum::response::Response> {
    let company = state
        .company_repo
        .update(&id, body.name.as_deref(), body.mission.as_deref(), body.budget_limit)
        .map_err(api_error)?;
    Ok(Json(company))
}

async fn delete_company(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, axum::response::Response> {
    state.company_repo.delete(&id).map_err(api_error)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct CreateDepartmentBody {
    company_id: String,
    name: String,
    description: Option<String>,
}

async fn create_department(
    State(state): State<AppState>,
    Json(body): Json<CreateDepartmentBody>,
) -> Result<Json<Department>, axum::response::Response> {
    let input = NewDepartment {
        company_id: body.company_id,
        name: body.name,
        description: body.description,
    };
    let dept = state.department_repo.create(&input).map_err(api_error)?;
    Ok(Json(dept))
}

#[derive(Debug, Deserialize)]
struct ListDepartmentsQuery {
    company_id: String,
}

async fn list_departments_by_company(
    State(state): State<AppState>,
    Query(query): Query<ListDepartmentsQuery>,
) -> Result<Json<Vec<Department>>, axum::response::Response> {
    let depts = state
        .department_repo
        .list_by_company(&query.company_id)
        .map_err(api_error)?;
    Ok(Json(depts))
}

async fn get_department(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Department>, axum::response::Response> {
    let dept = state.department_repo.get(&id).map_err(api_error)?;
    Ok(Json(dept))
}

#[derive(Debug, Deserialize)]
struct UpdateDepartmentBody {
    name: Option<String>,
    description: Option<String>,
}

async fn update_department(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateDepartmentBody>,
) -> Result<Json<Department>, axum::response::Response> {
    let dept = state
        .department_repo
        .update(&id, body.name.as_deref(), body.description.as_deref())
        .map_err(api_error)?;
    Ok(Json(dept))
}

async fn delete_department(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, axum::response::Response> {
    state.department_repo.delete(&id).map_err(api_error)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct CreateOrgPositionBody {
    company_id: String,
    department_id: Option<String>,
    agent_id: Option<String>,
    reports_to: Option<String>,
    role: String,
    title: Option<String>,
}

async fn create_org_position(
    State(state): State<AppState>,
    Json(body): Json<CreateOrgPositionBody>,
) -> Result<Json<OrgPosition>, axum::response::Response> {
    let input = NewOrgPosition {
        company_id: body.company_id,
        department_id: body.department_id,
        agent_id: body.agent_id,
        reports_to: body.reports_to,
        role: body.role,
        title: body.title,
    };
    let pos = state
        .org_position_repo
        .create(&input)
        .map_err(api_error)?;
    Ok(Json(pos))
}

#[derive(Debug, Deserialize)]
struct ListOrgPositionsQuery {
    company_id: String,
}

async fn list_org_positions_by_company(
    State(state): State<AppState>,
    Query(query): Query<ListOrgPositionsQuery>,
) -> Result<Json<Vec<OrgPosition>>, axum::response::Response> {
    let positions = state
        .org_position_repo
        .list_by_company(&query.company_id)
        .map_err(api_error)?;
    Ok(Json(positions))
}

#[derive(Debug, Deserialize)]
struct OrgChartQuery {
    company_id: Option<String>,
}

async fn get_org_chart(
    State(state): State<AppState>,
    Query(query): Query<OrgChartQuery>,
) -> Result<Json<org_model::CompanyOrgChart>, axum::response::Response> {
    let company = if let Some(id) = query.company_id {
        state.company_repo.get(&id).map_err(api_error)?
    } else {
        let companies = state.company_repo.list().map_err(api_error)?;
        companies
            .into_iter()
            .next()
            .ok_or_else(|| api_error(forge_core::error::ForgeError::Validation("no companies found".into())))?
    };

    let departments = state
        .department_repo
        .list_by_company(&company.id)
        .map_err(api_error)?;
    let positions = state
        .org_position_repo
        .list_by_company(&company.id)
        .map_err(api_error)?;

    // Map DB models into forge-org domain models.
    let company_model = org_model::Company {
        id: company.id.clone(),
        name: company.name.clone(),
        mission: company.mission.clone(),
        budget_limit: company.budget_limit,
        budget_used: company.budget_used,
    };

    let dept_models: Vec<org_model::Department> = departments
        .iter()
        .map(|d| org_model::Department {
            id: d.id.clone(),
            company_id: d.company_id.clone(),
            name: d.name.clone(),
            description: d.description.clone(),
        })
        .collect();

    let pos_models: Vec<org_model::OrgPosition> = positions
        .iter()
        .map(|p| org_model::OrgPosition {
            id: p.id.clone(),
            company_id: p.company_id.clone(),
            department_id: p.department_id.clone(),
            agent_id: p.agent_id.clone(),
            reports_to: p.reports_to.clone(),
            role: p.role.clone(),
            title: p.title.clone(),
        })
        .collect();

    let chart = org_service::build_org_chart(company_model, dept_models, pos_models);
    Ok(Json(chart))
}

