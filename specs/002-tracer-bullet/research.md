# Phase 0: Research & Decisions

## SQLite Crate Selection
- **Decision**: `rusqlite` wrapped in `tokio::task::spawn_blocking`
- **Rationale**: The deep think context established an Actor Model using `use_coroutine`. Wrapping a synchronous lightweight SQLite client in `spawn_blocking` avoids the massive compilation and macro overhead of `sqlx` while keeping the Dioxus UI thread 100% unblocked.
- **Alternatives**: `sqlx` (Overkill for a simple local desktop database without complex migrations yet).

## LLM Streaming Parsers
- **Decision**: `reqwest-eventsource` for OpenAI, manual byte stream for Ollama.
- **Rationale**: OpenAI uses standard Server-Sent Events (SSE) which `reqwest-eventsource` handles robustly. Ollama returns NDJSON (Newline Delimited JSON), which can be efficiently parsed by splitting the raw `reqwest` byte stream on `\n`.
- **Alternatives**: Writing custom SSE parser (error prone and reinvents the wheel).
