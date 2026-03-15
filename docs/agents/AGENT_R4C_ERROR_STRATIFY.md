# Agent R4-C: ForgeError Stratification

> ForgeError is a flat enum. Stratify into Client/Domain/Retriable/System categories with `is_retriable()`, `http_status()`, and `error_code()` methods.

## Step 1: Read Context

- `CLAUDE.md`
- `crates/forge-core/src/error.rs` — current ForgeError enum and ForgeResult type
- `crates/forge-api/src/error.rs` — HTTP error mapping (how errors become responses)
- `crates/forge-api/src/middleware.rs` — MiddlewareError type
- `crates/forge-api/src/routes/run.rs` — how middleware errors map to HTTP status codes
- Read 2-3 route handlers to see how errors are returned

## Step 2: Analyze Current Error Structure

Understand the current ForgeError variants and how they're used. Take notes on:
- Which variants are returned from repos (database errors)
- Which variants come from middleware (rate limit, circuit breaker, budget)
- Which variants come from process spawn
- How the API layer maps them to HTTP status codes

## Step 3: Redesign ForgeError

In `crates/forge-core/src/error.rs`, restructure the enum:

```rust
use http::StatusCode;

/// Stratified error types for the forge platform.
/// Errors are categorized by who's at fault and whether they're retriable.
#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    // ── Client errors (4xx) — caller's fault, don't retry ──
    #[error("validation error: {0}")]
    Validation(String),

    #[error("{entity} not found: {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("conflict: {0}")]
    Conflict(String),

    // ── Domain errors (4xx) — business logic rejection ──
    #[error("budget exceeded: cost {cost:.2} >= limit {limit:.2}")]
    BudgetExceeded { cost: f64, limit: f64 },

    #[error("approval required: {approval_type}")]
    ApprovalRequired { approval_type: String },

    #[error("rate limited, retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    // ── Retriable errors (5xx) — try again later ──
    #[error("CLI unavailable: {0}")]
    CliUnavailable(String),

    #[error("circuit breaker open, resets in {reset_in_ms}ms")]
    CircuitOpen { reset_in_ms: u64 },

    #[error("operation timed out: {0}")]
    Timeout(String),

    // ── System errors (5xx) — something is broken ──
    #[error("database error: {0}")]
    Database(Box<dyn std::error::Error + Send + Sync>),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("process error: {0}")]
    Process(String),
}
```

**IMPORTANT:** You must migrate ALL existing code that constructs ForgeError variants. Search the entire codebase for each variant name and update. Common patterns:

- `ForgeError::Database(Box::new(e))` — keep as-is
- `ForgeError::Internal(...)` — keep as-is
- Any new variants need call-site updates

## Step 4: Add Behavioral Methods

```rust
impl ForgeError {
    /// Whether the client should retry this request.
    pub fn is_retriable(&self) -> bool {
        matches!(self, Self::CliUnavailable(_) | Self::CircuitOpen { .. } | Self::Timeout(_) | Self::RateLimited { .. })
    }

    /// Map to HTTP status code.
    pub fn http_status(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::BudgetExceeded { .. } => StatusCode::PAYMENT_REQUIRED,
            Self::ApprovalRequired { .. } => StatusCode::FORBIDDEN,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::CliUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::CircuitOpen { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Process(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Machine-readable error code for API consumers.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::BudgetExceeded { .. } => "BUDGET_EXCEEDED",
            Self::ApprovalRequired { .. } => "APPROVAL_REQUIRED",
            Self::RateLimited { .. } => "RATE_LIMITED",
            Self::CliUnavailable(_) => "CLI_UNAVAILABLE",
            Self::CircuitOpen { .. } => "CIRCUIT_OPEN",
            Self::Timeout(_) => "TIMEOUT",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Process(_) => "PROCESS_ERROR",
        }
    }
}
```

## Step 5: Update API Error Response

In `crates/forge-api/src/error.rs`, use the new methods:

```rust
impl IntoResponse for ForgeError {
    fn into_response(self) -> Response {
        let status = self.http_status();
        let body = serde_json::json!({
            "error": self.to_string(),
            "code": self.error_code(),
            "retriable": self.is_retriable(),
        });
        (status, axum::Json(body)).into_response()
    }
}
```

## Step 6: Add http Dependency

In `crates/forge-core/Cargo.toml`, add:
```toml
http = "1"
```

Or use the workspace version if it's defined there.

## Step 7: Update All Call Sites

Search for every ForgeError construction across the workspace:
```bash
grep -r "ForgeError::" crates/ --include="*.rs" -l
```

For each file, ensure the variant names match. If you renamed variants, update all construction sites.

**Key principle:** Do NOT change behavior — only restructure. Every error that was a 500 before should still be a 500. Every error that was a 400 before should still be a 400.

## Step 8: Verify

```bash
cargo check --workspace 2>&1 | head -20   # zero warnings
cargo test --workspace 2>&1 | tail -10     # all pass
```

## Rules

- Touch: `crates/forge-core/src/error.rs`, `crates/forge-core/Cargo.toml`, `crates/forge-api/src/error.rs`
- You may need to touch other files to update ForgeError construction sites
- Do NOT change any behavior — only restructure the error types
- Do NOT touch `frontend/`, `site-docs/`, `CLAUDE.md`, `.github/workflows/`
- Do NOT touch `crates/forge-core/src/event_bus.rs` or `crates/forge-db/src/pool.rs`
- Run `cargo check` and `cargo test --workspace` before reporting done

## Report

When done, create `docs/agents/REPORT_R4C.md`:

```
STATUS: COMPLETE | PARTIAL | BLOCKED
VARIANTS_BEFORE: [count]
VARIANTS_AFTER: [count]
FILES_MODIFIED: [list]
METHODS_ADDED: is_retriable(), http_status(), error_code()
API_RESPONSE_FORMAT: { error, code, retriable }
CARGO_CHECK: pass/fail
CARGO_TEST: pass/fail
```
