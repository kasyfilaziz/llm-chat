use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use crate::domains::chat::state::Message;

pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            role TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub async fn save_message(conn: Arc<Mutex<Connection>>, message: Message) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO messages (id, role, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![message.id, message.role, message.content, message.created_at],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn load_messages(conn: Arc<Mutex<Connection>>) -> Result<Vec<Message>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, role, content, created_at FROM messages ORDER BY created_at ASC")
            .map_err(|e| e.to_string())?;
        
        let msg_iter = stmt.query_map([], |row| {
            Ok(Message {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut messages = Vec::new();
        for msg in msg_iter {
            messages.push(msg.map_err(|e| e.to_string())?);
        }
        Ok(messages)
    }).await.map_err(|e| e.to_string())?
}
