# Wave Next — Agent Coordination Brief

> **Coordinator**: main session (this file's author)
> **Date**: 2026-03-15
> **Target**: E2 Sprint 1 stories + E1 polish
> **Agents**: 4 parallel, each in a Zellij tab

---

## Current State

- **E1 complete**: 112 personas seeded, hire flow working, org chart, goals, approvals
- **Frontend polish in progress** (separate session): Inter font, Lucide icons, refined dark theme, Markdown component, active nav highlighting, transitions, focus states, scrollbar styling
- **Build**: zero warnings, 160 tests pass, release binary works
- **Version**: v0.6.0-dev on main

---

## Agent Roster

| Agent | Tab | Scope | Files Owned |
|-------|-----|-------|-------------|
| **A: Skills Importer** | Tab 1 | E2-S1 + E2-S9: Import 14 superpowers + 6 plugin skills | `skills/superpowers/**`, `skills/plugins/**` |
| **B: Task Detector** | Tab 2 | E2-S2 + E2-S4: TaskType detection engine + middleware | `crates/forge-process/src/task_type.rs` |
| **C: Security Scanner** | Tab 3 | E2-S5 + E2-S6: 9 OWASP pattern scanner + middleware | `crates/forge-safety/src/scanner.rs` |
| **D: E1 Backend Polish** | Tab 4 | Company detail page API, department CRUD, persona detail | `crates/forge-api/src/routes/org.rs`, `crates/forge-api/src/routes/personas.rs` |

---

## Agent A: Skills Importer

### Mission
Import 14 superpowers skills and 6 claude-code plugin skills into Forge's skill system. Convert each to Forge's YAML frontmatter format.

### Context

**Read these files first:**
```
agentforge-hq/CLAUDE.md
agentforge-hq/NORTH_STAR.md
```

**Existing skills format** (study these as template):
```
agentforge-hq/skills/*.md  (10 existing skills)
```

**Source material to convert:**
```
/Users/bm/cod/trend/10-march/superpowers/skills/   (14 directories, each has SKILL.md)
/Users/bm/cod/trend/10-march/claude-code/plugins/   (14 directories, pick 6 most relevant)
```

**Skill loading code** (understand how skills are parsed):
```
agentforge-hq/crates/forge-db/src/repos/skills.rs  (SkillRepo.load_from_dir)
```

### Instructions

1. Read 2-3 existing skills in `agentforge-hq/skills/` to understand the YAML frontmatter format (name, description, tags, triggers, content).
2. Read the `SkillRepo.load_from_dir()` method in `crates/forge-db/src/repos/skills.rs` to understand what fields are required and how nested directories are handled.
3. For each of the 14 superpowers skills, read the source `SKILL.md` and convert to Forge format:
   - Create `skills/superpowers/<skill-name>.md` with proper YAML frontmatter
   - Preserve the core instructional content
   - Add appropriate tags and trigger patterns
4. For claude-code plugins, pick the 6 most useful for development workflows:
   - Recommended: `code-review`, `security-guidance`, `feature-dev`, `pr-review-toolkit`, `explanatory-output-style`, `commit-commands`
   - Create `skills/plugins/<plugin-name>.md`
5. Verify: `SkillRepo.load_from_dir()` should find and parse all new files. Check by reading the code to understand what it expects.

### Deliverables
- `skills/superpowers/` — 14 skill files
- `skills/plugins/` — 6 skill files
- Total: 30 skills (10 existing + 20 new)

### Verify
```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-db -- skills 2>&1  # skill tests pass
```

### Do NOT
- Modify any Rust code (only create .md files in skills/)
- Touch existing skills in `skills/*.md`
- Modify Cargo.toml or any crate

### Report Format
When done, output:
```
STATUS: done
FILES_CREATED: [list]
SKILLS_COUNT: N superpowers + N plugins = N total
ISSUES: [any problems encountered]
```

---

## Agent B: Task Type Detection Engine

### Mission
Build a keyword-based task type classifier that categorizes user prompts into 6 types, plus a middleware that plugs it into the run pipeline.

### Context

**Read these files first:**
```
agentforge-hq/CLAUDE.md
agentforge-hq/NORTH_STAR.md
```

**Understand the middleware chain:**
```
agentforge-hq/crates/forge-api/src/middleware.rs  (existing 8-middleware chain)
agentforge-hq/crates/forge-process/src/lib.rs     (crate public API)
```

**Epic spec (your stories):**
- E2-S2: TaskType detection — keyword + pattern matching, NOT LLM-based
- E2-S4: TaskTypeDetection middleware — runs after SkillInjection

### Instructions

1. Read the existing middleware chain in `crates/forge-api/src/middleware.rs` to understand the `Middleware` trait, `RunContext`, and how middlewares are chained.
2. Read `crates/forge-process/src/lib.rs` to see what's publicly exported.
3. Create `crates/forge-process/src/task_type.rs`:
   ```rust
   pub enum TaskType { NewFeature, BugFix, CodeReview, Refactor, Research, General }

   pub struct TaskTypeDetector { /* keyword lists */ }
   impl TaskTypeDetector {
       pub fn new() -> Self { /* default keyword lists */ }
       pub fn classify(&self, prompt: &str) -> TaskType { /* keyword matching */ }
   }
   ```
4. Add `pub mod task_type;` to `crates/forge-process/src/lib.rs`.
5. Write 8+ tests in the same file:
   - `test_classify_new_feature_keywords`
   - `test_classify_bugfix_keywords`
   - `test_classify_code_review_keywords`
   - `test_classify_refactor_keywords`
   - `test_classify_research_keywords`
   - `test_classify_ambiguous_returns_general`
   - `test_classify_case_insensitive`
   - `test_classify_empty_prompt_returns_general`
6. **Do NOT create the middleware yet** — just the detection engine. The middleware wiring depends on understanding how SkillInjection currently works, which is complex. Leave middleware for the coordinator to integrate.

### Key Design Rules
- Case-insensitive matching
- Multiple keyword hits → most specific type wins (CodeReview > General)
- Configurable keyword lists (constructor takes optional overrides)
- No external dependencies beyond what forge-process already has

### Deliverables
- `crates/forge-process/src/task_type.rs` — TaskType enum + TaskTypeDetector + tests
- One-line addition to `crates/forge-process/src/lib.rs`

### Verify
```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-process -- task_type 2>&1  # all tests pass
```

### Do NOT
- Modify middleware.rs (coordinator handles integration)
- Add dependencies to Cargo.toml
- Touch any frontend code
- Touch any other crate

### Report Format
When done, output:
```
STATUS: done
FILES_CREATED: [list]
FILES_MODIFIED: [list]
TESTS_ADDED: N
TASK_TYPES: [list the 6 types with example keywords]
ISSUES: [any problems encountered]
```

---

## Agent C: Security Scanner

### Mission
Build a regex-based security scanner that detects 9 OWASP vulnerability patterns in agent-generated code.

### Context

**Read these files first:**
```
agentforge-hq/CLAUDE.md
agentforge-hq/NORTH_STAR.md
```

**Understand the safety crate:**
```
agentforge-hq/crates/forge-safety/src/lib.rs  (CircuitBreaker, RateLimiter, CostTracker)
agentforge-hq/crates/forge-safety/Cargo.toml  (current dependencies)
```

### Instructions

1. Read `crates/forge-safety/src/lib.rs` to understand the crate's structure.
2. Create `crates/forge-safety/src/scanner.rs`:
   ```rust
   pub struct SecurityFinding {
       pub pattern: String,      // e.g. "command_injection"
       pub severity: Severity,   // Critical, High, Medium, Low
       pub line: usize,
       pub snippet: String,      // the offending line
       pub description: String,  // human-readable explanation
   }

   pub enum Severity { Critical, High, Medium, Low }

   pub struct SecurityScanner { /* compiled regexes */ }
   impl SecurityScanner {
       pub fn new() -> Self { /* compile 9 patterns */ }
       pub fn scan(&self, code: &str) -> Vec<SecurityFinding> { /* scan each line */ }
   }
   ```
3. Implement 9 OWASP detection patterns:
   1. `command_injection` — `os.system`, `subprocess.call` with format strings / f-strings (Critical)
   2. `xss_dangerous_html` — `innerHTML`, `dangerouslySetInnerHTML`, `v-html` (High)
   3. `eval_injection` — `eval(`, `exec(`, `Function(` with dynamic input (Critical)
   4. `sql_injection` — string-formatted SQL: `f"SELECT`, `"SELECT" + `, `.format(` near SQL (Critical)
   5. `path_traversal` — `../` in file operations, unsanitized path joins (High)
   6. `pickle_deserialization` — `pickle.loads`, `pickle.load` on untrusted data (High)
   7. `hardcoded_secrets` — patterns like `api_key = "`, `password = "`, `secret = "` (Medium)
   8. `insecure_random` — `Math.random()`, `random.random()` in security contexts (Low)
   9. `open_redirect` — `redirect(request.GET`, unvalidated URL redirects (Medium)
4. Add `pub mod scanner;` to `crates/forge-safety/src/lib.rs`.
5. If `regex` is not already in Cargo.toml, add it.
6. Write 12+ tests:
   - One test per pattern (9 tests), each with a realistic code snippet
   - `test_clean_code_passes`
   - `test_multiple_findings_returns_all`
   - `test_scanner_handles_empty_input`

### Deliverables
- `crates/forge-safety/src/scanner.rs` — SecurityScanner + SecurityFinding + tests
- One-line addition to `crates/forge-safety/src/lib.rs`
- Possibly `regex` dep added to `crates/forge-safety/Cargo.toml`

### Verify
```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-safety -- scanner 2>&1  # all tests pass
```

### Do NOT
- Modify middleware.rs (coordinator handles integration)
- Touch any frontend code
- Touch any other crate besides forge-safety

### Report Format
When done, output:
```
STATUS: done
FILES_CREATED: [list]
FILES_MODIFIED: [list]
TESTS_ADDED: N
PATTERNS: [list 9 patterns with severity]
ISSUES: [any problems encountered]
```

---

## Agent D: E1 Backend Polish

### Mission
Add missing backend APIs that the frontend needs for a complete E1 experience: company detail, company update/delete, department CRUD, persona detail view enrichment.

### Context

**Read these files first:**
```
agentforge-hq/CLAUDE.md
agentforge-hq/NORTH_STAR.md
```

**Understand existing routes:**
```
agentforge-hq/crates/forge-api/src/routes/org.rs        (company, department, org-position routes)
agentforge-hq/crates/forge-api/src/routes/personas.rs    (persona list, get, hire)
agentforge-hq/crates/forge-api/src/routes/governance.rs  (goals, approvals)
agentforge-hq/crates/forge-api/src/lib.rs                (route wiring + tests)
```

**Understand existing repos:**
```
agentforge-hq/crates/forge-db/src/repos/companies.rs
agentforge-hq/crates/forge-db/src/repos/departments.rs
agentforge-hq/crates/forge-db/src/repos/org_positions.rs
```

### Instructions

1. Read existing routes and repos to understand the patterns used.
2. Add these missing endpoints (if not already present):

   **Companies:**
   - `GET /api/v1/companies/:id` — single company detail
   - `PATCH /api/v1/companies/:id` — update name, mission, budget_limit
   - `DELETE /api/v1/companies/:id` — delete company (cascade departments + positions)

   **Departments:**
   - `GET /api/v1/departments/:id` — single department detail
   - `PATCH /api/v1/departments/:id` — update name, description
   - `DELETE /api/v1/departments/:id` — delete department

   **Personas:**
   - `GET /api/v1/personas/divisions` — list all divisions with counts (for filter dropdown)

3. For each new endpoint:
   - Add repo method if needed (in the appropriate `repos/*.rs` file)
   - Add route handler in the appropriate `routes/*.rs` file
   - Wire into the router

4. Add integration tests in `crates/forge-api/src/lib.rs` using the existing `test_state()` helper and `json_post`/`json_get`/`json_patch` helpers.

### Key Rules
- Follow existing patterns: `api_error()` for error mapping, `Json<T>` responses
- Use repo methods, never raw SQL in routes
- Zero warnings policy

### Deliverables
- Updated route files with new endpoints
- Updated repo files with new methods
- Integration tests for each new endpoint

### Verify
```bash
cargo check 2>&1 | grep -c warning  # must be 0
cargo test -p forge-api 2>&1         # all tests pass
cargo test -p forge-db 2>&1          # all tests pass
```

### Do NOT
- Touch frontend code
- Touch middleware
- Modify migrations (use existing schema)
- Change the AppState struct

### Report Format
When done, output:
```
STATUS: done
FILES_MODIFIED: [list]
ENDPOINTS_ADDED: [list with methods]
TESTS_ADDED: N
ISSUES: [any problems encountered]
```

---

## Coordination Rules (All Agents)

### Before Starting
1. Read `CLAUDE.md` and `NORTH_STAR.md`
2. Read the files listed in your Context section
3. Understand existing patterns before writing code

### While Working
- **Zero warnings**: `cargo check` must produce 0 warnings
- **Tests pass**: Run your crate's tests after changes
- **Commit when done**: Create a single commit with descriptive message
- **Do NOT touch files outside your scope**

### Commit Message Format
```
<type>: <short description>

<what changed and why>

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
```

### When Done
Output your report, then stop. The coordinator will:
1. Pull and verify each agent's work
2. Run full `cargo test` and `cargo check`
3. Resolve any conflicts
4. Update NORTH_STAR.md

---

## Launch Commands

Open 4 Zellij tabs and run one agent per tab:

```bash
# Tab 1 — Agent A: Skills Importer
cd /Users/bm/cod/trend/10-march/agentforge-hq
claude --print "$(cat docs/agents/WAVE_NEXT.md)" --prompt "You are Agent A: Skills Importer. Follow the instructions in the Agent A section above. Start by reading the files listed in your Context section, then proceed with your task. When done, output your report."

# Tab 2 — Agent B: Task Detector
cd /Users/bm/cod/trend/10-march/agentforge-hq
claude --print "$(cat docs/agents/WAVE_NEXT.md)" --prompt "You are Agent B: Task Type Detection Engine. Follow the instructions in the Agent B section above. Start by reading the files listed in your Context section, then proceed with your task. When done, output your report."

# Tab 3 — Agent C: Security Scanner
cd /Users/bm/cod/trend/10-march/agentforge-hq
claude --print "$(cat docs/agents/WAVE_NEXT.md)" --prompt "You are Agent C: Security Scanner. Follow the instructions in the Agent C section above. Start by reading the files listed in your Context section, then proceed with your task. When done, output your report."

# Tab 4 — Agent D: E1 Backend Polish
cd /Users/bm/cod/trend/10-march/agentforge-hq
claude --print "$(cat docs/agents/WAVE_NEXT.md)" --prompt "You are Agent D: E1 Backend Polish. Follow the instructions in the Agent D section above. Start by reading the files listed in your Context section, then proceed with your task. When done, output your report."
```

---

## Risk Matrix

| Risk | Impact | Mitigation |
|------|--------|------------|
| Agent A + B + C all touch Cargo.toml | Merge conflict | Only C may add `regex` dep; A and B should not touch Cargo.toml |
| Agent B modifies forge-process/src/lib.rs | Minimal | Only adding one `pub mod` line |
| Agent C modifies forge-safety/src/lib.rs | Minimal | Only adding one `pub mod` line |
| Agent D touches routes used by other agents | Possible | D owns org.rs + personas.rs; no overlap with B/C |
| Frontend polish session conflicts | Low | That session only touches frontend/; these agents only touch backend + skills |
