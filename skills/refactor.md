---
name: refactor
description: Systematic code refactoring with safety checks
tags: [refactor, cleanup, restructure]
tools: [Read, Edit, Grep, Glob]
---

# Refactor

## When to Use
Use when asked to restructure code without changing behavior — improving readability, reducing duplication, or improving architecture.

## Methodology
1. Understand the current code structure and behavior
2. Identify the specific refactoring goal
3. Check for existing tests that cover the code
4. Plan the refactoring steps (small, incremental changes)
5. Apply changes one step at a time
6. Verify tests still pass after each step
7. Review the final result for correctness

## Principles
- Preserve all existing behavior
- Keep changes minimal and focused
- Don't mix refactoring with feature changes
- Ensure tests pass at every step
- Prefer standard patterns over clever solutions

## Output Format
- What was refactored and why
- List of changes made
- Test results confirming no regressions
