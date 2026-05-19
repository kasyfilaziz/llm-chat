# Implementation Plan: Core Interface Pages — Session Management & Chat Interface

**Branch**: `004-session-chat-interface` | **Date**: 2026-05-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/004-session-chat-ui/spec.md`

## Summary

Implement two core UI pages for the Lumina desktop app: (1) **Session Management** as the default landing page with folder-based session organization, directory tree, bento grid of session cards, CRUD operations, and drag-and-drop; (2) **Chat Interface** refactored into the new Terra dark-themed design with rich markdown, typing indicator, hover actions, and prompt suggestions. The existing chat orchestration (coroutine, store, DB) is preserved — only the visual layer changes. Both pages share a persistent sidebar navigation.

## Technical Context

**Language/Version**: Rust 1.80.0+, Dioxus 0.7.x  
**Primary Dependencies**: dioxus, dioxus-stores, rusqlite (bundled), tokio, reqwest, reqwest-eventsource, serde, serde_json, pulldown-cmark, uuid, dioxus-router  
**Storage**: SQLite via rusqlite (bundled) — local `lumina.db`  
**Testing**: cargo test  
**Target Platform**: Desktop (Linux WebKitGTK, Windows WebView2, macOS WKWebView)  
**Project Type**: Desktop application (Rust + Dioxus native webview)  
**Performance Goals**: 60fps rendering during LLM streaming, <2s session grid load, <3s first token, <100ms interactive feedback  
**Constraints**: Local-first (no remote DB), single-user desktop, no CDN-dependent assets (icons must use local fallbacks), sidebar persistent across all routes  
**Scale/Scope**: Single-user desktop app, target <1000 sessions per folder without degradation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Justification |
|-----------|--------|---------------|
| **I. Local-First Architecture** | ✅ PASS | All folder/session data stored in local SQLite. No external services required. |
| **II. Cross-Platform Consistency** | ✅ PASS | Uses Dioxus shared components with Tailwind CSS — sidebar, session grid, and chat canvas render identically on all platforms. |
| **III. Extensible Tooling via MCP** | ✅ N/A | This feature does not introduce new MCP tools. The MCP orchestrator is untouched. |
| **IV. Source-Grounded Integrity** | ✅ N/A | Session Management does not involve RAG or document citations. Chat Interface preserves existing LLM streaming — citation features are future (Context Vault). |
| **Technical Constraints** | ✅ PASS | Uses Dioxus 0.7, Rust stable, Tailwind CSS v4, SQLite — all within the constitution's allowed stack. |
| **Security & Privacy** | ✅ PASS | No new API keys, no credentials, no cloud sync. All data stays local. |

**Result**: GATE PASSED — no violations. Proceeding to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/004-session-chat-ui/
├── plan.md              # This file
├── spec.md              # Feature specification (speckit.specify)
├── research.md          # Phase 0 output: technical research & decisions
├── data-model.md        # Phase 1 output: entity definitions & relationships
├── quickstart.md        # Phase 1 output: setup & run instructions
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── contracts/           # Phase 1 output: interface contracts
```

### Source Code (repository root)

```text
src/
├── layout.rs                         # NEW: Shared layout with sidebar + Outlet
├── app.rs                            # MODIFY: Add sessions route, layout wrapper
├── db.rs                             # UNCHANGED
├── main.rs                           # UNCHANGED
├── components/                       # ADD: shared sidebar promoted from chat
│   ├── mod.rs
│   └── sidebar.rs                    # MOVED from domains/chat/widgets/
├── domains/
│   ├── chat/                         # MODIFY: widget-by-widget UI replacement
│   │   ├── mod.rs
│   │   ├── screen.rs                 # REWRITE RSX only; preserve coroutine/store
│   │   ├── state.rs
│   │   ├── repo.rs
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── message.rs            # REWRITE for Terra dark theme
│   │       ├── typing_indicator.rs   # NEW: bouncing dots
│   │       └── input_area.rs         # NEW: suggestions + textarea + toolbar
│   ├── sessions/                     # NEW: session management domain
│   │   ├── mod.rs
│   │   ├── screen.rs                 # Session Management page component
│   │   ├── state.rs                  # FolderStore + SessionStore
│   │   ├── repo.rs                   # SQLite queries for folders + sessions
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── directory_tree.rs     # Folder explorer panel
│   │       ├── session_card.rs       # Individual card in bento grid
│   │       └── session_grid.rs       # Bento grid container
│   ├── llm/                          # UNCHANGED
│   ├── mcp/                          # UNCHANGED
│   └── settings/                     # UNCHANGED
```

**Structure Decision**: Follows the established Pragmatic Hybrid DDD pattern — each domain gets `screen.rs`, `state.rs`, `repo.rs`, `widgets/`. The sidebar moves from `chat/widgets/` to `components/` since it's now shared. A new `layout.rs` at `src/` level wraps all routes with the persistent sidebar.

## Complexity Tracking

No constitution violations detected. No complexity justification needed.
