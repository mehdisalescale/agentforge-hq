# Agent A2: Skill Router (Task Type → Skill Injection)

> You are Agent A2. Your job: create a SkillRouter that maps TaskType to methodology skills and enhance SkillInjectionMiddleware to use it.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
crates/forge-api/src/middleware.rs                  — FULL FILE, study SkillInjectionMiddleware carefully
crates/forge-api/src/routes/run.rs                  — how chain is assembled
crates/forge-process/src/task_type.rs               — TaskType enum + TaskTypeDetector
crates/forge-db/src/repos/skills.rs                 — SkillRepo, UpsertSkill
```

## Step 2: Create SkillRouter

Create `crates/forge-process/src/skill_router.rs`:

```rust
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
```

Add `pub mod skill_router;` to `crates/forge-process/src/lib.rs`.

## Step 3: Create TaskTypeDetection Middleware

Add a new middleware to `crates/forge-api/src/middleware.rs` — add it AFTER the existing `SkillInjectionMiddleware` implementation, BEFORE the `PersistMiddleware`:

```rust
/// Classifies the prompt into a TaskType and injects methodology skills.
pub struct TaskTypeDetectionMiddleware {
    pub skill_repo: Arc<SkillRepo>,
}

impl Middleware for TaskTypeDetectionMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>> {
        Box::pin(async move {
            let detector = forge_process::task_type::TaskTypeDetector::new();
            let task_type = detector.classify(&ctx.prompt);
            ctx.metadata.insert("task_type".into(), format!("{:?}", task_type));

            let router = forge_process::skill_router::SkillRouter::new();
            let skill_names = router.skills_for(task_type);

            if !skill_names.is_empty() {
                if let Ok(all_skills) = self.skill_repo.list() {
                    let mut injected: Vec<String> = ctx.metadata
                        .get("injected_skills")
                        .map(|s| vec![s.clone()])
                        .unwrap_or_default();

                    for skill in &all_skills {
                        if skill_names.iter().any(|n| n == &skill.name) {
                            injected.push(format!("## Methodology: {}\n{}", skill.name, skill.content));
                        }
                    }
                    if !injected.is_empty() {
                        ctx.metadata.insert("injected_skills".into(), injected.join("\n\n"));
                    }
                }
            }

            next.run(ctx).await
        })
    }

    fn name(&self) -> &str {
        "task_type_detection"
    }
}
```

## Step 4: Wire into Chain

In `crates/forge-api/src/routes/run.rs`, add the new middleware AFTER SkillInjection:

```rust
// existing:
chain.add(SkillInjectionMiddleware { ... });
// ADD THIS:
chain.add(TaskTypeDetectionMiddleware {
    skill_repo: Arc::clone(&state.skill_repo),
});
// existing:
chain.add(PersistMiddleware { ... });
```

Add the import at the top of run.rs:
```rust
use crate::middleware::TaskTypeDetectionMiddleware;
```

## Step 5: Write Tests

Add tests in `crates/forge-api/src/middleware.rs` tests module:

```rust
#[tokio::test]
async fn task_type_detection_sets_metadata() {
    let conn = forge_db::DbPool::in_memory().unwrap();
    { let c = conn.connection(); forge_db::Migrator::new(&c).apply_pending().unwrap(); }
    let skill_repo = Arc::new(SkillRepo::new(conn.conn_arc()));
    let mw = TaskTypeDetectionMiddleware { skill_repo };
    let mut chain = MiddlewareChain::new();
    chain.add(mw);
    let mut ctx = test_context();
    ctx.prompt = "fix the login bug".into();
    let result = chain.execute(&mut ctx).await;
    assert!(result.is_ok());
    assert_eq!(ctx.metadata.get("task_type"), Some(&"BugFix".to_string()));
}

#[tokio::test]
async fn task_type_general_injects_no_methodology() {
    let conn = forge_db::DbPool::in_memory().unwrap();
    { let c = conn.connection(); forge_db::Migrator::new(&c).apply_pending().unwrap(); }
    let skill_repo = Arc::new(SkillRepo::new(conn.conn_arc()));
    let mw = TaskTypeDetectionMiddleware { skill_repo };
    let mut chain = MiddlewareChain::new();
    chain.add(mw);
    let mut ctx = test_context();
    ctx.prompt = "hello world".into();
    let result = chain.execute(&mut ctx).await;
    assert!(result.is_ok());
    assert_eq!(ctx.metadata.get("task_type"), Some(&"General".to_string()));
    // No methodology skills injected for General
}
```

## Step 6: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-api -- task_type 2>&1       # new tests pass
cargo test -p forge-process -- skill_router 2>&1 # skill_router tests pass
cargo test -p forge-api 2>&1                     # all API tests pass
```

## Rules

- Create `crates/forge-process/src/skill_router.rs` (new file)
- Add `pub mod skill_router;` to `crates/forge-process/src/lib.rs`
- Add `TaskTypeDetectionMiddleware` to `crates/forge-api/src/middleware.rs`
- Wire it into `crates/forge-api/src/routes/run.rs`
- Do NOT modify the existing `SkillInjectionMiddleware` — TaskTypeDetection is a separate middleware that runs AFTER it
- Do NOT touch forge-safety, forge-db, or frontend
- Commit with: `feat(api): add SkillRouter and TaskTypeDetection middleware`

## Report
```
STATUS: done | blocked
FILES_CREATED: [list]
FILES_MODIFIED: [list]
TESTS_ADDED: N
CHAIN_ORDER: [list middleware order after changes]
ISSUES: [any]
```
