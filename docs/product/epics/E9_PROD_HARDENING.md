# Epic E9: Production Hardening

> **Authentication, E2E testing, performance optimization, documentation, and release polish.**

---

## Business Value

Everything built in E1-E8 must be production-grade before v1.0.0. This epic covers the cross-cutting concerns that make the difference between a demo and a product.

## Acceptance Gate

1. Authentication works (JWT, multi-user)
2. E2E test suite covers all critical paths
3. Performance benchmarks meet targets (startup <2s, P95 API <50ms)
4. Full OpenAPI spec generated and verified
5. Docker Compose deploys all services
6. Migration path from v0.6.0 documented and tested
7. README, quickstart, and deployment guide complete

---

## User Stories (10)

### E9-S1: Authentication System
- Better-auth integration or custom JWT
- Login/register endpoints
- API key auth for MCP and external clients
- Company membership (user → company role)

### E9-S2: RBAC (Role-Based Access Control)
- Roles: owner, admin, member, viewer
- Company-scoped permissions
- Agent actions respect caller's role

### E9-S3: E2E Test Suite
- Playwright tests for all 20 frontend pages
- API integration tests for all 50+ endpoints
- Multi-backend E2E (Claude + mock Hermes)
- Budget enforcement E2E
- Approval flow E2E

### E9-S4: Performance Benchmarking
- Startup time benchmark (target: <2s)
- API latency benchmark (target: P95 <50ms)
- Concurrent session benchmark (target: 50+)
- Memory footprint benchmark (target: <100MB idle)
- Event throughput benchmark (target: 1000/sec)

### E9-S5: Connection Pooling
- Replace single `Arc<Mutex<Connection>>` with read/write pool
- Read replicas for analytics, KB search, session list
- Write path remains single-writer (SQLite WAL)

### E9-S6: Docker Compose Deployment
- Dockerfile for Forge binary (multi-stage, ~50MB)
- docker-compose.yml: forge + optional astrbot + optional openclaw
- Health checks and restart policies
- Volume mounts for data persistence
- Environment variable documentation

### E9-S7: Migration Path (v0.6.0 → v1.0.0)
- All new migrations are additive (no breaking changes to existing tables)
- Default company auto-created for existing data
- Existing agents assigned to default company
- Existing skills preserved alongside new imports
- Migration guide document

### E9-S8: OpenAPI Spec Completion
- All 50+ routes annotated with utoipa
- Request/response schemas generated
- Scalar UI at /docs with all endpoints
- TypeScript types generated from OpenAPI for desktop client

### E9-S9: Documentation Suite
- README.md rewrite (quickstart, features, architecture)
- docs/quickstart.md (5-minute getting started)
- docs/deployment.md (Docker, binary, cloud)
- docs/api-reference.md (auto-generated from OpenAPI)
- docs/architecture.md (crate dependency graph, data flow)
- docs/persona-authoring.md (how to write custom personas)

### E9-S10: Release Pipeline
- GitHub Actions: test → build → package → release
- 3 binary targets (macOS ARM64, macOS x64, Linux x64)
- Desktop app builds (macOS, Windows, Linux)
- Docker image push to ghcr.io
- Changelog generation from git history

---

## Story Point Estimates: **45 total** across S8-S9
