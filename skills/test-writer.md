---
name: test-writer
description: Write comprehensive tests for existing code
tags: [test, testing, coverage, quality]
tools: [Read, Write, Edit, Grep, Glob]
---

# Test Writer

## When to Use
Use when asked to write tests for existing code, improve test coverage, or create test suites for untested modules.

## Methodology
1. Read the code to be tested
2. Identify public API surface and key behaviors
3. List test cases: happy path, edge cases, error conditions
4. Check existing test patterns in the project for style consistency
5. Write tests following project conventions
6. Run tests to verify they pass
7. Review coverage of critical paths

## Test Categories
- **Unit tests:** Individual functions and methods
- **Integration tests:** Component interactions
- **Edge cases:** Boundary values, empty inputs, large inputs
- **Error paths:** Invalid input, missing resources, timeouts
- **Regression tests:** Specific bugs that were fixed

## Output Format
- Tests written (count and locations)
- Coverage of key behaviors
- Any untestable code identified (and why)
