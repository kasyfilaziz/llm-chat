# Problem

Implement the Session Management page as the default landing page for the Lumina desktop app (Dioxus 0.7 + rusqlite). The page requires: a persistent sidebar (shared across all pages) with 7 nav items (Chat, Sessions/active, Context Vault, Hooks, Settings, Help, Logout); a directory/folder explorer panel (left) showing a tree of workspaces/folders with subfolders and session counts; a bento grid of session cards (right) for the selected folder with hover effects, tag badges, date/turn info, and a pinned/featured card spanning 2 columns; full CRUD for folders (create, rename, delete); drag-and-drop to move sessions between folders; SQLite storage for a `folders` table plus `folder_id` on the existing `conversations` (sessions) table; and session cards showing icon, category tag, title, description (line-clamp-2), date, turn count, and hover accent animation. "Solved" means the domain layer (schema + repo + state/store) compiles cleanly, the UI components render without crashes, folder CRUD works end-to-end, and sessions can be moved between folders via drag-and-drop.

# User Context & Persona

- Sole developer, coming from Python, learning Rust.
- Wants to ship quickly but also learn idiomatic Rust patterns.
- Works in a single-person codebase; no team coordination overhead.
- Has existing Dioxus 0.7 + rusqlite codebase with a "Pragmatic Hybrid DDD" pattern already established in the `chat` domain.
- Has dioxus-stores 0.7.9 already in Cargo.toml.
- Is comfortable reading the `chat` domain as a reference, but needs Rust-specific concepts explained (ownership, traits, Result/Option, async via `spawn_blocking`).

# External Research

## Dioxus 0.7 Store / State Management Patterns

