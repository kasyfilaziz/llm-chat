# Implementation Plan: Phase 2 Unified Domain

**Branch**: `feature/002-tracer-bullet` | **Date**: 2026-05-15 | **Spec**: specs/003-phase2-unified-domain/spec.md
**Input**: Feature specification from `/specs/003-phase2-unified-domain/spec.md`

## Summary

Expand Lumina's core architecture by introducing isolated domains for Conversation History (SQLite), Application Settings (YAML), and Model Context Protocol (MCP) Foundation (Stdio process hosting). This phase establishes persistent sidebars, dynamic configurations via global state, and non-blocking external tool execution using a Pragmatic Hybrid DDD structure.

## Technical Context

**Language/Version**: Rust 1.80.0+  
**Primary Dependencies**: Dioxus 0.7 (`GlobalSignal`, `#[derive(Store)]`), `rusqlite` (Database), `serde_yaml_ng` (YAML Persistence), `rmcp` (MCP SDK), `tokio` (Async runtime)  
**Storage**: SQLite (`lumina.db`) for Chat; YAML (`settings.yaml`) for Configuration.  
**Testing**: `cargo test`  
**Target Platform**: Desktop (Windows, macOS, Linux) via System Native WebView  
**Project Type**: desktop-app  
**Performance Goals**: <100ms SQLite history loads, <50ms UI settings propagation, 60 fps rendering  
**Constraints**: UI thread must NEVER block during database I/O, file writes, or MCP child process execution.  
**Scale/Scope**: Support for thousands of messages per thread (via pagination) and multiple concurrent local MCP servers.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Local-First Architecture**: PASS. All history remains in SQLite; settings are in local YAML; MCP tools run locally via stdio.
- **II. Cross-Platform Consistency**: PASS. UI components continue to use Dioxus and Tailwind CSS without native platform deviations.
- **III. Extensible Tooling via MCP**: PASS. This phase directly implements the host-side foundation for MCP via `rmcp`.
- **IV. Source-Grounded Integrity**: N/A for this specific phase (no RAG implemented yet, but MCP provides the tool paths for future phases).
- **Security & Privacy**: PASS. UI explicit errors mandated for MCP crashes prevent silent failures. API keys moved from `.env` to a dedicated `settings.yaml` under the user's secure config directory.

## Project Structure

### Documentation (this feature)

```text
specs/003-phase2-unified-domain/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
└── quickstart.md        # Phase 1 output
```

### Source Code (repository root)

```text
src/
├── main.rs                 # Entry point
├── app.rs                  # Global router, layout wrap, and MCP Orchestrator Coroutine
├── db.rs                   # SQLite schemas (updated for 'conversations')
├── components/             # SHARED UI: Dumb components only
│   └── mod.rs
├── domains/                # SMART DOMAINS: DDD boundaries
│   ├── chat/
│   │   ├── mod.rs
│   │   ├── screen.rs       # Main Chat page
│   │   ├── state.rs        # ConversationStore (Pagination logic)
│   │   ├── repo.rs         # SQLite queries (conversations + messages)
│   │   └── widgets/        # Sidebar, MessageBubble
│   ├── settings/           # NEW DOMAIN
│   │   ├── mod.rs
│   │   ├── screen.rs       # Settings UI Modal/Page
│   │   ├── state.rs        # GlobalSignal<AppSettings>
│   │   └── repo.rs         # serde_yaml_ng async file I/O
│   ├── mcp/                # NEW DOMAIN
│   │   ├── mod.rs
│   │   ├── client.rs       # rmcp TokioChildProcess wrapper
│   │   └── state.rs        # McpEvent enums and use_coroutine logic
│   └── llm/
│       └── ...             # (Phase 1 untouched, except reading from Global Settings)
└── utils/                  # Generic helpers
```

**Structure Decision**: "Unified Domain" Extension. We maintain the "Pragmatic Hybrid DDD" established in Phase 1, isolating the new logic for `settings` and `mcp` into their own domains, preventing monoliths.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| *None* | *N/A* | *The proposed architecture aligns directly with existing patterns.* |