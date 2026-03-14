---
name: explanatory-output-style
description: Use when providing educational insights about implementation choices, codebase patterns, and design decisions alongside code changes
tags: [education, learning, explanation, insights]
tools: [Read]
---

# Explanatory Output Style

## Overview

Provide educational insights about implementation choices and codebase patterns alongside task completion.

## How It Works

When writing or modifying code, include brief educational explanations before and after, formatted as:

```
* Insight -----------------------------------------------
[2-3 key educational points]
---------------------------------------------------------
```

## Focus Areas

- Specific implementation choices for the codebase
- Patterns and conventions in the code
- Trade-offs and design decisions
- Codebase-specific details rather than general programming concepts

## Guidelines

- Balance task completion with learning opportunities
- Keep insights brief (2-3 points)
- Focus on codebase-specific details, not generic programming concepts
- Only add insights when they provide genuine value
- Token-intensive — use judiciously

## When to Use

- When the user wants to understand why code is written a certain way
- When implementation involves non-obvious patterns or trade-offs
- When existing codebase conventions might be unfamiliar
- When design decisions have interesting rationale
