# Problem

Implement a Session Management page as the default landing page for the Lumina Dioxus app. The page needs: a persistent sidebar with nav items (Chat, Sessions [active], Context Vault, Hooks, Settings, Help, Logout), a directory/folder explorer panel showing a tree of workspaces with subfolders and session counts, a bento grid of session cards (with a pinned card spanning 2 columns), full CRUD for folders (create, rename, delete), drag-and-drop to move sessions between folders, and SQLite persistence for both folders and the `folder_id` on sessions. The user is a solo Rust learner coming from Python who wants to see visual progress quickly.

# User Context & Persona

- Sole developer, coming from Python, learning Rust
- Already has a working Dioxus 0.7 app with chat, settings, MCP, and sidebar
- Familiar with the project's patterns: `use_signal`, `use_context`, `Arc<Mutex<Connection>>` for DB, `rsx!` with Tailwind, screens in `domains/{domain}/screen.rs`
- Wants immediate visual feedback to stay motivated — the "visual-fast" approach suits this
- Has an existing HTML template at `assets/stich/session_management_updated_layout/code.html` to convert

# External Research

1. **Dioxus 0.7 RSX syntax**: Official docs at https://dioxuslabs.com/learn/0.7/essentials/ui/rsx — attributes in `class: "..."` format, text children as `"string"`, event handlers as `onclick: move |evt| ...`. The `dx translate` CLI tool can convert HTML to RSX automatically: `dx translate --file code.html` (https://dioxuslabs.com/learn/0.7/guides/tools/translate/).

2. **Dioxus 0.7 Signals and Context**: https://dioxuslabs.com/learn/0.7/essentials/basics/context/ — `use_context::<T>()` to consume, `use_context_provider(|| T::new())` to provide, signals bundled in structs with `Copy` derive. Global signals via `static X: GlobalSignal<T> = Signal::global(|| ...)` for truly global state.

3. **Tailwind CSS with Dioxus**: https://dioxuslabs.com/learn/0.7/guides/utilities/tailwind/ — uses `class:` attribute in RSX, needs `@import "tailwindcss"` in `input.css` and `@source "./src/**/*.{rs,html,css}"` for class scanning. VSCode extension regex: `"class: \"(.*)\""`.

4. **Material Symbols with Dioxus**: `dioxus-material-icons` crate (https://lib.rs/crates/dioxus-material-icons) provides `MaterialIcon { name: "settings" }` component. Alternatively, use raw HTML `<span class="material-symbols-outlined">` with the Google Fonts stylesheet for the most flexibility.

5. **Drag and Drop in Dioxus**: Dioxus has `DragEvent` in its events module (https://docs.rs/dioxus/latest/dioxus/events/index.html) — supports `ondragstart`, `ondragover`, `ondrop` on elements. Uses HTML5 Drag and Drop API which works in WebView.

6. **HTML to Dioxus Component conversion**: The `dx translate --component` flag generates a full component from HTML (https://dioxuslabs.com/learn/0.7/guides/tools/translate/). For complex templates, convert in sections.

# Solution Summary

**Template-First Reactive Scaffold (Visual-Fast)** — Convert the HTML template directly into Dioxus RSX components, rendering the entire layout statically first (sidebar, directory tree, bento grid). Then incrementally replace hardcoded data with signals and store bindings, and finally add SQLite persistence for folders and folder_id on sessions. The core mechanism is starting with a fully rendered UI that looks like the final product, then wiring up reactivity from the outside in.

# Assumptions

- The existing `Arc<Mutex<Connection>>` pattern (from `src/db.rs`) will be reused for SQLite access
- The existing `Button` and `Input` components from `src/components/mod.rs` are available for reuse
- The project already has Tailwind CSS configured (there's an `assets/tailwind.css`)
- The HTML template's Material Symbols approach (Google Fonts stylesheet + `<span>` tags) will be used rather than a Rust crate wrapper, for maximum fidelity
- The sidebar will be shared as a common component (extracted or co-located with a nav module)
- The `dioxus-router` from Cargo.toml is available for routing (used as `Router::<Route>` in `app.rs`)
- Drag-and-drop will use the HTML5 Drag and Drop API (ondragstart/ondragover/ondrop) which works in Dioxus webview/desktop
- The user has `dx` CLI installed and can run `dx serve --desktop` for testing
- `folder_id` column already exists or can be added to the conversations table (currently Phase 1->2 migration shows `ALTER TABLE messages ADD COLUMN conversation_id` pattern)
- The existing `ConversationStore` will be extended or replaced with a `SessionStore` that includes folder data

# Detailed Implementation

## Phase 1: Directory Structure & Module Setup

Create the sessions domain following the existing `chat` domain structure:

```
src/domains/sessions/
├── mod.rs          # pub mod screen; pub mod state; pub mod repo;
├── screen.rs       # SessionScreen component — main page
├── state.rs        # Folder, SessionStore structs + signals
├── repo.rs         # SQLite CRUD for folders + folder_id on conversations
└── widgets/
    ├── mod.rs       # pub mod sidebar; pub mod folder_tree; pub mod session_card;
    ├── sidebar.rs   # Persistent sidebar (shared nav)
    ├── folder_tree.rs  # Directory explorer panel (left)
    └── session_card.rs # Bento grid card component (right)
```

### Add to module tree

In `src/domains/mod.rs`, add:
```rust
pub mod sessions;
```

## Phase 2: Define State Models (state.rs)

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

/// A folder/workspace that can contain conversations (sessions)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,   // for nesting; None = root
    pub created_at: i64,
    pub updated_at: i64,
    pub session_count: i64,          // denormalized, computed on load
}

impl Folder {
    pub fn new(name: &str, parent_id: Option<String>) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            parent_id,
            created_at: now,
            updated_at: now,
            session_count: 0,
        }
    }
}

/// The session (conversation) model extended with folder_id
/// This can be merged with the existing Conversation struct in chat/state.rs
/// or kept separate. For simplicity, extend the existing one.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub category: String,            // "Drafting", "Development", etc.
    pub description: String,         // first ~120 chars of first message
    pub created_at: i64,
    pub updated_at: i64,
    pub turn_count: i64,             // message count
    pub is_pinned: bool,             // featured card
}

