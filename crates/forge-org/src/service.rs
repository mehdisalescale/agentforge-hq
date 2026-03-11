use std::collections::HashMap;

use crate::model::{Company, CompanyOrgChart, Department, OrgChartNode, OrgPosition};

/// Build a hierarchical org chart from flat position rows.
pub fn build_org_chart(
    company: Company,
    departments: Vec<Department>,
    positions: Vec<OrgPosition>,
) -> CompanyOrgChart {
    let nodes: HashMap<String, OrgChartNode> = positions
        .into_iter()
        .map(|p| {
            let id = p.id.clone();
            (
                id,
                OrgChartNode {
                    position: p,
                    children: Vec::new(),
                },
            )
        })
        .collect();

    let mut roots = Vec::new();

    // Attach children to their parents while preserving all nodes.
    let mut by_id: HashMap<String, OrgChartNode> = HashMap::new();
    let mut parent_map: HashMap<String, Option<String>> = HashMap::new();

    for (id, node) in nodes.into_iter() {
        parent_map.insert(id.clone(), node.position.reports_to.clone());
        by_id.insert(id, node);
    }

    // Build child edges.
    let ids: Vec<String> = by_id.keys().cloned().collect();
    for id in &ids {
        if let Some(Some(parent_id)) = parent_map.get(id) {
            if let Some(child) = by_id.get(id).cloned() {
                if let Some(parent) = by_id.get_mut(parent_id) {
                    parent.children.push(child);
                }
            }
        }
    }

    // Roots are nodes without a valid parent.
    for id in ids {
        let parent_id = parent_map.get(&id).and_then(|p| p.clone());
        if parent_id.is_none() || !by_id.contains_key(parent_id.as_ref().unwrap()) {
            if let Some(node) = by_id.get(&id) {
                roots.push(node.clone());
            }
        }
    }

    CompanyOrgChart {
        company,
        departments,
        roots,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_org_chart_creates_tree() {
        let company = Company {
            id: "c1".into(),
            name: "Test Co".into(),
            mission: None,
            budget_limit: None,
            budget_used: 0.0,
        };

        let departments = vec![Department {
            id: "d1".into(),
            company_id: "c1".into(),
            name: "Engineering".into(),
            description: None,
        }];

        let ceo = OrgPosition {
            id: "p1".into(),
            company_id: "c1".into(),
            department_id: None,
            agent_id: None,
            reports_to: None,
            role: "ceo".into(),
            title: Some("CEO".into()),
        };
        let manager = OrgPosition {
            id: "p2".into(),
            company_id: "c1".into(),
            department_id: Some("d1".into()),
            agent_id: None,
            reports_to: Some("p1".into()),
            role: "manager".into(),
            title: Some("Eng Manager".into()),
        };
        let ic = OrgPosition {
            id: "p3".into(),
            company_id: "c1".into(),
            department_id: Some("d1".into()),
            agent_id: None,
            reports_to: Some("p2".into()),
            role: "ic".into(),
            title: Some("Engineer".into()),
        };

        let chart = build_org_chart(company, departments.clone(), vec![ceo, manager, ic]);

        assert_eq!(chart.company.id, "c1");
        assert_eq!(chart.departments.len(), 1);
        assert_eq!(chart.roots.len(), 1);
        let root = &chart.roots[0];
        assert_eq!(root.position.role, "ceo");
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].position.role, "manager");
        assert_eq!(root.children[0].children.len(), 1);
        assert_eq!(root.children[0].children[0].position.role, "ic");
    }
}

