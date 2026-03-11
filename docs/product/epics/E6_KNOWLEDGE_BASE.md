# Epic E6: Knowledge Base

> **Shared organizational knowledge with document upload, chunking, and FTS5 search.**
>
> Source: AstrBot (knowledge base concepts, adapted to zero-dep Rust)

---

## Business Value

Agents currently have no shared context beyond their own memory. This epic gives every company a knowledge base: upload documents, chunk them, search via FTS5, and auto-inject relevant context into agent runs. An engineering agent can reference API docs; a support agent can search product FAQs.

## Acceptance Gate

1. Users can upload documents (text, markdown, PDF)
2. Documents are chunked and indexed via FTS5
3. Search returns ranked chunks with relevance scoring
4. KnowledgeInjection middleware auto-injects relevant KB context
5. KB is scoped to companies
6. 20+ tests

---

## User Stories

### E6-S1: Document Model & Chunking
**As a** user, **I want** documents parsed into searchable chunks.

```gherkin
GIVEN I upload a 5000-word markdown document
WHEN the document parser processes it
THEN it splits into chunks of ~500 tokens each (paragraph-aware boundaries)
AND each chunk stores: content, chunk_index, source_document_id

GIVEN I upload a PDF file
WHEN the parser processes it
THEN text is extracted and chunked (using lopdf or pdf-extract Rust crate)
```

Tests: `test_chunk_markdown_by_paragraphs`, `test_chunk_respects_token_limit`, `test_pdf_text_extraction`

### E6-S2: FTS5 Search Index
**As a** user, **I want** full-text search across all knowledge base documents.

```gherkin
GIVEN 50 documents with 500 chunks total are indexed
WHEN I search "authentication flow"
THEN chunks containing those terms are returned ranked by BM25 relevance
AND results include document title, chunk content, and relevance score

GIVEN I search within a specific company
THEN only that company's documents are searched
```

Tests: `test_fts5_search_returns_ranked`, `test_search_scoped_to_company`, `test_search_empty_returns_empty`

### E6-S3: KB API Endpoints
```
POST   /api/v1/knowledge/documents     — upload + parse + chunk + index
GET    /api/v1/knowledge/documents     — list documents (by company)
DELETE /api/v1/knowledge/documents/:id — remove document + chunks
POST   /api/v1/knowledge/search        — FTS5 search
GET    /api/v1/knowledge/chunks/:doc_id — list chunks for a document
```

### E6-S4: KnowledgeInjection Middleware
**As a** system, **I want** relevant KB context auto-injected into agent system prompts.

```gherkin
GIVEN an agent run in a company with KB documents
WHEN KnowledgeInjection middleware runs
THEN it searches KB for top-5 relevant chunks based on the user prompt
AND prepends them to the system prompt as "Reference Context"

GIVEN no KB documents exist
WHEN KnowledgeInjection middleware runs
THEN it passes through without modification (no-op)
```

Tests: `test_injects_top_5_chunks`, `test_noop_when_no_kb`, `test_company_scoped`

### E6-S5: Knowledge Base Frontend Page
- Document list with source type badges
- Upload form (drag & drop)
- Search bar with highlighted results
- Chunk preview panel
- Per-company scope selector

### E6-S6: Document Embedding (Future-Ready)
**As a** system, **I want** an optional embedding endpoint for semantic search.

```gherkin
GIVEN FORGE_EMBEDDING_API_URL is configured
WHEN a document is chunked
THEN each chunk's embedding is computed and stored
AND search uses cosine similarity ranking alongside FTS5

GIVEN FORGE_EMBEDDING_API_URL is NOT configured
WHEN search is performed
THEN pure FTS5 keyword search is used (no degradation)
```

### E6-S7: MCP Knowledge Tools
- `forge_knowledge_search` — search KB from MCP client
- `forge_knowledge_upload` — add document via MCP

---

## Story Point Estimates: **28 total** across S5-S6
