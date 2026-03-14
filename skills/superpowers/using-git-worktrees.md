---
name: using-git-worktrees
description: Use when starting feature work that needs isolation from current workspace or before executing implementation plans
tags: [git, worktree, isolation, branching]
tools: [Bash]
---

# Using Git Worktrees

## Overview

Git worktrees create isolated workspaces sharing the same repository, allowing work on multiple branches simultaneously without switching.

**Core principle:** Systematic directory selection + safety verification = reliable isolation.

## Directory Selection Process

### 1. Check Existing Directories

Check for `.worktrees/` (preferred) or `worktrees/` in priority order.

### 2. Check CLAUDE.md

Look for worktree directory preferences in project configuration.

### 3. Ask User

If no directory exists and no config preference, ask the user to choose between project-local or global location.

## Safety Verification

For project-local directories: **MUST verify directory is gitignored** before creating worktree. If not ignored, add to .gitignore and commit.

For global directories: No .gitignore verification needed.

## Creation Steps

1. **Detect Project Name** from git root
2. **Create Worktree** with `git worktree add <path> -b <branch-name>`
3. **Run Project Setup** — auto-detect from project files (package.json → npm install, Cargo.toml → cargo build, etc.)
4. **Verify Clean Baseline** — run tests. If tests fail, report and ask whether to proceed.
5. **Report Location** — full path, test results, ready status

## Quick Reference

| Situation | Action |
|-----------|--------|
| `.worktrees/` exists | Use it (verify ignored) |
| `worktrees/` exists | Use it (verify ignored) |
| Both exist | Use `.worktrees/` |
| Neither exists | Check config → Ask user |
| Directory not ignored | Add to .gitignore + commit |
| Tests fail during baseline | Report failures + ask |

## Red Flags

**Never:**
- Create worktree without verifying it's ignored (project-local)
- Skip baseline test verification
- Proceed with failing tests without asking
- Assume directory location when ambiguous

**Always:**
- Follow directory priority: existing > config > ask
- Verify directory is ignored for project-local
- Auto-detect and run project setup
- Verify clean test baseline
