# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

- **Run desktop app (hot-reload):** `dx serve`
- **Build production binary:** `cargo build --release`
- **Run tests:** `cargo test`
- **Run single test:** `cargo test <test_name>`
- **Lint/check:** `cargo clippy`
- **Type-check only:** `cargo check`

## Environment Setup

- Copy `.env.example` to `.env` and configure `API_PROVIDER_TYPE` (ollama or openai)
- For Ollama: `API_OLLAMA_MODE=local`, endpoint defaults to `http://localhost:11434`
- For OpenAI: set `API_ENDPOINT_URL`, `API_MODEL_NAME`, `API_KEY`

## Architecture Overview

see @README.md

## Key Patterns

### Actor Model for Async
Chat uses `use_coroutine` with `ChatAction` enum. The coroutine handles all LLM streaming and DB writes — the UI only dispatches actions and reads reactive state. This keeps the UI thread unblocked.

### Two-Layer Rule
- **`src/components/`** — pure UI components. Accept props, emit events. Cannot access global state, DB, or network.
- **`src/domains/`** — business logic, state, DB. Domains can import components, but not vice versa.

### Database Pattern
All SQLite operations use `tokio::task::spawn_blocking`. The connection is `Arc<Mutex<Connection>>` passed via Dioxus context.

### State Management
- Local: `use_signal`
- Cross-component: `use_context::<Signal<ConversationStore>>`
- Global: `Signal::global()` for SETTINGS and MCP_STATE

### LLM Provider Flow
`process_stream` matches on `ProviderType`:
- **OpenAI:** SSE via `reqwest-eventsource` — parses `ChatResponseChunk.choices[0].delta.content`
- **Ollama:** NDJSON byte stream — parses `OllamaChatResponse.message.content`
