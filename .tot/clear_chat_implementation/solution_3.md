# Problem
Implement a highly responsive "Clear Chat" feature using an optimistic UI update and background database purge via the actor model (`use_coroutine`).

# User Context & Persona
- **Developer Profile:** Interested in Dioxus actor-model and async integration.
- **Goal:** Verify that destructive actions can be handled safely in the background without blocking the UI.

# External Research
- **Dioxus Actor Model:** `use_coroutine` provides a sequential mailbox. Sending a `Clear` message ensures it doesn't race with `SendMessage` or `SaveMessage`.
- **Optimistic UI:** Updating the `messages` signal immediately provides "instant" feedback even if the DB operation takes time.
- **Error Handling:** Background tasks must handle their own errors or report back via signals.

# Solution Summary
This solution leverages the existing `chat_service` (Coroutine). We add a `ClearChat` variant to `ChatAction`. When received, the UI is cleared immediately, and the coroutine handles the DB deletion sequentially, preventing any race conditions with ongoing message saves.

# Assumptions
- Users value UI responsiveness.
- The `chat_service` is the primary actor for chat logic.

# Detailed Implementation

### 1. `src/domains/chat/screen.rs`
Update the `ChatAction` enum and the coroutine loop:

```rust
pub enum ChatAction {
    SendMessage(String),
    ClearChat, // New action
}

// Inside ChatScreen's use_coroutine:
while let Some(action) = rx.next().await {
    match action {
        ChatAction::ClearChat => {
            // 1. Optimistic UI update (optional, can be done in button handler)
            // 2. Background DB Purge
            let _ = clear_all_messages(conn.clone()).await;
            info!("Database cleared in background");
        }
        ChatAction::SendMessage(text) => { ... }
    }
}

// Button in UI:
Button {
    onclick: move |_| {
        messages.set(Vec::new()); // Optimistic clear
        chat_service.send(ChatAction::ClearChat);
    },
    "Clear Chat"
}
```

### 2. `src/domains/chat/repo.rs`
Use the same `clear_all_messages` function as Solution 1.

# Trade-offs (Table)
| Aspect | Strength | Weakness |
| :--- | :--- | :--- |
| **Responsiveness** | Highest (Instant UI reset). | Possible out-of-sync state if DB fail. |
| **Architecture** | Best (Actor model usage). | More complex event flow. |
| **Safety** | High (Sequential processing). | Requires robust error reporting. |

# Consequences
- **Positive:** Verifies the Actor model for non-LLM tasks. Great UX.
- **Risks:** If a `SendMessage` is in flight while `ClearChat` is sent, we need to ensure the DB state remains consistent.
- **Second-Order:** Sets a pattern for all async/destructive side effects.

# Verdict
Recommended to fully exercise the Dioxus actor model and provide a "snappy" feel.
