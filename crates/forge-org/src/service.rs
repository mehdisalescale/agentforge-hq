use std::collections::HashMap;

use crate::model::{Company, CompanyOrgChart, Department, OrgChartNode, OrgPosition};

/// Build a hierarchical org chart from flat position rows.
pub fn build_org_chart(
    company: Company,
    departments: Vec<Department>,
    positions: Vec<OrgPosition>,
) -> CompanyOrgChart {
    let mut nodes: HashMap<String, OrgChartNode> = positions
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

    // Attach children to their parents.
    let ids: Vec<String> = nodes.keys().cloned().collect();
    for id in ids {
        // Clippy prefers if let over match for simple Some branches.
        if let Some(parent_id) = nodes
            .get(&id)
            .and_then(|n| n.position.reports_to.clone())
        {
            if let (Some(child), Some(parent)) = {
                // Borrow checker: fetch indices first, then split the map borrow.
                let child_opt = nodes.remove(&id);
                let parent_opt = nodes.get_mut(&parent_id);
                (child_opt, parent_opt)
            } {
                parent.children.push(child);
            } else if let Some(child) = nodes.remove(&id) {
                // Parent id not found; treat as root to keep the tree total.
                roots.push(child);
            }
        }
    }

    // Any nodes that were never attached (no reports_to) are roots.
    for (_id, node) in nodes {
        if node.position.reports_to.is_none() {
            roots.push(node);
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

