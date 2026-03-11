# Epic E2: Development Methodology

> **Equip engineering agents with structured TDD, debugging, brainstorming, planning, and code review workflows.**
>
> Source repos: superpowers (14 skills), claude-code plugins (13 plugins, 9 security patterns)

---

## Business Value

Engineering agents currently execute prompts with no methodology. This epic gives them structured workflows: TDD (red-green-refactor), systematic debugging (4-phase root cause), design brainstorming (Socratic), implementation planning (bite-sized tasks), and code review (confidence-scored parallel agents). The result: agents produce higher-quality, more predictable output.

## Acceptance Gate

**The epic is DONE when:**
1. Task-type detection correctly classifies prompts into 5 categories
2. Appropriate skills are auto-injected based on task type
3. Security scanner detects all 9 OWASP patterns in generated code
4. Code review plugin runs 6 parallel evaluations with confidence scoring
5. All 14 superpowers skills + 6 plugin skills load without errors
6. 30+ tests covering detection, injection, scanning, and review

---

## User Stories

### E2-S1: Skill Import Pipeline (Superpowers + Plugins → Forge Skills)

**As a** system administrator,
**I want** superpowers and claude-code plugin skills imported into Forge's skill system,
**So that** they're available for injection into agent runs.

**Acceptance Criteria:**

```gherkin
GIVEN 14 superpowers skill files exist in skills/superpowers/
WHEN the server starts and loads skills
THEN all 14 are parsed and stored in the skills table

GIVEN 6 adapted plugin skills exist in skills/plugins/
WHEN the server starts
THEN all 6 are parsed and stored

GIVEN a skill has YAML frontmatter with tags and trigger patterns
WHEN it is loaded
THEN tags, triggers, and auto-activation rules are stored correctly
```

**Technical Notes:**
- Adapt superpowers SKILL.md format to Forge's YAML frontmatter format
- Adapt claude-code plugin prompts to skill format
- Extend `SkillRepo.load_from_dir()` to handle nested directories

**Test Plan:**
- `test_load_superpowers_skills_all_14`
- `test_load_plugin_skills_all_6`
- `test_skill_frontmatter_parsing`
- `test_nested_directory_loading`

---

### E2-S2: Task Type Detection Engine

**As a** system,
**I want** to automatically classify user prompts into task types,
**So that** the right methodology skills are injected.

**Acceptance Criteria:**

```gherkin
GIVEN a prompt containing "add a new endpoint for user profiles"
WHEN TaskTypeDetector.classify(prompt) is called
THEN it returns TaskType::NewFeature

GIVEN a prompt containing "the login is returning 500 errors"
WHEN TaskTypeDetector.classify(prompt) is called
THEN it returns TaskType::BugFix

GIVEN a prompt containing "review this PR" or "check this code"
WHEN TaskTypeDetector.classify(prompt) is called
THEN it returns TaskType::CodeReview

GIVEN a prompt containing "refactor the database layer"
WHEN TaskTypeDetector.classify(prompt) is called
THEN it returns TaskType::Refactor

GIVEN a prompt containing "how does the auth system work"
WHEN TaskTypeDetector.classify(prompt) is called
THEN it returns TaskType::Research

GIVEN an ambiguous prompt
WHEN TaskTypeDetector.classify(prompt) is called
THEN it returns TaskType::General (no methodology injection)
```

**Technical Notes:**
- Keyword + pattern matching (not LLM-based — too slow for middleware)
- `TaskType` enum: `NewFeature | BugFix | CodeReview | Refactor | Research | General`
- Configurable keyword lists per type
- `TaskTypeDetector` in `forge-process` crate

**Test Plan:**
- `test_classify_new_feature_keywords`
- `test_classify_bugfix_keywords`
- `test_classify_code_review_keywords`
- `test_classify_refactor_keywords`
- `test_classify_research_keywords`
- `test_classify_ambiguous_returns_general`
- `test_classify_case_insensitive`

---

### E2-S3: Skill Router (Task Type → Skill Injection)

**As a** system,
**I want** the correct methodology skills injected into the system prompt based on task type,
**So that** agents follow structured workflows.

**Acceptance Criteria:**

