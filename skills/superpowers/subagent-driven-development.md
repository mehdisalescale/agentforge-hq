---
name: subagent-driven-development
description: Use when executing implementation plans with independent tasks in the current session
tags: [subagent, implementation, execution, workflow]
tools: [Read, Write, Edit, Bash]
---

# Subagent-Driven Development

Execute plan by dispatching fresh subagent per task, with two-stage review after each: spec compliance review first, then code quality review.

**Core principle:** Fresh subagent per task + two-stage review (spec then quality) = high quality, fast iteration

## When to Use

- Have an implementation plan with mostly independent tasks
- Want to stay in current session (vs. executing-plans for parallel sessions)
- Fresh subagent per task prevents context pollution

## The Process

1. **Read plan** — extract all tasks with full text, note context, create task tracking
2. **Per task:**
   - Dispatch implementer subagent with full task text + context
   - If implementer asks questions → answer, re-dispatch
   - Implementer implements, tests, commits, self-reviews
   - Dispatch spec compliance reviewer → verify code matches spec
   - If spec issues found → implementer fixes → re-review until approved
   - Dispatch code quality reviewer → check implementation quality
   - If quality issues found → implementer fixes → re-review until approved
   - Mark task complete
3. **After all tasks** — dispatch final code reviewer for entire implementation
4. **Use finishing-a-development-branch** to complete

## Model Selection

- **Mechanical tasks** (isolated functions, clear specs, 1-2 files): use fast, cheap model
- **Integration tasks** (multi-file coordination, debugging): use standard model
- **Architecture/design/review tasks**: use most capable model

## Handling Implementer Status

- **DONE:** Proceed to spec compliance review
- **DONE_WITH_CONCERNS:** Read concerns, address if correctness/scope related, then review
- **NEEDS_CONTEXT:** Provide missing context and re-dispatch
- **BLOCKED:** Assess blocker — provide more context, use more capable model, break task down, or escalate to human

## Red Flags

**Never:**
- Start implementation on main/master without explicit user consent
- Skip reviews (spec compliance OR code quality)
- Proceed with unfixed issues
- Dispatch multiple implementation subagents in parallel (conflicts)
- Skip review loops (reviewer found issues = implementer fixes = review again)
- Start code quality review before spec compliance is approved

## Integration

**Required workflow skills:**
- **using-git-worktrees** - Set up isolated workspace before starting
- **writing-plans** - Creates the plan this skill executes
- **requesting-code-review** - Code review template for reviewer subagents
- **finishing-a-development-branch** - Complete development after all tasks
