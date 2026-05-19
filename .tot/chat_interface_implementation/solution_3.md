# Problem

The Lumina chat application needs a new Chat Interface UI defined by the HTML template at `assets/stich/chat_interface_4/code.html`. The existing `src/domains/chat/screen.rs` (298 lines) is a monolith that mixes two concerns in one component: (1) a `use_coroutine` containing all streaming/AI logic, DB operations, LLM client construction, and auto-titling, and (2) the RSX rendering of the UI. Currently the UI is a minimal Tailwind layout with a sidebar, message bubbles, and a simple input — it does not match the target template's Terra theme (dark "surface" color palette), its rich message bubbles (user avatar, AI icon with markdown), hover action bars, typing indicator, prompt suggestion chips, attachment button, model selector dropdown, or fixed gradient input area.

"Solved" means: a `screen.rs` that renders the full Terra-themed UI from the template, while all streaming/coroutine/DB logic lives in a separate `service.rs` that the screen imports as a dependency. The service layer must be testable independent of Dioxus, and the screen must be a "dumb" presentation component.

---

# User Context & Persona

- **Sole developer**, Python background learning Rust
- Wants **idiomatic Rust** (map, and_then, iterators, closures, etc.)
- The project uses **Dioxus 0.7** with signals, `use_coroutine`, and `UnboundedReceiver`-based actor pattern
- Existing chat implementation is a baseline to refactor — "clean slate, forget old UI"
- Chat function must work first; hover actions, typing indicator, suggestions are at implementor's discretion
- Has explicit preference for domain-driven design (smart domain, dumb UI)
- Learning curve: needs explanations for ownership, `move`, closures, `Result`/`Option` idioms, and trait objects

---

# External Research

The following searches validated architectural patterns and Rust-specific techniques used in this solution:

1. **Dioxus 0.7 custom hooks** — Dioxus docs show custom hooks created by composing built-in hooks via `use_hook`, enabling extraction of logic from components. Source: https://dioxuslabs.com/learn/0.7/essentials/advanced/custom_hooks/

2. **`use_coroutine` documentation** — The official docs.rs page details how `use_coroutine` provides an `UnboundedReceiver`-based channel system and is well-suited for centralizing async event loops. Source: https://docs.rs/dioxus-hooks/latest/dioxus_hooks/fn.use_coroutine.html

3. **Service Layer pattern** — Martin Fowler's P of EAA defines the Service Layer as "an application's boundary with a layer of services that establishes a set of available operations." Source: https://martinfowler.com/eaaCatalog/serviceLayer.html

4. **Rust module organization** — The Rust Book chapter 7 covers packages, crates, and modules, and dev.to articles show idiomatic patterns of service/domain/infrastructure layers in Rust projects. Source: https://dev.to/sgchris/how-to-structure-a-rust-project-idiomatically-500k

5. **thiserror vs anyhow** — Community consensus (2024-2026): use `thiserror` for library/service types (custom, matchable error enums), use `anyhow` for application-level error propagation. Source: https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view

6. **Rust async traits (1.75+)** — Native `async fn` in traits stabilised in Rust 1.75, enabling clean service trait definitions without the `async-trait` macro. Source: https://medium.com/@ashusk_1790/rust-async-traits-what-finally-works-now-7b7f46529718

7. **Streams as async iterators** — The Rust Book chapter 17 clarifies that `Stream` is the async equivalent of `Iterator`, crucial for understanding `process_stream` in providers.rs. Source: https://doc.rust-lang.org/book/ch17-04-streams.html

---

# Solution Summary

**Logic Extraction + Clean Rebuild (Structural).** Extract the entire `use_coroutine` closure body (streaming, DB I/O, LLM client construction, auto-titling) out of `screen.rs` into a standalone `ChatService` struct in a new `service.rs` module. Then rewrite `screen.rs` purely as RSX markup that mirrors the HTML template — the screen only calls `ChatService::send_message()` and `ChatService::clear_chat()` via a coroutine handle, never touching streams, DB connections, or LLM clients directly. The service module exposes a clean `ChatAction`/`ChatEvent` protocol and a `ChatService::new()` factory that accepts its dependencies (DB connection, settings reader) explicitly.

---

