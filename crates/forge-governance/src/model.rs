use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub company_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    pub id: String,
    pub company_id: String,
    pub approval_type: String,
    pub status: String,
    pub requester: String,
    pub approver: Option<String>,
    pub data_json: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_roundtrip_serde() {
        let goal = Goal {
            id: "g1".into(),
            company_id: "c1".into(),
            parent_id: None,
            title: "Test Goal".into(),
            description: Some("Desc".into()),
            status: "planned".into(),
        };
        let json = serde_json::to_string(&goal).unwrap();
        let back: Goal = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "g1");
        assert_eq!(back.company_id, "c1");
    }

    #[test]
    fn approval_roundtrip_serde() {
        let approval = Approval {
            id: "a1".into(),
            company_id: "c1".into(),
            approval_type: "hire".into(),
            status: "pending".into(),
            requester: "user".into(),
            approver: None,
            data_json: "{}".into(),
        };
        let json = serde_json::to_string(&approval).unwrap();
        let back: Approval = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "a1");
        assert_eq!(back.status, "pending");
    }
}

