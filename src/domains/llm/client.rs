use reqwest::Client;
use crate::domains::llm::models::ChatRequest;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq)]
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

    fn adapt_request(&self, mut rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mode = crate::utils::env::get_var_optional("API_OLLAMA_MODE").unwrap_or_else(|| "local".to_string());
        
        if self.provider == ProviderType::Ollama && mode == "cloud" {
            // Adapt for Ollama Cloud (injecting auth and project headers)
            if !self.key.is_empty() && self.key != "not-needed" {
                // rb = rb.header("X-Ollama-Auth", &self.key);
                rb = rb.header("Authorization", format!("Bearer {}", self.key));
            }
            rb = rb.header("X-Ollama-Project", "lumina-default");
        }
        
        rb
    }

    pub async fn stream_chat(&self, request: ChatRequest) -> reqwest::Result<reqwest::Response> {
        info!("Sending request to {}: {:?}", self.url, request);
        let mut rb = self.prepare_chat_request(request);
        rb = self.adapt_request(rb);
        rb.send().await
    }
}
