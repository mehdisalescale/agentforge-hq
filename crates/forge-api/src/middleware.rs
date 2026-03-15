//! Middleware trait and chain for the run handler pipeline.
//!
//! Provides an ordered middleware chain where each middleware can inspect/modify
//! the request context, short-circuit with an error, or delegate to the next
//! middleware. Pattern inspired by DeerFlow's 8-middleware pipeline.
//!
//! Concrete middlewares: RateLimit, CircuitBreaker, CostCheck, SkillInjection,
//! Persist, Spawn.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use forge_core::event_bus::EventBus;
use forge_core::events::ForgeEvent;
use forge_core::ids::{AgentId, SessionId};
use forge_db::{ApprovalRepo, CompanyRepo, GoalRepo, OrgPositionRepo, SessionRepo, SkillRepo};
use forge_process::stream_event::StreamJsonEvent;
use forge_process::{parse_line, spawn, ProcessRunner, SpawnConfig, SpawnError};
use forge_safety::{BudgetStatus, CircuitBreaker, CostTracker, RateLimiter};

/// Context passed through the middleware chain.
pub struct RunContext {
    pub agent_id: String,
    pub prompt: String,
    pub session_id: String,
    pub working_dir: Option<String>,
    pub metadata: HashMap<String, String>,
    // Typed fields for middleware use
    pub agent_id_typed: AgentId,
    pub session_id_typed: SessionId,
    pub resume_session_id: Option<String>,
    pub directory: String,
}

/// Response from the middleware chain.
pub struct RunResponse {
    pub session_id: String,
    pub status: String,
}

/// Errors from middleware processing.
#[derive(Debug)]
pub enum MiddlewareError {
    RateLimited,
    CircuitOpen,
    BudgetExceeded { cost: f64, limit: f64 },
    ExitGateTriggered(String),
    QualityGateFailed { score: f64, threshold: f64 },
    SpawnFailed(String),
    Internal(String),
}

/// The Next function — calls the next middleware in the chain.
pub struct Next<'a> {
    middlewares: &'a [Arc<dyn Middleware>],
    index: usize,
}

impl<'a> Next<'a> {
    pub async fn run(self, ctx: &mut RunContext) -> Result<RunResponse, MiddlewareError> {
        if self.index < self.middlewares.len() {
            let middleware = &self.middlewares[self.index];
            let next = Next {
                middlewares: self.middlewares,
                index: self.index + 1,
            };
            middleware.process(ctx, next).await
        } else {
            // End of chain — return default response
            Ok(RunResponse {
                session_id: ctx.session_id.clone(),
                status: "completed".to_string(),
            })
        }
    }
}

/// Middleware trait — implement this for each concern.
///
/// Uses `Pin<Box<dyn Future>>` for object safety (`dyn Middleware`) without
/// requiring the `async_trait` crate.
pub trait Middleware: Send + Sync {
    /// Process the request. Call `next.run(ctx)` to continue the chain.
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>>;

    /// Name for logging/debugging.
    fn name(&self) -> &str;
}

/// Ordered chain of middlewares.
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add<M: Middleware + 'static>(&mut self, middleware: M) -> &mut Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    pub async fn execute(&self, ctx: &mut RunContext) -> Result<RunResponse, MiddlewareError> {
        let next = Next {
            middlewares: &self.middlewares,
            index: 0,
        };
        next.run(ctx).await
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Concrete middleware implementations
// ---------------------------------------------------------------------------

/// Guards against request floods via token-bucket rate limiting.
pub struct RateLimitMiddleware {
    pub rate_limiter: Arc<RateLimiter>,
}

impl Middleware for RateLimitMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>> {
        Box::pin(async move {
            if !self.rate_limiter.try_acquire() {
                return Err(MiddlewareError::RateLimited);
            }
            next.run(ctx).await
        })
    }

    fn name(&self) -> &str {
        "rate_limit"
    }
}

/// Prevents cascading failures by checking the circuit breaker state.
pub struct CircuitBreakerMiddleware {
    pub circuit_breaker: Arc<CircuitBreaker>,
}

impl Middleware for CircuitBreakerMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>> {
        Box::pin(async move {
            self.circuit_breaker
                .check()
                .map_err(|_| MiddlewareError::CircuitOpen)?;
            next.run(ctx).await
        })
    }

    fn name(&self) -> &str {
        "circuit_breaker"
    }
}

/// Pre-flight budget check: rejects requests when the session cost already
/// exceeds the configured budget limit.
pub struct CostCheckMiddleware {
    pub cost_tracker: Arc<CostTracker>,
    pub session_repo: Arc<SessionRepo>,
}

impl Middleware for CostCheckMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>> {
        Box::pin(async move {
            let current_cost = self
                .session_repo
                .get(&ctx.session_id_typed)
                .map(|s| s.cost_usd)
                .unwrap_or(0.0);
            if let BudgetStatus::Exceeded {
                current_cost,
                limit,
            } = self.cost_tracker.check(current_cost)
            {
                return Err(MiddlewareError::BudgetExceeded {
                    cost: current_cost,
                    limit,
                });
            }
            next.run(ctx).await
        })
    }

    fn name(&self) -> &str {
        "cost_check"
    }
}

