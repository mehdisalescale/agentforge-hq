---
name: commit-commands
description: Use when creating git commits, pushing branches, creating PRs, or cleaning up stale local branches
tags: [git, commit, pr, workflow]
tools: [Bash]
---

# Commit Commands

Git workflow commands for commits, PRs, and branch cleanup.

## Commit

Create a single git commit based on current changes.

1. Check git status and diff
2. Stage relevant files
3. Create commit with appropriate message based on the changes

## Commit-Push-PR

Complete workflow: commit, push, and create PR in one step.

1. Create a new branch if on main
2. Create a single commit with appropriate message
3. Push the branch to origin
4. Create a pull request using `gh pr create`

## Clean Gone

Clean up local branches that have been deleted from remote.

1. Run `git fetch --prune` to update remote tracking
2. Find branches with `[gone]` tracking status
3. For each gone branch:
   - Check if it has an associated worktree
   - If worktree exists, remove it first
   - Delete the local branch
4. Report what was cleaned up

## Usage Notes

- Commit messages should be concise and descriptive
- PR titles should summarize the changes
- Branch cleanup is safe — only removes branches already deleted on remote
- Always check git status before committing
