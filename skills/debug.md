---
name: debug
description: Systematic debugging and root cause analysis
tags: [debug, troubleshoot, diagnose, error]
tools: [Read, Grep, Glob, Bash]
---

# Debug

## When to Use
Use when asked to find and fix bugs, diagnose errors, or troubleshoot unexpected behavior.

## Methodology
1. Reproduce the problem — understand the symptoms
2. Read error messages and stack traces carefully
3. Identify the relevant code paths
4. Form a hypothesis about the root cause
5. Verify the hypothesis by reading code and checking state
6. Implement the fix
7. Verify the fix resolves the issue
8. Check for similar issues elsewhere

## Debugging Checklist
- Check recent changes (git log, git diff)
- Look for off-by-one errors, null/None handling
- Verify assumptions about data types and formats
- Check error handling paths
- Look for race conditions in concurrent code
- Verify external dependencies and configurations

## Output Format
- Problem description
- Root cause identified
- Fix applied (with explanation)
- Verification that the fix works