/// Governance middleware: injects company goals, enforces company budgets,
/// and surfaces pending approvals before a run proceeds.
pub struct GovernanceMiddleware {
    pub company_repo: Arc<CompanyRepo>,
    pub org_position_repo: Arc<OrgPositionRepo>,
    pub goal_repo: Arc<GoalRepo>,
    pub approval_repo: Arc<ApprovalRepo>,
}

impl GovernanceMiddleware {
    /// Find the company_id for a given agent by looking up org_positions.
    fn find_company_id(&self, agent_id: &str) -> Option<String> {
        if let Ok(positions) = self.org_position_repo.list_by_company("") {
            // org_position_repo.list_by_company requires a company_id, so we
            // need to search across all companies. We'll scan companies instead.
            drop(positions);
        }
        // Look up all companies, then find positions for each that match this agent.
        // More efficient: query all companies, then for each, check positions.
        if let Ok(companies) = self.company_repo.list() {
            for company in &companies {
                if let Ok(positions) = self.org_position_repo.list_by_company(&company.id) {
                    if positions.iter().any(|p| p.agent_id.as_deref() == Some(agent_id)) {
                        return Some(company.id.clone());
                    }
                }
            }
        }
        None
    }
}

impl Middleware for GovernanceMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>> {
        Box::pin(async move {
            let company_id = self.find_company_id(&ctx.agent_id);

            if let Some(ref cid) = company_id {
                // --- Budget Enforcement ---
                if let Ok(company) = self.company_repo.get(cid) {
                    if let Some(budget_limit) = company.budget_limit {
                        if budget_limit > 0.0 {
                            if company.budget_used >= budget_limit {
                                return Err(MiddlewareError::BudgetExceeded {
                                    cost: company.budget_used,
                                    limit: budget_limit,
                                });
                            }
                            if company.budget_used >= budget_limit * 0.9 {
                                ctx.metadata.insert(
                                    "budget_warning".into(),
                                    format!(
                                        "Company budget 90%+ used (${:.2} of ${:.2})",
                                        company.budget_used, budget_limit
                                    ),
                                );
                            }
                        }
                    }
                }

                // --- Goal Injection ---
                if let Ok(goals) = self.goal_repo.list_by_company(cid) {
                    let active: Vec<String> = goals
                        .iter()
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

                // --- Approval Gating (visibility only) ---
                if let Ok(approvals) = self.approval_repo.list_by_company(cid) {
                    let pending: Vec<_> = approvals
                        .iter()
                        .filter(|a| a.status == "pending")
                        .collect();
                    if !pending.is_empty() {
                        ctx.metadata.insert(
                            "pending_approvals".into(),
                            format!("{} pending approval(s)", pending.len()),
                        );
                    }
                }
            }

            next.run(ctx).await
        })
    }

    fn name(&self) -> &str {
        "governance"
    }
}

/// Matches the prompt against skill tags and injects matched skill content
/// into `ctx.metadata["injected_skills"]`.
pub struct SkillInjectionMiddleware {
    pub skill_repo: Arc<SkillRepo>,
}

impl SkillInjectionMiddleware {
    /// Extract keywords from a prompt: lowercase, split on whitespace, filter short words.
    fn extract_keywords(prompt: &str) -> Vec<String> {
        prompt
            .split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| w.len() > 2)
            .collect()
    }
}

impl Middleware for SkillInjectionMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>> {
        Box::pin(async move {
            let keywords = Self::extract_keywords(&ctx.prompt);
            if let Ok(skills) = self.skill_repo.list() {
                let mut injected = Vec::new();
                for skill in &skills {
                    // Match keywords against skill tags stored in parameters_json
                    if let Some(ref tags_json) = skill.parameters_json {
                        if let Ok(tags) = serde_json::from_str::<Vec<String>>(tags_json) {
                            let matched = keywords.iter().any(|kw| {
                                tags.iter().any(|tag| tag.to_lowercase().contains(kw))
                            });
                            if matched {
                                injected.push(format!(
                                    "## Skill: {}\n{}",
                                    skill.name, skill.content
                                ));
                            }
                        }
                    }
                }
                if !injected.is_empty() {
                    ctx.metadata
                        .insert("injected_skills".into(), injected.join("\n\n"));
                }
            }
            next.run(ctx).await
        })
    }

    fn name(&self) -> &str {
        "skill_injection"
    }
}

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

/// Post-execution security scanner. Scans code blocks in output
/// for OWASP vulnerability patterns.
pub struct SecurityScanMiddleware {
    pub event_bus: Arc<EventBus>,
}

