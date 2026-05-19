use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use crate::domains::chat::state::{Message, Conversation};

pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )",
        [],
    )?;

    // Handle Phase 1 -> Phase 2 migration
    // Attempt to add the conversation_id column. We ignore the error if it already exists.
    let _ = conn.execute("ALTER TABLE messages ADD COLUMN conversation_id TEXT NOT NULL DEFAULT 'default'", []);

    Ok(())
}

pub async fn get_all_conversations(conn: Arc<Mutex<Connection>>) -> Result<Vec<Conversation>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        
        let iter = stmt.query_map([], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut convos = Vec::new();
        for c in iter {
            convos.push(c.map_err(|e| e.to_string())?);
        }
        Ok(convos)
    }).await.map_err(|e| e.to_string())?
}

pub async fn create_conversation(conn: Arc<Mutex<Connection>>, conversation: Conversation) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![conversation.id, conversation.title, conversation.created_at, conversation.updated_at],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn update_conversation_title(conn: Arc<Mutex<Connection>>, id: String, new_title: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_title, now, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn save_message(conn: Arc<Mutex<Connection>>, message: Message) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![message.id, message.conversation_id, message.role, message.content, message.created_at],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn load_messages(conn: Arc<Mutex<Connection>>, conversation_id: String, limit: i64, offset: i64) -> Result<Vec<Message>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, conversation_id, role, content, created_at FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC LIMIT ?2 OFFSET ?3")
            .map_err(|e| e.to_string())?;
        
        let msg_iter = stmt.query_map(params![conversation_id, limit, offset], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut messages = Vec::new();
        for msg in msg_iter {
            messages.push(msg.map_err(|e| e.to_string())?);
        }
        Ok(messages)
    }).await.map_err(|e| e.to_string())?
}

#[allow(dead_code)]
pub async fn clear_messages(conn: Arc<Mutex<Connection>>, conversation_id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM messages WHERE conversation_id = ?1", params![conversation_id]).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}
