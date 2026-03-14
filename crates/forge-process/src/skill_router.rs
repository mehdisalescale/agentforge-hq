use crate::task_type::TaskType;

/// Maps TaskType to a list of skill names that should be injected.
pub struct SkillRouter {
    routes: Vec<(TaskType, Vec<String>)>,
}

impl SkillRouter {
    /// Default mapping: task type → skill names from skills/ directory.
    pub fn new() -> Self {
        Self {
            routes: vec![
                (TaskType::NewFeature, vec![
                    "brainstorming".into(),
                    "writing-plans".into(),
                    "test-driven-development".into(),
                    "subagent-driven-development".into(),
                ]),
                (TaskType::BugFix, vec![
                    "systematic-debugging".into(),
                    "test-driven-development".into(),
                ]),
                (TaskType::CodeReview, vec![
                    "code-review".into(),
                    "security-guidance".into(),
                ]),
                (TaskType::Refactor, vec![
                    "refactor".into(),
                    "verification-before-completion".into(),
                ]),
                (TaskType::Research, vec![
                    "explore".into(),
                    "deep-research".into(),
                ]),
                // General: no methodology injection
            ],
        }
    }

    /// Given a TaskType, return the skill names to inject.
    pub fn skills_for(&self, task_type: TaskType) -> Vec<String> {
        for (tt, skills) in &self.routes {
            if *tt == task_type {
                return skills.clone();
            }
        }
        Vec::new() // General or unknown: no injection
    }
}

impl Default for SkillRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_feature_returns_expected_skills() {
        let router = SkillRouter::new();
        let skills = router.skills_for(TaskType::NewFeature);
        assert_eq!(skills.len(), 4);
        assert!(skills.contains(&"brainstorming".to_string()));
        assert!(skills.contains(&"test-driven-development".to_string()));
    }

    #[test]
    fn bugfix_returns_expected_skills() {
        let router = SkillRouter::new();
        let skills = router.skills_for(TaskType::BugFix);
        assert_eq!(skills.len(), 2);
        assert!(skills.contains(&"systematic-debugging".to_string()));
    }

    #[test]
    fn general_returns_empty() {
        let router = SkillRouter::new();
        let skills = router.skills_for(TaskType::General);
        assert!(skills.is_empty());
    }

    #[test]
    fn code_review_returns_expected_skills() {
        let router = SkillRouter::new();
        let skills = router.skills_for(TaskType::CodeReview);
        assert_eq!(skills.len(), 2);
        assert!(skills.contains(&"code-review".to_string()));
        assert!(skills.contains(&"security-guidance".to_string()));
    }

    #[test]
    fn refactor_returns_expected_skills() {
        let router = SkillRouter::new();
        let skills = router.skills_for(TaskType::Refactor);
        assert_eq!(skills.len(), 2);
        assert!(skills.contains(&"refactor".to_string()));
    }

    #[test]
    fn research_returns_expected_skills() {
        let router = SkillRouter::new();
        let skills = router.skills_for(TaskType::Research);
        assert_eq!(skills.len(), 2);
        assert!(skills.contains(&"explore".to_string()));
        assert!(skills.contains(&"deep-research".to_string()));
    }
}
