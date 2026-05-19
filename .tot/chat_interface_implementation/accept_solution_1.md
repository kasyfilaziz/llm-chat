# Problem

Implement the new Chat Interface UI for Lumina by refactoring the existing `src/domains/chat/` implementation. The new UI is defined in `assets/stich/chat_interface_4/code.html` — a dark "Terra" themed design with: user message bubbles (circular avatar), AI message bubbles (icon + rich markdown content + hover action bar with copy/regenerate/thumbs-up), a typing indicator (bouncing dots), and a fixed-bottom input area containing prompt suggestion chips, a multi-line textarea, an attachment button, a model selector dropdown, a send button, and a legal disclaimer ("may produce inaccurate information"). The existing codebase is a Phase 1 tracer-bullet with a simple light-themed chat. "Solved" means all visual elements from the HTML template are rendered in Dioxus RSX using the Terra Tailwind color palette, the chat send/stream loop works correctly, and the app compiles and runs without runtime panics.

# User Context & Persona

- **Role**: Sole developer, learning Rust from a Python background.
- **Skill Level**: Understands Rust syntax but is still building intuition around ownership, closures (`move`), iterators, `Option`/`Result` idioms, and async patterns. Python background means comfort with dynamic dispatch but unfamiliarity with borrow checker discipline.
- **Constraints**: Wants idiomatic Rust code (`.map()`, `.and_then()`, iterators). Complete implementation in a focused session. Needs explanations of Rust-specific concepts where they appear.
- **Existing Work**: Already has a working tracer-bullet chat (screen.rs orchestrator, use_coroutine actor for streaming, ConversationStore via context, SQLite via spawn_blocking). The core message send/stream loop works and must be preserved.
- **Desired Approach**: Widget-by-widget replacement to minimize risk and allow incremental verification.

# External Research

## Dioxus 0.7 Component Patterns & Event Handling

The Dioxus 0.7 docs confirm that components are plain functions annotated with `#[component]`, taking props that implement `PartialEq + Clone`, returning `Element`. Conditional rendering uses standard Rust `if/else` or `match` inside `rsx! {}` blocks. Event handlers use closures with the `move` keyword:

```rust
onclick: move |_| count += 1,
oninput: move |evt: FormEvent| input_text.set(evt.value()),
```

Source: https://dioxuslabs.com/learn/0.7/essentials/ui/components/ — "Components and Properties"
Source: https://dioxuslabs.com/learn/0.7/essentials/ui/conditional/ — "Conditional Rendering"
Source: https://dioxuslabs.com/learn/0.7/essentials/basics/event_handlers/ — "User Input / Event Handlers"

## Rust Idiomatic Patterns with Dioxus (Closures, Iterators in RSX)

Dioxus supports two list-rendering styles: inline iterators with `.map()` and the `for` sugar:

```rust
rsx! {
    {(0..10).map(|idx| rsx! { "item {idx}" })}
    for msg in messages.read().iter() {
        MessageBubble { message: msg.clone() }
    }
}
```

The `for` sugar is preferred for readability. Closures that capture state need `move` to transfer ownership into the closure, which is required for `onclick`, `oninput`, etc. This is a key Rust concept to explain: without `move`, the closure borrows local variables; with `move`, it takes ownership.

Source: https://dioxuslabs.com/learn/0.7/essentials/ui/iteration/ — "Rendering Lists"

## dioxus-stores Patterns

Dioxus 0.7 ships `dioxus-stores` (v0.7.9 in Cargo.toml) which provides `#[derive(Store)]` for fine-grained reactive access to nested state. The existing code does NOT use stores — it uses `use_context::<Signal<ConversationStore>>` and raw `use_signal(Vec<Message>)`. Stores enable per-field reactivity (e.g., updating one conversation title re-renders only that item, not the whole list). For this solution (widget-by-widget surgery), we do **not** introduce stores — the existing `use_context` + `use_signal` pattern is sufficient and less invasive. If the user later hits performance issues with large message lists, stores would be the upgrade path.

Source: https://dioxuslabs.com/learn/0.7/essentials/basics/collections/ — "Stores and Collections"
Source: https://crates.io/crates/dioxus-stores — "dioxus-stores crate"

