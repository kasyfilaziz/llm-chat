# Problem
Implement a session-aware "Clear Chat" feature that prepares the codebase for Phase 2 (multi-session support) while satisfying current "delete all" requirements.

# User Context & Persona
- **Developer Profile:** Architecture-minded, validates patterns for future growth.
- **Goal:** Ensure current "Tracer Bullet" code doesn't require a full rewrite when sessions are introduced.

# External Research
- **SQLite Performance:** Deleting with a `WHERE` clause is slightly slower than a full truncate but remains sub-millisecond for typical chat histories.
- **Dioxus 0.7 State:** Using a "Key" pattern (changing a session ID) can trigger a clean state reset across the entire UI tree.
- **Foreign Keys:** `PRAGMA foreign_keys = ON` is necessary if we eventually link messages to a `sessions` table.

# Solution Summary
This solution introduces a `delete_session` function in `repo.rs`. While we don't have a `session_id` column yet, we simulate the logic or use a "default" identifier, ensuring the API signature is ready for multi-session support.

# Assumptions
- The schema will eventually include a `session_id`.
- Current "single-threaded" use case treats all existing messages as belonging to a "global" session.

# Detailed Implementation

### 1. `src/domains/chat/repo.rs`
Add a targeted deletion function:
```rust
pub async fn delete_session_messages(conn: Arc<Mutex<Connection>>, _session_id: &str) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        // Currently deletes all, but signature accepts an ID for Phase 2
        conn.execute("DELETE FROM messages", []).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}
```

### 2. `src/domains/chat/screen.rs`
- Introduce a `session_id` signal (hardcoded to "default" for now).
- Wrap the chat UI in a container keyed by `session_id` to force a clean reset.

```rust
let mut session_id = use_signal(|| "default".to_string());

let handle_clear = {
    let conn = conn.clone();
    move |_| {
        let conn = conn.clone();
        spawn(async move {
            let _ = delete_session_messages(conn, &session_id.read()).await;
            // Changing the key resets the messages signal if it were inside a child
            // or we manually clear it here.
            messages.set(Vec::new());
        });
    }
};
```

# Trade-offs (Table)
| Aspect | Strength | Weakness |
| :--- | :--- | :--- |
| **Future Proofing** | High. | Adds boilerplate (ID parameters) not yet used. |
| **Safety** | Moderate. | Requires careful management of session IDs. |
| **Complexity** | Medium. | Slightly more cognitive overhead. |

# Consequences
- **Positive:** Minimal refactoring needed for Phase 2.
- **Risks:** Implementation might be "over-engineered" for a tracer bullet.
- **Second-Order:** Encourages thinking about data ownership early on.

# Verdict
Recommended if the developer plans to implement multi-session within the next 1-2 sprints.
