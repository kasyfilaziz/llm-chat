# Domain Interface Contracts

## 1. SessionRepo (Internal Domain Boundary)

Boundary between `domains/sessions/screen.rs` and `domains/sessions/repo.rs`.

```rust
// Folder CRUD
fn create_folder(conn: &Connection, name: &str, parent_id: Option<&str>) -> Result<Folder>;
fn rename_folder(conn: &Connection, id: &str, new_name: &str) -> Result<()>;
fn delete_folder(conn: &Connection, id: &str) -> Result<()>; // moves sessions to uncategorized
fn get_all_folders(conn: &Connection) -> Result<Vec<Folder>>;
fn get_folder(conn: &Connection, id: &str) -> Result<Option<Folder>>;

// Session-folder association
fn get_sessions_by_folder(conn: &Connection, folder_id: Option<&str>) -> Result<Vec<Session>>;
fn move_session_to_folder(conn: &Connection, session_id: &str, folder_id: Option<&str>) -> Result<()>;

// Session card data
fn get_session_card(conn: &Connection, id: &str) -> Result<Option<SessionCard>>;
fn delete_session(conn: &Connection, id: &str) -> Result<()>;
```

## 2. Sidebar Component Props

Shared dumb component used by `RootLayout` in `layout.rs`.

```rust
#[derive(Props, Clone, PartialEq)]
struct SidebarProps {
    active_route: Route, // Current route to highlight active nav item
}
```

The sidebar renders static nav items. Context Vault and Hooks are rendered as disabled items with `cursor-not-allowed` and a "Coming soon" tooltip.

## 3. Navigation Contract (Router)

Interface between sessions domain and chat domain via the router.

```rust
#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[layout(RootLayout)]
        #[route("/")]
        SessionsPage {},         // default landing page
        #[route("/chat/:id?")]   // optional session ID
        ChatScreen { id: Option<String> },
        #[route("/settings")]
        SettingsScreen {},
    #[end_layout]
}
```

- SessionsPage navigates to ChatScreen by setting `Route::ChatScreen { id: Some(session_id) }`
- Clicking a session card calls `navigator.push(Route::ChatScreen { id: Some(session_id) })`
- ChatScreen reads the route parameter to load the correct session
