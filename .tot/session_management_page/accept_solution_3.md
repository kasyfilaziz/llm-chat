# Problem

The user wants a functional Session Management page as the default landing page for their Dioxus desktop app (Lumina). The page requires: a persistent sidebar with nav items (Chat, Sessions active, Context Vault, Hooks, Settings, Help, Logout), a directory/folder explorer tree (left panel) with workspace folders showing session counts, a bento-grid of session cards (right panel) within the selected folder, full CRUD for folders (create, rename, delete), drag-and-drop to move sessions between folders, SQLite storage for a folders table plus a `folder_id` column on the existing conversations table, and session cards showing icon, category tag, title, description (line-clamp-2), date, turn count, and hover accent animation. The existing codebase has a Chat domain (screen, repo, state, widgets) and a Settings domain (screen, repo, state) that serve as structural references. The app currently routes only `ChatScreen` at `/`. "Solved" means the user can open the app, see the sessions page, browse folders, create/rename/delete folders, see sessions inside folders, and drag sessions between folders — all persisted to SQLite.

# User Context & Persona

Sole developer on the project, coming from Python, learning Rust. Comfortable with Python patterns (explicit is better than implicit, duck typing) but needs Rust-specific concepts explained (ownership, `Result`/`Option`, `Arc<Mutex<>>`, `spawn_blocking`, async). Wants complete implementation in one focused session. The user has already established domain conventions — the `chat/` domain with `state.rs`, `repo.rs`, `screen.rs`, `widgets/` pattern — so they expect the sessions domain to follow the same structure. They use Dioxus 0.7 with the `dx` CLI and Tailwind CSS v4.

# External Research

The following searches were conducted to validate architectural choices and understand Dioxus 0.7 patterns:

1. **Dioxus 0.7 Signals + use_coroutine patterns** — Dioxus official docs show `use_signal` for local state, `Signal::global()` for global state (used by Settings domain), `use_context_provider` for shared stores (used by ConversationStore), `use_effect` for side-effects on mount/deps change, and `use_coroutine` for long-running async tasks with message channels. Key insight: Signals are `Copy`, making them ergonomic in closures without cloning. `use_future` is for one-off async; `use_coroutine` is for state machines with message channels. Source: https://dioxuslabs.com/learn/0.7/essentials/basics/signals/, https://mintlify.com/DioxusLabs/dioxus/api/hooks/use-coroutine

2. **rusqlite patterns for hierarchical data** — A Stack Overflow discussion on `parent_id` + `mpath` (materialized path) for folder trees in SQLite. The `parent_id` column with a recursive CTE is the simplest approach — no need for `mpath` at this scale. For ordered children, a `sort_order` column works. The user's scale (single user, desktop app) doesn't need complex tree structures; `parent_id IS NULL` for root folders suffices. Source: https://stackoverflow.com/questions/54565945/how-to-represent-nested-folder-subfolder-structure-in-sqlite-database

