# Problem

The user is building Lumina, a local-first, cross-platform LLM chat interface using Rust and Dioxus. The application needs a directory structure that adheres to Domain-Driven Design (DDD) to handle expected rapid growth in specific areas (LLM provider integrations, UI components, state management). However, the structure must reside entirely within a single Rust repository crate (no workspace crates yet) and should prioritize straightforward navigation without over-engineering.

# User Profile

| Attribute        | Value |
|------------------|-------|
| Technical Level  | Expert (4 yrs Web Backend, 5 yrs Sr Data Engineer) |
| Role / Domain    | Lead Developer / Architect |
| Output Style     | Concise, direct directory trees with strict architectural rules |

> Note: All language, depth, and framing in this document are adapted to the profile above.

# Assumptions

1. The project will remain a single `lumina` cargo crate for the foreseeable future.
2. Dioxus is used for both the UI components and the reactive state (`use_signal`, `use_context`).
3. Database logic (rusqlite) is synchronous or lightly wrapped, avoiding extreme async abstracting.
4. Fast growth means we expect 10+ LLM providers and complex UI states soon.

# Solution Summary

A Pure Vertical Slice architecture where the root `src/` directory represents distinct feature domains. Each domain (e.g., `chat`, `llm`, `settings`) is a completely self-contained module owning its own UI components, state management, and database/infrastructure interactions.

# Detailed Implementation

In this approach, you organize code strictly by **business capability**, not technical layer. 

```text
src/
├── main.rs                 # App entry point, mounts the Dioxus root
├── app.rs                  # Root component, global router
├── db/                     # Global infrastructure: connection pool setup
│   ├── mod.rs
│   └── migrations.rs
├── chat/                   # DOMAIN: Chat
│   ├── mod.rs
│   ├── components/         # Dioxus elements strictly for chat (MessageBubble, ChatInput)
│   ├── screen.rs           # Main Chat view component
│   ├── state.rs            # Dioxus Signals or global state related to chat
│   └── repo.rs             # SQLite queries specific to chat history
├── llm/                    # DOMAIN: LLM Integrations
│   ├── mod.rs
│   ├── providers/          # OpenAI, Anthropic, Local (Ollama)
│   ├── client.rs           # Trait definitions for LLM interactions
│   └── state.rs            # State tracking active model, API keys
└── settings/               # DOMAIN: Settings
    ├── mod.rs
    ├── screen.rs
    └── repo.rs             # SQLite queries for user preferences
```

**Rules of Engagement:**
1. **Intra-domain coupling is high, inter-domain coupling is low.** `chat/screen.rs` can freely use `chat/state.rs` and `chat/repo.rs`. 
2. **Strict import direction.** If `chat` needs to trigger an LLM request, it depends on public interfaces exposed by the `llm` module. The `llm` module should know nothing about `chat`.
3. **No global "components" folder.** If a UI component is only used by Chat, it lives in `chat/components/`. If a component (like a generic Button) must be shared, it either gets duplicated (acceptable in pure vertical slices) or pushed to a generic `shared/` domain.

# Trade-offs

| Dimension        | Assessment |
|------------------|------------|
| Complexity       | Low conceptually, but forces early decisions on where UI belongs |
| Time to implement| Very fast for new features (just copy a domain folder) |
| Reversibility    | Medium (Extracting shared UI later can be tedious) |
| Risk level       | Low |
| Scalability      | High (Domains scale independently without merge conflicts) |
| Maintainability  | High (Everything about a feature is in one folder) |

# Consequences

## Positive Outcomes
- **Extreme Feature Isolation**: When building a new LLM provider, you only touch `src/llm/`. You don't have to bounce between `src/components/`, `src/services/`, and `src/models/`.
- **Merge Conflict Reduction**: If you hire or collaborate, developers can own entire slices without stepping on each other's toes.

## Risks & Failure Modes
- **UI Component Fragmentation**: Since Dioxus is UI-heavy, you might find yourself needing a "Modal" or "Button" in both `chat` and `settings`. Pure vertical slices resist shared UI, leading to code duplication or a messy `shared/` folder that eventually becomes a dumping ground.
- **State Leaks**: If `chat` needs to read a value from `settings` (e.g., "dark mode"), you must carefully expose and consume Dioxus context across domain boundaries, which can cause reactive cycle issues if not architected cleanly.

## Second-Order Effects
- As the project grows, the `shared/` directory will inevitably appear. If not aggressively pruned, it will morph into a monolithic layer, destroying the vertical slice paradigm.

# Verdict
This solution is best for users who prioritize feature velocity and cognitive locality (having all code for a feature in one folder) over strict UI consistency and reusability.
