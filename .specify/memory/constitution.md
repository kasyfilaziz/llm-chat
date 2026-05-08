<!--
Sync Impact Report:
- Version change: none -> 1.0.0
- List of modified principles: (Initial Creation)
  - I. Local-First Architecture
  - II. Cross-Platform Consistency
  - III. Extensible Tooling via MCP
  - IV. Source-Grounded Integrity
- Added sections: Technical Constraints, Security & Privacy
- Removed sections: none
- Templates requiring updates: none (✅ updated)
- Follow-up TODOs: none
-->

# Lumina Constitution

## Core Principles

### I. Local-First Architecture
Lumina operates primarily as a local application. All application logic, Model Context Protocol (MCP) interactions, and storage (sessions, source documents) MUST reside securely on the user's client device. External API communication is strictly reserved for necessary intelligence inference (e.g., Gemini API).

### II. Cross-Platform Consistency
The application MUST maintain a unified, responsive user interface across Desktop (Windows, macOS, Linux) and Mobile (iOS, Android). This is achieved using Dioxus with system native WebView rendering (Approach A) and Tailwind CSS, ensuring high performance with web-standard styling capabilities.

### III. Extensible Tooling via MCP
All feature integrations that interact with local systems, files, or external third-party services MUST be implemented using the Model Context Protocol (MCP). Lumina acts strictly as an MCP host, separating core app logic from tool execution logic to ensure modularity and security.

### IV. Source-Grounded Integrity
AI responses should prioritize verifiability. Any feature handling documents or RAG (Retrieval-Augmented Generation) MUST implement source grounding, where outputs provide direct, clickable citations mapped to the original ingested metadata.

## Technical Constraints

- **Language & Framework:** Rust (Stable 1.80.0+) and Dioxus 0.7.x.
- **Rendering:** System Native WebView (e.g., `libwebkit2gtk-4.1` on Linux) to keep binary sizes small (~10MB) and RAM overhead minimal.
- **Styling:** Tailwind CSS v4. No custom GPU kernels or experimental renderers that break web CSS compatibility.
- **State & Persistence:** Dioxus Signals for state, and a local database (SQLite or SurrealDB) for persistence. No remote databases.

## Security & Privacy

- **API Keys:** API keys and sensitive credentials MUST NOT be hardcoded, logged, or synced to cloud environments. They must be managed via local secure keyrings or `.env` files.
- **MCP Sandboxing:** Any MCP tool capable of destructive or mutative system actions (e.g., file deletion, system configuration changes) MUST prompt the user for explicit confirmation before execution.

## Governance

The Lumina Constitution serves as the foundational source of truth for architectural and design decisions, superseding other guides. All planning (`speckit.plan`) MUST explicitly verify compliance with these principles. Any amendments to this Constitution require a version bump following Semantic Versioning rules, along with a documented rationale for the change.

**Version**: 1.0.0 | **Ratified**: 2026-05-03 | **Last Amended**: 2026-05-03