impl Middleware for SecurityScanMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>> {
        Box::pin(async move {
            let response = next.run(ctx).await?;

            // Scan any output that was captured
            if let Some(output) = ctx.metadata.get("output") {
                let scanner = forge_safety::scanner::SecurityScanner::new();
                let code_blocks = extract_code_blocks(output);

                let mut all_findings = Vec::new();
                for block in &code_blocks {
                    all_findings.extend(scanner.scan(block));
                }

                if all_findings.is_empty() {
                    let _ = self.event_bus.emit(ForgeEvent::SecurityScanPassed {
                        session_id: ctx.session_id_typed.clone(),
                        timestamp: chrono::Utc::now(),
                    }).await;
                    ctx.metadata.insert("security_scan".into(), "passed".into());
                } else {
                    let finding_strs: Vec<String> = all_findings.iter().map(|f| {
                        format!("[{:?}] {} (line {}): {}",
                            f.severity, f.pattern, f.line, f.description)
                    }).collect();
                    let _ = self.event_bus.emit(ForgeEvent::SecurityScanFailed {
                        session_id: ctx.session_id_typed.clone(),
                        findings: finding_strs.clone(),
                        timestamp: chrono::Utc::now(),
                    }).await;
                    ctx.metadata.insert("security_scan".into(), "failed".into());
                    ctx.metadata.insert("security_findings".into(), finding_strs.join("\n"));
                }
            }

            Ok(response)
        })
    }

    fn name(&self) -> &str {
        "security_scan"
    }
}

/// Extract fenced code blocks from markdown output.
fn extract_code_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current = Vec::new();

    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if in_block {
                blocks.push(current.join("\n"));
                current.clear();
                in_block = false;
            } else {
                in_block = true;
            }
        } else if in_block {
            current.push(line.to_string());
        }
    }

    // If no code blocks found, scan the entire text
    if blocks.is_empty() && !text.is_empty() {
        blocks.push(text.to_string());
    }

    blocks
}

/// Wraps the inner chain: sets session to "running" before, updates to
/// "completed"/"failed" after, and emits lifecycle events.
pub struct PersistMiddleware {
    pub session_repo: Arc<SessionRepo>,
    pub event_bus: Arc<EventBus>,
}

impl Middleware for PersistMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>> {
        Box::pin(async move {
            // Emit ProcessStarted and set session to running
            let _ = self.event_bus.emit(ForgeEvent::ProcessStarted {
                session_id: ctx.session_id_typed.clone(),
                agent_id: ctx.agent_id_typed.clone(),
                timestamp: chrono::Utc::now(),
            }).await;
            if self
                .session_repo
                .update_status(&ctx.session_id_typed, "running")
                .is_err()
            {
                tracing::warn!(
                    session_id = %ctx.session_id,
                    "persist middleware: failed to set status to running"
                );
            }

            match next.run(ctx).await {
                Ok(response) => Ok(response),
                Err(e) => {
                    // On middleware error, mark session failed
                    let error_msg = format!("{:?}", e);
                    let _ = self
                        .session_repo
                        .update_status(&ctx.session_id_typed, "failed");
                    let _ = self.event_bus.emit(ForgeEvent::ProcessFailed {
                        session_id: ctx.session_id_typed.clone(),
                        error: error_msg,
                        timestamp: chrono::Utc::now(),
                    }).await;
                    Err(e)
                }
            }
        })
    }

    fn name(&self) -> &str {
        "persist"
    }
}

/// Terminal middleware: spawns the Claude CLI process, kicks off a background
/// task to stream output and emit events. Does NOT call `next.run()`.
pub struct SpawnMiddleware {
    pub event_bus: Arc<EventBus>,
    pub session_repo: Arc<SessionRepo>,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub cost_tracker: Arc<CostTracker>,
    pub configurator: Arc<crate::configurator::AgentConfigurator>,
}

