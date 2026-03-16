use std::collections::HashMap;

use crate::model::{Company, CompanyOrgChart, Department, OrgChartNode, OrgPosition};

/// Maximum recursion depth for org chart tree building.
/// Prevents stack overflow from circular `reports_to` references.
const MAX_ORG_DEPTH: usize = 50;

/// Build a hierarchical org chart from flat position rows.
pub fn build_org_chart(
    company: Company,
    departments: Vec<Department>,
    positions: Vec<OrgPosition>,
) -> CompanyOrgChart {
    // Index positions by id, tracking parent edges separately.
    let mut by_id: HashMap<String, OrgChartNode> = HashMap::new();
    let mut children_of: HashMap<String, Vec<String>> = HashMap::new();
    let mut root_ids: Vec<String> = Vec::new();

    for p in &positions {
        if let Some(ref parent_id) = p.reports_to {
            children_of
                .entry(parent_id.clone())
                .or_default()
                .push(p.id.clone());
        }
    }

    for p in positions {
        let id = p.id.clone();
        by_id.insert(
            id,
            OrgChartNode {
                position: p,
                children: Vec::new(),
            },
        );
    }

    // Identify roots (no parent, or parent not in the set).
    for (id, node) in &by_id {
        match &node.position.reports_to {
            None => root_ids.push(id.clone()),
            Some(pid) if !by_id.contains_key(pid) => root_ids.push(id.clone()),
            _ => {}
        }
    }

    // Recursively build the tree from roots downward.
    fn build_subtree(
        id: &str,
        by_id: &mut HashMap<String, OrgChartNode>,
        children_of: &HashMap<String, Vec<String>>,
        depth: usize,
    ) -> Option<OrgChartNode> {
        if depth >= MAX_ORG_DEPTH {
            // Stop recursing to prevent stack overflow from circular reports_to chains.
            return by_id.remove(id);
        }
        let mut node = by_id.remove(id)?;
        if let Some(child_ids) = children_of.get(id) {
            for cid in child_ids {
                if let Some(child) = build_subtree(cid, by_id, children_of, depth + 1) {
                    node.children.push(child);
                }
            }
        }
        Some(node)
    }

    let mut roots = Vec::new();
    for rid in root_ids {
        if let Some(tree) = build_subtree(&rid, &mut by_id, &children_of, 0) {
            roots.push(tree);
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