**Source**: Dioxus 0.7 Launch Guide (https://newreleases.io/project/github/DioxusLabs/dioxus/release/v0.7.0) and Dioxus docs on state (https://dioxuslabs.com/learn/0.7/essentials/basics/context)
**Findings**:
- Dioxus 0.7 introduces "Stores: A new primitive for nested reactive state" via the `dioxus-stores` crate (already at 0.7.9 in the project). Stores are aware of data structures like `Vec` and `HashMap` — when you push an item, only the parts of the app depending on that change re-run.
- `use_context_provider` + `use_context` is the canonical pattern for shared global state. The existing codebase uses `Signal<T>` wrapped state structs (e.g., `ConversationStore`) provided via `use_context_provider`.
- `use_coroutine` is used for async actor-like message processing (the chat screen uses it for LLM streaming).
- Signals (`use_signal`) are cheap to clone and pass — the Dioxus docs explicitly recommend passing them as props rather than using `Mutex` or `Rc<RefCell>` in props (anti-pattern: "Avoid Interior Mutability in Props").

## Dioxus Anti-Patterns

**Source**: https://dioxuslabs.com/learn/0.7/guides/tips/antipatterns/
**Findings**:
- **Avoid Large Groups of State**: Don't put all state into one monolithic struct. Break into smaller signals (users, logged_in, warnings) or use memos for derived state. The existing codebase follows this with separate `ConversationStore` and local `use_signal` for messages.
- **Avoid Updating State During Render**: Use `use_effect` for side effects that depend on state changes. The chat screen does this correctly, loading messages when `active_id` changes.
- **Incorrect Iterator Keys**: Must use stable unique keys (not index) in RSX `for` loops. The existing sidebar uses `key: "{id}"` correctly.
- **Interior Mutability in Props**: Pass `Signal<T>`, not `Mutex<T>` or `Rc<RefCell<T>>`, as props to child components. The codebase already uses `Arc<Mutex<Connection>>` only at the top level for DB, wrapped via `use_context`.

## rusqlite Hierarchical Data (parent_id / Folder Tree)

**Source**: https://www.geeksforgeeks.org/sqlite/how-to-create-a-sqlite-hierarchical-recursive-query/ and SQLite CTE docs
**Findings**:
- The adjacency list pattern (`parent_id` column on the `folders` table) is the simplest and most maintainable for folder hierarchies. It uses `parent_id INTEGER REFERENCES folders(id) ON DELETE CASCADE` for self-referencing.
- To render a full folder tree efficiently, use SQLite's `WITH RECURSIVE` CTE. However, for small/medium numbers of folders (which the Lumina persona will have), simply loading all folders into memory and building the tree in Rust is simpler and avoids CTE complexity.
- Deletion of a folder should cascade to subfolders and sessions (SQLite foreign key with `ON DELETE CASCADE`).

## Dioxus DDD / Project Structure Patterns

**Source**: Existing `plan.md` (specs/002-tracer-bullet/plan.md) and the chat domain
**Findings**:
- The project uses "Pragmatic Hybrid DDD": domains live under `src/domains/<domain>/` with `mod.rs`, `state.rs`, `repo.rs`, `screen.rs`, `widgets/`.
- The existing `chat` domain is the pattern to follow exactly for the new `sessions` domain.
- Shared components go in `src/components/`. Shared widgets (like the sidebar) are domain-specific in `chat/widgets/sidebar.rs` — but the user's requirement says the sidebar is persistent across ALL pages. So the sidebar should be extracted into a shared location or provided from the app root layout.

# Solution Summary

Create a new `src/domains/sessions/` domain following the exact file structure of `chat/`: schema migration in `repo.rs`, data models and store in `state.rs`, page component in `screen.rs`, and sub-widgets in `widgets/`. The DB layer adds a `folders` table and a `folder_id` column to the existing `conversations` table. The Dioxus UI is a new `SessionsScreen` component with a folder tree panel, a bento grid of session cards, inline CRUD for folders, and drag-and-drop for moving sessions between folders. The sidebar is extracted to a shared location at `src/components/sidebar.rs` and rendered from the `App` root layout so it persists across routes.

# Assumptions

1. **The user has `dioxus-stores = "0.7.9"` in Cargo.toml** — already confirmed by reading the file. This crate is used for the folder tree store.
2. **The existing `conversations` table represents "sessions"** — the user's request uses the terms interchangeably. The schema will add `folder_id TEXT` (nullable) to it.
3. **The user's existing `Move` button is placeholder** — the template has a "Move" button per session card; this solution implements actual drag-and-drop (HTML5 Drag and Drop API) for moving sessions between folders.
4. **The user wants `dx serve --desktop` for development** — the project is Dioxus desktop mode (`features = ["desktop", "router"]`).
5. **Folder count is small (<50)** — no need for pagination or lazy loading in the folder tree. The Rust-side tree construction is fine.
6. **The sidebar nav items (Chat, Sessions, Context Vault, Hooks, Settings, Help, Logout) are static links for now** — only Chat and Sessions have routes; the rest are placeholders.
7. **The sessions page is the default landing page (`/`)** — the app route changes from `ChatScreen` at `"/"` to `SessionsScreen` at `"/"` with Chat moved to `"/chat"`.
8. **The user is on Dioxus 0.7 with the `launch` + `Router` pattern** — confirmed by `app.rs`.
9. **Drag-and-drop uses HTML5 native DnD API** — works in Dioxus desktop via WebView. No external crate needed.
10. **CSS classes follow the same Tailwind convention as the existing code** — the template uses a custom color palette; this solution maps those to Tailwind classes compatible with what's already in `main.css`.

# Detailed Implementation

## Step 1: Database Migration — `folders` table + `folder_id` on `conversations`

Create `src/domains/sessions/repo.rs` following `chat/repo.rs` exactly:

- `init_db(conn)` that creates:
  - `folders` table: `id TEXT PRIMARY KEY`, `name TEXT NOT NULL`, `parent_id TEXT REFERENCES folders(id) ON DELETE CASCADE`, `created_at INTEGER NOT NULL`, `updated_at INTEGER NOT NULL`
  - Adds `folder_id TEXT` column to `conversations` (using `ALTER TABLE ... ADD COLUMN` with the `let _ =` ignore-if-exists pattern from the existing chat repo)

- CRUD functions (all async, using `tokio::task::spawn_blocking` — **Rust concept**: `spawn_blocking` moves a closure to a thread pool so SQLite's synchronous C API doesn't block the Dioxus main thread. The `move` keyword captures variables by value into the closure. The `|e| e.to_string()` pattern converts `rusqlite::Error` to `String` via the `Display` trait — this is a common Rust idiom for error propagation in single-person projects where you don't need custom error types):

  ```rust
  // Rust concept: Result<T, E> is an enum — Ok(T) or Err(E).
  // The ? operator unwraps Ok or returns early with Err.
  // .map_err(|e| e.to_string()) converts the error type.

  pub async fn get_all_folders(conn: Arc<Mutex<Connection>>) -> Result<Vec<Folder>, String> {
      tokio::task::spawn_blocking(move || {
          let conn = conn.lock().map_err(|e| e.to_string())?;
          let mut stmt = conn.prepare("SELECT id, name, parent_id, created_at, updated_at FROM folders ORDER BY name")
              .map_err(|e| e.to_string())?;
          let iter = stmt.query_map([], |row| {
              Ok(Folder {
                  id: row.get(0)?,
                  name: row.get(1)?,
                  parent_id: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
              })
          }).map_err(|e| e.to_string())?;
          let mut folders = Vec::new();
          for f in iter {
              folders.push(f.map_err(|e| e.to_string())?);
          }
          Ok(folders)
      }).await.map_err(|e| e.to_string())?
  }
  ```

  Same pattern for: `create_folder`, `rename_folder`, `delete_folder` (with cascade), `move_session` (UPDATE conversations SET folder_id = ? WHERE id = ?), `get_session_count_by_folder`.

**Rust-specific note on `folder_id`**: Use `Option<String>` for the field type since root-level sessions have no folder. SQLite `NULL` maps to `Option::None`. The `row.get::<_, Option<String>>(idx)?` pattern extracts nullable columns.

## Step 2: State Models — `src/domains/sessions/state.rs`

Following `chat/state.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,  // None = root folder
    pub created_at: i64,
    pub updated_at: i64,
}

impl Folder {
    pub fn new(name: &str, parent_id: Option<&str>) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
        }
    }
}
```

**Rust concept**: `Option<String>` is equivalent to Python's `Optional[str]` or `str | None`. The `.map(|s| s.to_string())` method transforms `Option<&str>` to `Option<String>`.

The `SessionStore` (analogous to `ConversationStore`):

```rust
#[derive(Default, Clone, PartialEq)]
pub struct SessionStore {
    pub folders: Vec<Folder>,
    pub selected_folder_id: Option<String>,
    pub folder_session_counts: Vec<(String, i64)>,  // (folder_id, count)
}
```

## Step 3: Register the new domain module

Edit `src/domains/mod.rs` — add `pub mod sessions;`

## Step 4: Integrate migration into app startup

Edit `src/app.rs`:
- Import `crate::domains::sessions::repo::init_db as init_sessions_db`
- In the `use_hook` that creates the connection, call `init_sessions_db(&lock)?` after the existing `init_db(&lock)?`

## Step 5: Provide the SessionStore as context

In `src/app.rs`, add:
```rust
use_context_provider(|| Signal::new(SessionStore::default()));
```

## Step 6: Extract sidebar to shared component

The existing `Sidebar` in `chat/widgets/sidebar.rs` is conversation-specific. The user's requirement has sidebar nav items (Chat, Sessions - active, Context Vault, Hooks, Settings, Help, Logout). This sidebar is persistent — it should contain nav links and NOT conversation list.

Create `src/components/sidebar.rs`:

```rust
#[component]
pub fn Sidebar() -> Element {
    rsx! {
        nav { class: "w-64 bg-[#faf6f0] dark:bg-stone-950 h-screen border-r border-stone-200/40 flex flex-col py-4 shrink-0",
            // Header: "Terra Workspace" branding
            // Nav items: Chat, Sessions(active), Context Vault, Hooks, Settings, Help, Logout
            // Uses Router Link for Chat("/chat") and Sessions("/")
        }
    }
}
```

The sidebar uses `Link { to: Route::Chat {} }` and `Link { to: Route::Sessions {} }` for navigation. Active state is determined by `use_route()`. The old `chat/widgets/sidebar.rs` becomes an inline panel within the `ChatScreen` or is removed.

## Step 7: Update routing — Sessions page as default

In `src/app.rs`, change the `Route` enum:

```rust
#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Sessions {},
    #[route("/chat")]
    ChatScreen {},
}
```

Also wrap Router inside a layout that includes the `Sidebar` so it's persistent across all pages. Use Dioxus's layout route or render sidebar alongside the router outlet:

```rust
rsx! {
    document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
    div { class: "flex h-screen",
        Sidebar {}
        Router::<Route> {}
    }
}
```

Remove the individual `Sidebar {}` calls from `ChatScreen`.

## Step 8: Build `src/domains/sessions/screen.rs` — the Sessions page

This is the main screen. It has:

### State:
```rust
let mut store = use_context::<Signal<SessionStore>>();
let conn = use_context::<Arc<Mutex<Connection>>>();
let mut folder_tree = use_signal(Vec::<FolderNode>::new);  // derived tree
```

**Rust concept**: `Vec<FolderNode>` is a tree built from the flat `Vec<Folder>` by nesting children under parents. `FolderNode` is a recursive struct:

```rust
#[derive(Clone)]
struct FolderNode {
    folder: Folder,
    children: Vec<FolderNode>,
    depth: u32,
}
```

### Folder Tree Panel (left):
- Load all folders via `get_all_folders()` in a `use_effect` on mount.
- Build tree in Rust: group folders by `parent_id`, assign children. **Not using SQL CTE** because iterating a <50-item vec in Rust is simpler.
- Render recursively: `fn render_tree(nodes: &[FolderNode], depth: u32) -> Element`.
- Each folder shows icon + name + session count badge.
- Clicking selects `store.selected_folder_id`.
- Inline rename: double-click or "Rename" button reveals an `<input>` (same pattern as the existing sidebar's inline rename).
- Delete button with confirmation (use existing `show_confirm` pattern from `ChatScreen`).
- "New Folder" button in the top action bar (from the HTML template) opens an input for name + optional parent selection.

### Create Folder Flow:
1. User clicks "New Folder" → `editing_new_name` signal becomes `Some("")`
2. Input appears inline at bottom of tree with autofocus
3. On Enter: `create_folder(conn, Folder::new(&name, parent_id)).await` → refresh tree
4. On Escape: cancel

### Rename Folder Flow:
1. User clicks "Rename" icon → `editing_folder_id` signal sets
2. Input replaces name text, pre-filled
3. On Enter: `rename_folder(conn, id, new_name).await` → update tree
4. On Escape: cancel

### Delete Folder Flow:
1. User clicks "Delete" → `confirm_delete_id` signal sets
2. Confirmation dialog appears (same pattern as ChatScreen's clear confirmation)
3. On confirm: `delete_folder(conn, id).await` → SQLite cascades to subfolders + sessions
4. Update local state: remove folder + reset selection if needed

### Bento Grid of Session Cards (right):
- Load conversations filtered by `selected_folder_id` (or all if `None`).
- Use `get_conversations_by_folder` repo function — basically the same as `get_all_conversations` but with `WHERE folder_id = ?1` (or `WHERE folder_id IS NULL` for root).
- Grid: `<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">`.
- Each card is a `SessionCard` component with props: `session: Conversation`, `category: String`, `is_pinned: bool`.

### Session Card Component:
```rust
#[component]
fn SessionCard(session: Conversation, category: String, is_pinned: bool) -> Element {
    // Decorative top accent (gradient bar, opacity 0 → 1 on hover)
    // Icon (from session category)
    // Category tag badge
    // Title (truncated)
    // Description (line-clamp-2)
    // Date + turn count in footer
    // If pinned: col-span-1 md:col-span-2, pinned icon, different bg
    // draggable="true" for drag-and-drop
}
```

**Rust concept**: Props are defined with the `#[component]` macro. The function takes a struct-like argument list. PartialEq is derived for props automatically by the macro.

### Drag-and-Drop:
- Session cards have `draggable: "true"`, `ondragstart` sets the session ID in `evt.data_transfer()`.
- Folder items have `ondragover` (prevent default to allow drop), `ondrop` that reads the session ID and calls `move_session(conn, session_id, target_folder_id).await`.
- **Rust concept**: Event handlers use `move |evt| { ... }` closures. `evt.stop_propagation()` prevents parent handlers from firing. `evt.data_transfer()` returns a `DataTransfer` object — but Dioxus's desktop WebView may not fully support it; a fallback is to use local state (e.g., a `dragging_session_id` signal) instead.

### Pinned/Featured Session:
- The first pinned session (or a manually selected one) spans 2 columns: `class: "col-span-1 md:col-span-2 lg:col-span-2"`.
- Store pinned state in the DB: add `is_pinned INTEGER DEFAULT 0` to the `conversations` table.

## Step 9: Build widgets

Create `src/domains/sessions/widgets/mod.rs`:
```rust
pub mod session_card;
pub mod folder_tree;
```

`session_card.rs`: The `SessionCard` component.
`folder_tree.rs`: A recursive `FolderTree` component that takes a `Vec<FolderNode>` and renders the expandable/collapsible tree.

## Step 10: Wire it all together

The full `SessionsScreen` follows the `chat/screen.rs` pattern:
- `use_effect` loads folders on mount
- Another `use_effect` loads sessions when `selected_folder_id` changes
- Action handlers send events through a `use_coroutine` for DB writes (or simpler: just call async functions directly from event handlers using `spawn` — the existing sidebar shows this pattern)

**Rust concept**: `spawn { async move { ... } }` runs a fire-and-forget async task. Use it for DB writes where you don't need to track completion. Use `use_coroutine` when you need to sequence operations or handle streaming responses.

# Trade-offs

| Dimension          | Assessment                                                                         |
|--------------------|------------------------------------------------------------------------------------|
| Complexity         | Medium — 5 new files, one schema migration, one extracted sidebar. Low Rust complexity because it follows existing patterns exactly. |
| Time to implement  | ~6-10 hours for a single session (one focused day).                               |
| Reversibility      | Easy — folders table and folder_id column are backward-compatible. The existing chat still works if Sessions is removed. |
| Risk level         | Low-Medium — DnD in Dioxus WebView can be flaky (depends on platform WebView). Inline editing state management can cause subtle bugs (use_effect order). |
| Scalability        | Will handle thousands of sessions per folder with pagination (add LIMIT/OFFSET later). Folder tree is in-memory, fine for <50 folders. |
| Maintainability    | High — clean DDD separation. A new developer can understand the sessions domain by reading chat/ side-by-side. |
| Fit for user persona | High — follows the exact same pattern as the existing chat domain, so the user (who is learning Rust from Python) has a concrete reference. No new Rust concepts beyond what's already in the codebase. |

# Consequences

## Positive Outcomes

- **Pattern consistency**: The user learns one DDD pattern (`state.rs` → `repo.rs` → `screen.rs` → `widgets/`) and can apply it to future domains (settings, vault, hooks).
- **SQLite migration learning**: The `ALTER TABLE ... ADD COLUMN` pattern with `let _ =` shows the user a pragmatic Rust idiom for handling schema evolution without formal migration tools.
- **Reusable sidebar**: Extracting the sidebar to `src/components/` teaches the user that shared UI belongs outside domain boundaries.
- **Drag-and-drop in Dioxus**: Proves that HTML5 DnD works in Dioxus desktop, which is valuable knowledge for the user's future feature work.
- **SQL recursive vs in-memory trade-off**: The user sees a concrete decision point where "load all, build tree in Rust" is chosen over "WITH RECURSIVE CTE", teaching them to evaluate simplicity vs. query complexity at small scale.

## Risks & Failure Modes

- **Drag-and-drop WebView compatibility**: The Dioxus desktop renderer uses the system WebView (WebKit on macOS, WebView2 on Windows). `evt.data_transfer()` behavior can vary. **Mitigation**: Use a local `dragging_session_id: Signal<Option<String>>` as a fallback, and set/get it directly. Add a "Move to folder" context menu as an alternative path.
- **Tree re-render performance**: Every folder CRUD operation causes a full tree rebuild. For <50 folders this is negligible. **Mitigation**: If it becomes a problem, use `use_memo` to cache the derived tree and only recompute when the flat `folders` vec reference changes.
- **Inline editing state race**: The `editing_folder_id` and `editing_new_name` signals could conflict if the user clicks "New Folder" while another is being renamed. **Mitigation**: Use an enum for editing state: `enum EditingState { None, New(Option<String>), Rename(String, String) }` where the tuple is `(folder_id, current_name)`.
- **`folder_id` NULL confusion**: `Option<String>` in Rust maps to `NULL` in SQLite, but `""` (empty string) does NOT map to `NULL`. All code paths must consistently use `None` (not `Some("")`) for root-level sessions. **Mitigation**: Add a `const ROOT_FOLDER: &str = "__root__"` sentinel value if Option becomes cumbersome, but prefer `Option<String>` for correctness.

## Second-Order Effects

- **Six months later**: The user adds server-side sync or multi-user support. The flat-folder model (adjacency list) is easy to replicate to a remote DB. The in-memory tree construction may need to move server-side for larger shared workspaces, using the same `WITH RECURSIVE` CTE that was deliberately skipped here.
- **The `chat` domain becomes redundant**: If sessions are just conversations with folders, the `ConversationStore` and `SessionStore` may need to be unified. This is a natural refactoring point — merge `chat` and `sessions` into `conversations` with folder support. The clean DDD boundaries make this merge easier.
- **Codebase grows to 10+ domains**: The user will have a `domains/` directory with many small modules. This is fine in Rust (fast compile times with small files). The user should watch for cargo-chef or sccache when the project grows.
- **The "persistent sidebar" assumption breaks**: If a future route needs a completely different layout (e.g., a popup or settings wizard), the current layout (sidebar + router outlet) won't work. The user should add layout routes at that point: `Route::Layout(SidebarLayout { child: Route })` pattern.

# Verdict

This solution is best for users who prioritize codebase consistency and maintainability over rapid visual prototyping, and who have at least a few hours of focused time to work through the domain layer before seeing the UI render.
