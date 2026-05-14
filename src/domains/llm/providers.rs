use futures_util::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use tracing::info;
use crate::domains::llm::client::{LlmClient, ProviderType};
use crate::domains::llm::models::{ChatResponseChunk, OllamaChatResponse, ChatRequest};

pub async fn process_stream(
    client: &LlmClient,
    request: ChatRequest,
) -> Result<impl futures_util::Stream<Item = Result<String, String>>, String> {
    match client.provider {
        ProviderType::OpenAI => {
            let rb = client.prepare_chat_request(request);
            let es = EventSource::new(rb).map_err(|e| format!("Failed to create EventSource: {}", e))?;
            Ok(es.map(|event| {
                match event {
                    Ok(Event::Message(msg)) => {
                        if msg.data == "[DONE]" {
                            return Ok("".to_string());
                        }
                        let chunk: ChatResponseChunk = serde_json::from_str(&msg.data)
                            .map_err(|e| format!("Failed to parse OpenAI chunk: {}", e))?;
                        let content = chunk.choices[0].delta.content.clone().unwrap_or_default();
                        if !content.is_empty() {
                            info!("Received token: {}", content);
                        }
                        Ok(content)
                    }
                    Ok(_) => Ok("".to_string()),
                    Err(e) => Err(format!("SSE error: {}", e)),
                }
            }).boxed())
        }
        ProviderType::Ollama => {
            let response = client.stream_chat(request).await.map_err(|e| format!("Request error: {}", e))?;
            
            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(format!("Ollama API error ({}): {}", status, body));
            }

            let stream = response.bytes_stream();
            Ok(stream.map(|item| {
                match item {
                    Ok(bytes) => {
                        let line = String::from_utf8_lossy(&bytes);
                        let mut content = String::new();
                        for part in line.lines() {
                            if let Ok(res) = serde_json::from_str::<OllamaChatResponse>(part) {
                                let content_part = &res.message.content;
                                if !content_part.is_empty() {
                                    info!("Received token: {}", content_part);
                                }
                                content.push_str(content_part);
                            }
                        }
                        Ok(content)
                    }
                    Err(e) => Err(format!("Ollama stream error: {}", e)),
                }
            }).boxed())
        }
    }
}
