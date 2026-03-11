use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub mission: Option<String>,
    pub budget_limit: Option<f64>,
    pub budget_used: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgPosition {
    pub id: String,
    pub company_id: String,
    pub department_id: Option<String>,
    pub agent_id: Option<String>,
    pub reports_to: Option<String>,
    pub role: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgChartNode {
    pub position: OrgPosition,
    pub children: Vec<OrgChartNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyOrgChart {
    pub company: Company,
    pub departments: Vec<Department>,
    pub roots: Vec<OrgChartNode>,
}

