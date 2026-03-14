# Agent B: Task Type Detection Engine

> You are Agent B. Your job: build a keyword-based task type classifier in forge-process.

## Step 1: Read Context

```
CLAUDE.md                                        — project rules
NORTH_STAR.md                                     — current state
crates/forge-process/src/lib.rs                   — crate public API
crates/forge-process/Cargo.toml                   — current deps
crates/forge-api/src/middleware.rs                 — understand middleware chain (read-only)
```

## Step 2: Create TaskType Module

Create `crates/forge-process/src/task_type.rs`:

```rust
/// Task type classification for methodology routing.
///
/// Classifies user prompts into categories so the skill router
/// can inject appropriate methodology skills.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    NewFeature,
    BugFix,
    CodeReview,
    Refactor,
    Research,
    General,
}

pub struct TaskTypeDetector {
    // keyword lists per type
}

impl TaskTypeDetector {
    pub fn new() -> Self { ... }
    pub fn classify(&self, prompt: &str) -> TaskType { ... }
}
```

### Classification Rules

**NewFeature** keywords: "add", "create", "implement", "build", "new feature", "new endpoint", "introduce"

**BugFix** keywords: "fix", "bug", "broken", "error", "crash", "failing", "doesn't work", "not working", "500", "404", "regression"

**CodeReview** keywords: "review", "check this", "look at this PR", "code review", "PR review", "feedback on"

**Refactor** keywords: "refactor", "clean up", "reorganize", "restructure", "extract", "move to", "rename", "simplify"

**Research** keywords: "how does", "explain", "what is", "understand", "explore", "investigate", "why does", "documentation"

**General**: default when no strong match

### Matching Logic
- Case-insensitive
- Check all keyword lists against the prompt
- Count matches per type
- Return type with most matches
- If tie or zero matches, return General

## Step 3: Register Module

Add to `crates/forge-process/src/lib.rs`:
```rust
pub mod task_type;
```

## Step 4: Write Tests (8+)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_new_feature_keywords() {
        let d = TaskTypeDetector::new();
        assert_eq!(d.classify("add a new endpoint for user profiles"), TaskType::NewFeature);
        assert_eq!(d.classify("implement pagination"), TaskType::NewFeature);
    }

    #[test]
    fn test_classify_bugfix_keywords() {
        let d = TaskTypeDetector::new();
        assert_eq!(d.classify("fix the login bug"), TaskType::BugFix);
        assert_eq!(d.classify("the API is returning 500 errors"), TaskType::BugFix);
    }

    #[test]
    fn test_classify_code_review_keywords() { ... }
    #[test]
    fn test_classify_refactor_keywords() { ... }
    #[test]
    fn test_classify_research_keywords() { ... }
    #[test]
    fn test_classify_ambiguous_returns_general() { ... }
    #[test]
    fn test_classify_case_insensitive() { ... }
    #[test]
    fn test_classify_empty_prompt_returns_general() { ... }
}
```

## Step 5: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-process -- task_type 2>&1  # all 8+ tests pass
```

## Rules

- Only create `crates/forge-process/src/task_type.rs`
- Only add one `pub mod task_type;` line to `crates/forge-process/src/lib.rs`
- Do NOT add dependencies to Cargo.toml
- Do NOT modify middleware.rs or any other file
- Do NOT touch frontend code
- Commit with message: `feat(process): add TaskTypeDetector for methodology routing`

## Report

When done, output:
```
STATUS: done | blocked
FILES_CREATED: [list]
FILES_MODIFIED: [list]
TESTS_ADDED: N
TASK_TYPES: [list 6 types with 2-3 example keywords each]
ISSUES: [any problems]
```
