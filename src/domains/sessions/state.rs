use serde::{Deserialize, Serialize};

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
    pub category: String,
    pub description: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub turn_count: i64,
    pub folder_id: Option<String>,
    pub is_pinned: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionStore {
    pub folders: Vec<Folder>,
    pub active_folder_id: Option<String>,
    pub sessions: Vec<SessionSummary>,
    pub search_query: String,
    pub loading: bool,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            folders: Vec::new(),
            active_folder_id: None,
            sessions: Vec::new(),
            search_query: String::new(),
            loading: true,
        }
    }

    pub fn select_folder(&mut self, id: Option<String>) {
        self.active_folder_id = id;
    }

    pub fn add_folder(&mut self, folder: Folder) {
        self.folders.push(folder);
    }

    pub fn remove_folder(&mut self, id: &str) {
        self.folders.retain(|f| f.id != id);
        if self.active_folder_id.as_deref() == Some(id) {
            self.active_folder_id = None;
        }
    }

    pub fn rename_folder(&mut self, id: &str, new_name: String) {
        if let Some(f) = self.folders.iter_mut().find(|f| f.id == id) {
            f.name = new_name;
        }
    }

    pub fn set_sessions(&mut self, sessions: Vec<SessionSummary>) {
        self.sessions = sessions;
    }

    pub fn move_session(&mut self, session_id: &str, target_folder_id: Option<String>) {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == session_id) {
            s.folder_id = target_folder_id;
        }
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}
