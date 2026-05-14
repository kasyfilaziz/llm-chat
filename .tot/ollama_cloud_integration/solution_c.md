# Problem

The user wants to integrate Ollama Cloud and OpenAI-compatible services into Lumina while maintaining a "local-first" identity. The specific challenge is handling the subtle environmental differences between a local binary endpoint and a cloud-hosted API. "Solved" means an implementation that "just works" by detecting the operating mode from the environment and adapting the request pipeline accordingly.

# User Profile

| Attribute        | Value |
|------------------|-------|
| Technical Level  | Technical / Expert |
| Role / Domain    | Lead Developer |
| Output Style     | Full technical detail |

> Note: All language, depth, and framing in this document are adapted to the profile above.

# Assumptions

1. The developer wants to avoid a full refactor but needs something more robust than "Solution A."
2. Ollama Cloud might require specific non-standard headers (e.g., custom tokens or project IDs).
3. The "Local-First" core should remain as "pure" as possible, with cloud logic isolated.

# Solution Summary

This approach uses **Request Middleware** to adapt the existing `LlmClient` pipeline. By introducing a "Mode" flag in the environment, we inject conditional behavior into the `prepare_chat_request` method. This allows the client to adapt its authentication and endpoint logic dynamically without changing the core streaming parsers.

# Detailed Implementation

### 1. Update Environment Schema
Add an explicit mode for the provider.

```env
API_PROVIDER_TYPE=ollama
API_OLLAMA_MODE=cloud # or 'local'
API_ENDPOINT_URL=https://api.ollama.com
API_KEY=your_cloud_key
```

### 2. Implementation in `LlmClient`
We introduce an internal `adapt_request` helper.

```rust
// src/domains/llm/client.rs

impl LlmClient {
    fn adapt_request(&self, mut rb: RequestBuilder) -> RequestBuilder {
        let mode = env::get_var_optional("API_OLLAMA_MODE").unwrap_or("local".to_string());
        
        if self.provider == ProviderType::Ollama && mode == "cloud" {
            // Apply Cloud-specific adaptations
            rb = rb.header("X-Ollama-Cloud-Auth", &self.key);
            rb = rb.header("X-Ollama-Project", "lumina-default");
        }
        
        rb
    }
}
```

### 3. Pipeline Integration
The main `stream_chat` method remains clean, calling the adapter before execution.

```rust
pub async fn stream_chat(&self, request: ChatRequest) -> reqwest::Result<reqwest::Response> {
    let mut rb = self.prepare_chat_request(request);
    rb = self.adapt_request(rb); // Inject adaptations
    rb.send().await
}
```

# Trade-offs

| Dimension        | Assessment |
|------------------|------------|
| Complexity       | Medium |
| Time to implement| 1–2 hours  |
| Reversibility    | Easy |
| Risk level       | Low |
| Scalability      | Medium (Can handle cloud nuances without a full refactor) |
| Maintainability  | Medium (Cloud logic is isolated to the adapter) |

# Consequences

## Positive Outcomes
- **Surgical Changes**: No massive refactor of existing working code.
- **Protocol Safety**: Handles cases where the cloud API is *almost* but not *quite* the same as local.

## Risks & Failure Modes
- **Hidden Logic**: The "Mode" flag adds a layer of configuration that might be non-obvious to new contributors.
- **Conditional Bloat**: If too many modes are added, the `adapt_request` function becomes a large switch statement.

## Second-Order Effects
- It creates a pattern for "Provider Profiles" in the environment, which could eventually be moved to a `config.yaml` or UI setting.

# Verdict
This solution is best for users who want to **safely handle cloud nuances** without committing to a full architectural refactor during the early development phases.