```gherkin
GIVEN TaskType::NewFeature detected
WHEN SkillRouter selects skills
THEN it injects: brainstorming, writing-plans, test-driven-development, subagent-driven-development

GIVEN TaskType::BugFix detected
WHEN SkillRouter selects skills
THEN it injects: systematic-debugging, test-driven-development

GIVEN TaskType::CodeReview detected
WHEN SkillRouter selects skills
THEN it injects: code-review (confidence-scored), security-guidance

GIVEN TaskType::General detected
WHEN SkillRouter selects skills
THEN no methodology skills are injected (only keyword-matched skills as before)

GIVEN a custom skill_task_routes configuration
WHEN the admin has mapped TaskType::BugFix to additional skills
THEN the custom mapping is used instead of defaults
```

**Technical Notes:**
- DB table: `skill_task_routes` (task_type, skill_ids[], priority)
- Default mappings seeded in migration
- SkillRouter replaces/extends existing SkillInjectionMiddleware

**Test Plan:**
- `test_new_feature_injects_4_skills`
- `test_bugfix_injects_2_skills`
- `test_code_review_injects_2_skills`
- `test_general_injects_none`
- `test_custom_routing_overrides_defaults`

---

### E2-S4: TaskTypeDetection Middleware

**As a** system,
**I want** task type detection as a middleware in the run pipeline,
**So that** it integrates seamlessly with existing middleware chain.

**Acceptance Criteria:**

```gherkin
GIVEN a run request with prompt "fix the login bug"
WHEN the middleware chain executes
THEN TaskTypeDetection middleware runs after SkillInjection
AND sets ctx.metadata["task_type"] = "bug_fix"
AND SkillRouter injects systematic-debugging + TDD skills into context

GIVEN a run request with prompt "tell me about the codebase"
WHEN the middleware chain executes
THEN TaskTypeDetection sets task_type = "research"
AND only exploration skills are injected
```

**Technical Notes:**
- New middleware: `TaskTypeDetectionMiddleware`
- Position: after SkillInjection (slot 5 in chain)
- Reads prompt from `ctx.prompt`, writes to `ctx.metadata`
- SkillRouter reads from `ctx.metadata["task_type"]`

**Test Plan:**
- `test_middleware_sets_task_type_metadata`
- `test_middleware_chain_order_correct`
- `test_middleware_passes_through_on_general`

---

### E2-S5: Security Scanner (9 OWASP Patterns)

**As a** system administrator,
**I want** all agent-generated code scanned for security vulnerabilities,
**So that** dangerous code is flagged before it reaches production.

**Acceptance Criteria:**

```gherkin
GIVEN agent output containing `eval(user_input)`
WHEN SecurityScanner.scan(output) runs
THEN it returns SecurityScanResult::Failed with
  pattern: "eval_injection", line: 42, severity: "critical"

GIVEN agent output containing `innerHTML = userContent`
WHEN SecurityScanner.scan(output) runs
THEN it returns Failed with pattern: "xss_dangerous_html"

GIVEN agent output containing `os.system(f"rm {user_path}")`
WHEN SecurityScanner.scan(output) runs
THEN it returns Failed with pattern: "command_injection"

GIVEN agent output with no security issues
WHEN SecurityScanner.scan(output) runs
THEN it returns SecurityScanResult::Passed

The 9 patterns to detect:
1. command_injection (os.system, subprocess with user input)
2. xss_dangerous_html (innerHTML, dangerouslySetInnerHTML)
3. eval_injection (eval, exec with dynamic input)
4. sql_injection (string-formatted SQL queries)
5. path_traversal (../ in file paths from user input)
6. pickle_deserialization (pickle.loads on untrusted data)
7. hardcoded_secrets (API keys, passwords in source)
8. insecure_random (Math.random for security purposes)
9. open_redirect (unvalidated redirect URLs)
```

**Technical Notes:**
- Rust regex patterns (compiled once, reused)
- `SecurityScanner` in `forge-safety` crate
- Returns `Vec<SecurityFinding>` with pattern, line, severity, snippet

**Test Plan:**
- One test per pattern (9 tests)
- `test_clean_code_passes`
- `test_multiple_findings_returns_all`
- `test_scanner_handles_empty_input`

---

### E2-S6: SecurityScan Middleware

**As a** system,
**I want** security scanning as a post-execution middleware,
**So that** every agent run is automatically scanned.

**Acceptance Criteria:**

```gherkin
GIVEN an agent run completes with code output
WHEN SecurityScan middleware runs post-execution
THEN all code blocks in the output are scanned
AND findings are emitted as SecurityScanPassed or SecurityScanFailed events
AND findings are stored in session events

GIVEN a critical security finding
WHEN SecurityScan middleware detects it
THEN a warning is emitted (not a hard block — user decides)
AND the finding details appear in the session event stream
```

