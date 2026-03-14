---
name: writing-skills
description: Use when creating new skills, editing existing skills, or verifying skills work before deployment
tags: [skills, documentation, process, meta]
tools: [Read, Write, Edit]
---

# Writing Skills

## Overview

Writing skills IS Test-Driven Development applied to process documentation.

You write test cases (pressure scenarios), watch them fail (baseline behavior), write the skill, watch tests pass (agents comply), and refactor (close loopholes).

**Core principle:** If you didn't watch an agent fail without the skill, you don't know if the skill teaches the right thing.

## What is a Skill?

A **skill** is a reference guide for proven techniques, patterns, or tools.

**Skills are:** Reusable techniques, patterns, tools, reference guides
**Skills are NOT:** Narratives about how you solved a problem once

## Skill Types

- **Technique** — Concrete method with steps (condition-based-waiting, root-cause-tracing)
- **Pattern** — Way of thinking about problems (flatten-with-flags, test-invariants)
- **Reference** — API docs, syntax guides, tool documentation

## SKILL.md Structure

```markdown
---
name: skill-name
description: Use when [specific triggering conditions]
tags: [tag1, tag2]
tools: [Tool1, Tool2]
---

# Skill Name

## Overview - Core principle in 1-2 sentences.
## When to Use - Symptoms and use cases. When NOT to use.
## Core Pattern - Before/after code comparison.
## Quick Reference - Table or bullets for scanning.
## Common Mistakes - What goes wrong + fixes.
```

## Key Rules

- Name uses only letters, numbers, hyphens
- Description starts with "Use when..." (triggering conditions only, NOT workflow summary)
- Keep descriptions under 500 characters
- One excellent example beats many mediocre ones
- Use flowcharts ONLY for non-obvious decision points

## Skill Creation Checklist (TDD)

**RED:** Run pressure scenario WITHOUT skill — document baseline behavior
**GREEN:** Write minimal skill addressing specific failures — verify agents now comply
**REFACTOR:** Find new rationalizations → add counters → re-test until bulletproof

## Anti-Patterns

- Narrative examples ("In session X, we found...")
- Multi-language dilution
- Code in flowcharts
- Generic labels (helper1, step3)
- Deploying untested skills