/// Reactive store for the sessions page
#[derive(Clone, Copy)]
pub struct SessionStore {
    pub folders: Signal<Vec<Folder>>,
    pub sessions: Signal<Vec<Session>>,
    pub selected_folder_id: Signal<Option<String>>,
    pub search_query: Signal<String>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            folders: Signal::new(Vec::new()),
            sessions: Signal::new(Vec::new()),
            selected_folder_id: Signal::new(None),
            search_query: Signal::new(String::new()),
        }
    }

    /// Get sessions filtered by selected folder and search query
    pub fn filtered_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read();
        let folder_id = self.selected_folder_id.read().clone();
        let query = self.search_query.read().to_lowercase();

        sessions.iter()
            .filter(|s| folder_id.as_ref().map_or(true, |fid| s.folder_id.as_ref() == Some(fid)))
            .filter(|s| query.is_empty() || s.title.to_lowercase().contains(&query) || s.description.to_lowercase().contains(&query))
            .cloned()
            .collect()
    }

    /// Get root-level folders
    pub fn root_folders(&self) -> Vec<Folder> {
        self.folders.read().iter()
            .filter(|f| f.parent_id.is_none())
            .cloned()
            .collect()
    }

    /// Get child folders for a given parent
    pub fn child_folders(&self, parent_id: &str) -> Vec<Folder> {
        self.folders.read().iter()
            .filter(|f| f.parent_id.as_deref() == Some(parent_id))
            .cloned()
            .collect()
    }
}
```

## Phase 3: SQLite Repository (repo.rs)

Extend the `init_db` function pattern from `chat/repo.rs`:

```rust
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use crate::domains::sessions::state::{Folder, Session};

pub fn init_sessions_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
        );

        -- Add folder_id to existing conversations table
        ALTER TABLE conversations ADD COLUMN folder_id TEXT;
        ALTER TABLE conversations ADD COLUMN category TEXT NOT NULL DEFAULT 'General';
        ALTER TABLE conversations ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0;
        "
    )?;
    Ok(())
}
```

**CRUD Operations** (each follows the `tokio::task::spawn_blocking` pattern from `chat/repo.rs`):

- `get_all_folders(conn)` — SELECT * FROM folders ORDER BY name
- `create_folder(conn, folder)` — INSERT INTO folders
- `rename_folder(conn, id, new_name)` — UPDATE folders SET name = ?1, updated_at = ?2
- `delete_folder(conn, id)` — DELETE FROM folders WHERE id = ?1 (cascade handles children)
- `move_session_to_folder(conn, session_id, folder_id)` — UPDATE conversations SET folder_id = ?1
- `get_sessions_by_folder(conn, folder_id)` — SELECT with JOIN to count messages for turn_count

**Important Rust concept**: Each function is `async`, takes `Arc<Mutex<Connection>>`, and uses `spawn_blocking` because `rusqlite::Connection` is not `Send` in a way that works with async runtimes. The inner closure locks the mutex, runs the query, and returns the result.

```rust
pub async fn get_all_folders(conn: Arc<Mutex<Connection>>) -> Result<Vec<Folder>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT f.id, f.name, f.parent_id, f.created_at, f.updated_at,
                    (SELECT COUNT(*) FROM conversations WHERE folder_id = f.id) as session_count
             FROM folders f ORDER BY f.name"
        ).map_err(|e| e.to_string())?;

        let iter = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                session_count: row.get(5)?,
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

