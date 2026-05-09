# Problem

The user needs to implement Phase 1 of Lumina: a Dioxus UI, SQLite database, and OpenAI-compatible API streaming integration. The architecture must be clean, maintainable, and prevent the UI thread from blocking, while adhering to the previously established Pragmatic Hybrid DDD folder structure.

# User Profile

| Attribute        | Value |
|------------------|-------|
| Technical Level  | Expert (4 yrs Web Backend, 5 yrs Sr Data Engineer) |
| Role / Domain    | Lead Developer / Architect |
| Output Style     | Concise, direct, architectural but pragmatic |

> Note: All language, depth, and framing in this document are adapted to the profile above.

# Assumptions

1. The developer wants enterprise-grade separation of concerns without the boilerplate of Clean Architecture.
2. Dioxus 0.7 `use_coroutine` is stable and ideal for handling long-lived asynchronous event streams.
3. We must handle streaming tokens seamlessly without thrashing the database or blocking the UI.

# Solution Summary

The Actor Model (Event-Driven via Coroutines). The application is split into three independent actors: the UI, the Database, and the LLM Network Client. They communicate exclusively by passing strongly-typed messages (`UserMessage`, `StreamChunk`, `StreamFinished`) through a centralized Dioxus `use_coroutine` channel.

# Detailed Implementation

**Step 1: Define the Event Enum**
```rust
pub enum ChatEvent {
    SendUserMessage(String),
    ReceiveToken(String),
    StreamCompleted(String), // Full message for final DB save
    Error(String),
}
```

**Step 2: The Orchestrator (Dioxus Coroutine)**
In your `chat/screen.rs`, start a `use_coroutine`. This acts as the brain.
```rust
let chat_service = use_coroutine(|mut rx: UnboundedReceiver<ChatEvent>| async move {
    while let Some(event) = rx.next().await {
        match event {
            ChatEvent::SendUserMessage(text) => {
                // 1. Fire and forget DB save (tokio::spawn_blocking)
                // 2. Trigger HTTP LLM request
                // 3. As HTTP stream yields tokens, push ChatEvent::ReceiveToken to the UI signal
            },
            // ... handle other events
        }
    }
});
```

**Step 3: The UI Layer**
The Dioxus UI is completely dumb. It holds a `use_signal<Vec<Message>>`. 
When the user clicks "Send", it calls `chat_service.send(ChatEvent::SendUserMessage(text))`.
When the coroutine receives a `ChatEvent::ReceiveToken`, it mutates the `use_signal` to update the screen.

**Step 4: The Database Layer**
The database is only written to twice per message cycle: once when the User sends a message, and once when the `StreamCompleted` event fires containing the *entire* assistant response.

# Trade-offs

| Dimension        | Assessment |
|------------------|------------|
| Complexity       | Medium/High (Requires understanding async channels) |
| Time to implement| Medium |
| Reversibility    | High |
| Risk level       | Low |
| Scalability      | Very High (Easy to add new event types later) |
| Maintainability  | Very High |

# Consequences

## Positive Outcomes
- **Perfect Smoothness**: The UI is updated via in-memory signals (super fast), while the database is only written to twice per cycle (no I/O thrashing).
- **Strict Decoupling**: The HTTP client has no idea Dioxus exists. It just yields strings. The Coroutine handles the glue.

## Risks & Failure Modes
- **Partial Data Loss**: Like Solution A, if the app crashes mid-stream, the partial assistant response is not saved to SQLite, because we wait for `StreamCompleted`. (For a chat app, this is usually acceptable).
- **Async Lifetimes**: Passing database connection pools and HTTP clients into a long-lived async coroutine can occasionally cause Rust borrow checker / `Send + Sync` headaches.

## Second-Order Effects
- This sets an incredible foundation for Phase 2 (MCP Integration). When you add tools, you just add `ChatEvent::ToolCallRequested` to the enum, and the orchestrator handles it cleanly without touching the UI.

# Verdict
This solution is best for users who prioritize a highly scalable, event-driven architecture that elegantly balances UI responsiveness with clean backend separation.