# Assumptions

| # | Assumption | Risk if Wrong |
|---|-----------|---------------|
| 1 | The existing `process_stream` function in `providers.rs` and `LlmClient::new` remain unchanged | If those signatures change, the service module needs updates too |
| 2 | The `Arc<Mutex<Connection>>` pattern for SQLite is acceptable long-term | If migrated to connection pool, the service constructor must change |
| 3 | Settings are read from a global static `SETTINGS` during coroutine execution | If settings become reactive signals, the coroutine needs signal access |
| 4 | The user keeps Dioxus 0.7 with the current `use_coroutine` API | Upgrading Dioxus may change the coroutine hook signature |
| 5 | The template's Material Symbols icons can be rendered as text/emoji fallbacks or CSS classes | If icons must be actual SVG/icon fonts, HTML element rendering differs |
| 6 | The sidebar widget from the existing code is still wanted in the new UI | If sidebar redesign is expected, that's a separate widget change |
| 7 | The `conversation_id` migration in `repo.rs` already handled the `messages` table schema | If not, loading messages by conversation_id will fail |
| 8 | The user has Tailwind CSS classes available in the Dioxus asset pipeline | If not, the template classes will not apply and the UI will look unstyled |

---

# Detailed Implementation

## Step 1: Create `src/domains/chat/service.rs`

This is the core structural change. Define a **Rust struct** (not a Dioxus component) that owns all chat business logic. It accepts dependencies via constructor injection.

### 1a. Define a custom error type with `thiserror`

```rust
// service.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatServiceError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("LLM request failed: {0}")]
    Llm(String),
    #[error("Stream error: {0}")]
    Stream(String),
}

// This lets us convert existing String-based errors cheaply:
impl From<String> for ChatServiceError {
    fn from(s: String) -> Self {
        ChatServiceError::Llm(s)
    }
}
```

**Why thiserror?** It derives `Display`, `Debug`, and `Error` with one annotation — much less boilerplate than manual impls. The `#[error("...")]` attribute generates `Display` for each variant. The `#[derive(Error)]` is a procedural macro that implements `std::error::Error`.

### 1b. Define the service interface

```rust
// Re-export the actions so screen.rs can use them
#[derive(Debug)]
pub enum ChatAction {
    SendMessage(String),
    ClearChat,
}

pub struct ChatService {
    pub conn: Arc<Mutex<Connection>>,
}
```

**Note:** `ChatAction` moves here from `screen.rs`. The `ChatService` struct is plain Rust — no Dioxus traits, no signals. This makes it unit-testable: you can instantiate it in a test with a mock DB and call methods.

### 1c. Extract the streaming/LLM logic into a method

Take the inner body of the `ChatAction::SendMessage` match arm from the old coroutine and make it a standalone `async fn`:

```rust
impl ChatService {
    pub async fn send_message(
        &self,
        text: String,
        messages: &mut Vec<Message>,
        store: &mut ConversationStore,
    ) -> Result<Option<String>, ChatServiceError> {
        // 1. Read settings (same as before: from global static)
        let settings = crate::domains::settings::state::SETTINGS.read().clone();
        let provider_type = ProviderType::from(settings.provider_preferences.default_llm.as_str());

        let (url, key) = match provider_type {
            ProviderType::Ollama => {
                let base_url = settings.provider_preferences.ollama_endpoint.trim_end_matches('/');
                (format!("{base_url}/api/chat"), String::new())
            }
            ProviderType::OpenAI => (
                "https://api.openai.com/v1/chat/completions".into(),
                settings.provider_preferences.openai_api_key.clone(),
            ),
        };

        let model = if matches!(provider_type, ProviderType::Ollama) {
            "gemma4:31b-cloud".into()
        } else {
            "gpt-4-turbo".into()
        };

        let client = LlmClient::new(provider_type, url, key, model);

        // 2. Create/get conversation (inserted into store by caller)
        let is_new = store.active_id.is_none();
        let conv_id = if let Some(ref id) = store.active_id {
            id.clone()
        } else {
            let conv = Conversation::new("New Chat");
            let cid = conv.id.clone();
            create_conversation(self.conn.clone(), conv.clone()).await?;
            store.list.insert(0, conv);
            store.active_id = Some(cid.clone());
            cid
        };

        // 3. Save user message
        let user_msg = Message::new(&conv_id, "user", &text);
        save_message(self.conn.clone(), user_msg.clone()).await?;
        messages.push(user_msg);

        // 4. Build request from existing messages
        let request = ChatRequest {
            model: client.model.clone(),
            messages: messages.iter().map(|m| ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            }).collect(),
            stream: true,
        };

        // 5. Stream response
        let mut assistant_msg = Message::new(&conv_id, "assistant", "");
        let assistant_id = assistant_msg.id.clone();
        messages.push(assistant_msg.clone());

        let mut stream = process_stream(&client, request)
            .await
            .map_err(ChatServiceError::from)?;

        while let Some(chunk) = stream.next().await {
            let token = chunk.map_err(|e| ChatServiceError::Stream(e))?;
            if let Some(msg) = messages.iter_mut().find(|m| m.id == assistant_id) {
                msg.content.push_str(&token);
                assistant_msg.content.push_str(&token);
            }
        }

        // 6. Persist assistant message
        save_message(self.conn.clone(), assistant_msg.clone()).await?;

        // 7. Auto-title if new conversation
        if is_new {
            self.auto_title(&client, &text, &assistant_msg.content, &conv_id, store).await;
        }

        Ok(if is_new { Some(conv_id) } else { None })
    }

    async fn auto_title(
        &self,
        client: &LlmClient,
        user_text: &str,
        assistant_text: &str,
        conv_id: &str,
        store: &mut ConversationStore,
    ) {
        let title_req = ChatRequest {
            model: client.model.clone(),
            messages: vec![
                ChatMessage { role: "system".into(), content: "Generate a very short 3-5 word title for this conversation. Return ONLY the title.".into() },
                ChatMessage { role: "user".into(), content: user_text.into() },
                ChatMessage { role: "assistant".into(), content: assistant_text.into() },
            ],
            stream: false,
        };

        if let Ok(mut title_stream) = process_stream(client, title_req).await {
            let mut new_title = String::new();
            while let Some(Ok(token)) = title_stream.next().await {
                new_title.push_str(&token);
            }
            let trimmed = new_title.replace('"', "").trim().to_string();
            if !trimmed.is_empty() {
                let _ = update_conversation_title(self.conn.clone(), conv_id.into(), trimmed.clone()).await;
                if let Some(c) = store.list.iter_mut().find(|c| c.id == conv_id) {
                    c.title = trimmed;
                }
            }
        }
    }
}
```

**Key Rust concepts explained:**
- `&self`: immutable reference to the service — the service does not own the data (messages/store are passed as `&mut` parameters from the coroutine)
- `Result<_, ChatServiceError>`: the idiomatic Rust error return. `?` operator unwraps `Ok` or propagates `Err` after automatic conversion via `From`
- `.iter().map(|m| ...).collect()`: iterator chain converting `Vec<Message>` into `Vec<ChatMessage>`. `map` takes a closure, `collect` gathers into the target collection
- `clone()` is used pervasively because the compiler needs owned data across `.await` points (the borrow cannot live across a yield)

### 1d. Add `clear_chat` method

```rust
impl ChatService {
    pub async fn clear_chat(
        &self,
        conversation_id: Option<String>,
        messages: &mut Vec<Message>,
    ) -> Result<(), ChatServiceError> {
        if let Some(id) = conversation_id {
            clear_messages(self.conn.clone(), id)
                .await
                .map_err(|e| ChatServiceError::Database(e))?;
            messages.clear();
        }
        Ok(())
    }
}
```

## Step 2: Rewrite `screen.rs` — Pure UI, Thin Logic

The new `screen.rs` imports `ChatService` but the service is never called directly — it communicates via a coroutine channel just like before. The difference: the coroutine body is now a thin dispatcher.

### 2a. New coroutine: thin dispatcher