## Phase 4: Convert HTML Template → RSX (The Core of Solution 2)

This is the defining step of the Template-First approach. Rather than building the UI from scratch, we:

1. **Use `dx translate`** to convert the HTML template sections to RSX:
   ```bash
   dx translate --file assets/stich/session_management_updated_layout/code.html --component
   ```
   This generates a single Rust file with the whole layout as one component.

2. **Split into widgets**: Take the generated RSX and manually split it into:
   - `sidebar.rs` — The `<nav>` sidebar (navigation tabs + user info)
   - `folder_tree.rs` — The `<aside>` directory explorer panel
   - `session_card.rs` — Individual bento grid card
   - `screen.rs` — The top-level `div.flex` that assembles everything

3. **Key RSX conversion rules** for a Python developer learning Rust:
   - HTML `<div class="foo bar">` → RSX `div { class: "foo bar", ... }`
   - HTML `<span>text</span>` → RSX `span { "text" }`
   - HTML `<img src="..." alt="...">` → RSX `img { src: "...", alt: "..." }`
   - HTML `onclick="fn()"` → RSX `onclick: move |_| { /* Rust code */ }`
   - HTML data attributes `data-foo="bar"` → RSX `data_foo: "bar"` (hyphens become underscores)
   - HTML self-closing tags → RSX empty element `input { class: "...", value: "..." }`
   - Tailwind classes keep their hyphenated form in the string: `class: "flex items-center gap-2"`

4. **Material Symbols in RSX**: Use raw `<span>` tags with the Google Fonts stylesheet:
   ```rust
   // In app.rs or screen.rs, add the stylesheet link
   document::Link { rel: "stylesheet", href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL@20..48,100..700,0..1&display=swap" }
   ```
   Then use: `span { class: "material-symbols-outlined", "folder" }`

5. **For the folder tree with subfolders**, create a recursive component pattern:
   ```rust
   #[component]
   fn FolderNode(folder: Folder, depth: u32) -> Element {
       rsx! {
           li {
               button { class: "w-full flex items-center justify-between px-3 py-2.5 rounded-lg ...",
                   div { class: "flex items-center gap-2",
                       span { class: "material-symbols-outlined text-primary icon-fill", "folder" }
                       span { "{folder.name}" }
                   }
                   span { class: "text-xs ...", "{folder.session_count}" }
               }
               // Recursive subfolders (need store access)
               ...
           }
       }
   }
   ```
   **Rust note for Python devs**: Recursive components need to use `use_context` to access the store rather than passing it as a prop, because Rust's type system makes recursive prop types complex.

## Phase 5: Wire Up State → Static Layout

Replace hardcoded data with signal reads. The order matters for visual feedback:

1. **First, make folder names dynamic**: Replace the hardcoded "Project Horizon", "Client Memos" etc. with `store.folders().iter().map(...)`
2. **Make session cards dynamic**: Replace hardcoded card data with `store.filtered_sessions().iter().map(...)`
3. **Add interactivity**: Wire `onclick` on folder buttons to `store.selected_folder_id.set(...)`

**Pattern to use** — the store is provided in `app.rs` and consumed via `use_context`:
```rust
// In screen.rs
let store = use_context::<Signal<SessionStore>>();

// In sub-components (sidebar, folder_tree, session_card)
let store = use_context::<Signal<SessionStore>>();
```

## Phase 6: Add SQLite Persistence

In the `SessionScreen` component, load data on mount via `use_effect`:
```rust
let conn = use_context::<Arc<Mutex<Connection>>>();
let store = use_context::<Signal<SessionStore>>();

let load_conn = conn.clone();
use_effect(move || {
    let conn = load_conn.clone();
    spawn(async move {
        if let Ok(folders) = get_all_folders(conn).await {
            store.write().folders.set(folders);
        }
    });
});
```

