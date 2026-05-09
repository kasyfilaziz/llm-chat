# Problem

Lumina requires a strategic roadmap. As an alternative to resource-heavy Electron apps, the project must be meticulously planned. The user, an experienced Backend/Data Engineer, needs phases and explicit success criteria to guide development systematically from start to finish.

# User Profile

| Attribute        | Value |
|------------------|-------|
| Technical Level  | Expert (4 yrs Web Backend, 5 yrs Sr Data Engineer) |
| Role / Domain    | Lead Developer / Architect |
| Output Style     | Concise, direct, architectural but pragmatic |

> Note: All language, depth, and framing in this document are adapted to the profile above.

# Assumptions

1. The developer's strongest domain is backend architecture and data engineering.
2. A solid data model is the foundation of any scalable application; UI is merely a projection of that data.
3. Complex LLM features (RAG, vector search, parallel API calls) will eventually be required, necessitating a bulletproof backend core.

# Solution Summary

The Data-First Roadmap (Back-to-Front). This strategy builds the core engine (SQLite, HTTP clients, business logic) entirely in pure Rust first, validated by unit tests. The Dioxus UI is treated as a thin, secondary layer added only after the engine is complete.

# Detailed Implementation

**Phase 1: Core Engine & Data Persistence**
* **Goal**: Design the SQLite schema (conversations, messages, settings). Build the repository traits and rusqlite implementations.
* **Success Criteria**: 
  1. 100% unit test coverage on the repository layer.
  2. Database migrations run flawlessly from an empty state to the latest schema.

**Phase 2: LLM Network Layer**
* **Goal**: Build the HTTP clients to interact with external/local LLM providers. Handle token streaming, retries, and context window management (truncation logic).
* **Success Criteria**:
  1. Integration tests successfully send a prompt and receive a full streaming response.
  2. Token calculation logic correctly truncates chat history to fit within a provider's max context limit.

**Phase 3: Dioxus UI Binding**
* **Goal**: Build the UI and wire it directly to the battle-tested core engine.
* **Success Criteria**:
  1. UI correctly reflects the state of the database.
  2. User can send a message, and the UI reacts to the streaming response from the engine.

**Phase 4: Optimization & Release**
* **Goal**: Audit resource usage and finalize OS-specific features.
* **Success Criteria**: Binary size is minimized, and idle CPU usage is < 1%.

# Trade-offs

| Dimension        | Assessment |
|------------------|------------|
| Complexity       | Low (Standard backend engineering flow) |
| Time to implement| Fast (Unit tests are faster to write than UI debugging) |
| Reversibility    | Low (Once the DB is locked, changing it for UI reasons is hard) |
| Risk level       | Medium (Defers UI integration risks to the end) |
| Scalability      | High (The backend will be incredibly solid) |
| Maintainability  | High |

# Consequences

## Positive Outcomes
- **Mathematical Certainty**: You know exactly how your data flows, how fast your queries are, and how API errors are handled before a single pixel is drawn.
- **Leverages Strengths**: Plays directly into your 9 years of data/backend experience, ensuring immediate high-velocity progress.

## Risks & Failure Modes
- **The "Thin UI" Fallacy**: You might discover in Phase 3 that Dioxus requires data in a fundamentally different shape than your pure-Rust core provides, leading to heavy, inefficient mapping functions in the UI layer.
- **UI Performance Ignored**: By deferring the UI to Phase 3, you might find out too late that your data access patterns cause Dioxus to re-render the entire DOM tree, causing lag.

## Second-Order Effects
- This approach naturally leads to a strictly separated "Clean Architecture," making it very easy to eventually extract the core engine into its own crate and perhaps build a CLI client alongside the GUI.

# Verdict
This solution is best for users who prioritize data integrity and backend robustness over rapid UI prototyping.
