# Epic E8: Desktop Client

> **Native Electron desktop application connecting to Forge API.**
>
> Source: Open-Claude-Cowork (Electron + React)

---

## Business Value

Power users want a native desktop app: always-on, system tray, keyboard shortcuts, file drag-and-drop, native notifications. The web dashboard is good for management; the desktop app is for hands-on collaboration with agents.

## Acceptance Gate

1. Desktop app connects to Forge API (REST + WebSocket)
2. Chat interface with real-time streaming
3. Sidebar shows companies, agents, sessions
4. Permission control for agent tool execution
5. Builds for macOS, Windows, Linux
6. Works alongside web dashboard (both connect to same Forge server)

---

## User Stories (7)

### E8-S1: Forge API Client Module
- Replace @anthropic-ai/claude-agent-sdk with forge-client.ts
- REST client for all Forge endpoints
- WebSocket client for event streaming

### E8-S2: Chat Interface (Streaming)
- Token-by-token streaming via WebSocket
- Markdown rendering with syntax highlighting
- Tool call visualization with status indicators

### E8-S3: Company & Agent Sidebar
- Company switcher in header
- Agent list grouped by department
- Session history per agent
- Quick-hire from persona catalog

### E8-S4: Org Chart View
- Interactive tree visualization
- Agent cards with status indicators
- Drag-and-drop org chart editing

### E8-S5: Permission & Approval Integration
- Tool permission dialogs (approve/deny)
- Pending approval queue in sidebar
- Native OS notifications for approval requests

### E8-S6: Knowledge Base Search
- Search panel with instant results
- Document viewer
- Upload via drag-and-drop

### E8-S7: Build & Distribution
- Electron Builder: macOS (DMG), Windows (NSIS), Linux (AppImage)
- Auto-updater via GitHub Releases
- System tray with quick actions

---

## Story Point Estimates: **35 total** across S7-S8