3. **Dioxus drag-and-drop** — Dioxus 0.7 has built-in `DragEvent` support with `ondragstart`, `ondragover`, `ondrop`, and `DataTransfer` API. However, a known issue (Dioxus #2167) reports that HTML drag-and-drop events don't work on Windows WebView2 when a file-drop handler is provided. The Dioxus team has noted it's an upstream WebView2 limitation. For desktop, CSS-based drag simulation (mousedown/mousemove/mouseup) is more reliable. Source: https://github.com/DioxusLabs/dioxus/issues/2167, https://github.com/DioxusLabs/dioxus/blob/cece12b5/packages/html/src/events/drag.rs

4. **Rust async patterns with Dioxus** — The existing codebase already establishes the pattern: `tokio::task::spawn_blocking` for DB queries (rusqlite `Connection` is not `Send`), `use_effect` for initial data loading, and `use_coroutine` for long-running interaction loops. `Arc<Mutex<Connection>>` is the shared DB handle. Source: codebase `repo.rs` and `screen.rs`.

5. **Dioxus project structure with multiple domains** — The official Dioxus project structure guide and the existing codebase both show a flat `domains/` directory with one folder per domain (`chat/`, `settings/`, `llm/`, `mcp/`). Each domain has `state.rs` (models + store), `repo.rs` (DB queries), `screen.rs` (page component), and optionally `widgets/`. A `components/` directory at `src/` level holds shared dumb components. This pattern is well-suited for the sessions domain. Source: https://mintlify.com/DioxusLabs/dioxus/routing/nested-routes, codebase structure.

6. **Dioxus nested routes and layouts** — The router supports `#[layout(...)]` and `#[nest(...)]` for shared layouts with `Outlet`. This is how the persistent sidebar should be implemented — extract it from `ChatScreen` and move it into a `RootLayout` component. Source: https://mintlify.com/DioxusLabs/dioxus/routing/nested-routes

# Solution Summary

Build the Session Management page in four incremental vertical slices, each producing a working, testable increment: (1) static UI scaffold with persistent sidebar and empty sessions page, (2) SQLite folders table + folder CRUD operations in the store, (3) session listing under a selected folder with bento-grid cards, (4) drag-and-drop to move sessions between folders. Each slice touches UI (`screen.rs`/`widgets/`), state (`state.rs`), and DB (`repo.rs`) in one shot, following the existing Dioxus domain pattern established by the `chat/` and `settings/` domains. New files are created under `src/domains/sessions/`, and the router is refactored to use layouts so the sidebar is persistent across all pages.

# Assumptions

1. **Tailwind CSS v4 classes in the HTML template will work as-is** — The HTML template uses Tailwind v4 with `@import "tailwindcss"` and the `dx` CLI's built-in Tailwind support. The existing `assets/tailwind.css` was generated by Tailwind v4. Any new utility classes used (e.g., `line-clamp-2`, `grid-cols-3`, `col-span-2`) are already present in the generated CSS or will be picked up automatically by Tailwind's JIT engine via the `dx` CLI.
2. **Drag-and-drop on desktop will use a mouse-event simulation approach** — Native HTML drag-and-drop events have known issues on Windows WebView2 (Dioxus #2167). The solution implements drag intent detection via `mousedown`/`mousemove`/`mouseup` with visual feedback and a folder-target highlight, falling back to a context menu "Move to folder" option.
3. **The existing `conversations` table maps to "sessions"** — The user's request uses "sessions" as a synonym for conversations. The existing `conversations` table in SQLite becomes the sessions table. A `folder_id` column (nullable TEXT, FK to `folders.id`) will be added via `ALTER TABLE`.
4. **Folder hierarchy is single-level** — The HTML template shows workspaces with subfolders, but for MVP simplicity, the solution assumes a flat folder list with optional `parent_id` for nesting. The initial slice implements flat folders; nesting comes via `parent_id` in slice 2.
5. **No authentication or multi-user support** — The application is a single-user desktop app. The Logout/Help sidebar items are decorative stubs.
6. **The user has `dx` CLI installed and can run `cargo build`** — The solution assumes the Rust toolchain and Dioxus build toolchain are already set up.
7. **Session "turns" = message count per conversation** — The turn count on session cards is computed as `SELECT COUNT(*) FROM messages WHERE conversation_id = ?`.
8. **Material Symbols font is available** — The HTML template uses Google Material Symbols (`material-symbols-outlined` class). For a desktop Dioxus app, either the font must be bundled locally or the icons should be replaced with inline SVG or emoji. The solution assumes local fallback (emoji or text labels).

# Detailed Implementation

The overall plan is structured as 4 sequential vertical slices. Each slice produces a compilable, runnable increment.

## Slice 0: Preparation — Refactor router to shared layout

Before building sessions, extract the sidebar from `ChatScreen` into a shared `RootLayout` so it persists across all routes.

### Files to create/modify

**New: `src/layout.rs`** — Shared layout component with sidebar + `Outlet`
```
// Mirrors the HTML sidebar structure from code.html lines 46-104.
// Uses Dioxus Router's Outlet component to render child routes.
// The sidebar nav items are active-highlighted based on current route.
// Props: none (gets route from use_route)
```

**Modify: `src/app.rs`** — Refactor Route enum to use layout
```rust
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(RootLayout)]
        #[route("/")]
        SessionsPage {},
        #[route("/chat")]
        ChatScreen {},
        #[route("/settings")]
        SettingsScreen {},
        // Future: vault, hooks, help
    #[end_layout]
}
```

**Key Rust concepts explained:**
- `#[layout(RootLayout)]` wraps all nested routes in a component that renders an `<Outlet/>` — this is where child route content appears.
- `use_route::<Route>()` in the layout lets you read the current route to highlight the active nav item.
- The `Sidebar` component currently lives in `chat/widgets/sidebar.rs` — it should be promoted to `src/components/sidebar.rs` or kept in `layout.rs` since it's now shared.
- `provide_context(conn)` and `use_context_provider(|| Signal::new(ConversationStore::default()))` remain in `App` (the root component), not in the layout.

**Modify: `src/main.rs`** — Add `mod layout;`

**Action items:**
1. Create `src/layout.rs` with `RootLayout` component containing the persistent sidebar + `<Outlet::<Route>{}`
2. Move the sidebar UI from `chat/widgets/sidebar.rs` into `layout.rs` (the sidebar becomes part of the layout, not a child of ChatScreen)
3. Update `Route` enum in `app.rs`: add `SessionsPage` at `/`, keep `ChatScreen` at `/chat`, add `SettingsScreen` at `/settings`
4. Remove `Sidebar {}` from `ChatScreen` (it's now in the layout)
5. Compile and verify: app starts, sidebar renders, clicking nav items changes routes

## Slice 1: Static UI scaffold — Sessions page with empty folders/sessions panels

### Files to create

**New: `src/domains/sessions/mod.rs`**
```rust
pub mod state;
pub mod repo;
pub mod screen;
pub mod widgets;
```

**New: `src/domains/sessions/state.rs`** — Data models
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: i64,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub category: String,       // derived or stored tag
    pub description: String,    // first N chars of first message
    pub created_at: i64,
    pub updated_at: i64,
    pub turn_count: i64,
    pub folder_id: Option<String>,
    pub is_pinned: bool,
}

#[derive(Clone, PartialEq)]
pub struct SessionStore {
    pub folders: Vec<Folder>,
    pub active_folder_id: Option<String>,
    pub sessions: Vec<SessionSummary>,
    pub search_query: String,
}
```

**Key Rust concepts explained:**
- `#[derive(...)]` — Rust's auto-implementation of traits. `Debug` for printing, `Clone` for `.clone()`, `Serialize`/`Deserialize` for JSON/YAML, `PartialEq` for `==` comparisons.
- `pub` visibility — makes fields accessible outside the module. In Python everything is public by default; Rust is explicit.
- `Option<String>` — Rust's way of saying "this might be null". `None` means no parent (root folder). `Some(x)` means has a parent. Python uses `None`; Rust uses `Option`.

**New: `src/domains/sessions/repo.rs`** — Stub, no real DB yet
```rust
// Empty for Slice 1. Will be filled in Slice 2.
```

**New: `src/domains/sessions/screen.rs`** — Main page component
```
// Renders the split layout from code.html lines 129-276.
// Left: folder explorer panel (hardcoded sample folders for now)
// Right: bento grid of session cards (hardcoded sample sessions)
// Search bar + "New Folder" button (non-functional)
```

**New: `src/domains/sessions/widgets/mod.rs`**
```rust
pub mod folder_tree;
pub mod session_card;
pub mod bento_grid;
```

**New: `src/domains/sessions/widgets/folder_tree.rs`** — Folder explorer sidebar
```rust
#[component]
pub fn FolderTree(
    folders: Signal<Vec<Folder>>,
    active_folder_id: Signal<Option<String>>,
    on_create: EventHandler<()>,
    on_rename: EventHandler<(String, String)>,
    on_delete: EventHandler<String>,
) -> Element { ... }
```

**New: `src/domains/sessions/widgets/session_card.rs`** — Individual card
```
// Renders one session card from code.html lines 216-229.
// Props: session: SessionSummary, is_pinned: bool
// Visual: icon (top-left), category tag (top-right), title, description (line-clamp-2),
//         date + turn count (bottom), hover accent animation (top gradient bar)
```

**New: `src/domains/sessions/widgets/bento_grid.rs`** — Grid container
```
// Renders the grid from code.html lines 213-275.
// First pinned/featured card spans 2 columns (col-span-2).
// Responsive: grid-cols-1 md:grid-cols-2 lg:grid-cols-3
```

### Files to modify

**Modify: `src/domains/mod.rs`** — Add `pub mod sessions;`

**Modify: `src/main.rs`** — No change needed (layout handles routing)

**Modify: `src/app.rs`** — Add `SessionsPage` route variant
```rust
#[route("/")]
SessionsPage {},
```

### Action items:
1. Create all new files listed above with hardcoded sample data
2. Wire `SessionsPage` into the router
3. Compile and verify: navigating to `/` shows sessions layout with sample folders and cards
4. Make sure the sidebar nav correctly highlights "Sessions" as active

## Slice 2: SQLite folders table + folder CRUD in the store

### Files to modify

**Modify: `src/db.rs` or `src/domains/sessions/repo.rs`** — Database initialization
```sql
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id TEXT REFERENCES folders(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);

-- Migration: add folder_id to conversations
ALTER TABLE conversations ADD COLUMN folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL;
```

**Rust note:** `ALTER TABLE` will fail silently if column already exists — wrap in `let _ = conn.execute(...)` as the existing codebase does for `conversation_id` on `messages`.

**Modify: `src/domains/sessions/repo.rs`** — Full CRUD functions
```
pub fn init_db(conn: &Connection) -> rusqlite::Result<()>  -- creates folders table + migrates conversations
pub async fn get_all_folders(conn: Arc<Mutex<Connection>>) -> Result<Vec<Folder>, String>
pub async fn create_folder(conn: Arc<Mutex<Connection>>, folder: Folder) -> Result<(), String>
pub async fn rename_folder(conn: Arc<Mutex<Connection>>, id: String, new_name: String) -> Result<(), String>
pub async fn delete_folder(conn: Arc<Mutex<Connection>>, id: String) -> Result<(), String>  -- cascades: sets sessions' folder_id to NULL
pub async fn reorder_folder(conn: Arc<Mutex<Connection>>, id: String, new_order: i64) -> Result<(), String>
pub async fn get_sessions_by_folder(conn: Arc<Mutex<Connection>>, folder_id: Option<&str>) -> Result<Vec<SessionSummary>, String>
pub async fn update_session_folder(conn: Arc<Mutex<Connection>>, session_id: String, folder_id: Option<&str>) -> Result<(), String>
pub async fn get_folder_session_counts(conn: Arc<Mutex<Connection>>) -> Result<Vec<(String, i64)>, String>  -- for badge numbers
```

**Key Rust concepts explained:**
- `Arc<Mutex<Connection>>` — `Arc` (Atomic Reference Counting) lets multiple threads share ownership. `Mutex` ensures only one thread accesses the DB at a time. `Connection` is not `Send` (can't be transferred between threads safely), so `Arc<Mutex<>>` makes it shareable. Every DB function clones the `Arc` (cheap, just increments a counter) and passes it into `spawn_blocking`.
- `spawn_blocking` — Offloads blocking I/O (SQLite queries) to a thread pool so the UI thread stays responsive. Returns a `JoinHandle<Result<T, E>>`. The `.await` waits for the thread to finish.
- `Result<T, E>` — Rust's error handling. `Ok(T)` for success, `Err(E)` for failure. The `?` operator unwraps `Ok` or returns early with `Err`. This is like Python's `try`/`except` but encoded in the type system.
- `params![]` macro — rusqlite's way of binding parameters safely (avoids SQL injection). `?1`, `?2` are positional.

**Modify: `src/domains/sessions/state.rs`** — Add store methods
```rust
impl SessionStore {
    pub fn new() -> Self { ... }
    pub fn select_folder(&mut self, id: Option<String>) { ... }
    pub fn add_folder(&mut self, folder: Folder) { ... }
    pub fn remove_folder(&mut self, id: &str) { ... }
    pub fn rename_folder(&mut self, id: &str, new_name: String) { ... }
    pub fn set_sessions(&mut self, sessions: Vec<SessionSummary>) { ... }
    pub fn move_session(&mut self, session_id: &str, target_folder_id: Option<String>) { ... }
}
```

**Modify: `src/domains/sessions/screen.rs`** — Connect to real DB
```
- Add use_effect to load folders on mount (calls get_all_folders)
- Add use_effect to load sessions when active_folder_id changes (calls get_sessions_by_folder)
- Wire "New Folder" button to create_folder via spawn
- Wire rename dialog (inline editing on folder name click) to rename_folder
- Wire delete button with confirmation dialog to delete_folder
- The search bar filters sessions client-side by title
```

**Modify: `src/app.rs`** — Init sessions DB table
```rust
// In App(), after init_db for chat:
let _ = crate::domains::sessions::repo::init_db(&lock);
```

### Action items:
1. Add `init_db` function to sessions repo with CREATE TABLE + ALTER TABLE migrations
2. Implement all CRUD functions using `spawn_blocking` pattern
3. Add `SessionStore` with methods in state.rs
4. Update screen.rs to load from DB and wire up button handlers
5. Run a manual test: create a folder, rename it, delete it, verify via `sqlite3 lumina.db "SELECT * FROM folders;"`

## Slice 3: Session listing under a folder — real data in bento grid

### No new files needed — modify existing

**Modify: `src/domains/sessions/repo.rs`** — Add session query functions
```rust
pub async fn get_session_summaries(conn: Arc<Mutex<Connection>>, folder_id: Option<&str>) -> Result<Vec<SessionSummary>, String>
```
Implementation:
```sql
SELECT c.id, c.title, '' as category, 
       COALESCE((SELECT content FROM messages WHERE conversation_id = c.id ORDER BY created_at ASC LIMIT 1), '') as description,
       c.created_at, c.updated_at,
       (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as turn_count,
       c.folder_id,
       0 as is_pinned
FROM conversations c
WHERE (?1 IS NULL AND c.folder_id IS NULL) OR c.folder_id = ?1
ORDER BY c.updated_at DESC
```

**Note for Python dev:** This is a SQL correlated subquery. The `(SELECT COUNT(*) FROM messages WHERE conversation_id = c.id)` runs once per row. For a desktop app with hundreds of sessions this is fine; for thousands, a `LEFT JOIN` with `GROUP BY` would be faster.

**Modify: `src/domains/sessions/widgets/session_card.rs`** — Accept real data, format dates
```rust
// Format timestamps: use chrono to display "Oct 24, 2:30 PM" or "2 hours ago"
// Truncate description to first 200 chars with line-clamp-2
// Show category tag (could be derived from LLM or stored on conversation)
```

**Modify: `src/domains/sessions/widgets/folder_tree.rs`** — Show real session counts
```
// Each folder row shows a badge with session count (from get_folder_session_counts)
// Active folder gets highlighted background + colored icon
```

### Action items:
1. Implement `get_session_summaries` and `get_folder_session_counts` in repo.rs
2. Update screen.rs to load sessions when folder is selected
3. Add a `use_effect` that watches `active_folder_id` and reloads sessions
4. Verify: clicking a folder shows its sessions in the bento grid

## Slice 4: Drag-and-drop to move sessions between folders

### Approach

Given the WebView2 drag-event limitations (Dioxus #2167), use a two-pronged strategy:

**Primary: Mouse-event simulation** — Works on all platforms
```
1. Session cards get onmousedown handler that sets a "dragging" signal
2. onmousemove (on document/window) shows a ghost overlay at cursor
3. onmouseup checks if cursor is over a folder target (hit-test via element positions)
4. If valid target, call update_session_folder in DB + update local store
5. Visual feedback: folder highlights on dragover, session card dims while dragging
```

**Fallback: Context menu** — Right-click on session card shows "Move to folder" submenu
```
1. Right-click on session card opens a popover menu
2. Menu lists all folders
3. Clicking a folder triggers update_session_folder
```

**Files to modify:**

**Modify: `src/domains/sessions/widgets/session_card.rs`** — Add drag handlers
```rust
// New prop: on_drag_start: EventHandler<String>  (session_id)
// New prop: on_drag_end: EventHandler<String>     (session_id)
// The card sets a global "dragging_session" signal on mousedown
```

**Modify: `src/domains/sessions/widgets/folder_tree.rs`** — Add drop targets
```rust
// Each folder monitors the global "dragging_session" signal
// On mouseover while dragging: highlight the folder
// On click (after drag): execute move
```

**Modify: `src/domains/sessions/screen.rs`** — Wire DnD orchestration
```
// Coroutine that manages drag state: from mousedown on card to mouseup on folder
// On drop: call repo::update_session_folder, then refresh session list for both old and new folders
```

**Modify: `src/domains/sessions/state.rs`** — Add drag state
```rust
pub drag_active: bool,
pub drag_session_id: Option<String>,
pub drag_over_folder_id: Option<String>,
```

### Rust-specific implementation notes for DnD:

```rust
// In session_card.rs — the mousedown approach
let onmousedown = move |evt: MouseEvent| {
    // Only respond to left button (button 0)
    if evt.held_buttons() == dioxus::input::MouseButtonSet::PRIMARY {
        drag_state.write().drag_active = true;
        drag_state.write().drag_session_id = Some(session_id.clone());
    }
};
```

Since desktop Dioxus doesn't have native `mousemove` on unmounted elements, a better approach is to track the mouse position relative to folder elements using a combination of:
1. A transparent overlay div that appears when dragging starts (captures all mouse events)
2. `onmousemove` on this overlay to track cursor
3. Hit-testing against stored folder element rectangles (obtained via JS eval or manual layout tracking)

**Simpler fallback** for the MVP: Skip visual drag-and-drop and implement "Move to folder" via:
1. Right-click context menu on session card
2. Dropdown selector in the folder explorer header ("Move here" when a session is selected for moving)
3. A "Move" button on each session card that opens a folder picker dialog

### Action items:
1. Implement drag state machine in screen.rs or a dedicated hook
2. Add mouse-event handlers to session card and folder tree widgets
3. Implement context menu for "Move to folder"
4. Wire DB update and local store refresh on move completion
5. Test: move a session from one folder to another, refresh page (or reload), verify persistence

## Post-slice polish

1. **Search bar** — Filter sessions client-side by title match (case-insensitive `contains`)
2. **Category tags** — Store a `category` column on conversations table (default empty string), allow editing in session card context menu
3. **Pinned sessions** — Add `is_pinned` column to conversations, show pinned cards at top of grid spanning 2 columns
4. **Empty states** — Show "No sessions yet" with illustration when a folder is empty
5. **Error handling** — Wrap all DB calls in error handling with toast/alert display

# Trade-offs

| Dimension | Assessment |
|---|---|
| Complexity | **Low** — Each slice is self-contained; the pattern is already established by the `chat/` domain. No new dependencies beyond what's in `Cargo.toml`. |
| Time to implement | **8–12 hours** — 4 slices at ~2–3 hours each. Drag-and-drop (Slice 4) is the riskiest and could take 4+ hours if the mouse-event approach needs debugging. |
| Reversibility | **Easy** — Each slice produces a working increment. If drag-and-drop is too hard, the context-menu fallback is trivially reversible. Adding new columns via ALTER TABLE is non-destructive. |
| Risk level | **Low** — The vertical-slice approach means risk is contained within each slice. SQLite schema changes are backwards-compatible. The main risk is Dioxus drag-and-drop on Windows (research confirms this is a known issue). |
| Scalability | **Limited to single-user desktop scale** — SQLite with `spawn_blocking` handles thousands of folders and sessions. The `parent_id` tree approach supports arbitrary nesting depth via recursive CTEs. Not suitable for multi-user or server deployment. |
| Maintainability | **High** — Follows the exact same pattern as the existing `chat/` and `settings/` domains. A future developer (including the user 6 months later) can understand the structure immediately. Each file has a single responsibility. |
| Fit for user persona | **High** — The user is a Python developer learning Rust. The vertical-slice approach lets them see working results quickly (reinforcing learning), and the slice boundaries align with clear architectural boundaries that map to familiar concepts (models → state.rs, queries → repo.rs, views → screen.rs/widgets/). |

# Consequences

## Positive Outcomes

1. **Working increments at every step** — After each slice, the user can run the app and see real (if limited) functionality. This provides rapid feedback and motivation, which is especially valuable for someone learning a new language.
2. **Pattern reinforcement** — The user sees the same `state.rs` / `repo.rs` / `screen.rs` / `widgets/` pattern repeat across slices, reinforcing the Rust module system and DDD organization. By slice 4, the user will intuitively know where to add any new feature.
3. **No new dependencies** — Everything uses existing libraries (rusqlite, tokio, dioxus, chrono, uuid). No third-party drag-and-drop crate needed.
4. **Backwards-compatible DB** — The `ALTER TABLE` migration for `folder_id` on conversations won't break existing data. Sessions without a folder simply show in the "Uncategorized" root view.
5. **Shared layout refactor benefits all pages** — Once the sidebar is extracted into `RootLayout`, adding new pages (Context Vault, Hooks, etc.) becomes trivial — just add a route and component.

## Risks & Failure Modes

1. **Drag-and-drop may not work on Windows** — This is a confirmed Dioxus issue (source: Dioxus #2167). The mouse-event simulation approach is a workaround but may feel less polished than native drag-and-drop. Mitigation: the context-menu fallback must be implemented as a default, with visual drag-and-drop as progressive enhancement. The solution document explicitly budgets for this risk.
2. **`use_context` pattern becomes chat-specific** — Currently, `ConversationStore` is provided via `use_context_provider` in `App()`. If the sessions store also uses `use_context`, there could be name collisions or confusion. Mitigation: use separate context providers with distinct types, or use `Signal::global()` (like Settings does) for the sessions store to avoid nesting issues. The document recommends `Signal::global()` for `SessionStore` to follow the Settings pattern.
3. **`spawn_blocking` thread exhaustion** — If many DB operations fire simultaneously (e.g., rapid folder creation/deletion), the tokio thread pool could be exhausted. Mitigation: at desktop scale this is extremely unlikely; the default pool has 512 threads.
4. **Tailwind CSS classes missing in build** — The `dx` CLI's zero-config Tailwind watches `.rs` files for class usage. If a class is only used in an HTML template or a string, it might not be included in the build. Mitigation: verify all utility classes are used in `.rs` files or manually listed. The existing `tailwind.css` already has most needed classes.
5. **Folder deletion cascading** — If a folder with sessions is deleted, what happens to the sessions? The schema uses `ON DELETE SET NULL`, so sessions lose their folder_id. This is the safe default; the user may want `ON DELETE CASCADE` (delete all sessions in the folder). The document makes this explicit and recommends a confirmation dialog.

## Second-Order Effects

1. **Route design constrains future page structure** — Using `#[layout(RootLayout)]` means all pages share the sidebar. Adding a page that shouldn't have the sidebar (e.g., a full-screen login or modal) would require a separate route group without the layout.
2. **`folder_id` on conversations is permanent** — Once deployed to users, removing this column or changing the FK constraint requires a migration. The `ALTER TABLE` migration approach in this solution is additive-only, which is safe.
3. **The sessions page becomes the default landing page** — This changes the app's navigation flow: users now see "Session Vault" on launch instead of a chat input. The user requested this explicitly, but it means the "New Chat" flow now requires clicking the Chat nav item first.
4. **Folder CRUD creates a new UI interaction pattern** — The existing `ChatScreen` has inline rename (click the pencil icon on a conversation). The sessions page will have similar inline rename for folders. This establishes a pattern that future pages (Context Vault, Hooks) should follow for consistency.
5. **Drag-and-drop implementation becomes a reusable abstraction** — If the mouse-event drag simulation is extracted into a custom hook (e.g., `use_drag_drop`), it can be reused for other DnD scenarios (reordering sessions, organizing vault items). The document recommends extracting this in post-slice polish.

# Verdict

This solution is best for users who prioritize **low risk, incremental progress, and pattern consistency** and have the **patience to work through four sequential rounds of implementation**, each producing a working increment that reinforces their understanding of the Rust module system and Dioxus patterns.
