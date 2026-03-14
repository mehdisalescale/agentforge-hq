# Agent D2: Code Review Engine

> You are Agent D2. Your job: build a code review engine with 6 specialist evaluators and confidence-scored aggregation.

## Step 1: Read Context

```
CLAUDE.md                                          — project rules
NORTH_STAR.md                                      — current state
crates/forge-process/src/concurrent.rs              — ConcurrentRunner, SubTask, SubTaskResult
crates/forge-process/src/lib.rs                     — crate exports
crates/forge-process/Cargo.toml                     — current deps
```

## Step 2: Create Review Engine Module

Create `crates/forge-process/src/review.rs`:

### Design

The review engine defines 6 specialist reviewer roles. Each produces findings with a confidence score. The engine aggregates results, filtering by confidence threshold.

```rust
use serde::{Deserialize, Serialize};

/// A specialist reviewer role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewAspect {
    PrComments,
    TestCoverage,
    ErrorHandling,
    TypeDesign,
    CodeQuality,
    Simplification,
}

impl ReviewAspect {
    pub fn all() -> Vec<ReviewAspect> {
        vec![
            Self::PrComments,
            Self::TestCoverage,
            Self::ErrorHandling,
            Self::TypeDesign,
            Self::CodeQuality,
            Self::Simplification,
        ]
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::PrComments => "PR Comments Quality",
            Self::TestCoverage => "Test Coverage Adequacy",
            Self::ErrorHandling => "Error Handling Completeness",
            Self::TypeDesign => "Type Design Correctness",
            Self::CodeQuality => "Code Quality (SOLID, DRY)",
            Self::Simplification => "Simplification Opportunities",
        }
    }

    /// Generate the specialist prompt for this review aspect.
    pub fn system_prompt(&self) -> String {
        match self {
            Self::PrComments => "You are a PR review specialist. Evaluate the quality of PR comments, commit messages, and documentation changes. Score confidence 0-100 on how well the PR communicates intent.".into(),
            Self::TestCoverage => "You are a test coverage specialist. Evaluate whether the code has adequate test coverage for new/changed functionality. Check for edge cases, error paths, and integration tests. Score confidence 0-100.".into(),
            Self::ErrorHandling => "You are an error handling specialist. Evaluate whether errors are properly handled, propagated, and communicated. Check for swallowed errors, missing error types, and panic risks. Score confidence 0-100.".into(),
            Self::TypeDesign => "You are a type system specialist. Evaluate type correctness, proper use of generics, newtypes, and whether the type design prevents invalid states. Score confidence 0-100.".into(),
            Self::CodeQuality => "You are a code quality specialist. Evaluate adherence to SOLID principles, DRY, naming conventions, and overall code structure. Score confidence 0-100.".into(),
            Self::Simplification => "You are a simplification specialist. Identify unnecessary complexity, over-engineering, and opportunities for reducing code while maintaining functionality. Score confidence 0-100.".into(),
        }
    }
}

/// A single finding from a specialist reviewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub aspect: ReviewAspect,
    pub confidence: u8,    // 0-100
    pub severity: ReviewSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewSeverity {
    Critical,   // confidence >= 90
    Important,  // confidence 80-89
    Minor,      // confidence < 80
}

impl ReviewFinding {
    pub fn severity_from_confidence(confidence: u8) -> ReviewSeverity {
        match confidence {
            90..=100 => ReviewSeverity::Critical,
            80..=89 => ReviewSeverity::Important,
            _ => ReviewSeverity::Minor,
        }
    }
}

/// Aggregated review report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub findings: Vec<ReviewFinding>,
    pub aspects_reviewed: Vec<ReviewAspect>,
    pub overall_confidence: f64,
}

impl ReviewReport {
    /// Create a report from raw findings, filtering by confidence threshold.
    pub fn from_findings(
        findings: Vec<ReviewFinding>,
        confidence_threshold: u8,
    ) -> Self {
        let aspects_reviewed: Vec<ReviewAspect> = ReviewAspect::all();

        let filtered: Vec<ReviewFinding> = findings
            .into_iter()
            .filter(|f| f.confidence >= confidence_threshold)
            .collect();

        let overall_confidence = if filtered.is_empty() {
            100.0
        } else {
            filtered.iter().map(|f| f.confidence as f64).sum::<f64>()
                / filtered.len() as f64
        };

        ReviewReport {
            findings: filtered,
            aspects_reviewed,
            overall_confidence,
        }
    }

    /// Count findings by severity.
    pub fn count_by_severity(&self, severity: &ReviewSeverity) -> usize {
        self.findings.iter().filter(|f| &f.severity == severity).count()
    }
}

/// Default confidence threshold for including findings.
pub const DEFAULT_CONFIDENCE_THRESHOLD: u8 = 80;
```

