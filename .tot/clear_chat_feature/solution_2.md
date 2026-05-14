# Problem
The user needs a way to permanently delete chat history from the Lumina application. This requires both a database operation (SQLite hard delete) and a UI update to reflect the empty state. Crucially, because this is a destructive action, a confirmation step is required to prevent accidental data loss.

# User Context & Persona
The user is a **Python developer learning Rust**. 
- **Familiarity**: Comfortable with imperative logic and dynamic typing, but new to Rust's strict ownership, borrowing rules, and explicit async/await patterns.
- **Mental Model**: Likely expects state changes to be as simple as assigning a value to a variable (e.g., `self.show_modal = True`).
- **Goal**: Wants to understand how "Lumina" (the app) handles UI state and component communication in an idiomatic Rust/Dioxus way.

# External Research
To ensure this implementation follows Dioxus 0.7 standards, the following research was conducted:
- **Dioxus 0.7 Signal System**: Verified that `use_signal` is the standard for reactive state. Signals are `Copy` and automatically track dependencies, simplifying the ownership mental model for Python developers. (Source: `DIOXUS_0.7_REPORT.md`)
- **Conditional Rendering in RSX**: Confirmed that Dioxus 0.7 supports native Rust `if` statements directly inside the `rsx!` macro, making it much more intuitive than older "macro-heavy" versions. (Source: Google Search / Dioxus Docs)
- **Modal Patterns**: Identified that the "Portal" pattern or simple conditional rendering with a high z-index backdrop is the standard for modals in Dioxus. (Source: Google Search)

# Solution Summary
The **Component Architect** approach focuses on building a reusable, native Dioxus `ConfirmModal` component. This solution teaches:
1. **State Orchestration**: Using `Signals` to manage the visibility of the modal.
2. **Prop Drilling & Callbacks**: How a child component (the Modal) communicates back to the parent (the Chat Screen) using `EventHandler`.
3. **Conditional Rendering**: Using standard Rust `if` logic to mount/unmount UI elements.
4. **Hard Delete Logic**: Interacting with the database layer to ensure permanent deletion.

# Assumptions
- The application uses Tailwind CSS for styling (as evidenced in `src/domains/chat/screen.rs`).
- The database connection is available via Dioxus context (as seen in `ChatScreen`).
- We are using Dioxus 0.7 shorthand attributes and signal syntax.

# Detailed Implementation

## 1. The Reactivity Concept: Signals vs. Python Variables
In Python, if you change a variable `show_modal = True`, the UI doesn't know it needs to redraw. You would typically need a framework like Flask or FastAPI to handle the request/response, or a UI library to manually trigger an update.

In Dioxus, we use **Signals**:
```rust
let mut show_confirm = use_signal(|| false);
```
Think of a Signal as a "smart box." 
- When you "read" it (`show_confirm()`), Dioxus takes a note: "This part of the UI depends on this box."
- When you "write" it (`show_confirm.set(true)`), Dioxus says: "The box changed! I need to re-run the code that depends on it."

## 2. The Custom Modal Component
We'll create a generic confirmation modal. In Rust, we define "Props" (properties) as a struct to tell Dioxus what data the component needs.

```rust
// src/components/mod.rs (Append this)

#[derive(Props, Clone, PartialEq)]
pub struct ConfirmModalProps {
    title: String,
    message: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
}

#[component]
pub fn ConfirmModal(props: ConfirmModalProps) -> Element {
    rsx! {
        // Backdrop: Fixed overlay covering the whole screen
        div { 
            class: "fixed inset-0 bg-slate-900/50 backdrop-blur-sm z-50 flex items-center justify-center p-4",
            onclick: move |_| props.on_cancel.call(()), // Close if clicking background

            // Modal Box
            div { 
                class: "bg-white rounded-2xl shadow-xl max-w-sm w-full p-6 space-y-4",
                onclick: |evt| evt.stop_propagation(), // Don't close when clicking inside

                h2 { class: "text-xl font-bold text-slate-800", "{props.title}" }
                p { class: "text-slate-600", "{props.message}" }

                div { class: "flex gap-3 justify-end",
                    button { 
                        class: "px-4 py-2 text-slate-500 hover:bg-slate-100 rounded-lg transition-colors",
                        onclick: move |_| props.on_cancel.call(()), 
                        "Cancel" 
                    }
                    button { 
                        class: "px-4 py-2 bg-red-500 hover:bg-red-600 text-white rounded-lg transition-colors font-semibold",
                        onclick: move |_| props.on_confirm.call(()), 
                        "Clear Everything" 
                    }
                }
            }
        }
    }
}
```