```rust
// Inside ChatScreen component:
let service = ChatService { conn: conn.clone() };
let chat_tx = use_coroutine(move |mut rx: UnboundedReceiver<ChatAction>| {
    let service = ChatService { conn: conn.clone() }; // fresh for each coroutine restructure...
    // Actually, ChatService is cheap to clone if we make fields Clone+cheap (Arc)
    // Better: wrap conn in Arc, make ChatService hold Arc fields
    // But for simplicity, capture service directly (it only holds Arc<Mutex<Connection>>)
    async move {
        while let Some(action) = rx.next().await {
            match action {
                ChatAction::SendMessage(text) => {
                    let mut msgs = messages.write();
                    let mut store_mut = store.write();
                    if let Err(e) = service.send_message(
                        text,
                        &mut *msgs,
                        &mut *store_mut,
                    ).await {
                        error_msg.set(Some(e.to_string()));
                    }
                }
                ChatAction::ClearChat => {
                    let cid = store.read().active_id.clone();
                    let mut msgs = messages.write();
                    if let Err(e) = service.clear_chat(cid, &mut *msgs).await {
                        error_msg.set(Some(e.to_string()));
                    }
                }
            }
        }
    }
});
```

**Note on `&mut *msgs`:** `messages.write()` returns a `SignalGuard<Vec<Message>>` which implements `DerefMut`. The `*` dereferences through the guard to get `&mut Vec<Message>`, then `&mut` borrows it mutably. This is the idiomatic way to get a mutable reference from a Dioxus signal.

### 2b. RSX from the template

The RSX should mirror the HTML template's structure. Major sections:

1. **Global layout**: `main` with `bg-surface-bright`, `flex-1 flex flex-col h-full`
2. **Header**: hidden on desktop (`md:hidden`), shows menu icon + "Lumina Chat" title
3. **Chat canvas**: scrollable container with `pb-40` to avoid input overlap
4. **Message rendering**: user messages with avatar `img` (circular), AI messages with `bg-primary-container` icon + `Markdown` content
5. **Action bar per AI message**: copy, regenerate, thumbs up — `opacity-0 group-hover:opacity-100`
6. **Typing indicator**: bouncing dots (`flex gap-1`, three `rounded-full` divs with `animate-bounce` and staggered `animation-delay`)
7. **Input area**: fixed bottom with gradient-to-transparent padding
8. **Prompt suggestion chips**: horizontal scrollable row of `rounded-full` buttons
9. **Input box**: textarea + toolbar with attachment button, model selector, send button
10. **Disclaimer**: small text "Lumina may produce inaccurate information..."

