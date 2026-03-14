# Agent C: Security Scanner

> You are Agent C. Your job: build a regex-based security scanner in forge-safety that detects 9 OWASP vulnerability patterns.

## Step 1: Read Context

```
CLAUDE.md                                    — project rules
NORTH_STAR.md                                — current state
crates/forge-safety/src/lib.rs               — crate structure (CircuitBreaker, RateLimiter, CostTracker)
crates/forge-safety/Cargo.toml               — current dependencies
```

## Step 2: Add regex dependency (if needed)

Check `crates/forge-safety/Cargo.toml`. If `regex` is not listed, add it:
```toml
regex = "1"
```

If it is already present (possibly through another dep), skip this step.

## Step 3: Create Scanner Module

Create `crates/forge-safety/src/scanner.rs`:

```rust
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub pattern: String,
    pub severity: Severity,
    pub line: usize,
    pub snippet: String,
    pub description: String,
}

pub struct SecurityScanner {
    patterns: Vec<ScanPattern>,
}

struct ScanPattern {
    name: String,
    severity: Severity,
    regex: Regex,
    description: String,
}
```

### The 9 Patterns

| # | Pattern | Severity | Regex targets |
|---|---------|----------|---------------|
| 1 | `command_injection` | Critical | `os\.system\(`, `subprocess\.(call\|run\|Popen)\(.*\bf"`, `exec\(` with shell commands |
| 2 | `xss_dangerous_html` | High | `\.innerHTML\s*=`, `dangerouslySetInnerHTML`, `v-html` |
| 3 | `eval_injection` | Critical | `eval\(`, `exec\(`, `new Function\(` with dynamic input |
| 4 | `sql_injection` | Critical | `f".*SELECT`, `"SELECT.*"\s*\+`, `\.format\(.*SELECT`, `f".*INSERT`, `f".*UPDATE`, `f".*DELETE` |
| 5 | `path_traversal` | High | `\.\./` in file open/read contexts, `os\.path\.join\(.*request` |
| 6 | `pickle_deserialization` | High | `pickle\.loads?\(`, `yaml\.load\(` (without SafeLoader) |
| 7 | `hardcoded_secrets` | Medium | `(api_key\|password\|secret\|token)\s*=\s*["'](?!$\|{)` — strings that look like real values |
| 8 | `insecure_random` | Low | `Math\.random\(\)`, `random\.random\(\)` near auth/crypto/token context |
| 9 | `open_redirect` | Medium | `redirect\(request\.(GET\|POST\|query)`, unvalidated URL params |

### API

```rust
impl SecurityScanner {
    pub fn new() -> Self { /* compile all 9 patterns */ }
    pub fn scan(&self, code: &str) -> Vec<SecurityFinding> {
        // For each line in code:
        //   For each pattern:
        //     If regex matches, create SecurityFinding with line number, snippet, etc.
        // Return all findings sorted by severity (Critical first)
    }
}
```

## Step 4: Register Module

Add to `crates/forge-safety/src/lib.rs`:
```rust
pub mod scanner;
```

## Step 5: Write Tests (12+)

One test per pattern with realistic code snippets:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_command_injection() {
        let s = SecurityScanner::new();
        let findings = s.scan(r#"os.system(f"rm -rf {user_input}")"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern, "command_injection");
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn detects_xss_dangerous_html() {
        let s = SecurityScanner::new();
        let findings = s.scan(r#"element.innerHTML = userContent;"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern, "xss_dangerous_html");
    }

    // ... one test per pattern ...

    #[test]
    fn clean_code_passes() {
        let s = SecurityScanner::new();
        let findings = s.scan("let x = 1 + 2;\nfn hello() { println!(\"hi\"); }");
        assert!(findings.is_empty());
    }

    #[test]
    fn multiple_findings_returns_all() {
        let s = SecurityScanner::new();
        let code = "eval(user_input)\nelement.innerHTML = data\nos.system(cmd)";
        let findings = s.scan(code);
        assert!(findings.len() >= 3);
    }

    #[test]
    fn handles_empty_input() {
        let s = SecurityScanner::new();
        assert!(s.scan("").is_empty());
    }
}
```

## Step 6: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-safety -- scanner 2>&1  # all 12+ tests pass
```

## Rules

- Only create `crates/forge-safety/src/scanner.rs`
- Only modify `crates/forge-safety/src/lib.rs` (add `pub mod scanner;`)
- Only modify `crates/forge-safety/Cargo.toml` (add `regex` if needed)
- Do NOT touch middleware.rs or any route file
- Do NOT touch frontend code
- Do NOT touch any other crate
- Commit with message: `feat(safety): add SecurityScanner with 9 OWASP patterns`

## Report

When done, output:
```
STATUS: done | blocked
FILES_CREATED: [list]
FILES_MODIFIED: [list]
TESTS_ADDED: N
PATTERNS: [list 9 with severities]
ISSUES: [any problems]
```
