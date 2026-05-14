# Problem
The Lumina application requires a way to permanently delete all chat history. This action must be a "hard delete," meaning records are removed from the underlying SQLite database and the user interface is updated immediately to reflect an empty chat state. To prevent accidental data loss, a confirmation dialog is mandatory before the operation executes.

# User Context & Persona
- **Persona:** A Python developer transitioning to Rust.
- **Background:** Comfortable with Python's `threading`, `multiprocessing`, and `queue` modules.
- **Challenge:** New to Rust's strict ownership model, `async/await` runtime (Tokio), and thread-safety guarantees (`Send`, `Sync`, `Arc`, `Mutex`).
- **Goal:** Understand how to build a robust, non-blocking UI that can scale to handle many background tasks without "freezing" the application.

# External Research
To design this solution, the following technical patterns were researched:
1.  **Dioxus `use_coroutine` for Centralized State:**
    - The `use_coroutine` hook is ideal for creating a "brain" for a component. It runs a long-lived background task that processes messages from an `UnboundedReceiver`.
    - Source: [Dioxus Documentation / Community Patterns](https://docs.rs/dioxus/latest/dioxus/prelude/fn.use_coroutine.html)
2.  **Actor Model in Rust UI:**
    - Using message-passing (via `coroutine.send()`) decouples the UI from business logic. This is similar to Python's `queue.Queue` pattern where the main thread UI pushes tasks to a worker thread.
    - Source: [Vertex AI Search Grounding - Dioxus use_coroutine pattern](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEWxHG86LTKGPyaHz0UPuumZwzdO5XyqFF7kbcoSxTl24p4KmkjnLcJVZpwK25Jiwuz0Xi9iXo2oLMi8k05TlKqjMe3oIDHFWQCRpG66XH0MyeDXFjnpMGEzplNBt9TqC1EYNTeH77L-Qkea068d7DeEuBEjxWNVBFd-CsD)
3.  **Multi-threaded SQLite Access:**
    - For scaling, SQLite performs best with **Write-Ahead Logging (WAL)** mode enabled, allowing simultaneous readers and a single writer. While `Arc<Mutex<Connection>>` works for simple cases, it serializes all access.
    - Source: [Reddit / StackOverflow - SQLite Rust Best Practices](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQG5YxnDvMvtm4zTMHTi3ajdOyVCl83TEOw-jk5XOUy8OSJ4rOfCXa6ABlhAo4GcvQRUpaWujHRT9FLt-whGqWUJ5SFyCmGd6hMqiUSjc-hc1D6y-NM4_SF3EX_eC9-7BVyU4dR2nzu-e6IAR7OhbEq8v_U9Zu6_leNqNpXlTUOCFroyz0S38jlE1V8GVotDVQV3SXvD)

# Solution Summary: The Event-Driven Actor
This solution treats the chat functionality as an **Actor**. The UI is the "Sender," and a background coroutine is the "Receiver." 

Instead of writing logic directly in the "Clear" button, we send a `ClearChat` message to the `chat_service`. The service handles the database deletion and then tells the UI to reset its state. This mirrors how a Python developer might use a `worker_thread` with a `Queue` to keep a GUI (like Tkinter or PyQt) responsive.

# Assumptions
- The application uses `rusqlite` for database access.
- The `use_coroutine` hook is available and already managing chat messages.
- `Arc<Mutex<Connection>>` is the current method for sharing the database handle.

# Detailed Implementation

### 1. Update the Actor's Protocol (Enum)
In `src/domains/chat/screen.rs`, we add the `ClearChat` action to our message types.

```rust
pub enum ChatAction {
    SendMessage(String),
    ClearChat, // New: The instruction to wipe history
}
```

### 2. Implement Database Hard Delete
In `src/domains/chat/repo.rs`, we add the function to wipe the table. We use `spawn_blocking` because database operations are synchronous and would otherwise block the async event loop (similar to how `time.sleep()` blocks in Python's `asyncio`).

```rust
pub async fn clear_messages(conn: Arc<Mutex<Connection>>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM messages", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}
```

### 3. Handle the Action in the Coroutine
The "Actor" (coroutine) now knows how to handle the `ClearChat` message.

```rust
let chat_service = use_coroutine(move |mut rx: UnboundedReceiver<ChatAction>| {
    let conn = coroutine_conn.clone();
    async move {
        while let Some(action) = rx.next().await {
            match action {
                ChatAction::SendMessage(text) => { /* existing logic */ }
                ChatAction::ClearChat => {
                    info!("Clearing all messages...");
                    // 1. Wipe from DB
                    if let Err(e) = clear_messages(conn.clone()).await {
                        error!("Failed to clear DB: {}", e);
                        error_msg.set(Some(format!("Database error: {}", e)));
                    } else {
                        // 2. Wipe from UI state (Signal)
                        messages.set(Vec::new());
                    }
                }
            }
        }
    }
});
```

### 4. UI with Confirmation Logic
In the `ChatScreen` component, we add a local state for the confirmation dialog and the button itself.

```rust
// Local state for the "Are you sure?" dialog
let mut show_confirm = use_signal(|| false);

// Inside rsx!
footer { class: "p-6 bg-white ...",
    div { class: "max-w-4xl mx-auto flex flex-col gap-3",
        // The confirmation overlay/dialog
        if *show_confirm.read() {
            div { class: "bg-red-50 p-4 rounded-xl border border-red-200 flex justify-between items-center mb-2 animate-in fade-in slide-in-from-bottom-2",
                span { class: "text-red-700 text-sm font-medium", "Permanently delete all messages?" }
                div { class: "flex gap-2",
                    Button { 
                        class: "bg-red-600 hover:bg-red-700 !py-1 !px-4 text-xs",
                        onclick: move |_| {
                            chat_service.send(ChatAction::ClearChat);
                            show_confirm.set(false);
                        },
                        "Yes, Clear"
                    }
                    Button { 
                        class: "bg-slate-200 !text-slate-700 hover:bg-slate-300 !py-1 !px-4 text-xs",
                        onclick: move |_| show_confirm.set(false),
                        "Cancel"
                    }
                }
            }
        }

        div { class: "flex gap-3",
            // Clear Button (Triggers confirmation)
            Button {
                class: "bg-slate-100 !text-slate-500 hover:bg-red-50 hover:!text-red-600 !px-3 shadow-none",
                onclick: move |_| show_confirm.toggle(),
                "🗑️"
            }
            Input { /* ... */ }
            Button { /* Send Button ... */ }
        }
    }
}
```

# Trade-offs
- **Complexity:** This adds more "moving parts" (Enums, message matching, separate functions) compared to just putting the logic inside a button's `onclick`.
- **Latency:** Because the message goes through a queue, there is a micro-delay (nanoseconds) before the action starts.
- **Indirectness:** To see what a button does, you have to look at where the message is defined and where it is handled.

# Consequences
- **Thread Safety:** Rust ensures that the `messages` signal and the `conn` handle are accessed safely across threads. If you tried to do something unsafe, the compiler would stop you—this is the biggest difference from Python where you'd only find out via a `RuntimeError` or a crash.
- **Scalability:** If you later add a "Cloud Sync" or "Export to PDF" feature, you just add a new `ChatAction`. The UI stays simple, and the coroutine handles the heavy lifting.
- **Hard Delete:** Messages are gone forever. No "Undo" button is possible without significant architectural changes (like a Trash table).

# Verdict
This is the **"Pro" way** to handle state in Dioxus. While it feels like more boilerplate initially, it prevents the UI from becoming a "spaghetti" of logic. It teaches the fundamental Rust concept of **message passing** (Communicating by sharing data, rather than sharing data by communicating), which is the key to mastering high-performance Rust applications.
