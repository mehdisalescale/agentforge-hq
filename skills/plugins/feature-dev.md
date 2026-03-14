---
name: feature-dev
description: Use when implementing a new feature that requires codebase understanding, architecture design, and systematic implementation
tags: [feature, development, architecture, implementation]
tools: [Read, Write, Edit, Grep, Glob, Bash]
---

# Feature Development

Guided feature development with codebase understanding and architecture focus.

## Core Principles

- **Ask clarifying questions** — Identify all ambiguities, edge cases, and underspecified behaviors before designing
- **Understand before acting** — Read and comprehend existing code patterns first
- **Simple and elegant** — Prioritize readable, maintainable, architecturally sound code

## Phase 1: Discovery

**Goal:** Understand what needs to be built

1. Create task tracking with all phases
2. If feature unclear, ask: What problem? What should it do? Constraints?
3. Summarize understanding and confirm with user

## Phase 2: Codebase Exploration

**Goal:** Understand relevant existing code and patterns

1. Launch 2-3 explorer agents targeting different aspects (similar features, architecture, UI patterns)
2. Read all key files identified by agents
3. Present comprehensive summary of findings

## Phase 3: Clarifying Questions

**Goal:** Fill in gaps and resolve all ambiguities before designing

**CRITICAL: DO NOT SKIP.** Identify underspecified aspects: edge cases, error handling, integration points, scope boundaries, design preferences, backward compatibility, performance needs. Present all questions and wait for answers.

## Phase 4: Architecture Design

**Goal:** Design multiple implementation approaches

1. Launch 2-3 architect agents with different focuses (minimal changes, clean architecture, pragmatic balance)
2. Present approaches with trade-offs and your recommendation
3. Ask user which approach they prefer

## Phase 5: Implementation

**Goal:** Build the feature

**DO NOT START WITHOUT USER APPROVAL.** Implement following chosen architecture, follow codebase conventions strictly.

## Phase 6: Quality Review

**Goal:** Ensure code quality

1. Launch 3 reviewer agents (simplicity/DRY, bugs/correctness, conventions/abstractions)
2. Present findings and ask what to fix now vs. later

## Phase 7: Summary

Document what was built, key decisions, files modified, suggested next steps.
