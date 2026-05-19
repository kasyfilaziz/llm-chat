#![allow(dead_code)]

use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use crate::domains::sessions::state::{Folder, SessionSummary};

pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id TEXT REFERENCES folders(id) ON DELETE CASCADE,
            created_at INTEGER NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0
        );"
    )?;

    let _ = conn.execute("ALTER TABLE conversations ADD COLUMN folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL", []);
    let _ = conn.execute("ALTER TABLE conversations ADD COLUMN deleted_at INTEGER", []);
    let _ = conn.execute("ALTER TABLE conversations ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0", []);

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS session_tags (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
            tag TEXT NOT NULL
        );"
    )?;

    Ok(())
}

pub async fn get_all_folders(conn: Arc<Mutex<Connection>>) -> Result<Vec<Folder>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, created_at, sort_order FROM folders ORDER BY sort_order ASC"
        ).map_err(|e| e.to_string())?;

        let iter = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                created_at: row.get(3)?,
                sort_order: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut folders = Vec::new();
        for f in iter {
            folders.push(f.map_err(|e| e.to_string())?);
        }
        Ok(folders)
    }).await.map_err(|e| e.to_string())?
}

pub async fn create_folder(conn: Arc<Mutex<Connection>>, folder: &Folder) -> Result<(), String> {
    let folder = folder.clone();
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO folders (id, name, parent_id, created_at, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![folder.id, folder.name, folder.parent_id, folder.created_at, folder.sort_order],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn rename_folder(conn: Arc<Mutex<Connection>>, id: String, new_name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE folders SET name = ?1 WHERE id = ?2",
            params![new_name, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn delete_folder(conn: Arc<Mutex<Connection>>, id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute("UPDATE conversations SET folder_id = NULL WHERE folder_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM folders WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn get_tags_for_session(conn: Arc<Mutex<Connection>>, session_id: String) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT tag FROM session_tags WHERE session_id = ?1 ORDER BY tag ASC")
            .map_err(|e| e.to_string())?;
        let iter = stmt.query_map(params![session_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut tags = Vec::new();
        for t in iter {
            tags.push(t.map_err(|e| e.to_string())?);
        }
        Ok(tags)
    }).await.map_err(|e| e.to_string())?
}

pub async fn add_tag_to_session(conn: Arc<Mutex<Connection>>, session_id: String, tag: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO session_tags (id, session_id, tag) VALUES (?1, ?2, ?3)",
            params![uuid::Uuid::new_v4().to_string(), session_id, tag],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn remove_tag_from_session(conn: Arc<Mutex<Connection>>, session_id: String, tag: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM session_tags WHERE session_id = ?1 AND tag = ?2",
            params![session_id, tag],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn get_sessions_by_folder(conn: Arc<Mutex<Connection>>, folder_id: Option<String>) -> Result<Vec<SessionSummary>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        let sql =             "SELECT c.id, c.title, '' as category,
            COALESCE((SELECT content FROM messages WHERE conversation_id = c.id ORDER BY created_at ASC LIMIT 1), '') as description,
            c.created_at, c.updated_at,
            (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as turn_count,
            c.folder_id, c.is_pinned,
            COALESCE((SELECT GROUP_CONCAT(tag, '||') FROM session_tags WHERE session_id = c.id), '') as tags
            FROM conversations c
            WHERE c.deleted_at IS NULL
            AND (?1 IS NULL AND c.folder_id IS NULL) OR c.folder_id = ?1
            ORDER BY c.is_pinned DESC, c.updated_at DESC";

        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let iter = stmt.query_map(params![folder_id], |row| {
            let tags_str: String = row.get(9).unwrap_or_default();
            let tags: Vec<String> = if tags_str.is_empty() { Vec::new() } else { tags_str.split("||").map(|s| s.to_string()).collect() };
            Ok(SessionSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                turn_count: row.get(6)?,
                folder_id: row.get(7)?,
                is_pinned: row.get::<_, i64>(8)? != 0,
                tags,
            })
        }).map_err(|e| e.to_string())?;

        let mut sessions = Vec::new();
        for s in iter {
            sessions.push(s.map_err(|e| e.to_string())?);
        }
        Ok(sessions)
    }).await.map_err(|e| e.to_string())?
}

pub async fn move_session_to_folder(conn: Arc<Mutex<Connection>>, session_id: String, folder_id: Option<String>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE conversations SET folder_id = ?1 WHERE id = ?2",
            params![folder_id, session_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn get_folder_session_counts(conn: Arc<Mutex<Connection>>) -> Result<Vec<(String, i64)>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT folder_id, COUNT(*) as cnt FROM conversations WHERE deleted_at IS NULL AND folder_id IS NOT NULL GROUP BY folder_id"
        ).map_err(|e| e.to_string())?;

        let iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        }).map_err(|e| e.to_string())?;

        let mut counts = Vec::new();
        for c in iter {
            counts.push(c.map_err(|e| e.to_string())?);
        }
        Ok(counts)
    }).await.map_err(|e| e.to_string())?
}

pub async fn soft_delete_session(conn: Arc<Mutex<Connection>>, session_id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE conversations SET deleted_at = ?1 WHERE id = ?2",
            params![now, session_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

pub async fn toggle_pin_session(conn: Arc<Mutex<Connection>>, session_id: String, is_pinned: bool) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE conversations SET is_pinned = ?1 WHERE id = ?2",
            params![is_pinned as i64, session_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}