**Technical Notes:**
- Position: last in middleware chain (after QualityGate)
- Extracts code blocks from output (markdown ``` fences)
- Non-blocking: logs findings as events, doesn't reject output
- New events: `SecurityScanPassed`, `SecurityScanFailed`

**Test Plan:**
- `test_middleware_scans_code_blocks`
- `test_middleware_emits_passed_event`
- `test_middleware_emits_failed_event_with_findings`
- `test_middleware_handles_no_code_blocks`

---

### E2-S7: Code Review Engine (Confidence-Scored Parallel Review)

**As a** user requesting code review,
**I want** multiple specialist reviewers analyzing different aspects in parallel,
**So that** I get comprehensive, high-confidence feedback.

**Acceptance Criteria:**

```gherkin
GIVEN a code review task
WHEN the review engine runs
THEN 6 parallel sub-agents analyze:
  1. PR comments quality
  2. Test coverage adequacy
  3. Error handling completeness
  4. Type design correctness
  5. Code quality (SOLID, DRY)
  6. Simplification opportunities

GIVEN each sub-agent returns findings with confidence scores
WHEN results are aggregated
THEN only findings with confidence >= 80 are included in the final report
AND findings are classified: Critical (90+), Important (80-89), Minor (<80 but included as info)

GIVEN the review completes
WHEN the report is generated
THEN it is posted as a session event with structured findings
AND a ReviewCompleted event is emitted
```

**Technical Notes:**
- Uses existing `ConcurrentRunner` for parallel execution
- 6 specialist prompts derived from claude-code PR review toolkit
- Confidence scoring via structured output parsing
- Threshold configurable via `FORGE_REVIEW_CONFIDENCE_THRESHOLD` (default: 80)

**Test Plan:**
- `test_review_runs_6_parallel_agents`
- `test_confidence_filtering_at_80`
- `test_severity_classification`
- `test_review_event_emitted`

---

### E2-S8: Methodology Frontend Enhancements

**As a** user,
**I want** to see which methodology was applied to each agent run,
**So that** I understand the agent's approach and can configure it.

**Acceptance Criteria:**

```gherkin
GIVEN a session ran with TDD methodology
WHEN I view the session detail
THEN I see a "Methodology: TDD" badge
AND the skill names that were injected are listed

GIVEN I'm on the Settings page
WHEN I navigate to the "Methodology" section
THEN I see the task type → skill mapping table
AND I can enable/disable specific skill routes
AND I can add custom skill routes

GIVEN a security scan found issues
WHEN I view the session detail
THEN I see a security findings panel with severity badges
AND each finding shows the pattern, line, and code snippet
```

**Test Plan:**
- E2E: methodology badge shows on session
- E2E: settings page allows skill route configuration
- E2E: security findings display correctly

---

### E2-S9: Skills Directory Population

**As a** developer,
**I want** all 20 new skills (14 superpowers + 6 plugins) committed to the repo,
**So that** they're available on fresh installs.

**Acceptance Criteria:**

```gherkin
GIVEN a fresh clone of the repository
WHEN I run the server
THEN 30 skills load (10 original + 14 superpowers + 6 plugins)
AND each skill has valid YAML frontmatter with name, description, tags

GIVEN the superpowers TDD skill
WHEN it is injected into a system prompt
THEN the agent receives clear instructions for RED-GREEN-REFACTOR cycle
```

**Technical Notes:**
- Adapt each superpowers SKILL.md to Forge format (YAML frontmatter)
- Adapt each plugin to a standalone skill file
- Organize: `skills/superpowers/`, `skills/plugins/`

**Test Plan:**
- `test_all_30_skills_load_on_startup`
- `test_each_skill_has_valid_frontmatter`

---

## Story Point Estimates

| Story | Points | Sprint |
|-------|--------|--------|
| E2-S1 Skill Import | 3 | S1 |
| E2-S2 Task Type Detection | 3 | S1 |
| E2-S3 Skill Router | 3 | S1 |
| E2-S4 Detection Middleware | 2 | S1 |
| E2-S5 Security Scanner | 5 | S2 |
| E2-S6 SecurityScan Middleware | 2 | S2 |
| E2-S7 Code Review Engine | 8 | S2 |
| E2-S8 Frontend Enhancements | 3 | S2 |
| E2-S9 Skills Population | 2 | S1 |
| **Total** | **31** | |