```rust
rsx! {
    main { class: "flex-1 flex flex-col h-full relative bg-surface-bright min-w-0",
        // Mobile header
        header { class: "flex md:hidden justify-between items-center px-6 h-16 w-full bg-surface border-b border-outline-variant/40 shrink-0",
            button { class: "text-secondary hover:bg-primary/10 p-2 rounded-full transition-colors",
                span { class: "material-symbols-outlined", "menu" }
            }
            div { class: "font-headline text-xl font-bold text-primary", "Lumina Chat" }
        }

        // Chat canvas
        div { class: "flex-1 overflow-y-auto px-4 md:px-8 py-8 scroll-smooth pb-40",
            div { class: "max-w-3xl mx-auto space-y-10",

                // Intro / session header
                div { class: "text-center mb-12",
                    h2 { class: "font-headline text-3xl font-medium text-on-surface mb-2",
                        "Chat Session"
                    }
                    p { class: "text-secondary text-sm",
                        "Session started just now"
                    }
                }

                // Messages
                for msg in messages.read().iter() {
                    if msg.role == "user" {
                        // User bubble with avatar
                        div { class: "flex gap-4 items-start w-full",
                            div { class: "w-8 h-8 rounded-full overflow-hidden shrink-0 mt-1 border border-outline-variant/30",
                                // Using a placeholder avatar div since we can't load external images reliably
                                div { class: "w-full h-full bg-primary flex items-center justify-center text-on-primary text-xs", "U" }
                            }
                            div { class: "flex-1 min-w-0 pt-1",
                                div { class: "text-on-surface text-lg leading-relaxed font-body whitespace-pre-wrap",
                                    "{msg.content}"
                                }
                            }
                        }
                    } else {
                        // AI bubble with icon
                        div { class: "flex gap-4 items-start w-full group",
                            div { class: "w-8 h-8 rounded-full bg-primary-container text-on-primary-container flex items-center justify-center shrink-0 mt-1",
                                span { class: "material-symbols-outlined text-sm", "eco" }
                            }
                            div { class: "flex-1 min-w-0 pt-1",
                                div { class: "text-on-surface text-lg leading-relaxed font-body",
                                    Markdown { content: msg.content.clone() }
                                }
                                // Action bar (hover)
                                div { class: "flex items-center gap-2 mt-4 opacity-0 group-hover:opacity-100 transition-opacity",
                                    button {
                                        class: "text-secondary hover:text-primary hover:bg-primary/5 p-1.5 rounded-full transition-colors text-xs flex items-center gap-1",
                                        onclick: move |_| {
                                            // Copy to clipboard
                                            let _ = spawn(async move {
                                                let content = msg.content.clone();
                                                // clipboard API not available in desktop Dioxus — use a workaround
                                            });
                                        },
                                        "Copy"
                                    }
                                    button { class: "text-secondary hover:text-primary hover:bg-primary/5 p-1.5 rounded-full transition-colors",
                                        onclick: move |_| chat_tx.send(ChatAction::SendMessage(last_user_text.clone())),
                                        "↻"
                                    }
                                    button { class: "text-secondary hover:text-primary hover:bg-primary/5 p-1.5 rounded-full transition-colors",
                                        "👍"
                                    }
                                }
                            }
                        }
                    }
                }

                // Typing indicator
                if is_typing() {
                    div { class: "flex gap-4 items-start w-full",
                        div { class: "w-8 h-8 rounded-full bg-primary-container text-on-primary-container flex items-center justify-center shrink-0 mt-1",
                            span { class: "material-symbols-outlined text-sm animate-pulse", "eco" }
                        }
                        div { class: "flex-1 min-w-0 pt-1 flex items-center h-8",
                            div { class: "flex gap-1",
                                div { class: "w-2 h-2 rounded-full bg-tertiary/60 animate-bounce" }
                                div { class: "w-2 h-2 rounded-full bg-tertiary/60 animate-bounce", animation_delay: "0.2s" }
                                div { class: "w-2 h-2 rounded-full bg-tertiary/60 animate-bounce", animation_delay: "0.4s" }
                            }
                        }
                    }
                }
            }
        }

        // Fixed input area
        div { class: "absolute bottom-0 left-0 right-0 bg-gradient-to-t from-surface-bright via-surface-bright to-transparent pt-10 pb-6 px-4 md:px-8 z-20",
            div { class: "max-w-3xl mx-auto",
                // Prompt suggestions
                div { class: "flex gap-2 mb-3 overflow-x-auto pb-2",
                    for chip in prompt_chips.read().iter() {
                        button {
                            class: "whitespace-nowrap px-4 py-1.5 rounded-full bg-surface-container hover:bg-surface-container-high border border-outline-variant/30 text-xs text-on-surface-variant transition-colors",
                            onclick: move |_| {
                                input_text.set(chip.clone());
                            },
                            "{chip}"
                        }
                    }
                }
                // Main input
                div { class: "relative bg-surface rounded-2xl border border-outline-variant/50 shadow-sm focus-within:border-primary focus-within:ring-1 focus-within:ring-primary transition-all duration-200",
                    textarea {
                        class: "w-full bg-transparent border-none focus:ring-0 resize-none py-4 pl-4 pr-12 text-on-surface placeholder:text-secondary/60 text-base min-h-[56px] max-h-48 rounded-2xl",
                        placeholder: "Ask Lumina anything...",
                        value: "{input_text}",
                        oninput: move |evt| input_text.set(evt.value()),
                        onkeydown: move |evt| {
                            if evt.key() == Key::Enter && !input_text.read().is_empty() {
                                let text = input_text.read().clone();
                                chat_tx.send(ChatAction::SendMessage(text));
                                input_text.set(String::new());
                            }
                        }
                    }
                    // Toolbar
                    div { class: "flex justify-between items-center px-3 pb-3 pt-1",
                        div { class: "flex items-center gap-1",
                            button { class: "text-secondary hover:text-primary hover:bg-primary/10 p-2 rounded-full transition-colors",
                                "📎" // Attachment placeholder
                            }
                            div { class: "cursor-pointer flex items-center gap-1 text-xs font-medium text-secondary hover:text-primary bg-surface-container-low hover:bg-surface-container px-3 py-1.5 rounded-lg transition-colors border border-outline-variant/20 ml-1",
                                span { "🧠" }
                                span { "Lumina Core" }
                                span { "▾" }
                            }
                        }
                        button {
                            class: "bg-primary text-on-primary hover:bg-primary-container hover:text-on-primary-container p-2 rounded-xl transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                            disabled: input_text.read().is_empty(),
                            onclick: move |_| {
                                if !input_text.read().is_empty() {
                                    let text = input_text.read().clone();
                                    chat_tx.send(ChatAction::SendMessage(text));
                                    input_text.set(String::new());
                                }
                            },
                            "↑"
                        }
                    }
                }
                // Disclaimer
                div { class: "text-center mt-2",
                    span { class: "text-[10px] text-secondary/70",
                        "Lumina may produce inaccurate information. Verify important details."
                    }
                }
            }
        }
    }
}
```

