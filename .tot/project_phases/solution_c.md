# Problem

Lumina requires a development roadmap. The goal is to build a lightweight, native-feeling LLM client. The user needs defined phases and success criteria. Because this is a greenfield project utilizing an evolving UI framework (Dioxus 0.7) paired with system-level Rust, the integration points are the highest source of risk.

# User Profile

| Attribute        | Value |
|------------------|-------|
| Technical Level  | Expert (4 yrs Web Backend, 5 yrs Sr Data Engineer) |
| Role / Domain    | Lead Developer / Architect |
| Output Style     | Concise, direct, architectural but pragmatic |

> Note: All language, depth, and framing in this document are adapted to the profile above.

# Assumptions

1. The hardest part of building cross-platform Rust apps is not the database or the UI in isolation, but how they talk to each other without blocking the main thread.
2. Momentum is critical for personal projects; seeing a working, end-to-end feature early sustains motivation.
3. The "Hybrid DDD" folder structure we agreed upon favors building feature-by-feature rather than layer-by-layer.

# Solution Summary

The Tracer Bullet Roadmap (Vertical Slices). Instead of building layers (All UI, or All Database), we build complete features end-to-end. Phase 1 focuses on building the absolute minimum viable path from a UI text input, through the database, to an LLM, and back to the screen.

# Detailed Implementation

**Phase 1: The Core Loop (Tracer Bullet)**
* **Goal**: Build an ugly but functional chat interface that takes user input, saves it to SQLite, sends it to a mocked (or simple real) LLM endpoint, and displays the response.
* **Success Criteria**:
  1. A user can type a message and hit enter.
  2. The message is persisted to `lumina.db` instantly.
  3. The UI updates asynchronously with a response without the application freezing (proving async Rust works harmoniously with Dioxus coroutines).

**Phase 2: The Domain Expansion**
* **Goal**: Flesh out the domains. Add Conversation History (sidebar), Settings (API keys), and multiple LLM provider support.
* **Success Criteria**:
  1. User can switch between past conversations; UI hydrates from SQLite instantly.
  2. User can securely store an OpenAI API key and switch models seamlessly.

**Phase 3: The UX & Performance Polish**
* **Goal**: Implement streaming tokens (server-sent events), markdown rendering, and strict memory profiling.
* **Success Criteria**:
  1. Markdown and code blocks render perfectly in the UI.
  2. Application idle RAM usage is verified to be significantly lower than a comparable Electron app (e.g., < 100MB).

**Phase 4: Distribution**
* **Goal**: Package for Windows, Linux, and macOS.
* **Success Criteria**: Automated GitHub Actions produce downloadable, runnable artifacts.

# Trade-offs

| Dimension        | Assessment |
|------------------|------------|
| Complexity       | Medium (Requires context switching between UI, DB, and Network constantly) |
| Time to implement| Fast (Gets to a working MVP immediately) |
| Reversibility    | Medium (Refactoring an end-to-end slice is harder than refactoring a single layer) |
| Risk level       | Lowest (Identifies integration blockers immediately) |
| Scalability      | High |
| Maintainability  | High |

# Consequences

## Positive Outcomes
- **Immediate Integration Validation**: You prove in Phase 1 that Dioxus `use_coroutine` can successfully handle async database writes and HTTP requests without locking the UI thread.
- **Continuous Motivation**: At the end of every phase, you have a fully working, usable application, just with varying levels of features.

## Risks & Failure Modes
- **Technical Debt**: Because Phase 1 focuses on connecting the pipes quickly, you might write messy, tightly coupled code just to get the "Tracer Bullet" working, requiring a refactor in Phase 2.
- **Context Whiplash**: As a developer, you have to constantly switch mental contexts from CSS styling to SQLite SQL syntax to async HTTP handling, all in the same afternoon.

## Second-Order Effects
- This roadmap aligns perfectly with the "Pragmatic Hybrid DDD" folder structure we chose. By building Phase 1, you will naturally populate `domains/chat` completely, rather than leaving empty files waiting for future phases.

# Verdict
This solution is best for users who prioritize rapid end-to-end validation, continuous momentum, and early mitigation of integration risks.
