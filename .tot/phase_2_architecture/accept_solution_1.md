# Problem
The user wants to continue the development of Lumina into Phase 2, designing the architecture for three new components: Conversation History (SQLite), Settings (YAML), and the Model Context Protocol (MCP) Foundation (host-side implementation). The design needs to be production-ready, cleanly integrate with the existing "Tracer Bullet" Phase 1 code, and leverage Rust, Dioxus 0.7, SQLite, and Tailwind CSS.

# User Context & Persona
- **Goal**: Establish a production-ready architectural design for Phase 2 that handles the three new domains without compromising the existing UI-Aware Domain-Driven Design (DDD) principles.
- **Storage Strategy**: Session/chat history in SQLite; Settings in YAML.
- **MCP Foundation Goal**: Local MCP server connection via host-side implementation.
- **Current Stack**: Rust, Dioxus 0.7, SQLite (`rusqlite`), Tailwind CSS.
- **Expectation**: A balanced, idiomatic approach that avoids monolithic codebases, keeps Dioxus UI fast, and maintains local-first and cross-platform consistency.

# External Research
1. **Dioxus 0.7 State Management**:
   - Best practices indicate using `use_signal` for local UI state and `use_context` (Context API) for deeply nested shared state. For complex collections (like a list of conversations), the new `#[derive(Store)]` API is highly recommended to provide fine-grained reactivity and prevent full-component re-renders. Global state (like application Settings) can be elegantly handled using `GlobalSignal`.
2. **`rust-mcp-sdk` (`rmcp`) Host Implementation**:
   - Best practices for local tools (like Claude Desktop implementations) dictate using `StdioTransport` wrapping a `tokio::process::Child` for secure, efficient local process communication.
   - It's critical to derive JSON schemas using `schemars` for type-safe tool invocations and rigorously check `ServerCapabilities` during initialization.
3. **Rust `serde_yaml` Persistence**:
   - `serde_yaml` combined with `serde`'s derive macros (`Serialize`, `Deserialize`) is the standard approach for mapping YAML directly to Rust structs.
   - **Anti-pattern / Risk**: The `serde_yaml` crate is officially unmaintained. Production environments are increasingly migrating to `serde_yml` (a maintained fork) to ensure long-term stability and security updates. It is a best practice to handle file I/O operations asynchronously using `tokio::fs` to avoid blocking the main or UI threads.

# Solution Summary
The "Unified Domain" Extension expands Lumina's existing Pragmatic Hybrid DDD structure. It adds `history`, `settings`, and `mcp` into the `domains/` folder, preserving the separation of dumb UI components and smart business logic. Conversation history integrates naturally with the existing `chat` domain via `rusqlite`; Settings are serialized to YAML (using `serde_yml` as a robust alternative) and exposed via `GlobalSignal`; and MCP is implemented via `rmcp` using an async `StdioTransport` background actor.

# Assumptions
- The database schema for `lumina.db` can be gracefully updated to include a `conversations` table using basic SQL scripts, as no heavy migration framework (e.g., SQLx migrations) is strictly required at this scale.
- The use of `serde_yaml` can be seamlessly substituted with the maintained `serde_yml` fork without breaking user expectations, mitigating the unmaintained crate risk.
- Local MCP servers are provided as external binaries (e.g., via Node/npx or Python) and do not need to be packaged inside the Lumina binary.
- Dioxus coroutines (`use_coroutine`) will be used to encapsulate all blocking or async I/O (Database, File System, MCP Stdio), preventing UI thread starvation.

# Detailed Implementation

### 1. Conversation History (SQLite + Dioxus Store)
- **Database (`domains/chat/repo.rs`)**: Introduce a `conversations` table. Add functions `get_all_conversations(conn)` and `create_conversation(conn)`. Ensure all `rusqlite` calls are wrapped in `tokio::task::spawn_blocking` to avoid blocking the async runtime.
- **State (`domains/chat/state.rs`)**: Implement a Dioxus `Store` (e.g., `#[derive(Store)] struct ConversationStore { list: Vec<Conversation> }`) to hold the sidebar history. This provides fine-grained reactivity when a specific conversation is renamed or deleted.
- **UI (`domains/chat/screen.rs`)**: Create a sidebar component that reads from the `ConversationStore` and dispatches selection events to the main chat window.

### 2. Settings Domain (YAML + GlobalSignal)
- **Structure**: Create a new `domains/settings` module.
- **Persistence (`domains/settings/repo.rs`)**: Define `#[derive(Serialize, Deserialize)] pub struct AppSettings`. Implement asynchronous `load` and `save` methods using `tokio::fs` and `serde_yml` to read/write from `~/.config/lumina/settings.yaml` (or OS equivalent via the `directories` crate).
- **State (`domains/settings/state.rs`)**: Expose the settings globally using `GlobalSignal<AppSettings>`. This allows any domain (like `llm/client.rs` needing API keys) to instantly read the user's config without prop drilling.

### 3. MCP Foundation (rmcp + Actor Coroutine)
- **Structure**: Create a new `domains/mcp` module.
- **Connection (`domains/mcp/client.rs`)**: Use `rmcp`'s `TokioChildProcess` to wrap `StdioTransport`. Spawn the local MCP server (e.g., `npx -y @modelcontextprotocol/server-everything`) using `tokio::process::Command`. Initialize the client and negotiate capabilities.
- **State & Integration (`domains/mcp/state.rs`)**: Use an Event-Driven Actor model. Launch a Dioxus `use_coroutine` in the global `app.rs` layout that holds the active `McpClient`. The UI or Chat domain sends `McpEvent::CallTool { name, args }` to this coroutine via an `UnboundedSender`, and the coroutine executes the tool asynchronously, avoiding UI freezes during long tool executions.

# Trade-offs
| Dimension          | Assessment                        |
|--------------------|-----------------------------------|
| Complexity         | Medium                            |
| Time to implement  | 1-2 weeks                         |
| Reversibility      | Easy                              |
| Risk level         | Low                               |
| Scalability        | High for single-user local growth |
| Maintainability    | High (Strict DDD boundaries)      |
| Fit for user persona | High (Idiomatic, Tracer-aligned)|

# Consequences
## Positive Outcomes
- **Clean Separation of Concerns**: Dumb UI components remain pure, while complex business logic (YAML parsing, MCP process lifecycle) is strictly confined to its specific domain.
- **Performance**: By utilizing Dioxus Stores, Global Signals, and async I/O, the UI will maintain 60fps even when loading large conversation histories or saving settings.

## Risks & Failure Modes
- **Orphaned MCP Processes**: If Lumina crashes or panics without gracefully dropping the `TokioChildProcess`, background MCP servers might leak and continue running on the host OS.
- **Dependency Stagnation**: Relying directly on `serde_yaml` exposes the app to unpatched security vulnerabilities. (Mitigated by switching to `serde_yml`).
- **File I/O Blocking**: If YAML serialization is mistakenly performed on the main UI thread synchronously, the app will stutter during configuration saves.

## Second-Order Effects
- The introduction of a global settings signal means any domain can react to setting changes in real-time. E.g., changing the theme or LLM provider in the Settings UI immediately updates the active Chat screen without requiring a restart.
- The MCP foundation acts as a precursor to Phase 3/4. By routing tools through an asynchronous coroutine early, future additions of complex tools (like codebase search or web fetching) will not require architectural rewrites.

# Verdict
This solution is best for users who prioritize a balanced, strictly idiomatic extension of the existing Domain-Driven architecture and have the capability to diligently manage async file I/O and process lifecycles.