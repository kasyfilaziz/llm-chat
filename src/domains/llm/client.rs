use reqwest::Client;
use crate::domains::llm::models::ChatRequest;

#[derive(Debug, Clone, Copy)]
pub enum ProviderType {
    OpenAI,
    Ollama,
}

impl From<&str> for ProviderType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ollama" => ProviderType::Ollama,
            _ => ProviderType::OpenAI,
        }
    }
}

pub struct LlmClient {
    client: Client,
    pub provider: ProviderType,
    pub url: String,
    pub key: String,
    pub model: String,
}

impl LlmClient {
    pub fn new(provider: ProviderType, url: String, key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            provider,
            url,
            key,
            model,
        }
    }

    pub fn prepare_chat_request(&self, request: ChatRequest) -> reqwest::RequestBuilder {
        let mut rb = self.client.post(&self.url);
        
        if matches!(self.provider, ProviderType::OpenAI) {
            rb = rb.header("Authorization", format!("Bearer {}", self.key));
        }

        rb.json(&request)
    }

    pub async fn stream_chat(&self, request: ChatRequest) -> reqwest::Result<reqwest::Response> {
        self.prepare_chat_request(request).send().await
    }
}
