use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}

impl Message {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: role.to_string(),
            content: content.to_string(),
            created_at: Utc::now().timestamp(),
        }
    }
}
