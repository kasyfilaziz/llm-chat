# Problem

The user wants to support both local and cloud LLM services (OpenAI-compatible and Ollama) while maintaining a local-first architecture. The core challenge is that "Local Ollama" and "Cloud Ollama" share the same message format but have different transport requirements (Auth, HTTPS, Discovery). "Solved" means an architecture that can scale to multiple providers and locations without leaking transport details into the business logic.

# User Profile

| Attribute        | Value |
|------------------|-------|
| Technical Level  | Technical / Expert |
| Role / Domain    | Lead Developer |
| Output Style     | Full technical detail |

> Note: All language, depth, and framing in this document are adapted to the profile above.

# Assumptions

1. The application will eventually need to support hybrid modes (e.g., local for fast RAG, cloud for deep reasoning).
2. Different providers might use the same protocol (e.g., DeepSeek using OpenAI protocol).
3. The developer is willing to invest in a one-time architectural refactor to ensure long-term stability.

# Solution Summary

This approach refactors the `llm` domain to decouple the **Protocol** (Message Schema & Parsing) from the **Transport** (HTTP Client, Auth Headers, Endpoint Logic). By creating an `LlmProvider` trait, we can compose different configurations: `OpenAIProtocol + CloudTransport`, `OllamaProtocol + LocalTransport`, or `OllamaProtocol + CloudTransport`.

# Detailed Implementation

### 1. Define Abstractions
We move away from a single `LlmClient` to a trait-based system.

```rust
// src/domains/llm/client.rs

pub trait LlmProtocol {
    fn format_request(&self, req: ChatRequest) -> serde_json::Value;
    fn parse_chunk(&self, chunk: String) -> Result<String, String>;
}

pub trait LlmTransport {
    async fn send_request(&self, url: &str, body: serde_json::Value) -> Result<reqwest::Response, String>;
}
```

### 2. Implement Composites
We can then implement `OllamaProtocol` and `OpenAITransport`.

```rust
pub struct OllamaProtocol;
impl LlmProtocol for OllamaProtocol { ... }

pub struct AuthenticatedTransport {
    pub key: String,
}
impl LlmTransport for AuthenticatedTransport {
    async fn send_request(...) {
        // Injects Authorization: Bearer {key}
    }
}
```

### 3. Factory Selection
The `LlmClient` becomes a holder for these traits, initialized based on `.env`.

```rust
pub struct LlmClient {
    protocol: Box<dyn LlmProtocol>,
    transport: Box<dyn LlmTransport>,
}
```

# Trade-offs

| Dimension        | Assessment |
|------------------|------------|
| Complexity       | High |
| Time to implement| 4–8 hours  |
| Reversibility    | Hard (Core architectural change) |
| Risk level       | Low (Very robust once implemented) |
| Scalability      | Excellent (Easy to add new protocols like Anthropic) |
| Maintainability  | High (Separation of concerns) |

# Consequences

## Positive Outcomes
- **Extreme Flexibility**: Can use DeepSeek with OpenAI protocol over an authenticated transport effortlessly.
- **Future-Proof**: Supports hybrid local/cloud orchestration in the same session.

## Risks & Failure Modes
- **Over-Engineering**: For a "Tracer Bullet," this might be too much abstraction too early.
- **Performance**: Minor overhead of dynamic dispatch (`Box<dyn ...>`), though negligible for LLM network I/O.

## Second-Order Effects
- The UI can now offer a "Provider Registry" where users save multiple configurations, moving away from a single global `.env`.

# Verdict
This solution is best for users who prioritize **architectural robustness and long-term scalability**, intending to build a professional-grade multi-model platform.
