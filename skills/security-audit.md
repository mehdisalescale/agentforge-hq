---
name: security-audit
description: Security vulnerability assessment and hardening
tags: [security, audit, vulnerability, safety]
tools: [Read, Grep, Glob]
---

# Security Audit

## When to Use
Use when asked to review code for security vulnerabilities, assess attack surface, or harden an application.

## Methodology
1. Map the attack surface (inputs, APIs, file access, network)
2. Review authentication and authorization logic
3. Check for OWASP Top 10 vulnerabilities
4. Analyze data flow for injection points
5. Review secrets management and configuration
6. Check dependency versions for known CVEs
7. Assess error handling (no sensitive data leaks)

## OWASP Top 10 Checklist
- Injection (SQL, command, XSS)
- Broken authentication
- Sensitive data exposure
- XML external entities (XXE)
- Broken access control
- Security misconfiguration
- Cross-site scripting (XSS)
- Insecure deserialization
- Using components with known vulnerabilities
- Insufficient logging and monitoring

## Output Format
- Vulnerabilities found (severity: critical/high/medium/low)
- Attack vectors described
- Remediation steps for each issue
- Overall security posture assessment
