# Research: Core Interface Pages

## Overview

Technical research conducted to resolve architecture decisions for implementing the Session Management page and Chat Interface UI refactor in Lumina.

## Decisions

### Decision 1: Session domain follows existing chat domain pattern

- **Decision**: Create `src/domains/sessions/` with `state.rs`, `repo.rs`, `screen.rs`, `widgets/` mirroring the chat domain structure.
- **Rationale**: The codebase already has an established Dioxus domain pattern in `domains/chat/` and `domains/settings/`. Following the same pattern reduces cognitive overhead for a solo developer learning Rust.
- **Alternatives considered**:
  - Putting folder/session logic into a new `src/views/` or `src/pages/` directory — rejected because it would break the existing domain convention.
  - Adding folder CRUD directly into the chat domain — rejected because Session Management is conceptually separate from the chat interface.

### Decision 2: Sidebar extracted to shared `layout.rs` with Dioxus Router `#[layout]`

- **Decision**: Create `src/layout.rs` with a `RootLayout` component containing the sidebar and `<Outlet/>`. Refactor `app.rs` to use `#[layout(RootLayout)]` wrapping all routes.
- **Rationale**: Dioxus 0.7 Router supports `#[layout]` + `#[end_layout]` for shared UI with `Outlet`. This is the canonical way to implement persistent navigation. The existing sidebar code in `chat/widgets/sidebar.rs` moves to `src/components/sidebar.rs`.
- **Alternatives considered**:
  - Duplicating sidebar code in each screen — rejected (violates DRY).
  - Using `use_context` to render sidebar conditionally — rejected (layout is the idiomatic Dioxus pattern).

### Decision 3: Folder hierarchy uses `parent_id` with recursive CTE

- **Decision**: Folders table has a nullable `parent_id` column (self-referential FK). Queries use recursive CTE for tree traversal.
- **Rationale**: Single-user desktop app with <100 folders doesn't need nested sets, materialized paths, or other heavy tree representations. `parent_id` + recursive CTE is the simplest SQLite-compatible approach.
- **Alternatives considered**:
  - Materialized Path (mpath) — over-engineered for this scale.
  - Nested Sets — complex inserts/updates for a single-user app.
  - Flat folder list without nesting — rejected because the spec requires at least one level of nesting.

### Decision 4: Drag-and-drop uses mouse-event simulation

- **Decision**: Implement drag-and-drop via `mousedown`/`mousemove`/`mouseup` event handlers rather than native HTML5 DnD API.
- **Rationale**: Dioxus has known WebView2 issues with native DnD events (GitHub issue #2167 — drag events don't fire on Windows). Mouse-event simulation is more reliable across platforms.
- **Alternatives considered**:
  - Native DnD via `ondragstart`/`ondrop` — rejected due to known cross-platform compatibility issues.
  - Context menu "Move to folder" option — implemented as fallback for when drag is unavailable.

### Decision 5: Icons use Unicode/emoji fallbacks

- **Decision**: Replace Material Symbols icon references with Unicode/emoji equivalents or inline SVG.
- **Rationale**: Desktop webview cannot reliably load Google Fonts CDN. The font may fail silently, leaving empty spaces where icons should be. Unicode fallbacks (e.g., 📁 for folder, 💬 for chat) are universally available across platforms.
- **Alternatives considered**:
  - Bundling Material Symbols font locally — possible but adds complexity for a solo developer.
  - Inline SVG icons — most reliable but more verbose in RSX.

### Decision 6: Chat refactor preserves existing coroutine

- **Decision**: The `use_coroutine` actor in `screen.rs`, the `ConversationStore` signal, and the `spawn_blocking` DB pattern remain unchanged. Only the RSX rendering layer is replaced.
- **Rationale**: The existing streaming/DB logic is proven working (Phase 1 tracer-bullet). Rewriting it alongside the UI would increase risk and debugging surface. Separate commits for widget swaps allow rollback.
- **Alternatives considered**:
  - Full screen rewrite — rejected due to high risk.
  - Extracting coroutine to `service.rs` — architecturally cleaner but adds an extra refactoring step for this session.

## References

- Dioxus 0.7 Layout + Outlet: https://dioxuslabs.com/learn/0.7/routing/nested-routes
- Dioxus Tailwind CSS: https://dioxuslabs.com/learn/0.7/guides/utilities/tailwind/
- Dioxus Store pattern: https://dioxuslabs.com/learn/0.7/essentials/basics/collections/
- rusqlite recursive CTE: https://www.sqlite.org/lang_with.html
- Dioxus DnD issue #2167: https://github.com/DioxusLabs/dioxus/issues/2167
