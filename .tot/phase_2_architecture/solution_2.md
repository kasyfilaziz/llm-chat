# Problem
The user wants to implement Phase 2 of the project, which requires designing the architecture for three new domains: Conversation History, Settings, and an MCP (Model Context Protocol) Foundation. This phase must integrate cleanly with the existing "Tracer Bullet" code (Rust, Dioxus 0.7, SQLite, Tailwind CSS) while adhering to a production-ready standard, especially handling the unpredictability of local third-party MCP servers.

# User Context & Persona
- **Goal:** Wants a production-ready, stable architecture for Phase 2.
- **Storage Strategy:** SQLite for session/chat history, YAML for application settings.
- **MCP Foundation:** Lumina acts as a host connecting to local MCP servers.
- **Current Stack:** Rust, Dioxus 0.7, SQLite, Tailwind CSS.

# External Research
- **Dioxus 0.7 Global State:** Research shows a shift from older context-based shared state to the new `GlobalSignal` pattern (`static GLOBAL: GlobalSignal<T> = Global::new(|| ...)`) for app-wide state (like Settings). For scoped state, Context APIs with `Signal` are preferred. Complex collections benefit from custom Stores to minimize re-renders.
- **Rust Circuit Breakers:** Best practices for distributed/external connections in Rust involve using state machines (Closed, Open, Half-Open). Crates like `tower-resilience` (if using tower services) or `failsafe-rs`/`circuitbreaker-rs` are standard. It's critical to only trip breakers on actual failures (timeouts, internal errors) and not on valid client rejections (4xx).
- **MCP Host in Rust:** Connecting to local MCP servers typically involves `stdio` transports. Because these servers are separate processes controlled by third parties, they are prone to crashes or hangs, making the circuit breaker pattern an essential production requirement.

# Solution Summary
Solution 2 introduces the "Resilient Middleware" Architecture, prioritizing stability and fault tolerance. It leverages an `AppStore` using Dioxus 0.7 `GlobalSignal`s to sync reactive UI state with the YAML backend, and wraps the `mcp-host` integration in isolated circuit breakers to prevent third-party server crashes from locking up the application UI.

# Assumptions
- The application is running in a desktop/local environment where file I/O (SQLite/YAML) is synchronous or fast enough to not block the main event loop significantly.
- Local MCP servers will communicate over `stdio`.
- Third-party MCP servers are inherently untrusted in terms of performance and uptime; the host must protect itself.
- Dioxus 0.7 `use_coroutine` or background tasks can be effectively used to offload file writing.

# Detailed Implementation

1. **State Management (`AppStore`)**
   - Implement an `AppStore` module to manage application Settings.
   - Use Dioxus 0.7's `GlobalSignal` to hold the in-memory settings: `static SETTINGS: GlobalSignal<AppSettings> = Global::new(|| AppSettings::default());`.
   - On startup, the application reads the YAML file, parses it via `serde_yaml`, and initializes the `GlobalSignal`.
   - Implement a background coroutine (`use_coroutine`) that listens for specific update events. When the user modifies a setting via the UI, the signal updates instantly for reactivity, and an event is sent to the coroutine to persist the change back to the YAML file, preventing UI stutter during disk I/O.

2. **Conversation History (SQLite)**
   - Extend the Phase 1 `db.rs` module with robust schemas for `sessions` and `messages`.
   - Use Dioxus `use_resource` to reactively load history data. When the active session ID changes, the resource automatically re-fetches the corresponding messages without manual orchestration.

3. **MCP Foundation (Resilient Middleware)**
   - Create an `McpClient` module responsible for managing subprocesses (the local MCP servers via `stdio`).
   - **Circuit Breaker Integration:** Wrap each connected MCP server instance with a circuit breaker (e.g., using `failsafe-rs` or a custom implementation leveraging `parking_lot::RwLock`).
   - The breaker maintains `Closed`, `Open`, and `HalfOpen` states. 
   - If an MCP tool call hangs or returns a critical error (e.g., process exited), the failure is recorded. Upon reaching a failure threshold, the circuit trips to `Open`.
   - When `Open`, any LLM attempts to call tools on that server are immediately short-circuited with a local error, and the UI can reflect this degraded state (e.g., greying out the tool).
   - After a cooldown period, the circuit moves to `HalfOpen`, allowing a test request through to see if the server has recovered or restarted.

4. **Integration with Phase 1**
   - The LLM provider logic from Phase 1 is updated to request the list of available tools from the `McpClient`.
   - The UI components simply subscribe to the `GlobalSignal` for settings and the `use_resource` for history, keeping the render logic completely decoupled from the data fetching and persistence layers.

# Trade-offs
| Dimension          | Assessment                        |
|--------------------|-----------------------------------|
| Complexity         | Medium / High                     |
| Time to implement  | 2 - 3 weeks                       |
| Reversibility      | Hard (Core middleware pattern)    |
| Risk level         | Low                               |
| Scalability        | High (Handles many MCP servers gracefully) |
| Maintainability    | High (Clean separation of concerns) |
| Fit for user persona| High (Meets production/stability standards) |

# Consequences

## Positive Outcomes
- **Extreme UI Resilience:** A hanging or crashed third-party MCP server will never freeze the chat interface or crash the main app.
- **Snappy UX:** Optimistic UI updates on Settings with asynchronous YAML writes provide a highly responsive user experience.
- **Clear Separation:** The middleware pattern isolates the nasty reality of IPC and subprocess management from the clean Dioxus reactive UI components.

## Risks & Failure Modes
- **State De-sync:** If the background YAML write fails (e.g., due to file permissions), the in-memory `GlobalSignal` will be out of sync with disk, requiring error-handling callbacks to revert the UI state.
- **Circuit Flapping:** Poorly tuned circuit breaker thresholds (too few errors allowed, or too short a cooldown) could lead to tools rapidly toggling between available and unavailable, frustrating the user and the LLM.

## Second-Order Effects
- By establishing a robust error-handling middleware early, future extensions (like connecting to remote MCP servers or adding complex local agent loops) will be much easier to integrate safely.

# Verdict
This solution is best for users who prioritize rock-solid application stability and have the technical context to manage complex, decoupled state synchronization and middleware patterns.