---
name: writing-plans
description: Use when you have a spec or requirements for a multi-step task, before touching code
tags: [planning, implementation, architecture, workflow]
tools: [Read, Write, Grep, Glob]
---

# Writing Plans

## Overview

Write comprehensive implementation plans assuming the engineer has zero context. Document everything they need: which files to touch, code, testing, docs to check, how to test. Give them the whole plan as bite-sized tasks. DRY. YAGNI. TDD. Frequent commits.

## Scope Check

If the spec covers multiple independent subsystems, suggest breaking into separate plans — one per subsystem. Each plan should produce working, testable software on its own.

## File Structure

Before defining tasks, map out which files will be created or modified. Design units with clear boundaries and well-defined interfaces. Prefer smaller, focused files over large ones. Follow established patterns in existing codebases.

## Bite-Sized Task Granularity

Each step is one action (2-5 minutes):
- "Write the failing test" - step
- "Run it to make sure it fails" - step
- "Implement the minimal code" - step
- "Run the tests" - step
- "Commit" - step

## Plan Document Header

Every plan MUST start with:
- Feature name
- Goal (one sentence)
- Architecture (2-3 sentences)
- Tech stack

## Task Structure

Each task includes:
- **Files:** Create/Modify/Test paths
- **Steps:** Checkbox syntax for tracking with exact code and commands
- **Verification:** Expected output for each step

## Remember

- Exact file paths always
- Complete code in plan (not "add validation")
- Exact commands with expected output
- DRY, YAGNI, TDD, frequent commits

## Execution Handoff

After saving the plan:
- If subagents available: Use subagent-driven-development
- If no subagents: Use executing-plans
