---
name: receiving-code-review
description: Use when receiving code review feedback, before implementing suggestions, especially if feedback seems unclear or technically questionable
tags: [code-review, feedback, review, quality]
tools: [Read, Grep, Glob]
---

# Code Review Reception

## Overview

Code review requires technical evaluation, not emotional performance.

**Core principle:** Verify before implementing. Ask before assuming. Technical correctness over social comfort.

## The Response Pattern

1. **READ:** Complete feedback without reacting
2. **UNDERSTAND:** Restate requirement in own words (or ask)
3. **VERIFY:** Check against codebase reality
4. **EVALUATE:** Technically sound for THIS codebase?
5. **RESPOND:** Technical acknowledgment or reasoned pushback
6. **IMPLEMENT:** One item at a time, test each

## Handling Unclear Feedback

If any item is unclear, STOP — do not implement anything yet. Ask for clarification on unclear items. Items may be related; partial understanding leads to wrong implementation.

## Source-Specific Handling

### From Your Team
- Implement after understanding
- Still ask if scope unclear
- No performative agreement — skip to action or technical acknowledgment

### From External Reviewers
Before implementing:
1. Check: Technically correct for THIS codebase?
2. Check: Breaks existing functionality?
3. Check: Reason for current implementation?
4. Check: Works on all platforms/versions?
5. Check: Does reviewer understand full context?

If suggestion seems wrong: Push back with technical reasoning.

## YAGNI Check

If reviewer suggests "implementing properly" — grep codebase for actual usage. If unused: suggest removing (YAGNI). If used: implement properly.

## Implementation Order

For multi-item feedback:
1. Clarify anything unclear FIRST
2. Then implement in this order:
   - Blocking issues (breaks, security)
   - Simple fixes (typos, imports)
   - Complex fixes (refactoring, logic)
3. Test each fix individually
4. Verify no regressions

## When To Push Back

Push back when:
- Suggestion breaks existing functionality
- Reviewer lacks full context
- Violates YAGNI (unused feature)
- Technically incorrect for this stack
- Legacy/compatibility reasons exist
- Conflicts with architectural decisions

## Acknowledging Correct Feedback

When feedback IS correct: Just fix it and describe what changed. No performative agreement — actions speak.
