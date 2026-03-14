---
name: finishing-a-development-branch
description: Use when implementation is complete, all tests pass, and you need to decide how to integrate the work - guides completion via merge, PR, or cleanup
tags: [git, branch, merge, completion, workflow]
tools: [Bash]
---

# Finishing a Development Branch

## Overview

Guide completion of development work by presenting clear options and handling chosen workflow.

**Core principle:** Verify tests → Present options → Execute choice → Clean up.

## The Process

### Step 1: Verify Tests

Run the project's test suite before presenting options. If tests fail, stop and fix before continuing.

### Step 2: Determine Base Branch

Identify the base branch (main/master) this feature branched from.

### Step 3: Present Options

Present exactly these 4 options:

1. Merge back to base branch locally
2. Push and create a Pull Request
3. Keep the branch as-is (handle later)
4. Discard this work

### Step 4: Execute Choice

**Option 1: Merge Locally** — Switch to base, pull latest, merge, verify tests, delete feature branch, cleanup worktree.

**Option 2: Push and Create PR** — Push branch, create PR with summary and test plan, cleanup worktree.

**Option 3: Keep As-Is** — Report branch/worktree location. Don't cleanup.

**Option 4: Discard** — Require typed "discard" confirmation. Delete branch and worktree.

### Step 5: Cleanup Worktree

For Options 1, 2, 4: remove worktree if applicable.
For Option 3: keep worktree.

## Quick Reference

| Option | Merge | Push | Keep Worktree | Cleanup Branch |
|--------|-------|------|---------------|----------------|
| 1. Merge locally | Yes | - | - | Yes |
| 2. Create PR | - | Yes | Yes | - |
| 3. Keep as-is | - | - | Yes | - |
| 4. Discard | - | - | - | Yes (force) |

## Red Flags

**Never:**
- Proceed with failing tests
- Merge without verifying tests on result
- Delete work without confirmation
- Force-push without explicit request

**Always:**
- Verify tests before offering options
- Present exactly 4 options
- Get typed confirmation for Option 4
- Clean up worktree for Options 1 & 4 only