For CRUD operations, call the repo functions inside event handlers:
```rust
// Create folder
button {
    onclick: move |_| {
        let new_folder = Folder::new("New Folder", None);
        let folder_clone = new_folder.clone();
        let conn_clone = conn.clone();
        store.write().folders.write().push(new_folder);
        spawn(async move {
            let _ = create_folder(conn_clone, folder_clone).await;
        });
    },
    "+ New Folder"
}
```

## Phase 7: Drag-and-Drop

Use the HTML5 Drag and Drop API. In Dioxus, this maps to event handlers on RSX elements:

```rust
// On session cards (drag source):
div {
    class: "...",
    draggable: "true",
    ondragstart: move |evt| {
        evt.data_transfer().set_data("text/plain", &session.id);
        evt.data_transfer().set_effect("move");
    },
    // ... card content
}

// On folder items (drop target):
button {
    class: "...",
    ondragover: move |evt| {
        evt.prevent_default();  // Allow drop
        evt.data_transfer().set_effect("move");
    },
    ondrop: move |evt| {
        let session_id = evt.data_transfer().get_data("text/plain");
        let folder_id = folder.id.clone();
        let conn_clone = conn.clone();
        spawn(async move {
            let _ = move_session_to_folder(conn_clone, session_id, Some(folder_id)).await;
        });
    },
    // ... folder content
}
```

**Rust note**: `evt.data_transfer()` returns a `DataTransfer` object. Dioxus wraps the JS API. The methods `set_data`, `get_data`, and `set_effect` work the same as in standard web APIs.

## Phase 8: Hook Into App Router

Update `src/app.rs` to add the sessions route as the default landing page:

```rust
#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    SessionScreen {},   // Changed from ChatScreen to SessionScreen
    #[route("/chat")]
    ChatScreen {},
    // ... other routes
}

pub fn App() -> Element {
    // ... existing DB init ...
    // Provide the SessionStore alongside the existing ConversationStore
    use_context_provider(|| Signal::new(crate::domains::sessions::state::SessionStore::new()));
    // Initialize folders DB table
    {
        let lock = conn.lock().expect("Failed to lock DB");
        crate::domains::sessions::repo::init_sessions_db(&lock).expect("Failed to init sessions DB");
    }
    // ...
}
```

**For the Python dev**: The `#[derive(Clone, Routable, Debug, PartialEq)]` macro auto-generates routing code. The `#[route("/")]` attribute marks which component renders at that path. The `Router::<Route> {}` component in `rsx!` handles all the URL matching.

## Phase 9: Extract Shared Sidebar

Since the sidebar is "persistent across all pages", extract the nav HTML into a shared component at `src/components/sidebar.rs` or at `src/domains/sessions/widgets/sidebar.rs`. Either way, reuse it in both `ChatScreen` and `SessionScreen`.

**Approach**: Define a `Sidebar` component that takes an `active_tab` prop:

```rust
#[derive(PartialEq, Clone)]
pub enum NavTab {
    Chat,
    Sessions,
    ContextVault,
    Hooks,
    Settings,
    Help,
    Logout,
}

#[component]
pub fn Sidebar(active_tab: NavTab) -> Element {
    rsx! {
        nav { class: "w-64 ...",
            // User info
            // Nav items with conditional active styling
            ul {
                NavItem { icon: "chat", label: "Chat", tab: NavTab::Chat, active: active_tab == NavTab::Chat }
                NavItem { icon: "folder_managed", label: "Sessions", tab: NavTab::Sessions, active: active_tab == NavTab::Sessions }
                // ...
            }
        }
    }
}
```

## Migration Path: Existing Conversations

Since the existing `Conversation` model in `chat/state.rs` lacks `folder_id`, `category`, `is_pinned`, and `turn_count`, use an `ALTER TABLE` SQL migration (same as the existing Phase 1→2 migration pattern in `repo.rs`):

```rust
let _ = conn.execute("ALTER TABLE conversations ADD COLUMN folder_id TEXT", []);
let _ = conn.execute("ALTER TABLE conversations ADD COLUMN category TEXT NOT NULL DEFAULT 'General'", []);
let _ = conn.execute("ALTER TABLE conversations ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0", []);
```