## Dioxus Tailwind Styling

Dioxus 0.7 has zero-config Tailwind integration. DX auto-detects `tailwind.css` at the project root, runs the Tailwind CLI watcher, and outputs to `assets/tailwind.css`. The app includes it via `document::Stylesheet { href: asset!("/assets/tailwind.css") }`. Tailwind classes go in the `class:` attribute. Custom theme colors (like the Terra palette) are defined in `@theme` blocks in the input CSS:

```css
@import "tailwindcss";
@source "./src/**/*.{rs,html,css}";
@theme {
    --color-surface: #101411;
    --color-primary: #9dd3aa;
    --color-secondary: #d0c5b8;
    /* ... etc */
}
```

Source: https://dioxuslabs.com/learn/0.7/essentials/ui/styling/ — "Styling your app"
Source: https://dioxuslabs.com/learn/0.7/guides/utilities/tailwind/ — "Tailwind Guide"

# Solution Summary

**Solution 1: Widget-by-Widget Surgery** keeps the existing `screen.rs` orchestrator (including the `use_coroutine`-based chat actor, `use_effect` for loading conversations/messages, and `use_context` for `ConversationStore` and DB connection) fully intact. It replaces individual visual components one at a time: the message bubble component, the input area (inline in screen.rs or extracted to a widget), the hover action bar (as a sub-component of AI messages), the typing indicator (conditional rendering based on streaming state), and prompt suggestion chips. Each replacement is isolated so the app can be compiled and visually verified after each step.

# Assumptions

1. **The existing message send/stream loop in `use_coroutine` is correct and will not be rewritten.** Only its visual output changes.
2. **The Terra color palette will be defined in `tailwind.css` using `@theme` variables.** No inline color values in Rust components.
3. **Icons will use Unicode/emoji or simple inline SVG** rather than Material Symbols (which require a Google Fonts CDN link). The HTML template uses `<span class="material-symbols-outlined">eco</span>` but Dioxus desktop apps cannot load Google Fonts from CDN reliably; inline SVG or Unicode fallbacks are safer.
4. **The typing indicator activates when an assistant message with empty content exists** (i.e., the placeholder message created before streaming begins). This is already the existing pattern: `assistant_msg` is created with `content: ""` and pushed before streaming starts.
5. **The hover action bar appears on AI messages only** (not user messages), matching the HTML template.
6. **The attachment button and model selector dropdown are visual-only** in this phase (no file upload or model switching logic). Their UI is rendered but `onclick` is a no-op or logs to console.
7. **Prompt suggestion chips are static** (hardcoded strings from the HTML template) rather than dynamically generated.
8. **The existing `Button` and `Input` shared components will not be reused** for the new input area — the new input area is sufficiently different (textarea vs input, nested toolbar layout) that inline Dioxus elements are cleaner.
9. **The `Message` struct in `state.rs` does not need changes.** Its `role` and `content` fields are sufficient for the new rendering.
10. **The existing sidebar widget is preserved as-is.** The HTML template's secondary sidebar is out of scope for this solution.
11. **The project uses Dioxus 0.7** (confirmed in Cargo.toml: `dioxus = { version = "0.7", features = ["desktop", "router"] }`).

# Detailed Implementation

## Step 0: Add Terra Theme to Tailwind Config

**File**: `tailwind.css` (project root)

Extract the Terra dark color palette from the HTML template's `tailwind.config.theme.extend.colors` block and add it as `@theme` variables. The HTML template has dark-mode colors (surface: #101411, primary: #9dd3aa, etc.) — since the app is desktop, always-dark is fine.

