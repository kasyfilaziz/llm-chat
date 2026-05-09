use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatResponseChunk {
    pub choices: Vec<ChatResponseChoice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatResponseChoice {
    pub delta: ChatResponseDelta,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatResponseDelta {
    pub content: Option<String>,
}

// Ollama specific
#[derive(Debug, Deserialize, Clone)]
pub struct OllamaChatResponse {
    pub message: ChatMessage,
    #[allow(dead_code)]
    pub done: bool,
}
