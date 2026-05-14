# Problem
The Lumina application currently lacks a way for users to clear their chat history. Messages persist indefinitely in the SQLite database and the UI, which can lead to cluttered conversations and privacy concerns. Users need a reliable "Clear Chat" feature that permanently removes all messages from both the interface and the local database.

# User Context & Persona
The user is a Python developer transitioning to Rust. They are comfortable with high-level logic but find Rust's low-level concepts (ownership, borrowing, async) challenging. Their goal is to build a functional side project while gradually absorbing Rust idioms. 

This solution focuses on **Simplicity** and **Readability**, avoiding complex abstractions or new dependencies, and explaining the "why" behind the Rust-specific code used.

# External Research
To develop this solution, I researched the following areas:

- **Dioxus 0.7 Event Handling & Signals**: Dioxus 0.7 uses a signal-based state management system. Events are handled via closures that "capture" signals using the `move` keyword. 
    - *Source*: [Dioxus 0.7 Event Handling Guide](https://docs.rs/dioxus/latest/dioxus/events/index.html)
- **Rust rusqlite DELETE Queries**: Best practices include using parameterized queries to prevent SQL injection and using `spawn_blocking` when interacting with SQLite from an async context to avoid blocking the main UI thread.
    - *Source*: [rusqlite Documentation](https://docs.rs/rusqlite/latest/rusqlite/)
- **Dioxus Desktop Dialog Patterns**: While OS-native dialogs (via crates like `rfd`) are common, the most "minimalist" approach within the framework is to use state-controlled UI components (modals) to manage confirmation flows.
    - *Source*: [Dioxus Community Patterns & Samples](https://github.com/DioxusLabs/dioxus)

# Solution Summary
**The Native Minimalist (Quick & Simple)**

This solution implements a "Clear Chat" button directly in the footer, next to the "Send" button. It uses a Dioxus `signal` to toggle a simple, styled confirmation overlay. When confirmed, a message is sent to the existing chat coroutine, which handles the database deletion and state update. It is "Minimalist" because it leverages existing project patterns and requires zero new library dependencies.

# Assumptions
- The application is running in a Dioxus Desktop environment.
- The SQLite database is already initialized with a `messages` table.
- The project follows a domain-driven structure (Chat domain).

# Detailed Implementation

### 1. Database Repository (`src/domains/chat/repo.rs`)
First, we add a function to delete all messages from the database.

```rust
pub async fn clear_messages(conn: Arc<Mutex<Connection>>) -> Result<(), String> {
    // We use spawn_blocking because database operations are "slow" (synchronous).
    // If we ran this on the main thread, the app would freeze for a split second!
    tokio::task::spawn_blocking(move || {
        // .lock() ensures we are the only ones talking to the database right now.
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM messages", []).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}
```
**Concept Check for Python Devs:**
- **Arc (Atomic Reference Counted)**: Think of this as a shared library card. Many parts of your code can "hold" it to access the database.
- **Mutex (Mutual Exclusion)**: This is like a lock on a diary. Only one person can write in it at a time to prevent data corruption.
- **spawn_blocking**: This is like sending a heavy task to a background worker so your main app stays responsive.

### 2. Chat Action (`src/domains/chat/screen.rs`)
Update the `ChatAction` enum to include a clear command.

```rust
pub enum ChatAction {
    SendMessage(String),
    ClearChat, // New action!
}
```

### 3. Coroutine Handling (`src/domains/chat/screen.rs`)
Update the `chat_service` to respond to the `ClearChat` action.

```rust
// Inside ChatScreen coroutine while loop:
match action {
    ChatAction::ClearChat => {
        if let Ok(_) = crate::domains::chat::repo::clear_messages(conn.clone()).await {
            messages.set(Vec::new()); // Clear the UI state
        }
    }
    // ... SendMessage logic ...
}
```

### 4. UI Layout & Confirmation (`src/domains/chat/screen.rs`)
We add a `show_confirm` signal and the "Clear" button.

```rust
let mut show_confirm = use_signal(|| false);

// In the footer, next to the Button:
Button {
    class: "bg-red-500 hover:bg-red-600",
    onclick: move |_| show_confirm.set(true),
    "Clear"
}

// At the end of the ChatScreen rsx!, add the confirmation modal:
if *show_confirm.read() {
    div { class: "fixed inset-0 bg-slate-900/50 flex items-center justify-center z-50",
        div { class: "bg-white p-6 rounded-2xl shadow-xl max-w-sm mx-4",
            h2 { class: "text-lg font-bold mb-2", "Clear all messages?" }
            p { class: "text-slate-600 mb-6", "This action cannot be undone. All chat history will be permanently deleted." }
            div { class: "flex gap-3 justify-end",
                button { 
                    class: "px-4 py-2 text-slate-500 hover:bg-slate-100 rounded-lg transition-colors",
                    onclick: move |_| show_confirm.set(false),
                    "Cancel"
                }
                button { 
                    class: "px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors shadow-sm",
                    onclick: move |_| {
                        chat_service.send(ChatAction::ClearChat);
                        show_confirm.set(false);
                    },
                    "Clear Chat"
                }
            }
        }
    }
}
```

# Trade-offs
- **Pros**:
    - **No Bloat**: No new crates added to `Cargo.toml`.
    - **Easy to Understand**: Uses standard Dioxus signals and `rsx!` which are fundamental to the framework.
    - **Fast Development**: Minimal changes to the existing architecture.
- **Cons**:
    - **UI Only**: The "modal" is just a div; it doesn't use a separate window, so it won't persist if the app is navigated (though Lumina is currently a single screen).
    - **Repetition**: The styling for the modal is inline (Tailwind) rather than a reusable component.

# Consequences
- **Codebase**: Slight increase in complexity in `ChatScreen` due to the added signal and modal logic.
- **User Experience**: Immediate, high-contrast feedback for a destructive action.
- **Learning**: The user learns how to handle destructive state changes across the UI and Database layers in a single flow.

# Verdict
Ideal for the "Tracer Bullet" phase. It achieves the requirement with the least friction and provides a clear learning path for Dioxus state management.