```css
@import "tailwindcss";
@source "./src/**/*.{rs,html,css}";

@theme {
    --color-surface: #101411;
    --color-surface-dim: #101411;
    --color-surface-bright: #363a36;
    --color-surface-container: #1d211d;
    --color-surface-container-low: #191d19;
    --color-surface-container-lowest: #0b0f0c;
    --color-surface-container-high: #272b27;
    --color-surface-container-highest: #323632;
    --color-surface-variant: #323632;
    --color-on-surface: #e0e3de;
    --color-on-surface-variant: #c1c9bf;
    --color-primary: #9dd3aa;
    --color-primary-container: #4a7c59;
    --color-on-primary: #01391c;
    --color-on-primary-container: #e1ffe5;
    --color-secondary: #d0c5b8;
    --color-secondary-container: #4f483e;
    --color-on-secondary: #362f26;
    --color-on-secondary-container: #c1b7aa;
    --color-tertiary: #e2c284;
    --color-tertiary-container: #886e38;
    --color-on-tertiary: #402d00;
    --color-on-tertiary-container: #fff6ed;
    --color-error: #ffb4ab;
    --color-error-container: #93000a;
    --color-on-error: #690005;
    --color-on-error-container: #ffdad6;
    --color-outline: #8b938a;
    --color-outline-variant: #414942;
    --color-inverse-primary: #376847;
    --color-inverse-surface: #e0e3de;
    --color-inverse-on-surface: #2d312e;
}
```

**Rust explanation**: `@theme` is Tailwind v4's way of defining custom design tokens. Each `--color-{name}` variable generates utility classes like `bg-surface`, `text-on-surface`, `border-primary`, etc. The Dioxus CLI's Tailwind integration picks this up automatically.

## Step 1: Rewrite `MessageBubble` Widget (`widgets/message.rs`)

This is the most impactful change. The existing bubble is a simple colored rectangle. The new one must render:
- **User messages**: Right-aligned, circular avatar on the right, bubble with white text on primary background.
- **AI messages**: Left-aligned, icon badge (eco leaf) on the left, rich markdown content, hover action bar (Copy, Regenerate, Thumbs Up) below the content, initially hidden and revealed on group hover.

**New `widgets/message.rs`**:

```rust
use dioxus::prelude::*;
use crate::domains::chat::state::Message;
use crate::components::Markdown;

/// The hover action bar shown on AI messages.
/// Uses opacity-0 group-hover:opacity-100 to reveal on hover.
#[component]
fn ActionBar() -> Element {
    rsx! {
        div { class: "flex items-center gap-2 mt-4 opacity-0 group-hover:opacity-100 transition-opacity",
            button {
                class: "text-secondary hover:text-primary hover:bg-primary/5 p-1.5 rounded-full transition-colors text-xs flex items-center gap-1",
                title: "Copy",
                onclick: move |_| {},
                span { class: "material-symbols-outlined", style: "font-size: 16px;", "content_copy" }
            }
            button {
                class: "text-secondary hover:text-primary hover:bg-primary/5 p-1.5 rounded-full transition-colors text-xs flex items-center gap-1",
                title: "Regenerate",
                onclick: move |_| {},
                span { class: "material-symbols-outlined", style: "font-size: 16px;", "refresh" }
            }
            button {
                class: "text-secondary hover:text-primary hover:bg-primary/5 p-1.5 rounded-full transition-colors text-xs flex items-center gap-1",
                title: "Good response",
                onclick: move |_| {},
                span { class: "material-symbols-outlined", style: "font-size: 16px;", "thumb_up" }
            }
        }
    }
}

/// User avatar: a small circular image placeholder.
/// In production this would be the user's profile picture.
#[component]
fn UserAvatar() -> Element {
    rsx! {
        div { class: "w-8 h-8 rounded-full overflow-hidden shrink-0 mt-1 border border-outline-variant/30",
            // Placeholder avatar — a solid circle with initials or a generic icon
            div { class: "w-full h-full bg-primary/20 flex items-center justify-center text-xs text-on-primary-container font-bold",
                "U"
            }
        }
    }
}

/// AI avatar: an icon badge with the Terra eco leaf icon.
#[component]
fn AiAvatar() -> Element {
    rsx! {
        div { class: "w-8 h-8 rounded-full bg-primary-container text-on-primary-container flex items-center justify-center shrink-0 mt-1",
            span { class: "material-symbols-outlined text-sm", "eco" }
        }
    }
}

/// A single chat message bubble, styled per the Terra design system.
/// User messages are right-aligned with an avatar. AI messages are
/// left-aligned with an icon badge and a hover action bar.
#[component]
pub fn MessageBubble(message: Message) -> Element {
    let is_user = message.role == "user";

    // Key Rust concept: `move` transfers ownership of `message` into the closure.
    // Without it, the closure would borrow a local that doesn't live long enough.
    if is_user {
        rsx! {
            div { class: "flex gap-4 items-start w-full justify-end",
                div { class: "flex-1 min-w-0 pt-1 max-w-[85%]",
                    div { class: "text-on-surface text-lg leading-relaxed font-body whitespace-pre-wrap",
                        "{message.content}"
                    }
                }
                UserAvatar {}
            }
        }
    } else {
        // AI message — group container enables the hover effect on ActionBar.
        // Key Rust concept: `group` in Tailwind creates a parent scope that
        // child elements can reference with `group-hover:`.
        rsx! {
            div { class: "flex gap-4 items-start w-full group",
                AiAvatar {}
                div { class: "flex-1 min-w-0 pt-1",
                    div { class: "text-on-surface text-lg leading-relaxed font-body space-y-4",
                        Markdown { content: message.content }
                    }
                    ActionBar {}
                }
            }
        }
    }
}
```

