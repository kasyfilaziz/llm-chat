# Problem

Lumina requires a scalable directory structure. As a Dioxus-based LLM chat application built by a Senior Data Engineer / Backend Developer, it needs the rigorous organization of Domain-Driven Design (DDD) without the bureaucratic boilerplate of full Clean Architecture. It must comfortably support heavy UI component reuse alongside massive growth in backend LLM logic.

# User Profile

| Attribute        | Value |
|------------------|-------|
| Technical Level  | Expert (4 yrs Web Backend, 5 yrs Sr Data Engineer) |
| Role / Domain    | Lead Developer / Architect |
| Output Style     | Concise, direct directory trees with strict architectural rules |

> Note: All language, depth, and framing in this document are adapted to the profile above.

# Assumptions

1. Dioxus is a component-driven framework; shared UI elements (Buttons, Layouts, Inputs) are inevitable and should be treated as a first-class citizen.
2. The user has experience with complex backend architectures but specifically requested a "simple directory structure" to start.
3. Feature domains (Chat vs. LLM Providers) are naturally decoupled in business logic but heavily intertwined in the UI.

# Solution Summary

The Pragmatic Hybrid (UI-Aware DDD). This structure maintains a top-level shared `components` and `utils` layer for the frontend, but delegates all business logic, state management, and specific screens to isolated `domains`.

# Detailed Implementation

This structure bridges the gap between Frontend Component Driven Design and Backend Domain Driven Design.

```text
src/
├── main.rs                 # Entry point
├── app.rs                  # Global router and layout wrap
├── db.rs                   # Global SQLite connection pool management
├── components/             # SHARED UI: Dumb components only
│   ├── mod.rs
│   ├── buttons.rs
│   ├── layout.rs
│   └── modals.rs
├── domains/                # SMART DOMAINS: DDD boundaries
│   ├── chat/
│   │   ├── mod.rs
│   │   ├── screen.rs       # The Chat page (smart component)
│   │   ├── state.rs        # Global use_signal definitions for chat
│   │   ├── repo.rs         # SQLite queries for chat
│   │   └── widgets/        # Components specific ONLY to chat (e.g., MessageBubble)
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── client.rs       # Base HTTP client / Traits
│   │   ├── providers.rs    # OpenAI, Local implementations
│   │   └── models.rs       # Prompt/Response structs
│   └── settings/
│       ├── screen.rs
│       └── store.rs
└── utils/                  # Generic helpers (formatting, crypto)
```

**Rules of Engagement:**
1. **Dumb UI vs Smart Domains**: Files in `src/components/` must be pure functions/components. They accept props and emit events. They **cannot** access global state, database, or network.
2. **Domain Boundaries**: `src/domains/` owns everything else. A domain can import from `src/components/`, but a component can never import from `src/domains/`.
3. **Cross-Domain Comms**: If `domains/chat/screen.rs` needs to trigger an LLM inference, it calls a function in `domains/llm/client.rs`.

# Trade-offs

| Dimension        | Assessment |
|------------------|------------|
| Complexity       | Low (Very intuitive for both frontend and backend devs) |
| Time to implement| Fast |
| Reversibility    | Medium (Couples UI and Domain logic inside the domain folder) |
| Risk level       | Low |
| Scalability      | High (Shared UI scales, backend logic scales via domains) |
| Maintainability  | High |

# Consequences

## Positive Outcomes
- **Perfect Balance**: You get the reusability of a modern frontend framework (centralized `components/`) without losing the backend clarity of DDD (isolated `domains/`).
- **No Boilerplate**: You avoid writing empty interface traits just to pass data between an arbitrary "Core" and "UI" layer. You just call the database from the domain's `repo.rs` directly.

## Risks & Failure Modes
- **Fat Domains**: The `chat` domain might eventually become massive, housing thousands of lines of UI, state, and database logic.
- **State Sprawl**: Because state lives inside domains, passing a `GlobalSignal` from `settings` to `chat` requires careful attention to Dioxus context initialization order in `app.rs`.

## Second-Order Effects
- As the application grows, you may naturally evolve `domains/llm` into an entirely separate workspace crate because it contains zero Dioxus UI code and is purely backend logic, which this structure perfectly sets you up to do.

# Verdict
This solution is best for users who prioritize immediate productivity and frontend reusability while laying the exact groundwork needed for future workspace separation.
