# Quickstart: Core Interface Pages (004-session-chat-ui)

## Prerequisites

- **Rust**: 1.80.0+
- **Dioxus CLI**: `cargo install dioxus-cli --version 0.7.0`
- **System deps (Linux)**: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `pkg-config`, `libssl-dev`

## Setup

```bash
git clone <repo-url> && cd lumina

# Configure LLM provider (OpenAI or Ollama)
cp .env.example .env   # or create manually (see specs/002-tracer-bullet/quickstart.md)

# Database initializes automatically on first run (SQLite via rusqlite bundled)
```

## Running Dev Server

```bash
dx serve
```

Launches the desktop app with hot-reloading on WebView2/WKWebView/WebKitGTK. The Session Management page is the default landing route.

## Production Build

```bash
dx build --release
```

Outputs an optimised platform-native binary in `dist/`.

## Testing

```bash
cargo test
```

## Implementation Order

This feature is split into two parallel tracks:

### Track A — Session Management (4 vertical slices)

| Slice | What | Touches |
|-------|------|---------|
| 1 | Router refactor + shared layout with sidebar | `app.rs`, new `layout.rs`, `components/sidebar.rs` |
| 2 | Static UI scaffold | `domains/sessions/` — `screen.rs`, widget stubs |
| 3 | SQLite folders + CRUD | `repo.rs`, `state.rs` — folder tree, create/rename/delete |
| 4 | Session grid + drag-and-drop | `session_grid.rs`, `session_card.rs`, mouse-event DnD |

### Track B — Chat Widget Swaps (one-at-a-time replacements)

Existing `screen.rs` coroutine, `ConversationStore`, and DB pattern preserved — only RSX changes:

1. **Message bubbles** — Terra dark theme, rich markdown (pulldown-cmark)
2. **Input area** — textarea, send button, model selector, attachment button
3. **Hover action bar** — Copy, Regenerate, Thumbs Up on AI messages
4. **Typing indicator** — animated bouncing dots
5. **Prompt suggestion chips** — quick-action chips above input
