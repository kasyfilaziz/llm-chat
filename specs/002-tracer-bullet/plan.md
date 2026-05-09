# Implementation Plan: Phase 1 Core Loop (Tracer Bullet)

**Branch**: `feature/002-tracer-bullet` | **Date**: 2026-05-09 | **Spec**: specs/002-tracer-bullet/spec.md
**Input**: Feature specification from `specs/002-tracer-bullet/spec.md`

## Summary

Build an end-to-end "Tracer Bullet" chat interface that proves Dioxus UI, SQLite disk I/O, and asynchronous LLM streaming (OpenAI and Ollama) can work together without blocking the main thread, using an Event-Driven Actor model via Dioxus `use_coroutine`.

## Technical Context

**Language/Version**: Rust 1.80.0+  
**Primary Dependencies**: Dioxus 0.7, Tailwind CSS v4, `reqwest` (HTTP), `reqwest-eventsource` (OpenAI SSE), `rusqlite` (Database), `uuid` (v4), `tokio` (Async runtime)  
**Storage**: SQLite (`lumina.db` local file)  
**Testing**: `cargo test`  
**Target Platform**: Desktop (Windows, macOS, Linux) via System Native WebView  
**Project Type**: desktop-app  
**Performance Goals**: <100MB RAM overhead, 60fps streaming token updates without UI stutter  
**Constraints**: UI thread must NEVER block during database I/O or network requests. Credentials must be loaded from `.env`, not hardcoded.  
**Scale/Scope**: Single chat thread, `.env` file configuration, support for OpenAI OR Ollama API formats.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Local-First Architecture**: PASS. Data is stored purely locally in SQLite.
- **II. Cross-Platform Consistency**: PASS. Using Dioxus + Tailwind CSS.
- **Security & Privacy**: PASS. API credentials and Configuration are loaded via a `.env` file. No hardcoded secrets.

## Project Structure

### Documentation (this feature)

```text
specs/002-tracer-bullet/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
└── quickstart.md        # Phase 1 output
```

### Source Code (repository root)

```text
src/
├── main.rs                 # Entry point
├── app.rs                  # Global router and layout wrap
├── db.rs                   # Global SQLite connection pool management
├── components/             # SHARED UI: Dumb components only
│   └── mod.rs
├── domains/                # SMART DOMAINS: DDD boundaries
│   ├── chat/
│   │   ├── mod.rs
│   │   ├── screen.rs       # The Chat page (smart component)
│   │   ├── state.rs        # Global use_signal definitions for chat
│   │   ├── repo.rs         # SQLite queries (spawn_blocking)
│   │   └── widgets/        # Chat-specific components
│   └── llm/
│       ├── mod.rs
│       ├── client.rs       # Base HTTP client / Traits
│       ├── providers.rs    # OpenAI, Ollama parsing logic
│       └── models.rs       # Prompt/Response structs
└── utils/                  # Generic helpers (formatting, env loading)
```

**Structure Decision**: "Pragmatic Hybrid DDD" (UI-Aware DDD). This avoids massive Clean Architecture boilerplate while keeping UI and Business domains (Chat, LLM) cleanly isolated.