## Step 3: Update `mod.rs`

```rust
// src/domains/chat/mod.rs
pub mod state;
pub mod widgets;
pub mod screen;
pub mod repo;
pub mod service;  // ADD: expose the service module
```

Remove the `ChatAction` from `screen.rs` (it now lives in `service.rs`) and update the import path in the coroutine wiring.

## Step 4: Wire up typing indicator state

Add a signal in `screen.rs`:

```rust
let mut is_typing = use_signal(|| false);
```

Before sending a message, set `is_typing.set(true)`. After the stream completes (done inside the coroutine), set it to `false`. Since the coroutine cannot directly access the signal (it runs in a separate async task), pass a **sender** via a second channel, or use a simpler approach: attach a clone of a `Signal::new_sync()` that is `Send + Sync`.

Dioxus 0.7 provides `use_signal_sync(|| false)` for cross-thread signal access. The coroutine body can hold a cloned `Signal<bool>` and set it:

```rust
let mut is_typing_signal = use_signal_sync(|| false);
// Inside coroutine:
// before stream: is_typing_signal.set(true);
// after stream: is_typing_signal.set(false);
```

**Ownership detail:** `use_signal_sync` returns a `Signal<bool>` that implements `Send + Sync + Clone`. Cloning it gives each coroutine scope its own handle to write to the same reactive slot. The `move` keyword in the coroutine closure captures this clone by value.

## Step 5: Remove old code

- Delete the old import of `process_stream`, `ChatRequest`, `ChatMessage`, `LlmClient`, `save_message`, `clear_messages`, `create_conversation`, `update_conversation_title` from `screen.rs` — these now live in `service.rs`
- Keep the import of `ChatAction` but source it from `service` instead of local
- The `Message`, `Conversation`, `ConversationStore` types remain in `state.rs` and are used by both modules

## Step 6: Verify compilation

```bash
cargo check
```

Expected issues:
- `Signal<bool>` in a closure that crosses `.await` boundary — use `use_signal_sync` not `use_signal`
- The `material-symbols-outlined` font may not load in desktop Dioxus — replace with Unicode/emoji alternatives or add the font via `document::Link`
- `Key::Enter` handling in textarea vs. input — textarea captures Enter for newlines by default; you may need to use `Shift+Enter` for newline and plain `Enter` to send

---

# Trade-offs

| Dimension | Assessment |
|-----------|------------|
| **Complexity** | Medium — one new file, refactored coroutine pattern, but concept is straightforward (extract + rebuild) |
| **Time to implement** | 3–5 hours for a focused session (service extraction: 1h, UI rewrite: 2–3h, tweaks & edge cases: 1h) |
| **Reversibility** | Easy — the old `screen.rs` can be restored from git; the service module is additive, not destructive |
| **Risk level** | Medium — the extracted streaming logic is unchanged but now operates on `&mut Vec<Message>` instead of inline `messages.push()`; incorrect borrowing in the coroutine could cause panics |
| **Scalability** | High — the service layer can be extended with new methods (message editing, branching, export) without touching the UI; the screen can be swapped for a different renderer (e.g., TUI) |
| **Maintainability** | High — two small focused files instead of one large one; a contributor fixing a streaming bug touches `service.rs`, one fixing layout touches `screen.rs`, never both |
| **Fit for user persona** | High — this is the DDD pattern the user requested (smart domain, dumb UI); it demonstrates idiomatic Rust module organization, `thiserror`, and struct-based service pattern |

