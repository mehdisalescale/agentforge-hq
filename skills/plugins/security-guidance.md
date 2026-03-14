---
name: security-guidance
description: Use when editing files that may contain security-sensitive patterns like command injection, XSS, or unsafe code
tags: [security, xss, injection, vulnerability]
tools: [Read, Grep]
---

# Security Guidance

## Overview

Security reminder for common vulnerability patterns. Check for these when editing code.

## Patterns to Watch

### Command Injection

- **GitHub Actions workflows** — Never use untrusted input (issue titles, PR descriptions, commit messages) directly in `run:` commands. Use `env:` with proper quoting instead.
- **child_process.exec()** — Use `execFile` instead of `exec` to prevent shell injection. Pass arguments as arrays.
- **os.system()** — Only use with static arguments, never with user-controlled input.

### XSS Vulnerabilities

- **dangerouslySetInnerHTML** — Ensure all content is sanitized with DOMPurify or similar.
- **document.write()** — Use DOM manipulation methods (createElement, appendChild) instead.
- **innerHTML** — Use textContent for plain text. For HTML, sanitize with DOMPurify.

### Code Injection

- **eval()** — Consider JSON.parse() for data or alternative patterns. Major security risk.
- **new Function()** — Avoid with dynamic strings. Consider alternatives that don't evaluate arbitrary code.

### Deserialization

- **pickle** — Can lead to arbitrary code execution with untrusted content. Use JSON or safe formats instead.

## Risky GitHub Actions Inputs

Be especially careful with these event properties in workflow files:
- `github.event.issue.title` / `body`
- `github.event.pull_request.title` / `body`
- `github.event.comment.body`
- `github.event.commits.*.message`
- `github.event.head_commit.message`
- `github.head_ref`

## Safe Pattern Example

```yaml
# UNSAFE
run: echo "${{ github.event.issue.title }}"

# SAFE
env:
  TITLE: ${{ github.event.issue.title }}
run: echo "$TITLE"
```
