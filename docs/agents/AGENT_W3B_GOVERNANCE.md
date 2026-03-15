# Agent W3-B: Governance Wiring

> You are Agent W3-B. Your job: make governance real. Wire company budgets to CostTracker, add approval gating before spawn, and inject active goals into agent prompts.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
crates/forge-api/src/middleware.rs                  — FULL FILE, study CostCheckMiddleware
crates/forge-api/src/routes/run.rs                 — middleware chain assembly
crates/forge-safety/src/cost.rs                    — CostTracker
crates/forge-db/src/repos/companies.rs             — CompanyRepo, Company struct
crates/forge-db/src/repos/goals.rs                 — GoalRepo, Goal struct
crates/forge-db/src/repos/approvals.rs             — ApprovalRepo
crates/forge-db/src/repos/agents.rs                — AgentRepo (to find agent's company)
crates/forge-db/src/repos/org_positions.rs         — OrgPositionRepo (agent → company link)
stat-qou-plan/REVISED_PLAN.md                      — what we're doing and why
```

## Step 2: Goal Injection

Modify `CostCheckMiddleware` or add a new `GovernanceMiddleware` in `crates/forge-api/src/middleware.rs` that:

1. Looks up the agent's company (via org_positions: agent_id → company_id)
2. Fetches active goals for that company (status = "in_progress" or "planned")
3. Injects them into `ctx.metadata["company_goals"]` as a formatted string
4. The SpawnMiddleware should include these goals in the system prompt

```rust
// In the middleware, after finding the company:
if let Ok(goals) = goal_repo.list_by_company(&company_id) {
    let active: Vec<String> = goals.iter()
        .filter(|g| g.status == "planned" || g.status == "in_progress")
        .map(|g| format!("- {} ({})", g.title, g.status))
        .collect();
    if !active.is_empty() {
        ctx.metadata.insert(
            "company_goals".into(),
            format!("Active company goals:\n{}", active.join("\n")),
        );
    }
}
```

## Step 3: Budget Enforcement

Modify `CostCheckMiddleware` in `crates/forge-api/src/middleware.rs` to:

1. Look up the agent's company via org_positions
2. Read the company's `budget_limit` and `budget_used`
3. If `budget_used >= budget_limit`, block the run with an error: "Company budget exhausted ($X of $Y used)"
4. If `budget_used >= budget_limit * 0.9`, add a warning to metadata but allow the run

This replaces the env-var-only CostTracker for company-scoped runs.

You'll need `CompanyRepo` and `OrgPositionRepo` available in the middleware. Add them to the middleware struct:

```rust
pub struct CostCheckMiddleware {
    pub cost_tracker: Arc<CostTracker>,
    pub company_repo: Arc<CompanyRepo>,
    pub org_position_repo: Arc<OrgPositionRepo>,
    pub goal_repo: Arc<GoalRepo>,
}
```

Update the chain assembly in `run.rs` to pass these repos.

## Step 4: Approval Gating

Add a check in the governance middleware: before spawn, if the company has any pending approvals of type "budget_increase" or "run_authorization", add a warning to metadata. For v1, don't hard-block — just make it visible.

```rust
if let Ok(approvals) = approval_repo.list_by_company(&company_id) {
    let pending: Vec<_> = approvals.iter()
        .filter(|a| a.status == "pending")
        .collect();
    if !pending.is_empty() {
        ctx.metadata.insert(
            "pending_approvals".into(),
            format!("{} pending approval(s)", pending.len()),
        );
    }
}
```

## Step 5: Wire into Chain

In `crates/forge-api/src/routes/run.rs`, update the `CostCheckMiddleware` instantiation to include the new repos:

```rust
chain.add(CostCheckMiddleware {
    cost_tracker: Arc::clone(&state.cost_tracker),
    company_repo: Arc::clone(&state.company_repo),
    org_position_repo: Arc::clone(&state.org_position_repo),
    goal_repo: Arc::clone(&state.goal_repo),
});
```

Make sure `AppState` has these fields accessible (it should already from Epic 1).

## Step 6: Write Tests

Add tests in `crates/forge-api/src/middleware.rs`:

```rust
#[tokio::test]
async fn cost_check_blocks_over_budget() {
    // Create company with budget_limit = 100, budget_used = 100
    // Run through middleware
    // Assert: error returned, run blocked
}

#[tokio::test]
async fn cost_check_warns_near_budget() {
    // Create company with budget_limit = 100, budget_used = 91
    // Run through middleware
    // Assert: warning in metadata, run allowed
}

#[tokio::test]
async fn goal_injection_adds_active_goals() {
    // Create company with 2 goals (1 planned, 1 completed)
    // Run through middleware
    // Assert: metadata contains only the planned goal
}
```

## Step 7: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-api 2>&1         # all tests pass
cargo test 2>&1 | grep "FAILED"     # no failures
```

## Rules

- ONLY modify `crates/forge-api/src/middleware.rs` and `crates/forge-api/src/routes/run.rs`
- You may also modify `crates/forge-api/src/state.rs` if needed to expose repos
- Do NOT modify forge-db, forge-safety, forge-process, or forge-core
- Do NOT touch frontend files
- Do NOT touch main.rs
- Do NOT modify existing tests — only add new ones
- Commit with: `feat(api): wire budget enforcement, goal injection, and approval visibility`

## Report
```
STATUS: done | blocked
FILES_MODIFIED: [list]
TESTS_ADDED: N
GOVERNANCE_FEATURES: [budget enforcement, goal injection, approval visibility]
MIDDLEWARE_CHANGES: [describe]
ISSUES: [any]
```
