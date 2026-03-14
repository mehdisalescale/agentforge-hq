# Agent A: Skills Importer

> You are Agent A. Your job: import 14 superpowers skills + 6 claude-code plugin skills into Forge's skill system.

## Step 1: Read Context

Read these files to understand the project and the skill format:

```
CLAUDE.md                                    — project rules
NORTH_STAR.md                                — current state
skills/architect.md                          — example skill (study format)
skills/debug.md                              — another example
skills/test-writer.md                        — another example
crates/forge-db/src/repos/skills.rs          — how skills are loaded (load_from_dir method)
```

## Step 2: Understand Skill Format

Existing skills use YAML frontmatter:

```markdown
---
name: skill-name
description: One-line description
tags:
  - tag1
  - tag2
triggers:
  - keyword1
  - keyword2
---

Instructional content here...
```

Study `SkillRepo::load_from_dir()` to understand:
- What fields are required vs optional
- How nested directories are handled (or not — you may need to confirm)
- What validation is applied

## Step 3: Convert Superpowers Skills (14)

Source: `/Users/bm/cod/trend/10-march/superpowers/skills/`

Each directory contains a SKILL.md. For each:
1. Read the source SKILL.md
2. Convert to Forge format with proper YAML frontmatter
3. Save as `skills/superpowers/<skill-name>.md`

The 14 skills to convert:
- brainstorming
- dispatching-parallel-agents
- executing-plans
- finishing-a-development-branch
- receiving-code-review
- requesting-code-review
- subagent-driven-development
- systematic-debugging
- test-driven-development
- using-git-worktrees
- using-superpowers
- verification-before-completion
- writing-plans
- writing-skills

## Step 4: Convert Plugin Skills (6)

Source: `/Users/bm/cod/trend/10-march/claude-code/plugins/`

Pick the 6 most useful for development workflows. Recommended:
- `code-review`
- `security-guidance`
- `feature-dev`
- `pr-review-toolkit`
- `explanatory-output-style`
- `commit-commands`

For each:
1. Read the plugin's README.md or main content file
2. Extract the instructional prompt/content
3. Convert to Forge skill format
4. Save as `skills/plugins/<plugin-name>.md`

## Step 5: Verify

```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-db -- skills 2>&1  # skill tests pass
```

If `load_from_dir` doesn't handle nested directories, note this in your report but still create the files. The coordinator will handle the code change.

## Rules

- Only create .md files in `skills/superpowers/` and `skills/plugins/`
- Do NOT modify any Rust code
- Do NOT modify existing skills in `skills/*.md`
- Do NOT touch Cargo.toml
- Commit your work when done with message: `feat(skills): import 14 superpowers + 6 plugin skills`

## Report

When done, output:
```
STATUS: done | blocked
FILES_CREATED: [list all created files]
SKILLS_COUNT: N superpowers + N plugins = N total new
NESTED_DIR_SUPPORT: yes | no (does load_from_dir handle subdirs?)
ISSUES: [any problems]
```
