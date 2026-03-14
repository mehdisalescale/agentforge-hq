---
name: code-review
description: Use when reviewing a pull request for bugs, compliance violations, and code quality issues
tags: [review, pr, quality, bugs]
tools: [Read, Grep, Glob, Bash]
---

# Code Review

Provide a thorough code review for a pull request using multi-agent validation.

## Workflow

1. **Pre-check** — Verify PR is open, not a draft, and hasn't already been reviewed
2. **Gather context** — Find relevant CLAUDE.md / project config files in affected directories
3. **Summarize changes** — View the PR diff and create a summary
4. **Launch review agents in parallel:**
   - CLAUDE.md compliance audit (2 agents for coverage)
   - Bug detection (focus on diff only, flag only significant bugs)
   - Security/logic review (problems in introduced code only)
5. **Validate findings** — For each issue found, launch a validation agent to confirm with high confidence
6. **Filter false positives** — Remove unvalidated issues
7. **Report results** — Output summary to terminal; optionally post inline comments

## High Signal Issues Only

**Flag when:**
- Code will fail to compile or parse
- Code will definitely produce wrong results
- Clear, unambiguous config/style violations with exact rule quoted

**Do NOT flag:**
- Code style or quality concerns
- Potential issues depending on specific inputs
- Subjective suggestions
- Pre-existing issues
- Issues a linter will catch

## False Positive Filters

- Pre-existing issues
- Pedantic nitpicks a senior engineer would ignore
- General quality concerns unless explicitly required
- Issues silenced in code (lint ignore comments)

## Output Format

If no issues found: "No issues found. Checked for bugs and compliance."

If issues found: List each with description, file location, and severity. For small fixes, include suggestion blocks. For larger fixes, describe the issue and fix without code blocks.