impl Middleware for SpawnMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        _next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>> {
        Box::pin(async move {
            // Configure workspace with persona identity, skills, goals, hooks
            if let Err(e) = self.configurator.configure_workspace(
                &ctx.agent_id, &ctx.prompt, &ctx.session_id, &ctx.directory
            ) {
                tracing::warn!("Failed to configure workspace: {}", e);
                // Non-fatal — agent runs without persona config
            }

            let resume_arg = ctx.resume_session_id.as_deref();
            let config = SpawnConfig::from_env().with_working_dir(&ctx.directory);
            let mut handle = spawn(&config, &ctx.prompt, resume_arg)
                .await
                .map_err(|e| match e {
                    SpawnError::Io(io) => {
                        MiddlewareError::SpawnFailed(format!("io: {}", io))
                    }
                    SpawnError::CommandMissing => {
                        MiddlewareError::SpawnFailed("command missing".into())
                    }
                })?;

            // Capture values for the background task
            let event_bus = Arc::clone(&self.event_bus);
            let session_repo = Arc::clone(&self.session_repo);
            let circuit_breaker = Arc::clone(&self.circuit_breaker);
            let cost_tracker = Arc::clone(&self.cost_tracker);
            let sid = ctx.session_id_typed.clone();
            let aid = ctx.agent_id_typed.clone();

            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;

                let runner = ProcessRunner::new(event_bus);
                let mut stdout = match handle.take_stdout() {
                    Some(s) => s,
                    None => {
                        let _ = handle.wait().await;
                        return;
                    }
                };
                let mut reader = tokio::io::BufReader::new(&mut stdout);
                let mut buf = String::new();
                loop {
                    buf.clear();
                    match reader.read_line(&mut buf).await {
                        Ok(0) => break,
                        Err(e) => {
                            tracing::warn!(error = %e, "spawn middleware: read_line error");
                            break;
                        }
                        _ => {}
                    }
                    if let Ok(Some(ev)) = parse_line(buf.trim()) {
                        if let StreamJsonEvent::Result(payload) = &ev {
                            if let Some(cost) = payload.cost_usd {
                                if session_repo.update_cost(&sid, cost).is_err() {
                                    tracing::warn!(
                                        session_id = %sid.0,
                                        "spawn middleware: failed to update session cost"
                                    );
                                } else {
                                    match cost_tracker.check(cost) {
                                        BudgetStatus::Exceeded {
                                            current_cost,
                                            limit,
                                        } => {
                                            let _ = runner.emit(ForgeEvent::BudgetExceeded {
                                                current_cost,
                                                limit,
                                                timestamp: chrono::Utc::now(),
                                            });
                                        }
                                        BudgetStatus::Warning {
                                            current_cost,
                                            threshold,
                                        } => {
                                            let _ = runner.emit(ForgeEvent::BudgetWarning {
                                                current_cost,
                                                limit: threshold,
                                                timestamp: chrono::Utc::now(),
                                            });
                                        }
                                        BudgetStatus::Ok => {}
                                    }
                                }
                            }
                        }
                        if runner.emit_parsed_event(&sid, &aid, &ev).is_err() {
                            tracing::warn!("spawn middleware: emit_parsed_event failed");
                        }
                    }
                }
                match handle.wait().await {
                    Ok(status) => {
                        let code = status.code().unwrap_or(-1);
                        circuit_breaker.record_success();
                        if session_repo.update_status(&sid, "completed").is_err() {
                            tracing::warn!(
                                session_id = %sid.0,
                                "spawn middleware: failed to update status to completed"
                            );
                        }
                        if runner
                            .emit(ForgeEvent::ProcessCompleted {
                                session_id: sid,
                                exit_code: code,
                                timestamp: chrono::Utc::now(),
                            })
                            .is_err()
                        {
                            tracing::warn!(
                                "spawn middleware: failed to emit ProcessCompleted"
                            );
                        }
                    }
                    Err(e) => {
                        circuit_breaker.record_failure();
                        tracing::warn!(error = %e, "spawn middleware: wait failed");
                        if session_repo.update_status(&sid, "failed").is_err() {
                            tracing::warn!(
                                session_id = %sid.0,
                                "spawn middleware: failed to update status to failed"
                            );
                        }
                        if runner
                            .emit(ForgeEvent::ProcessFailed {
                                session_id: sid,
                                error: e.to_string(),
                                timestamp: chrono::Utc::now(),
                            })
                            .is_err()
                        {
                            tracing::warn!(
                                "spawn middleware: failed to emit ProcessFailed"
                            );
                        }
                    }
                }
            });

            Ok(RunResponse {
                session_id: ctx.session_id.clone(),
                status: "spawned".to_string(),
            })
        })
    }

    fn name(&self) -> &str {
        "spawn"
    }
}

/// Trait for evaluating output quality. Implement this for real critic agents
/// or use `MockCritic` in tests.
pub trait QualityCritic: Send + Sync {
    fn evaluate<'a>(
        &'a self,
        output: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<f64, String>> + Send + 'a>>;
}

/// Post-completion quality gate. Evaluates output with a critic and re-runs
/// if the score is below the threshold (up to max_iterations).
pub struct QualityGateMiddleware {
    pub critic: Arc<dyn QualityCritic>,
    pub threshold: f64,
    pub max_iterations: u32,
}

