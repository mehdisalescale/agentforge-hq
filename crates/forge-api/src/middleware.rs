//! Middleware trait and chain for the run handler pipeline.
//!
//! Provides an ordered middleware chain where each middleware can inspect/modify
//! the request context, short-circuit with an error, or delegate to the next
//! middleware. Pattern inspired by DeerFlow's 8-middleware pipeline.
//!
//! This module defines the infrastructure only — concrete middlewares (rate limit,
//! circuit breaker, skill injection, etc.) are added in Wave 3.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Context passed through the middleware chain.
pub struct RunContext {
    pub agent_id: String,
    pub prompt: String,
    pub session_id: String,
    pub working_dir: Option<String>,
    pub metadata: HashMap<String, String>,
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

#[cfg(test)]
mod tests {
    use super::*;

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
        }
    }

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
}