## Step 3: Register Module

Add to `crates/forge-process/src/lib.rs`:
```rust
pub mod review;
```

## Step 4: Write Tests (10+)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_finding(aspect: ReviewAspect, confidence: u8, message: &str) -> ReviewFinding {
        ReviewFinding {
            aspect,
            confidence,
            severity: ReviewFinding::severity_from_confidence(confidence),
            message: message.into(),
            suggestion: None,
        }
    }

    #[test]
    fn severity_from_confidence_critical() {
        assert_eq!(ReviewFinding::severity_from_confidence(95), ReviewSeverity::Critical);
        assert_eq!(ReviewFinding::severity_from_confidence(90), ReviewSeverity::Critical);
    }

    #[test]
    fn severity_from_confidence_important() {
        assert_eq!(ReviewFinding::severity_from_confidence(85), ReviewSeverity::Important);
        assert_eq!(ReviewFinding::severity_from_confidence(80), ReviewSeverity::Important);
    }

    #[test]
    fn severity_from_confidence_minor() {
        assert_eq!(ReviewFinding::severity_from_confidence(79), ReviewSeverity::Minor);
        assert_eq!(ReviewFinding::severity_from_confidence(50), ReviewSeverity::Minor);
    }

    #[test]
    fn report_filters_below_threshold() {
        let findings = vec![
            sample_finding(ReviewAspect::TestCoverage, 90, "Missing edge case tests"),
            sample_finding(ReviewAspect::CodeQuality, 60, "Minor naming issue"),
            sample_finding(ReviewAspect::ErrorHandling, 85, "Swallowed error in handler"),
        ];
        let report = ReviewReport::from_findings(findings, 80);
        assert_eq!(report.findings.len(), 2); // 90 and 85 pass, 60 filtered
    }

    #[test]
    fn report_count_by_severity() {
        let findings = vec![
            sample_finding(ReviewAspect::TestCoverage, 95, "Critical issue"),
            sample_finding(ReviewAspect::CodeQuality, 92, "Another critical"),
            sample_finding(ReviewAspect::ErrorHandling, 85, "Important issue"),
        ];
        let report = ReviewReport::from_findings(findings, 80);
        assert_eq!(report.count_by_severity(&ReviewSeverity::Critical), 2);
        assert_eq!(report.count_by_severity(&ReviewSeverity::Important), 1);
    }

    #[test]
    fn report_overall_confidence() {
        let findings = vec![
            sample_finding(ReviewAspect::TestCoverage, 90, "Issue A"),
            sample_finding(ReviewAspect::CodeQuality, 80, "Issue B"),
        ];
        let report = ReviewReport::from_findings(findings, 80);
        assert!((report.overall_confidence - 85.0).abs() < 0.01);
    }

    #[test]
    fn report_empty_findings_100_confidence() {
        let report = ReviewReport::from_findings(vec![], 80);
        assert_eq!(report.overall_confidence, 100.0);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn all_aspects_returns_six() {
        assert_eq!(ReviewAspect::all().len(), 6);
    }

    #[test]
    fn each_aspect_has_system_prompt() {
        for aspect in ReviewAspect::all() {
            let prompt = aspect.system_prompt();
            assert!(!prompt.is_empty());
            assert!(prompt.contains("confidence"));
        }
    }

    #[test]
    fn each_aspect_has_display_name() {
        for aspect in ReviewAspect::all() {
            assert!(!aspect.display_name().is_empty());
        }
    }

    #[test]
    fn finding_serializes_to_json() {
        let f = sample_finding(ReviewAspect::TestCoverage, 90, "Missing tests");
        let json = serde_json::to_string(&f).unwrap();
        assert!(json.contains("TestCoverage"));
        assert!(json.contains("90"));
    }
}
```

## Step 5: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-process -- review 2>&1  # all review tests pass
cargo test -p forge-process 2>&1            # ALL forge-process tests pass
```

## Rules

- Create `crates/forge-process/src/review.rs` (new file)
- Add `pub mod review;` to `crates/forge-process/src/lib.rs`
- Do NOT modify `concurrent.rs` — the review engine will USE ConcurrentRunner later, but for now just define the data model
- Do NOT touch middleware.rs, forge-api, forge-db, forge-safety, or frontend
- Do NOT modify existing tests
- Commit with: `feat(process): add code review engine with 6 specialist aspects`

## Report
```
STATUS: done | blocked
FILES_CREATED: [list]
FILES_MODIFIED: [list]
TESTS_ADDED: N
ASPECTS: [list 6 review aspects]
ISSUES: [any]
```
