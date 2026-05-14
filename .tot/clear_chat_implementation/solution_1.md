# Problem
Implement a "Clear Chat" feature to delete all messages from the SQLite database and reset the UI to a blank state immediately.

# User Context & Persona
- **Developer Profile:** Focuses on Rust/Dioxus architecture validation.
- **Goal:** Verify basic functionality of LLM client and database access (Tracer Bullet).
- **Preference:** Functional over polished UX, SQL logic in `repo.rs`.

# External Research
- **SQLite Performance:** `DELETE FROM table` is optimized by SQLite as a single operation (truncate optimization) if no `WHERE` clause is present.
- **Dioxus 0.7 State:** Signals are `Copy` and generational. Resetting a signal like `Vec<Message>` can be done via `.set(Vec::new())` in an event handler.
- **Safety:** Using `rusqlite`'s `execute` method is safe for hardcoded truncation queries.

# Solution Summary
This solution implements a direct "Truncate" approach. It adds a `clear_all_messages` function to `repo.rs` that executes `DELETE FROM messages`. The UI calls this function and clears the local signal immediately.

# Assumptions
- The application currently only supports a single chat session (all messages in one table).
- Data loss is acceptable as the feature is intended to purge the session.

# Detailed Implementation

### 1. `src/domains/chat/repo.rs`
Add the following function:
```rust
pub async fn clear_all_messages(conn: Arc<Mutex<Connection>>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM messages", []).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}
```

### 2. `src/domains/chat/screen.rs`
- Add a "Clear Chat" button in the header.
- Implement the `onclick` handler to call `clear_all_messages` and reset the signal.

```rust
// Inside ChatScreen component
let clear_chat = {
    let conn = conn.clone();
    move |_| {
        let conn = conn.clone();
        spawn(async move {
            if clear_all_messages(conn).await.is_ok() {
                messages.set(Vec::new());
            }
        });
    }
};

// UI Change (Header)
header { ...
    Button {
        onclick: clear_chat,
        "Clear Chat"
    }
}
```

# Trade-offs (Table)
| Aspect | Strength | Weakness |
| :--- | :--- | :--- |
| **Performance** | Highest (SQLite Truncate Optimization). | N/A (atomic for all rows). |
| **Complexity** | Lowest. | No granularity (deletes everything). |
| **Safety** | High. | No undo capability. |

# Consequences
- **Positive:** Extremely fast, simple to implement, verifies basic DB write-access.
- **Risks:** Accidentally deletes data if multi-session is added later without updating this logic.
- **Second-Order:** Establishes `repo.rs` as the source of truth for destructive operations.

# Verdict
Recommended for the Tracer Bullet phase to verify the simplest possible destructive path.