---

# Consequences

## Positive Outcomes

- **Clean separation of concerns** — the streaming/DB/LLM logic is now a plain Rust struct. It can be unit-tested without Dioxus: instantiate `ChatService::new(conn)`, call `send_message()`, assert on the returned message list. The Rust compiler catches entire classes of bugs at the module boundary.
- **Education value** — the user sees idiomatic Rust patterns in practice: `thiserror` for error types, `impl` blocks with async methods, iterator chains, `Result` propagation with `?`, and `Into`/`From` conversions.
- **Screen becomes a template** — the new `screen.rs` is essentially the HTML template transcribed to RSX. If the design changes, you diff against `code.html`, update the RSX, and never worry about breaking the streaming logic.
- **Reusable service** — `ChatService` could be used from multiple screens (e.g., a dedicated admin panel, a batch-processing script, or a CLI tool) without modification.

## Risks & Failure Modes

1. **Borrow conflicts in the coroutine** — The coroutine captures `messages` (a `Signal<Vec<Message>>`) and `store` (a `Signal<ConversationStore>`). The extracted `send_message` takes `&mut Vec<Message>` and `&mut ConversationStore`. Inside the coroutine, calling `messages.write()` returns a guard that blocks other reads. If the streaming loop holds the write guard across `.await` points, other components (like the sidebar) cannot read `store`. **Mitigation:** release the write guard before `.await` by extracting values, or use a pattern where the coroutine sends events to an intermediary that updates signals in a non-async context.

2. **`move` closure captures** — The coroutine closure uses `move` to capture `messages`, `store`, `error_msg`, and `is_typing`. If the component re-renders and creates new signals, the old coroutine still holds references to the old signals. Dioxus handles this internally (signals are reference-counted), but it's worth verifying that updates still propagate.

3. **`use_signal_sync` overhead** — Synchronizing a signal from a background task uses internal locking. For the typing indicator (toggled at most once per message), this is negligible.

4. **Material Symbols font** — The template uses Google's Material Symbols icon font via `@import`. On Dioxus desktop (WebView), external font loading may flash or fail. **Fallback:** use inline SVG paths or plain text labels ("Copy", "↻", "👍") as shown in the RSX above.

5. **Automatic textarea height** — The HTML template uses `rows="1"` with auto-grow. Dioxus RSX `textarea` supports `rows` attribute but not auto-resize without JavaScript. **Mitigation:** set a fixed `max-h-48` with CSS overflow or accept that the textarea stays one row until the user types.

## Second-Order Effects

- **Widget dependency shift** — The sidebar (`widgets/sidebar.rs`) currently accesses `ConversationStore` via `use_context`. After refactoring, the store is still provided at the `App` level, so the sidebar continues to work unchanged. However, the sidebar creates conversations directly via `repo::create_conversation`, bypassing `ChatService`. **6-month consequence:** there are two paths for conversation creation (sidebar button + chat auto-create). This is fine now but could lead to inconsistent logic if business rules are added (e.g., "limit 10 conversations per session"). Refactor the sidebar to also go through `ChatService` when that day comes.

- **Testing infrastructure** — Once `ChatService` is extracted, writing unit tests becomes straightforward. The user may start writing tests, which is a positive outcome, but it also means they need to set up test fixtures (in-memory SQLite via `rusqlite::Connection::open_in_memory`).

- **Future abstraction pressure** — `ChatService` currently depends on concrete types: `Arc<Mutex<Connection>>`, `LlmClient`, `process_stream`. If the user later wants to swap the LLM provider or the database, they need to introduce traits (`ChatRepository`, `LlmProvider`). This is a natural next step in the DDD journey but is outside this solution's scope.

---

# Verdict

**This solution is best for users who prioritize clean module boundaries, long-term maintainability, and the DDD "smart domain, dumb UI" principle, and who are willing to invest one extra refactoring step (service extraction) before seeing visual results.**
