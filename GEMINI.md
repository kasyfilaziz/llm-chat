<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
specs/002-tracer-bullet/plan.md
<!-- SPECKIT END -->

## Active Technologies
- Rust 1.80.0+ + Dioxus 0.7 (`GlobalSignal`, `#[derive(Store)]`), `rusqlite` (Database), `serde_yaml_ng` (YAML Persistence), `rmcp` (MCP SDK), `tokio` (Async runtime) (feature/002-tracer-bullet)
- SQLite (`lumina.db`) for Chat; YAML (`settings.yaml`) for Configuration. (feature/002-tracer-bullet)

## Recent Changes
- feature/002-tracer-bullet: Added Rust 1.80.0+ + Dioxus 0.7 (`GlobalSignal`, `#[derive(Store)]`), `rusqlite` (Database), `serde_yaml_ng` (YAML Persistence), `rmcp` (MCP SDK), `tokio` (Async runtime)
