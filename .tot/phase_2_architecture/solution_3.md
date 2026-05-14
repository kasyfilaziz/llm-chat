# Problem
The user wants to implement Phase 2 of their project, introducing three new core domains: Conversation History (persisted in SQLite), Settings (persisted in YAML), and a Model Context Protocol (MCP) Foundation (acting as a host to connect to existing local MCP servers). The architecture needs to be production-ready and integrate cleanly with the existing "Tracer Bullet" Phase 1 code built on Rust, Dioxus 0.7, SQLite, and Tailwind CSS.

# User Context & Persona
- **Goal**: Wants a production-ready architectural design for Phase 2 that handles three new domains while integrating cleanly with the existing code.
- **Storage Strategy**: Session/chat history stored in SQLite; Settings stored in YAML.
- **MCP Foundation Goal**: Lumina connects to existing local MCP servers (host-side implementation).
- **Quality Standard**: Production-ready architecture.
- **Current Stack**: Rust, Dioxus 0.7, SQLite, Tailwind CSS.

# External Research
- **Queried:** "Rust Actor Model" "Dioxus 0.7" use_coroutine channels event-driven UI
- **Found:** In Dioxus 0.7, the Actor Model is natively supported using the `use_coroutine` hook, which uses a built-in `futures_channel::mpsc` channel. This allows components to "fire and forget" messages (enums) to an async loop that lives for the component's lifetime. It is ideal for complex asynchronous logic, state management, and event-driven updates.
- **Queried:** "Model Context Protocol" "Rust" host implementation actor model
- **Found:** The official Rust SDK for MCP (`rmcp`) and community crates like `mcpr` provide client implementations. An actor-based host implementation is recommended because it isolates failures, manages state efficiently (JSON-RPC handshakes, tool executions), and handles concurrency across multiple servers without complex locking. A host orchestrator actor manages connections handled by individual server actors.

# Solution Summary
This approach adopts a fully decoupled, actor-orchestrated system where Chat, Settings, and MCP domains are modeled as isolated background tasks communicating via typed event channels (using Dioxus `use_coroutine` or dedicated `tokio` channels). This event-driven architecture ensures that heavy asynchronous workloads (like an MCP tool execution or database query) never block the UI thread, treating each domain as a pluggable, independent micro-service within the application.

# Assumptions
- The application will rely heavily on Dioxus 0.7's `use_coroutine` and `tokio`'s mpsc channels for cross-component and background communication.
- MCP tool execution might be long-running, demanding strict isolation from the UI thread to prevent freezing.
- Settings changes must be globally reactive, requiring a pub/sub or broadcast mechanism to update UI components instantly.
- The existing SQLite connection can be safely shared or managed by a dedicated database actor to prevent locking contention.
- The `rmcp` or similar crate will be used for standardizing the MCP JSON-RPC protocol over stdio or SSE.

# Detailed Implementation
1. **Define Domain Actors (Coroutines / Tokio Tasks):**
   - **Chat Actor:** Manages the active conversation state. It listens for `ChatEvent` messages (e.g., `SendMessage`, `LoadHistory`). It communicates with the SQLite database to persist messages and interfaces with the LLM.
   - **Settings Actor:** Listens for `SettingsEvent` messages (e.g., `UpdateTheme`, `SetMCPPath`). It reads/writes to the YAML file and broadcasts state updates to global Dioxus `Signal`s so the UI reacts immediately.
   - **MCP Host Actor (Orchestrator):** Manages the lifecycle of connected MCP servers. It listens for `McpEvent` messages (e.g., `ConnectServer`, `CallTool`, `ListTools`). It spawns child actors for each individual MCP server connection (handling stdio pipes).

2. **Establish the Event Bus:**
   - Define strict, strongly-typed Rust `enum`s representing actions for each domain (e.g., `ChatAction`, `SettingsAction`, `McpAction`).
   - Use Dioxus's `use_coroutine` at the root of the application (or relevant contexts) to initialize these actors.
   - The `use_coroutine` hook returns a `Coroutine<T>` handle containing an `UnboundedSender`, which can be accessed from any child component via `use_coroutine_handle::<T>()`.

3. **UI Integration (Fire and Forget):**
   - UI components are completely stateless regarding business logic. They read from shared `Signal`s (updated by the actors) to render the view.
   - User interactions (button clicks, form submissions) strictly send messages down the channels using the coroutine handles (e.g., `chat_actor.send(ChatAction::SendMessage(text))`).

4. **Handling the MCP Foundation:**
   - When the user adds an MCP server via Settings, a message is dispatched to the MCP Host Actor.
   - The MCP Host Actor spawns a standard `tokio` task (representing the specific server connection), establishing a stdio transport using an MCP SDK (like `rmcp`).
   - The server connection performs the handshake, discovers capabilities, and registers available tools into the global state.
   - When the LLM requests a tool call, the Chat Actor dispatches a message to the MCP Host Actor, which routes it to the specific Server Actor. The result is channeled back asynchronously.

5. **Database and File I/O Management:**
   - SQLite queries and YAML file writes are performed exclusively within the respective actor's async loops. This inherently serializes operations and prevents concurrency bugs without the need for extensive `Mutex` or `RwLock` usage.

# Trade-offs
| Dimension          | Assessment                        |
|--------------------|-----------------------------------|
| Complexity         | High                              |
| Time to implement  | 1 - 2 weeks                       |
| Reversibility      | Hard                              |
| Risk level         | Medium                            |
| Scalability        | Excellent (handles many MCP servers and concurrent operations effortlessly) |
| Maintainability    | High (clear separation of concerns, testable domains) |
| Fit for user persona | High (Provides the highest quality, production-ready architecture required for a complex local application) |

# Consequences
## Positive Outcomes
- **Ultimate Responsiveness:** The UI will never freeze, regardless of how long an LLM generation or MCP tool execution takes.
- **Robustness:** A crash or panic in a specific MCP server connection won't bring down the entire application or the main UI thread.
- **Testability:** Domain logic can be tested in complete isolation from the Dioxus UI by simply sending messages to the actors and asserting on the resulting state/outputs.

## Risks & Failure Modes
- **Message Tracing:** Debugging can become difficult if events cascade (e.g., a Settings event triggers a Chat event which triggers an MCP event). Tracing the flow of messages requires disciplined logging.
- **State Desync:** If an actor fails silently or drops a message, the UI might be left waiting in a loading state indefinitely because the "success" message was never sent back to update the `Signal`.
- **Boilerplate:** Requires writing extensive `enum` definitions and pattern matching for every possible action.

## Second-Order Effects
- **Extensibility:** Adding a new domain in Phase 3 (e.g., Cloud Sync) becomes trivial. You simply define a new `SyncActor` and its event enum without touching existing UI or Chat logic.
- **Performance:** Minimizes lock contention on shared resources (like the database connection pool) since ownership is centralized within specific actors.

# Verdict
This solution is best for users who prioritize rock-solid stability, non-blocking performance, and clean separation of concerns, and have the Rust experience to manage asynchronous event buses and actor lifecycles.