Turn count is computed dynamically: `SELECT COUNT(*) FROM messages WHERE conversation_id = ?1`.

## Testing the Implementation

```bash
dx serve --desktop
```

The app should open to the Sessions page showing:
- Left sidebar with navigation tabs (Sessions active)
- Folder tree panel with hardcoded → dynamic folder data
- Bento grid of session cards with hover effects
- All CRUD operations working via buttons and inline editing
- Drag-and-drop moving sessions between folders

# Trade-offs

| Dimension          | Assessment                        |
|--------------------|-----------------------------------|
| Complexity         | Medium (layout conversion is mechanical; state wiring requires understanding signals, but patterns are already established in the codebase) |
| Time to implement  | 8–12 hours (3–4 for RSX template conversion + widget split, 2–3 for state wiring, 2–3 for SQLite persistence, 1–2 for drag-drop + polish) |
| Reversibility      | Easy (new domain module, doesn't break existing chat functionality; routes can be swapped back) |
| Risk level         | Low-Medium (main risk: template-to-RSX conversion may miss Tailwind classes or Material Symbols fonts; drag-and-drop has inconsistent behavior across WebView versions) |
| Scalability        | Good for moderate folder/session counts (hundreds); may need virtualized lists at thousands of sessions since Dioxus renders all nodes eagerly |
| Maintainability    | Medium (initial code has RSX-heavy files due to template-first approach; refactoring into smaller components is deferred but necessary for clean architecture) |
| Fit for user persona | High — immediate visual output is highly motivating for a Rust learner; template-first matches "I can see it working" mindset from Python prototyping |

# Consequences

## Positive Outcomes

- The user sees a fully rendered, visually complete page within the first hour of implementation (template conversion), providing instant motivation
- Existing codebase patterns (signals, context + Arc<Mutex<Connection>>, spawn_blocking for SQLite) are reused, reinforcing learning
- The sidebar extraction solves a cross-cutting concern (persistent nav) that benefits the whole app
- Folder CRUD + drag-and-drop gives a true "file manager" feel for conversation organization
- The visual-first approach means the UI fidelity is high from day one — the template's custom color palette, Material Symbol integration, and bento grid layout are preserved

## Risks & Failure Modes

- **RSX conversion errors**: The `dx translate` tool may not perfectly handle complex Tailwind classes (custom gradients, group-hover variants, etc.). Manual cleanup will be needed. Mitigation: convert the template section by section (sidebar → folder tree → cards) and verify each renders independently.
- **Material Symbols not loading in WebView**: The Google Fonts CDN may not load in the desktop WebView context. Mitigation: download and self-host the Material Symbols font in `assets/`, or use the `dioxus-material-icons` crate. Fall back to emoji/unicode characters as placeholders.
- **Drag-and-drop in desktop WebView**: HTML5 DnD is supported in Wry (the WebView library Dioxus desktop uses), but behavior can differ from browsers. Testing on the target OS is essential. See also: `ondrag` events may need `evt.prevent_default()` on `ondragover` to enable drops.
- **Recursive component re-renders**: Each signal write triggers a re-render of all consuming components. If the folder tree is deeply nested, performance may suffer. Mitigation: use `use_memo` to derive child folder lists, and `key` attributes on list items for efficient diffing.
- **Second-Order Effects**: Extending `conversations` table with `folder_id` could affect the existing chat screen queries. The `get_all_conversations` and `load_messages` functions don't filter by folder_id yet, so all conversations still appear in the chat sidebar. Need a follow-up to filter conversations by the selected folder.

## Second-Order Effects

- **3–6 months later**: The template-first approach leaves RSX-heavy widget files that could benefit from refactoring into smaller, testable components. The deferred separation of concerns may start to feel painful as the page gains features (batch operations, sorting, filtering).
- **Routing ripple**: Making sessions the default `/` route and moving chat to `/chat` will require updating any bookmarks or sharing links. The existing `ConversationStore` and `ChatScreen` continue to work independently, but state sync between the sessions page's `SessionStore` and the chat page's `ConversationStore` may require a shared store or events.
- **Folder ↔ conversation ownership model**: Once folders are established, the existing "New Chat" button in the chat sidebar creates orphan conversations (no folder_id). The UX should either auto-assign to "Uncategorized" or prompt for folder selection. This is a natural follow-up feature.

# Verdict

This solution is best for users who prioritize **immediate visual motivation and rapid prototyping** and have the **ability to iterate on separation of concerns after the initial implementation**.
