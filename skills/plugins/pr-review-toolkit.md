---
name: pr-review-toolkit
description: Use when performing comprehensive PR review with specialized analysis agents for comments, tests, errors, types, code quality, and simplification
tags: [pr, review, quality, analysis]
tools: [Read, Grep, Glob, Bash]
---

# Comprehensive PR Review

Run a comprehensive pull request review using multiple specialized agents, each focusing on a different aspect of code quality.

## Review Aspects

- **comments** — Analyze code comment accuracy and maintainability
- **tests** — Review test coverage quality and completeness
- **errors** — Check error handling for silent failures
- **types** — Analyze type design and invariants (if new types added)
- **code** — General code review for project guidelines
- **simplify** — Simplify code for clarity and maintainability
- **all** — Run all applicable reviews (default)

## Workflow

1. **Determine scope** — Check git diff for changed files, parse arguments for specific aspects
2. **Identify applicable reviews** based on changes:
   - Always: code-reviewer
   - Test files changed: test-analyzer
   - Comments/docs added: comment-analyzer
   - Error handling changed: silent-failure-hunter
   - Types added/modified: type-design-analyzer
   - After passing: code-simplifier
3. **Launch agents** — Sequential (interactive) or parallel (comprehensive)
4. **Aggregate results** into: Critical (must fix), Important (should fix), Suggestions (nice to have), Positive observations
5. **Provide action plan** with prioritized findings and file:line references

## Agent Descriptions

| Agent | Focus |
|-------|-------|
| comment-analyzer | Comment accuracy, rot, documentation completeness |
| pr-test-analyzer | Behavioral coverage, critical gaps, test quality |
| silent-failure-hunter | Silent failures, catch blocks, error logging |
| type-design-analyzer | Type encapsulation, invariant expression, design quality |
| code-reviewer | Config compliance, bugs, general code quality |
| code-simplifier | Simplification, clarity, readability, standards |

## Tips

- Run early, before creating PR
- Address critical issues first
- Re-run after fixes to verify
- Use specific aspects when you know the concern
