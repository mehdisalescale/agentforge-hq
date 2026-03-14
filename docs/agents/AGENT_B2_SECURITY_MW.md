# Agent B2: SecurityScan Middleware

> You are Agent B2. Your job: create a post-execution SecurityScan middleware that scans agent output for vulnerabilities using the existing SecurityScanner, and add new ForgeEvent variants.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
crates/forge-safety/src/scanner.rs                  — SecurityScanner (already built)
crates/forge-api/src/middleware.rs                  — middleware chain, study SpawnMiddleware and QualityGateMiddleware
crates/forge-core/src/events.rs                     — ForgeEvent enum (35 variants)
crates/forge-core/src/lib.rs                        — core crate exports
```

## Step 2: Add ForgeEvent Variants

In `crates/forge-core/src/events.rs`, add two new variants to the `ForgeEvent` enum:

```rust
SecurityScanPassed {
    session_id: SessionId,
    timestamp: DateTime<Utc>,
},
SecurityScanFailed {
    session_id: SessionId,
    findings: Vec<String>,  // serialized finding descriptions
    timestamp: DateTime<Utc>,
},
```

Make sure the variants follow the existing pattern (derive Serialize/Deserialize, use the same field types). Update `all_event_variants_serialize` test if it exists to include the new variants.

## Step 3: Create SecurityScanMiddleware

Add a new middleware to `crates/forge-api/src/middleware.rs`. This is a **post-execution** middleware — it runs AFTER Spawn, inspecting the output. Since SpawnMiddleware spawns a background task and returns immediately, this middleware should run BEFORE Spawn (so it wraps the post-execution context), OR it should be designed differently.

**Recommended approach**: Make it a pre-chain middleware that adds metadata indicating security scanning is enabled, then have the spawn background task do the actual scanning after collecting output.

**Simpler approach** (preferred for now): Place it AFTER QualityGate but BEFORE Persist. It scans `ctx.metadata["output"]` if present (same pattern QualityGate uses). In the future, the SpawnMiddleware background task can set output.

```rust
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
                    });
                    ctx.metadata.insert("security_scan".into(), "passed".into());
                } else {
                    let finding_strs: Vec<String> = all_findings.iter().map(|f| {
                        format!("[{}] {} (line {}): {}",
                            format!("{:?}", f.severity), f.pattern, f.line, f.description)
                    }).collect();
                    let _ = self.event_bus.emit(ForgeEvent::SecurityScanFailed {
                        session_id: ctx.session_id_typed.clone(),
                        findings: finding_strs.clone(),
                        timestamp: chrono::Utc::now(),
                    });
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
```

## Step 4: Wire into Chain

In `crates/forge-api/src/routes/run.rs`, add SecurityScanMiddleware AFTER SkillInjection but BEFORE Persist:

```rust
chain.add(SkillInjectionMiddleware { ... });
// TaskTypeDetection goes here (Agent A2 adds this)
chain.add(SecurityScanMiddleware {
    event_bus: Arc::clone(&state.event_bus),
});
chain.add(PersistMiddleware { ... });
chain.add(SpawnMiddleware { ... });
```

Add the import:
```rust
use crate::middleware::SecurityScanMiddleware;
```

## Step 5: Write Tests

Add tests in `crates/forge-api/src/middleware.rs`:

```rust
#[tokio::test]
async fn security_scan_passes_clean_output() {
    let event_bus = Arc::new(EventBus::new(32));
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
    let event_bus = Arc::new(EventBus::new(32));
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
    let event_bus = Arc::new(EventBus::new(32));
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
```

## Step 6: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-core 2>&1        # event tests pass
cargo test -p forge-api -- security 2>&1  # new tests pass
cargo test 2>&1 | grep "FAILED"      # no failures
```

## Rules

- Add 2 variants to `crates/forge-core/src/events.rs`
- Add SecurityScanMiddleware + extract_code_blocks to `crates/forge-api/src/middleware.rs`
- Wire into `crates/forge-api/src/routes/run.rs`
- Do NOT modify forge-safety (scanner already exists)
- Do NOT modify forge-process
- Do NOT touch frontend
- **IMPORTANT**: If Agent A2 has already added TaskTypeDetectionMiddleware to run.rs, place SecurityScanMiddleware AFTER it. If not, just place it after SkillInjectionMiddleware. Either way, it goes BEFORE PersistMiddleware.
- Commit with: `feat(api): add SecurityScan middleware with OWASP pattern detection`

## Report
```
STATUS: done | blocked
FILES_CREATED: [list]
FILES_MODIFIED: [list]
TESTS_ADDED: N
NEW_EVENTS: [list new ForgeEvent variants]
CHAIN_ORDER: [list middleware order]
ISSUES: [any]
```