**Key Rust concepts to explain**:
- `move |_| {}`: The `move` keyword forces the closure to take ownership of any captured variables. In Dioxus event handlers, `move` is required because the closure lives longer than the current function scope.
- `group` / `group-hover:`: Tailwind utility — adding `group` to a parent div allows children to use `group-hover:opacity-100` to reveal on parent hover. This is purely CSS; no Rust state needed.
- The `rsx! {}` blocks inside `if/else` are valid because each branch evaluates to an `Element` (Dioxus's virtual node type). Rust's `if/else` is an expression, so it returns the chosen branch.

## Step 2: Add Typing Indicator Widget (`widgets/typing.rs`)

Create a new file for the typing indicator. This is a small component showing three bouncing dots inside an AI-message-shaped container (icon + dots).

**New file `widgets/typing.rs`**:

```rust
use dioxus::prelude::*;

/// A bouncing-dots typing indicator styled to match AI message layout.
/// Shows the eco icon badge on the left and three animated dots on the right.
#[component]
pub fn TypingIndicator() -> Element {
    rsx! {
        div { class: "flex gap-4 items-start w-full",
            div { class: "w-8 h-8 rounded-full bg-primary-container text-on-primary-container flex items-center justify-center shrink-0 mt-1",
                span { class: "material-symbols-outlined text-sm animate-pulse", "eco" }
            }
            div { class: "flex-1 min-w-0 pt-1 flex items-center h-8",
                div { class: "flex gap-1",
                    div { class: "w-2 h-2 rounded-full bg-tertiary/60 animate-bounce", style: "animation-delay: 0s;" }
                    div { class: "w-2 h-2 rounded-full bg-tertiary/60 animate-bounce", style: "animation-delay: 0.2s;" }
                    div { class: "w-2 h-2 rounded-full bg-tertiary/60 animate-bounce", style: "animation-delay: 0.4s;" }
                }
            }
        }
    }
}
```

**Rust explanation**: Nothing Rust-specific here — this is pure RSX/CSS. The `animate-bounce` and `animate-pulse` are Tailwind animation utilities. The staggered `animation-delay` on each dot creates the bouncing-cascade effect.

## Step 3: Register New Widgets in `widgets/mod.rs`

```rust
pub mod message;
pub mod sidebar;
pub mod typing;  // NEW
```

The `message.rs` changes are in-place; no module rename needed.

## Step 4: Refactor `screen.rs` — Add Streaming State + Typing Indicator

The existing code creates an empty `assistant_msg` before streaming, pushes it to `messages`, then fills it via the stream. We need a signal that tracks whether streaming is active so the UI can show the typing indicator instead of the empty assistant message placeholder.

**Changes in `screen.rs`**:

1. Add a `streaming` signal:
```rust
let mut streaming = use_signal(|| false);
```

2. In the `ChatAction::SendMessage` branch of the coroutine, set `streaming.set(true)` before the loop and `streaming.set(false)` after.

3. In the rendering section (inside the `for msg in messages.read().iter()` block), show the typing indicator when streaming is active and the last message is an empty assistant message.

**Key pattern** — conditional rendering of typing indicator:

```rust
// In the messages list, between the last real message and the footer:
if streaming() {
    TypingIndicator {}
}
```

However, there's a subtlety: when streaming starts, an empty assistant message is pushed to `messages`, and then tokens fill it. The typing indicator should replace the empty message or show alongside it. The cleanest approach: **remove the empty push** and instead conditionally render either the typing indicator or the completed message. But that changes the coroutine's message tracking.

**Less invasive approach**: Keep the empty push (it tracks the assistant message ID for token appending). In the render loop, filter out empty assistant messages when streaming is active, and show the typing indicator instead.

```rust
// Replace the raw iteration with conditional logic:
let msgs = messages.read();
let mut msg_iter = msgs.iter().peekable();

while let Some(msg) = msg_iter.next() {
    // If this is an empty assistant message and streaming is active, skip it
    // (the typing indicator handles the visual)
    if streaming() && msg.role == "assistant" && msg.content.is_empty() {
        continue;
    }
    MessageBubble { message: msg.clone() }
}

if streaming() {
    TypingIndicator {}
}
```

**Rust concept**: `.peekable()` converts an iterator into one that supports `.peek()` — useful for lookahead. `while let` is pattern matching: "while the option returned by `.next()` matches `Some(msg)`, bind `msg` and execute the block." This is more idiomatic than calling `.next()` in a loop condition.

## Step 5: Refactor Input Area in `screen.rs`

Replace the existing footer (Button + Input + Send) with the new Terra-themed input area containing:
- Prompt suggestion chips (horizontal scrollable row)
- Main input box (textarea with auto-resize, bottom toolbar)
- Attachment button, model selector, send button
- Legal disclaimer text

The new input area is complex enough to warrant its own widget file. Extract to `widgets/input_area.rs`.

**New file `widgets/input_area.rs`**:

```rust
use dioxus::prelude::*;

/// A single prompt suggestion chip — a rounded pill button.
#[component]
fn SuggestionChip(label: String) -> Element {
    rsx! {
        button {
            class: "whitespace-nowrap px-4 py-1.5 rounded-full bg-surface-container hover:bg-surface-container-high border border-outline-variant/30 text-xs text-on-surface-variant transition-colors",
            "{label}"
        }
    }
}

/// The fixed-bottom input area with suggestions, textarea, and toolbar.
#[component]
pub fn InputArea(
    value: String,
    oninput: EventHandler<FormEvent>,
    onsubmit: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "absolute bottom-0 left-0 right-0 bg-gradient-to-t from-surface-bright via-surface-bright to-transparent pt-10 pb-6 px-4 md:px-8 z-20",
            div { class: "max-w-3xl mx-auto",
                // Prompt Suggestions Bar
                div { class: "flex gap-2 mb-3 overflow-x-auto pb-2 scrollbar-hide",
                    SuggestionChip { label: "Extract Action Items".to_string() }
                    SuggestionChip { label: "Draft Executive Summary".to_string() }
                    SuggestionChip { label: "Compare with Q2".to_string() }
                }
                // Main Input Box
                div { class: "relative bg-surface rounded-2xl border border-outline-variant/50 shadow-[0_4px_20px_rgba(46,50,48,0.06)] focus-within:border-primary focus-within:ring-1 focus-within:ring-primary transition-all duration-200",
                    textarea {
                        class: "w-full bg-transparent border-none focus:ring-0 resize-none py-4 pl-4 pr-12 text-on-surface placeholder:text-secondary/60 text-base min-h-[56px] max-h-48 rounded-2xl",
                        placeholder: "Ask Terra anything...",
                        rows: "1",
                        value: "{value}",
                        oninput: move |evt| oninput.call(evt),
                    }
                    // Bottom Toolbar
                    div { class: "flex justify-between items-center px-3 pb-3 pt-1",
                        div { class: "flex items-center gap-1",
                            // Attachment Button (visual-only)
                            button {
                                "aria-label": "Attach file",
                                class: "text-secondary hover:text-primary hover:bg-primary/10 p-2 rounded-full transition-colors flex items-center justify-center",
                                onclick: move |_| {},
                                span { class: "material-symbols-outlined", "attach_file" }
                            }
                            // Model Selector (visual-only)
                            div { class: "relative flex cursor-pointer items-center gap-1 text-xs font-medium text-secondary hover:text-primary bg-surface-container-low hover:bg-surface-container px-3 py-1.5 rounded-lg transition-colors border border-outline-variant/20 ml-1",
                                span { class: "material-symbols-outlined", style: "font-size: 16px;", "psychology" }
                                "Terra Core v2"
                                span { class: "material-symbols-outlined", style: "font-size: 16px;", "expand_more" }
                            }
                        }
                        // Send Button
                        button {
                            class: "bg-primary text-on-primary hover:bg-primary-container hover:text-on-primary-container p-2 rounded-xl transition-colors flex items-center justify-center shadow-sm disabled:opacity-50 disabled:cursor-not-allowed",
                            onclick: move |evt| onsubmit.call(evt),
                            span { class: "material-symbols-outlined", "arrow_upward" }
                        }
                    }
                }
                // Legal Disclaimer
                div { class: "text-center mt-2",
                    span { class: "text-[10px] text-secondary/70",
                        "Terra may produce inaccurate information. Please verify important details."
                    }
                }
            }
        }
    }
}
```

**Register in `widgets/mod.rs`**:

```rust
pub mod message;
pub mod sidebar;
pub mod typing;
pub mod input_area;
```

**Replace footer in `screen.rs`**:

Old code (lines 240–294) is replaced with:

```rust
InputArea {
    value: input_text.read().clone(),
    oninput: move |evt: FormEvent| input_text.set(evt.value()),
    onsubmit: move |_| {
        if !input_text.read().is_empty() {
            chat_service.send(ChatAction::SendMessage(input_text.read().clone()));
            input_text.set(String::new());
        }
    },
}
```

Remove the `show_confirm` dialog (clear chat button) since the new UI template does not include it. Remove the old `Button`/`Input` component imports — they're no longer used in `screen.rs`.

**Key Rust concept**: The `oninput` prop on `InputArea` receives a `FormEvent`. We call `.set()` on `Signal<String>` to update it. The pattern `move |evt| oninput.call(evt)` delegates the event up to the parent. Compare with the textarea's `oninput: move |evt| oninput.call(evt)` which calls the prop's handler. Rust closures here capture `oninput` (an `EventHandler<FormEvent>`) by `move`, taking ownership, which is fine since the closure runs at most once per event.

## Step 6: Wire `streaming` Signal Through the Coroutine

In `screen.rs`, the coroutine block needs access to the `streaming` signal. Since `use_coroutine` captures variables that implement `Send` and the coroutine closure is `'static + Send`, we can clone references:

```rust
// Before the coroutine:
let mut streaming = use_signal(|| false);
let streaming_clone = streaming.clone();  // Signal implements Clone — both point to same value

let chat_service = use_coroutine(move |mut rx: UnboundedReceiver<ChatAction>| {
    let streaming = streaming_clone.clone();  // clone into coroutine
    // ... inside SendMessage match arm:
    streaming.set(true);
    // ... after stream completes:
    streaming.set(false);
});
```

**Rust concept**: `Signal` in Dioxus implements `Clone` — cloning a signal gives you a new handle to the same reactive value, not a copy of the value. This is similar to `Arc` but with reactive tracking.

## Step 7: Update Markdown Rendering for Dark Theme

The existing `Markdown` component in `components/markdown.rs` uses light-theme colors (`text-slate-900`, `bg-slate-100`, etc.). Update these to Terra theme colors:

| Old | New |
|-----|-----|
| `text-slate-900` | `text-on-surface` |
| `text-slate-800` | `text-on-surface` |
| `bg-slate-100` | `bg-surface-container` |
| `border-slate-200` | `border-outline-variant` |
| `text-pink-600` | `text-primary` |

This ensures code blocks, headings, and inline code use the dark palette.

## Step 8: Update `screen.rs` Layout Container

The wrapping div in `screen.rs`:

```rust
div { class: "flex h-screen bg-slate-50 text-slate-900 font-sans",
```

becomes:

```rust
div { class: "flex h-screen bg-surface text-on-surface font-body",
```

The chat history container:

```rust
div { class: "flex-1 overflow-y-auto px-4 py-8 max-w-4xl mx-auto w-full space-y-2",
```

becomes:

```rust
div { class: "flex-1 overflow-y-auto px-4 md:px-8 py-8 max-w-3xl mx-auto w-full space-y-10 scroll-smooth pb-40",
```

The header (lines 206–213) can be simplified or removed — the HTML template shows a minimal mobile-only header. For desktop, a thin top bar is fine. Replace with:

```rust
header { class: "flex md:hidden justify-between items-center px-6 h-16 w-full bg-surface border-b border-outline-variant/40 shrink-0",
    button { class: "text-secondary hover:bg-primary/10 p-2 rounded-full transition-colors",
        span { class: "material-symbols-outlined", "menu" }
    }
    div { class: "font-headline text-xl font-bold text-primary", "Lumina" }
    div { class: "flex gap-2",
        button { class: "text-secondary hover:bg-primary/10 p-2 rounded-full transition-colors",
            span { class: "material-symbols-outlined", "notifications" }
        }
    }
}
```

Or keep the existing header with updated colors. Simpler is better given the "surgery" approach — just update the class strings.

## Step 9: Handle Material Symbols Icons

The HTML template uses Material Symbols (`<span class="material-symbols-outlined">eco</span>`), which require loading `https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined` — a CDN font. Dioxus desktop (WebView) can load Google Fonts, but this requires an internet connection at first render and may not work in offline scenarios.

**Option A (recommended)**: Add a `document::Link` in `app.rs` to load the Material Symbols stylesheet:

```rust
document::Link {
    rel: "stylesheet",
    href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap",
}
```

**Option B**: Replace all icons with Unicode alternatives:
- `eco` → 🌿 or ⬡ SVG
- `content_copy` → 📋 or SVG
- `refresh` → 🔄 or SVG
- `thumb_up` → 👍 or SVG
- `attach_file` → 📎 or SVG
- `psychology` → 🧠 or SVG
- `expand_more` → ▼ or SVG
- `arrow_upward` → ↑ or SVG

Option A is simpler and matches the template exactly. Add the link to `app.rs` alongside the existing `document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }`.

## Step 10: Clean Up Unused Code

After the refactor:
- Remove unused imports in `screen.rs` (no more `Button`, `Input` from `crate::components`, no more `show_confirm` logic, no more `ChatAction::ClearChat` if unused).
- The `ChatAction` enum can be simplified to just `SendMessage(String)` if `ClearChat` is dropped.
- Remove the `clear_messages` import from `repo.rs` if no longer called.
- Remove `confirm_dialog` rendering from `screen.rs`.

## Verification Checklist

After each step, compile and run:

```bash
cargo build 2>&1 | head -50
dx serve --desktop
```

Verify:
1. App launches with dark Terra theme (no light flash).
2. User messages show right-aligned with avatar.
3. AI messages show left-aligned with eco icon + markdown.
4. Hovering over an AI message reveals the action bar (copy/regenerate/thumbs up).
5. Typing indicator appears when a message is being streamed.
6. Input area shows suggestion chips, textarea, attachment button, model selector, send button, disclaimer.
7. Sending a message works end-to-end (creates conversation, sends request, streams response).
8. No console errors about missing Material Symbols font (if using CDN).
9. MCP error banners still render correctly (lines 217–224 of old screen.rs).

# Trade-offs

| Dimension          | Assessment                                                                         |
|--------------------|------------------------------------------------------------------------------------|
| Complexity         | Low — each widget is small and self-contained, no architectural changes            |
| Time to implement  | 3–5 hours (1 hour per widget + wiring + theme setup)                               |
| Reversibility      | Easy — each widget change is isolated; revert a single file to go back             |
| Risk level         | Low — the coroutine/state layer is untouched; visual bugs are isolated             |
| Scalability        | Good — widget boundaries support future extraction of features (file upload, model switcher logic) |
| Maintainability    | Medium — `screen.rs` still has too many responsibilities (coroutine + layout), but that's existing tech debt |
| Fit for user persona | High — the user (Python→Rust learner) can understand each small widget in isolation and see Rust patterns applied |

# Consequences

## Positive Outcomes

- **Incremental verifiability**: After each widget replacement, the app compiles and runs. You can verify "does the bubble look right?" before moving to the next widget. This is critical for a Rust learner who may struggle with debugging opaque compiler errors.
- **Preserved working core**: The `use_coroutine` message send/stream loop is proven working. By not touching it, you avoid the #1 risk area (async streaming + state mutation).
- **Clear Rust pattern examples**: Each widget demonstrates a specific Rust-in-Dioxus concept: `move` closures in event handlers, conditional rendering with `if/else` expressions in `rsx!`, iterator patterns in list rendering, `Signal::clone()` for sharing state across async boundaries.
- **Tailwind theme mastery**: The user will understand Tailwind v4 `@theme` customization, which is directly transferable to future Dioxus projects.

## Risks & Failure Modes

- **Material Symbols font loading**: If the Google Fonts CDN is unreachable (offline/corporate proxy), icons will render as empty squares. The risk is real — Dioxus desktop uses the system WebView, which may have different network restrictions. **Mitigation**: Test with Option B (Unicode fallback icons) as a compile-time flag, or inline SVG as a dependency-free alternative. The Dioxus docs recommend `asset!()` for local assets; you could download the Material Symbols font and serve it from `assets/`.
- **Typing indicator timing**: The `streaming` signal may not toggle correctly if the coroutine panics or the stream errors before completing. **Mitigation**: Wrap the streaming loop in a `finally` equivalent using a `Drop` guard or ensure `streaming.set(false)` runs on all exit paths (success, error, cancellation). Rust's `Drop` trait is perfect here:

```rust
struct StreamingGuard(Signal<bool>);
impl Drop for StreamingGuard {
    fn drop(&mut self) {
        self.0.set(false);
    }
}
let _guard = StreamingGuard(streaming.clone());
```

- **Auto-resize textarea**: The HTML template uses `rows="1"` and `max-h-48` but Dioxus textarea does not auto-resize. The textarea stays at one row. **Mitigation**: Accept single-row display (common in chat UIs) or add JavaScript-like resize via `oninput` that sets `rows` based on line count. For a Rust learner, the simplest approach is to set a fixed reasonable height or use a larger `rows` value.

## Second-Order Effects

- **Widget file proliferation**: Adding `typing.rs` and `input_area.rs` creates 4 widget files. Over 3–6 months, if every new feature adds another widget, the `widgets/` directory could grow to 10+ files. This is manageable but signals the need for a subdirectory structure (e.g., `widgets/message/action_bar.rs`). Monitor file count and reorganize when it exceeds ~8 files.
- **`screen.rs` remains the god component**: The surgery approach explicitly avoids extracting the orchestration layer. Over time, `screen.rs` will accumulate more inline state, more child widgets, and more conditional rendering logic. The next architecture phase should extract the header, the message list, and the input container into separate "screen sections" — or move to a Store-based architecture where the entire screen state lives outside the component.
- **Coroutine closure captures grow**: Every new signal the coroutine needs (currently: `streaming`) adds another `clone()` before the coroutine closure. This pattern can become unwieldy. If 5+ signals are needed, consider bundling them into a single `ChatState` struct wrapped in a `Signal`. But for now, the pattern is straightforward.
- **Community pattern divergence**: The existing code uses `use_context` for `ConversationStore` + `Signal`. Adding more `use_signal` local state (like `streaming`) creates a mixed state strategy. Future contributors must learn both patterns. This is acceptable for a solo project but should be documented in `AGENTS.md`.

# Verdict

This solution is best for users who prioritize **low risk and incremental verification** and have **an existing working chat loop that must be preserved**. It is ideal for a Rust learner because each widget is small, focused, and demonstrates exactly one Rust-in-Dioxus concept at a time, making debugging tractable and building confidence with each successful compilation.
