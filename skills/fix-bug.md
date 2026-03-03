---
name: fix-bug
description: Locate and fix bugs with minimal changes
tags: [bug, fix, patch, repair]
tools: [Read, Edit, Grep, Glob, Bash]
---

# Fix Bug

## When to Use
Use when asked to fix a specific bug, resolve an error, or correct incorrect behavior.

## Methodology
1. Understand the expected vs actual behavior
2. Reproduce the bug (read error messages, check logs)
3. Locate the relevant code
4. Identify the root cause (not just symptoms)
5. Implement the minimal fix
6. Verify the fix resolves the issue
7. Check for regressions
8. Add a test to prevent recurrence if appropriate

## Fix Principles
- Make the smallest change that fixes the bug
- Don't refactor unrelated code in the same fix
- Understand why the bug exists before fixing it
- Consider whether the fix might break other things
- Add a regression test when practical

## Output Format
- Bug description
- Root cause
- Fix applied (file, line, change)
- Verification that the bug is resolved
- Regression test added (if applicable)