## 3. Database Layer Update
We add a hard delete function to the repository. Note the use of `spawn_blocking`—this is how we tell Rust "this task might take a while (like talking to a disk), don't freeze the UI while waiting."

```rust
// src/domains/chat/repo.rs

pub async fn clear_messages(conn: Arc<Mutex<Connection>>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM messages", []).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}
```

## 4. Integrating into ChatScreen
We add a new signal `show_confirm` and the "Clear" button.

```rust
// src/domains/chat/screen.rs

#[component]
pub fn ChatScreen() -> Element {
    let mut messages = use_signal(Vec::<Message>::new);
    let mut show_confirm = use_signal(|| false); // New state!
    let conn = use_context::<Arc<Mutex<Connection>>>();

    // ... (existing logic)

    let handle_clear = move |_| {
        let conn = conn.clone();
        spawn(async move {
            if let Ok(_) = clear_messages(conn).await {
                messages.set(Vec::new()); // Clear UI
                show_confirm.set(false); // Close modal
            }
        });
    };

    rsx! {
        div { class: "flex flex-col h-screen ...",
            
            // ... (Header)

            // 1. Conditional Rendering of Modal
            if show_confirm() {
                ConfirmModal {
                    title: "Clear Chat History?",
                    message: "This will permanently delete all messages from this conversation. This action cannot be undone.",
                    on_confirm: handle_clear,
                    on_cancel: move |_| show_confirm.set(false),
                }
            }

            // ... (Chat History)

            footer { class: "...",
                div { class: "max-w-4xl mx-auto flex gap-3",
                    // New "Clear" Button
                    button {
                        class: "p-2 text-slate-400 hover:text-red-500 transition-colors",
                        onclick: move |_| show_confirm.set(true),
                        title: "Clear Chat",
                        // Trash icon SVG
                        svg { width: "20", height: "20", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                            path { d: "M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" }
                        }
                    }

                    Input { ... }
                    Button { ... }
                }
            }
        }
    }
}
```

# Trade-offs
### Strengths
- **Type Safety**: Using `EventHandler<()>` ensures that the modal can only signal back in ways we expect.
- **Declarative UI**: The `if show_confirm() { ... }` block makes the UI logic very easy to follow—no manual "show/hide" DOM manipulation.
- **Explicit Communication**: Props and Events clearly define how components talk to each other, preventing the "spaghetti code" common in large Python scripts.

### Weaknesses
- **Boilerplate**: Compared to Python, defining a `Props` struct and wrapping logic in `spawn(async move { ... })` feels verbose.
- **Ownership Hurdles**: The Python developer might find `move |_|` and `.clone()` confusing. We have to clone the `conn` (database connection) because Rust needs to know exactly which part of the code "owns" that connection when it moves into an async block.

# Consequences
- **Learning Curve**: The user will be forced to confront `async` and `Signals` immediately. While harder at first, it builds the correct foundation for Rust development.
- **Architecture**: Encourages a "Single Source of Truth" where the `messages` signal is the only place the UI looks for data.
- **Visual Feedback**: The modal provides a high-quality, professional user experience that matches the "Lumina" aesthetic.

# Verdict
This solution is ideal for a learner because it demonstrates the "Dioxus Way" of handling state and components. It moves beyond simple scripts and introduces architectural patterns (Componentization, Reactive State, and Async Safety) that are essential for mastering Rust.