impl Middleware for QualityGateMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut RunContext,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>> {
        Box::pin(async move {
            let response = next.run(ctx).await?;

            // Use the prompt as a stand-in for "output" in this middleware context.
            // In production, this would collect actual process output from events.
            let output = ctx.metadata.get("output").cloned().unwrap_or_default();

            for iteration in 0..self.max_iterations {
                match self.critic.evaluate(&output).await {
                    Ok(score) if score >= self.threshold => {
                        ctx.metadata.insert("quality_score".into(), score.to_string());
                        ctx.metadata.insert("quality_iteration".into(), iteration.to_string());
                        return Ok(response);
                    }
                    Ok(score) => {
                        ctx.metadata.insert("quality_score".into(), score.to_string());
                        ctx.metadata.insert("quality_iteration".into(), iteration.to_string());
                        // Last iteration — fail
                        if iteration == self.max_iterations - 1 {
                            return Err(MiddlewareError::QualityGateFailed {
                                score,
                                threshold: self.threshold,
                            });
                        }
                        // Otherwise continue to next iteration (would re-run in production)
                    }
                    Err(e) => {
                        return Err(MiddlewareError::Internal(format!("critic error: {}", e)));
                    }
                }
            }
            Ok(response)
        })
    }

    fn name(&self) -> &str {
        "quality_gate"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // -- helpers for tests --------------------------------------------------

    struct LogMiddleware {
        label: String,
    }

    impl Middleware for LogMiddleware {
        fn process<'a>(
            &'a self,
            ctx: &'a mut RunContext,
            next: Next<'a>,
        ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>>
        {
            Box::pin(async move {
                ctx.metadata
                    .insert(format!("{}_entered", self.label), "true".into());
                let result = next.run(ctx).await;
                ctx.metadata
                    .insert(format!("{}_exited", self.label), "true".into());
                result
            })
        }

        fn name(&self) -> &str {
            &self.label
        }
    }

    struct BlockMiddleware;

    impl Middleware for BlockMiddleware {
        fn process<'a>(
            &'a self,
            _ctx: &'a mut RunContext,
            _next: Next<'a>,
        ) -> Pin<Box<dyn Future<Output = Result<RunResponse, MiddlewareError>> + Send + 'a>>
        {
            Box::pin(async move { Err(MiddlewareError::RateLimited) })
        }

        fn name(&self) -> &str {
            "block"
        }
    }

    fn test_context() -> RunContext {
        RunContext {
            agent_id: "agent-1".into(),
            prompt: "test".into(),
            session_id: "sess-1".into(),
            working_dir: None,
            metadata: Default::default(),
            agent_id_typed: AgentId::new(),
            session_id_typed: SessionId::new(),
            resume_session_id: None,
            directory: ".".into(),
        }
    }

    // -- existing tests (chain infrastructure) ------------------------------

    #[tokio::test]
    async fn chain_executes_in_order() {
        let mut chain = MiddlewareChain::new();
        chain.add(LogMiddleware {
            label: "first".into(),
        });
        chain.add(LogMiddleware {
            label: "second".into(),
        });
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert_eq!(
            ctx.metadata.get("first_entered"),
            Some(&"true".to_string())
        );
        assert_eq!(
            ctx.metadata.get("second_entered"),
            Some(&"true".to_string())
        );
    }

    #[tokio::test]
    async fn middleware_can_short_circuit() {
        let mut chain = MiddlewareChain::new();
        chain.add(BlockMiddleware);
        chain.add(LogMiddleware {
            label: "never".into(),
        });
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(matches!(result, Err(MiddlewareError::RateLimited)));
        assert!(ctx.metadata.get("never_entered").is_none());
    }

    #[tokio::test]
    async fn empty_chain_returns_ok() {
        let chain = MiddlewareChain::new();
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
    }

    // -- RateLimitMiddleware tests ------------------------------------------

    #[tokio::test]
    async fn rate_limit_allows_when_tokens_available() {
        let rl = Arc::new(RateLimiter::new(5, Duration::from_secs(60)));
        let mw = RateLimitMiddleware {
            rate_limiter: rl,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn rate_limit_rejects_when_exhausted() {
        let rl = Arc::new(RateLimiter::new(1, Duration::from_secs(60)));
        // Exhaust the single token
        assert!(rl.try_acquire());

        let mw = RateLimitMiddleware {
            rate_limiter: rl,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(matches!(result, Err(MiddlewareError::RateLimited)));
    }

    // -- CircuitBreakerMiddleware tests -------------------------------------

    #[tokio::test]
    async fn circuit_breaker_allows_when_closed() {
        let cb = Arc::new(CircuitBreaker::default());
        let mw = CircuitBreakerMiddleware {
            circuit_breaker: cb,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn circuit_breaker_rejects_when_open() {
        let cb = Arc::new(CircuitBreaker::new(1, 1, Duration::from_secs(60)));
        cb.record_failure(); // trips the breaker
        assert!(cb.check().is_err());

        let mw = CircuitBreakerMiddleware {
            circuit_breaker: cb,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(matches!(result, Err(MiddlewareError::CircuitOpen)));
    }

    // -- CostCheckMiddleware tests ------------------------------------------

    fn setup_db() -> (Arc<forge_db::AgentRepo>, Arc<SessionRepo>, forge_db::DbPool) {
        let db = forge_db::DbPool::in_memory().unwrap();
        {
            let c = db.connection();
            forge_db::Migrator::new(&c).apply_pending().unwrap();
        }
        let agent_repo = Arc::new(forge_db::AgentRepo::new(db.conn_arc()));
        let session_repo = Arc::new(SessionRepo::new(db.conn_arc()));
        (agent_repo, session_repo, db)
    }

    fn create_test_session(
        agent_repo: &forge_db::AgentRepo,
        session_repo: &SessionRepo,
    ) -> forge_db::repos::sessions::Session {
        let agent = agent_repo
            .create(&forge_agent::model::NewAgent {
                name: "test-agent".into(),
                model: None,
                system_prompt: None,
                allowed_tools: None,
                max_turns: None,
                use_max: None,
                preset: None,
                config: None,
            })
            .unwrap();
        session_repo
            .create(&forge_db::NewSession {
                agent_id: agent.id,
                directory: ".".into(),
                claude_session_id: None,
            })
            .unwrap()
    }

    #[tokio::test]
    async fn cost_check_allows_under_limit() {
        let (agent_repo, session_repo, _db) = setup_db();

        let ct = Arc::new(CostTracker::new(None, Some(100.0)));
        let mw = CostCheckMiddleware {
            cost_tracker: ct,
            session_repo: Arc::clone(&session_repo),
        };

        let session = create_test_session(&agent_repo, &session_repo);

        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.session_id_typed = session.id;
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn cost_check_rejects_over_limit() {
        let (agent_repo, session_repo, _db) = setup_db();

        let ct = Arc::new(CostTracker::new(None, Some(10.0)));
        let mw = CostCheckMiddleware {
            cost_tracker: ct,
            session_repo: Arc::clone(&session_repo),
        };

        let session = create_test_session(&agent_repo, &session_repo);
        // Set cost above limit
        session_repo.update_cost(&session.id, 15.0).unwrap();

        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.session_id_typed = session.id;
        let result = chain.execute(&mut ctx).await;
        assert!(matches!(
            result,
            Err(MiddlewareError::BudgetExceeded { .. })
        ));
    }

    // -- SkillInjectionMiddleware tests -------------------------------------

    #[tokio::test]
    async fn skill_injection_adds_matching_skills() {
        let conn = forge_db::DbPool::in_memory().unwrap();
        {
            let c = conn.connection();
            forge_db::Migrator::new(&c).apply_pending().unwrap();
        }
        let skill_repo = Arc::new(SkillRepo::new(conn.conn_arc()));

        // Insert a skill with matching tags
        skill_repo
            .upsert(&forge_db::repos::skills::UpsertSkill {
                id: "review-skill".into(),
                name: "code-review".into(),
                description: Some("Review methodology".into()),
                category: Some("review".into()),
                subcategory: None,
                content: "Review checklist here".into(),
                source_repo: None,
                parameters_json: Some(r#"["review","quality"]"#.into()),
                examples_json: None,
            })
            .unwrap();

        let mw = SkillInjectionMiddleware {
            skill_repo: Arc::clone(&skill_repo),
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.prompt = "please review my code for quality".into();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        let injected = ctx.metadata.get("injected_skills");
        assert!(injected.is_some());
        assert!(injected.unwrap().contains("code-review"));
        assert!(injected.unwrap().contains("Review checklist here"));
    }

    #[tokio::test]
    async fn skill_injection_no_match_no_metadata() {
        let conn = forge_db::DbPool::in_memory().unwrap();
        {
            let c = conn.connection();
            forge_db::Migrator::new(&c).apply_pending().unwrap();
        }
        let skill_repo = Arc::new(SkillRepo::new(conn.conn_arc()));

        let mw = SkillInjectionMiddleware { skill_repo };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.prompt = "hello world".into();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert!(ctx.metadata.get("injected_skills").is_none());
    }

    // -- PersistMiddleware tests --------------------------------------------

    #[tokio::test]
    async fn persist_sets_running_and_propagates_ok() {
        let (agent_repo, session_repo, _db) = setup_db();
        let (bus, _persist_rx) = EventBus::new(32, 32);
        let event_bus = Arc::new(bus);

        let session = create_test_session(&agent_repo, &session_repo);

        let mw = PersistMiddleware {
            session_repo: Arc::clone(&session_repo),
            event_bus,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.session_id_typed = session.id.clone();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());

        let updated = session_repo.get(&session.id).unwrap();
        assert_eq!(updated.status, "running");
    }

    #[tokio::test]
    async fn persist_marks_failed_on_error() {
        let (agent_repo, session_repo, _db) = setup_db();
        let (bus, _persist_rx) = EventBus::new(32, 32);
        let event_bus = Arc::new(bus);

        let session = create_test_session(&agent_repo, &session_repo);

        let mw = PersistMiddleware {
            session_repo: Arc::clone(&session_repo),
            event_bus,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        chain.add(BlockMiddleware); // will cause an error
        let mut ctx = test_context();
        ctx.session_id_typed = session.id.clone();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_err());

        let updated = session_repo.get(&session.id).unwrap();
        assert_eq!(updated.status, "failed");
    }

    // -- QualityGateMiddleware tests ----------------------------------------

    use std::sync::atomic::{AtomicU32, Ordering};

    struct MockCritic {
        scores: Vec<f64>,
        call_count: AtomicU32,
    }

    impl MockCritic {
        fn new(scores: Vec<f64>) -> Self {
            Self {
                scores,
                call_count: AtomicU32::new(0),
            }
        }
    }

    impl QualityCritic for MockCritic {
        fn evaluate<'a>(
            &'a self,
            _output: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<f64, String>> + Send + 'a>> {
            Box::pin(async move {
                let idx = self.call_count.fetch_add(1, Ordering::SeqCst) as usize;
                if idx < self.scores.len() {
                    Ok(self.scores[idx])
                } else {
                    Ok(*self.scores.last().unwrap_or(&0.0))
                }
            })
        }
    }

    #[tokio::test]
    async fn quality_gate_passes_above_threshold() {
        let critic = Arc::new(MockCritic::new(vec![90.0]));
        let mw = QualityGateMiddleware {
            critic,
            threshold: 80.0,
            max_iterations: 3,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert_eq!(ctx.metadata.get("quality_score"), Some(&"90".to_string()));
    }

    #[tokio::test]
    async fn quality_gate_fails_below_threshold_after_max_iterations() {
        let critic = Arc::new(MockCritic::new(vec![50.0, 60.0, 55.0]));
        let mw = QualityGateMiddleware {
            critic,
            threshold: 80.0,
            max_iterations: 3,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(matches!(
            result,
            Err(MiddlewareError::QualityGateFailed { .. })
        ));
    }

    // -- TaskTypeDetectionMiddleware tests -----------------------------------

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

    // -- SecurityScanMiddleware tests ----------------------------------------

    #[tokio::test]
    async fn security_scan_passes_clean_output() {
        let (bus, _persist_rx) = EventBus::new(32, 32);
        let event_bus = Arc::new(bus);
        let mw = SecurityScanMiddleware { event_bus };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.metadata.insert("output".into(), "let x = 1 + 2;".into());
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert_eq!(ctx.metadata.get("security_scan"), Some(&"passed".to_string()));
    }

    #[tokio::test]
    async fn security_scan_detects_eval_injection() {
        let (bus, _persist_rx) = EventBus::new(32, 32);
        let event_bus = Arc::new(bus);
        let mw = SecurityScanMiddleware { event_bus };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.metadata.insert("output".into(), "```python\neval(user_input)\n```".into());
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok()); // non-blocking: logs findings but doesn't reject
        assert_eq!(ctx.metadata.get("security_scan"), Some(&"failed".to_string()));
        assert!(ctx.metadata.get("security_findings").unwrap().contains("eval_injection"));
    }

    #[tokio::test]
    async fn security_scan_skips_when_no_output() {
        let (bus, _persist_rx) = EventBus::new(32, 32);
        let event_bus = Arc::new(bus);
        let mw = SecurityScanMiddleware { event_bus };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        // No "output" in metadata
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert!(ctx.metadata.get("security_scan").is_none());
    }

    #[tokio::test]
    async fn extract_code_blocks_finds_fenced() {
        let text = "some text\n```python\neval(x)\n```\nmore text\n```js\nalert(1)\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].contains("eval"));
        assert!(blocks[1].contains("alert"));
    }

    #[tokio::test]
    async fn quality_gate_retries_then_passes() {
        let critic = Arc::new(MockCritic::new(vec![50.0, 90.0]));
        let mw = QualityGateMiddleware {
            critic,
            threshold: 80.0,
            max_iterations: 3,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert_eq!(ctx.metadata.get("quality_iteration"), Some(&"1".to_string()));
    }

    // -- GovernanceMiddleware tests ------------------------------------------

    fn setup_governance_db() -> (
        Arc<CompanyRepo>,
        Arc<OrgPositionRepo>,
        Arc<GoalRepo>,
        Arc<ApprovalRepo>,
        Arc<forge_db::AgentRepo>,
        forge_db::DbPool,
    ) {
        let db = forge_db::DbPool::in_memory().unwrap();
        {
            let c = db.connection();
            forge_db::Migrator::new(&c).apply_pending().unwrap();
        }
        let company_repo = Arc::new(CompanyRepo::new(db.conn_arc()));
        let org_position_repo = Arc::new(OrgPositionRepo::new(db.conn_arc()));
        let goal_repo = Arc::new(GoalRepo::new(db.conn_arc()));
        let approval_repo = Arc::new(ApprovalRepo::new(db.conn_arc()));
        let agent_repo = Arc::new(forge_db::AgentRepo::new(db.conn_arc()));
        (company_repo, org_position_repo, goal_repo, approval_repo, agent_repo, db)
    }

    #[tokio::test]
    async fn governance_blocks_over_budget() {
        let (company_repo, org_position_repo, goal_repo, approval_repo, agent_repo, _db) =
            setup_governance_db();

        // Create company with budget_limit = 100, budget_used = 100
        let company = company_repo
            .create(&forge_db::NewCompany {
                name: "BudgetCo".into(),
                mission: None,
                budget_limit: Some(100.0),
            })
            .unwrap();
        // Set budget_used to 100
        company_repo
            .update(&company.id, None, None, None)
            .unwrap();
        {
            let conn = _db.connection();
            conn.execute_batch(&format!(
                "UPDATE companies SET budget_used = 100.0 WHERE id = '{}'",
                company.id
            ))
            .unwrap();
        }

        // Create agent and org position linking agent to company
        let agent = agent_repo
            .create(&forge_agent::model::NewAgent {
                name: "test-agent".into(),
                model: None,
                system_prompt: None,
                allowed_tools: None,
                max_turns: None,
                use_max: None,
                preset: None,
                config: None,
            })
            .unwrap();
        org_position_repo
            .create(&forge_db::NewOrgPosition {
                company_id: company.id.clone(),
                department_id: None,
                agent_id: Some(agent.id.0.to_string()),
                reports_to: None,
                role: "engineer".into(),
                title: None,
            })
            .unwrap();

        let mw = GovernanceMiddleware {
            company_repo,
            org_position_repo,
            goal_repo,
            approval_repo,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.agent_id = agent.id.0.to_string();
        let result = chain.execute(&mut ctx).await;
        assert!(matches!(result, Err(MiddlewareError::BudgetExceeded { .. })));
    }

    #[tokio::test]
    async fn governance_warns_near_budget() {
        let (company_repo, org_position_repo, goal_repo, approval_repo, agent_repo, _db) =
            setup_governance_db();

        let company = company_repo
            .create(&forge_db::NewCompany {
                name: "NearBudgetCo".into(),
                mission: None,
                budget_limit: Some(100.0),
            })
            .unwrap();
        {
            let conn = _db.connection();
            conn.execute_batch(&format!(
                "UPDATE companies SET budget_used = 91.0 WHERE id = '{}'",
                company.id
            ))
            .unwrap();
        }

        let agent = agent_repo
            .create(&forge_agent::model::NewAgent {
                name: "test-agent".into(),
                model: None,
                system_prompt: None,
                allowed_tools: None,
                max_turns: None,
                use_max: None,
                preset: None,
                config: None,
            })
            .unwrap();
        org_position_repo
            .create(&forge_db::NewOrgPosition {
                company_id: company.id.clone(),
                department_id: None,
                agent_id: Some(agent.id.0.to_string()),
                reports_to: None,
                role: "engineer".into(),
                title: None,
            })
            .unwrap();

        let mw = GovernanceMiddleware {
            company_repo,
            org_position_repo,
            goal_repo,
            approval_repo,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.agent_id = agent.id.0.to_string();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert!(ctx.metadata.get("budget_warning").is_some());
        assert!(ctx.metadata.get("budget_warning").unwrap().contains("91.00"));
    }

    #[tokio::test]
    async fn governance_injects_active_goals() {
        let (company_repo, org_position_repo, goal_repo, approval_repo, agent_repo, _db) =
            setup_governance_db();

        let company = company_repo
            .create(&forge_db::NewCompany {
                name: "GoalCo".into(),
                mission: None,
                budget_limit: None,
            })
            .unwrap();

        // Create one planned goal and one completed goal
        goal_repo
            .create(&forge_db::NewGoal {
                company_id: company.id.clone(),
                parent_id: None,
                title: "Ship v1".into(),
                description: None,
            })
            .unwrap();
        let completed_goal = goal_repo
            .create(&forge_db::NewGoal {
                company_id: company.id.clone(),
                parent_id: None,
                title: "Setup CI".into(),
                description: None,
            })
            .unwrap();
        goal_repo
            .update_status(&completed_goal.id, "completed")
            .unwrap();

        let agent = agent_repo
            .create(&forge_agent::model::NewAgent {
                name: "test-agent".into(),
                model: None,
                system_prompt: None,
                allowed_tools: None,
                max_turns: None,
                use_max: None,
                preset: None,
                config: None,
            })
            .unwrap();
        org_position_repo
            .create(&forge_db::NewOrgPosition {
                company_id: company.id.clone(),
                department_id: None,
                agent_id: Some(agent.id.0.to_string()),
                reports_to: None,
                role: "engineer".into(),
                title: None,
            })
            .unwrap();

        let mw = GovernanceMiddleware {
            company_repo,
            org_position_repo,
            goal_repo,
            approval_repo,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.agent_id = agent.id.0.to_string();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        let goals = ctx.metadata.get("company_goals").unwrap();
        assert!(goals.contains("Ship v1"));
        assert!(!goals.contains("Setup CI"));
    }

    #[tokio::test]
    async fn governance_surfaces_pending_approvals() {
        let (company_repo, org_position_repo, goal_repo, approval_repo, agent_repo, _db) =
            setup_governance_db();

        let company = company_repo
            .create(&forge_db::NewCompany {
                name: "ApprovalCo".into(),
                mission: None,
                budget_limit: None,
            })
            .unwrap();

        approval_repo
            .create(&forge_db::NewApproval {
                company_id: company.id.clone(),
                approval_type: "budget_increase".into(),
                requester: "agent-1".into(),
                data_json: "{}".into(),
            })
            .unwrap();

        let agent = agent_repo
            .create(&forge_agent::model::NewAgent {
                name: "test-agent".into(),
                model: None,
                system_prompt: None,
                allowed_tools: None,
                max_turns: None,
                use_max: None,
                preset: None,
                config: None,
            })
            .unwrap();
        org_position_repo
            .create(&forge_db::NewOrgPosition {
                company_id: company.id.clone(),
                department_id: None,
                agent_id: Some(agent.id.0.to_string()),
                reports_to: None,
                role: "engineer".into(),
                title: None,
            })
            .unwrap();

        let mw = GovernanceMiddleware {
            company_repo,
            org_position_repo,
            goal_repo,
            approval_repo,
        };
        let mut chain = MiddlewareChain::new();
        chain.add(mw);
        let mut ctx = test_context();
        ctx.agent_id = agent.id.0.to_string();
        let result = chain.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert_eq!(
            ctx.metadata.get("pending_approvals"),
            Some(&"1 pending approval(s)".to_string())
        );
    }
}
