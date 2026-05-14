# Problem

The user wants to extend the Lumina LLM chatbot to support "Ollama Cloud" and OpenAI-compatible services while maintaining the "local-first" principle (local storage, local models, but cloud access). The current implementation has a hard-coded assumption that Ollama does not require an API key and operates primarily on local endpoints. "Solved" means being able to switch between local Ollama, cloud Ollama, and OpenAI-compatible providers using only `.env` configuration without changing the core application logic.

# User Profile

| Attribute        | Value |
|------------------|-------|
| Technical Level  | Technical / Expert |
| Role / Domain    | Lead Developer |
| Output Style     | Full technical detail |

> Note: All language, depth, and framing in this document are adapted to the profile above.

# Assumptions

1. The Ollama Cloud API remains 100% wire-compatible with the local Ollama binary's NDJSON stream.
2. Authentication for Ollama Cloud follows standard Bearer token patterns or simple header injection.
3. The user prefers a "Tracer Bullet" approach that maximizes speed to delivery with minimal architectural overhead.

# Solution Summary

This approach treats "Ollama Cloud" as a simple variant of the existing `Ollama` provider type. It refactors `LlmClient` to stop ignoring the `API_KEY` field when the provider is `Ollama`, allowing the same logic to handle both local (where the key is empty/ignored) and cloud (where the key is passed as a header).

# Detailed Implementation

### 1. Refactor `LlmClient`
Currently, `LlmClient` only attaches the `Authorization` header for `ProviderType::OpenAI`. We will modify `prepare_chat_request` to attach the key if it's present, regardless of the provider, or specifically for Ollama if it's in a "Cloud" context.

```rust
// src/domains/llm/client.rs

pub fn prepare_chat_request(&self, request: ChatRequest) -> reqwest::RequestBuilder {
    let mut rb = self.client.post(&self.url);
    
    // Attach API Key if it's not the default "not-needed" or empty
    if !self.key.is_empty() && self.key != "not-needed" {
        rb = rb.header("Authorization", format!("Bearer {}", self.key));
    }

    rb.json(&request)
}
```

### 2. Update Environment Configuration
The `.env` file simply points to the Cloud URL.

```env
API_PROVIDER_TYPE=ollama
API_ENDPOINT_URL=https://api.ollama.cloud/v1/chat # Example cloud URL
API_MODEL_NAME=llama3
API_KEY=your_cloud_token_here
```

### 3. Stream Parser Consistency
Since the parser in `src/domains/llm/providers.rs` already handles Ollama's NDJSON format, no changes are needed there as long as the cloud response matches the local format.

# Trade-offs

| Dimension        | Assessment |
|------------------|------------|
| Complexity       | Low |
| Time to implement| < 1 hour |
| Reversibility    | Easy |
| Risk level       | Medium (Assumes 100% wire compatibility) |
| Scalability      | Low (Difficult to add cloud-only features later) |
| Maintainability  | High (Minimal code to maintain) |

# Consequences

## Positive Outcomes
- **Immediate Deployment**: Works immediately with existing infrastructure.
- **Zero Bloat**: No new traits or abstractions added to the codebase.

## Risks & Failure Modes
- **Protocol Drift**: If Ollama Cloud introduces specific headers (like `X-Ollama-Project`) or different auth schemes, this simple model will break.
- **Ambiguity**: It becomes harder to distinguish "Local" from "Cloud" in the UI because they share the same enum variant.

## Second-Order Effects
- As the project grows, we may find ourselves adding more and more "if cloud" checks inside the `LlmClient`, leading to "spaghetti configuration."

# Verdict
This solution is best for users who prioritize **speed and simplicity** over long-term architectural purity, especially during the initial Tracer Bullet phase.
