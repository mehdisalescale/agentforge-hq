---
name: explore
description: Navigate and understand unfamiliar codebases
tags: [explore, navigate, understand, codebase]
tools: [Read, Grep, Glob]
---

# Explore

## When to Use
Use when asked to understand a codebase, find where something is implemented, or map out code structure.

## Methodology
1. Start with entry points (main, lib, index files)
2. Read configuration files (Cargo.toml, package.json, etc.)
3. Map the directory structure and module organization
4. Identify key types, traits, and interfaces
5. Trace data flow through the system
6. Note patterns and conventions used

## Exploration Strategy
- **Top-down:** Start from entry points, follow imports
- **Bottom-up:** Start from the specific feature, trace callers
- **Keyword search:** Find all uses of a term across the codebase
- **Type-driven:** Find type definitions, then their implementations

## Output Format
- Codebase structure overview
- Key files and their roles
- Important types and relationships
- Patterns and conventions observed
- Entry points for specific features
