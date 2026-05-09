# Problem

The user needs a directory structure for Lumina, a local-first Rust/Dioxus LLM chat interface. The system is expected to scale heavily in UI complexity and external integrations (LLM providers). The user desires a Domain-Driven Design (DDD) approach within a single crate that keeps the structure clean but capable of sustaining enterprise-level complexity.

# User Profile

| Attribute        | Value |
|------------------|-------|
| Technical Level  | Expert (4 yrs Web Backend, 5 yrs Sr Data Engineer) |
| Role / Domain    | Lead Developer / Architect |
| Output Style     | Concise, direct directory trees with strict architectural rules |

> Note: All language, depth, and framing in this document are adapted to the profile above.

# Assumptions

1. The data layer (SQLite) and external API layer (HTTP clients) represent infrastructure details that change at different rates than the business logic.
2. Dioxus UI components change frequently and independently of the core business logic.
3. Keeping strict boundaries between UI, Domain, and Infrastructure inside a single crate is preferred over mixing concerns.

# Solution Summary

A Clean Layered Architecture that splits the repository strictly by technical boundary (`ui`, `core`, `infrastructure`) at the root. Within `core`, code is organized by business domain.

# Detailed Implementation

This approach ensures the UI cannot directly query the database, and the core domain has no knowledge of Dioxus or HTTP.

```text
src/
├── main.rs                 # Bootstraps the app, dependency injection setup
├── ui/                     # Layer: Presentation (Dioxus)
│   ├── components/         # Shared dumb UI (Button, Modal)
│   ├── views/              # Smart pages (ChatView, SettingsView)
│   └── state/              # Dioxus specific signals (use_signal)
├── core/                   # Layer: Business Logic (Pure Rust)
│   ├── chat/               # DOMAIN: Chat
│   │   ├── entities.rs     # Structs: Message, Conversation
│   │   └── use_cases.rs    # Core logic: process_incoming_message()
│   ├── llm/                # DOMAIN: LLM
│   │   ├── entities.rs     # Structs: Prompt, Response
│   │   └── ports.rs        # Traits: LlmProvider, ChatRepository
│   └── settings/
└── infrastructure/         # Layer: External Integrations
    ├── sqlite/             # Implements traits from core::chat::ports
    │   ├── schema.rs
    │   └── chat_repo.rs
    └── providers/          # Implements core::llm::ports::LlmProvider
        ├── openai.rs
        └── anthropic.rs
```

**Rules of Engagement:**
1. **Dependency Rule**: `ui` and `infrastructure` depend on `core`. `core` depends on nothing.
2. **Ports and Adapters**: If `core::chat` needs to save a message, it calls an interface (Trait) defined in `core::chat::ports`. `infrastructure::sqlite` implements this trait.
3. **No Dioxus in Core**: The `core` module cannot import `dioxus::prelude::*`. It uses pure standard Rust types. 

# Trade-offs

| Dimension        | Assessment |
|------------------|------------|
| Complexity       | High (Requires strict discipline and boilerplate traits) |
| Time to implement| Slower initially due to mapping between layers |
| Reversibility    | Easy to rip out the UI or DB entirely |
| Risk level       | Low (Very safe, highly testable) |
| Scalability      | Very High (Handles massive scale and team sizes perfectly) |
| Maintainability  | High (Separation of concerns is absolute) |

# Consequences

## Positive Outcomes
- **Ultimate Testability**: You can write `core` unit tests with mocked LLM providers and in-memory databases seamlessly.
- **Technology Agnosticism**: If you ever switch from Dioxus to Tauri/Leptos, or SQLite to Postgres, your `core` directory remains 100% untouched.

## Risks & Failure Modes
- **Boilerplate Fatigue**: Adding a simple feature (e.g., displaying the token count) requires touching `infrastructure` (to fetch it), `core` (to model it), and `ui` (to display it).
- **Over-engineering**: For a single-developer personal project, creating interfaces and mappers for every database call can feel like massive overkill, potentially slowing down momentum.

## Second-Order Effects
- As Dioxus evolves, you might find that keeping UI state strictly separated from Domain logic causes friction, as modern reactive frameworks blur the line between "application state" and "UI state" heavily via signals.

# Verdict
This solution is best for users who prioritize long-term architectural purity, testability, and strict separation of concerns over rapid prototyping.
