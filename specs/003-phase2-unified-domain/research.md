# Phase 2 Unified Domain Research

## 1. Dioxus 0.7 Global State & Store

- **Decision**: Use `GlobalSignal` for application settings (`AppSettings`), and `#[derive(Store)]` (via the `dioxus-stores` crate) for complex nested states like a paginated list of `Conversation`s.
- **Rationale**: 
  - `GlobalSignal` avoids context lookups and prop-drilling for simple, flat structures that need to be accessed application-wide. 
  - `#[derive(Store)]` is essential for deeply nested data or collections (like a `Vec<Conversation>`) because it scopes subscriptions to individual fields or collection items. This prevents the entire list from re-rendering when only a single conversation is modified.
- **Alternatives considered**: `use_context_provider` with `use_signal` (leads to whole-component re-renders on any change), and `fermi` (largely superseded by the Dioxus core signal architecture).
- **Code Example**:
  ```rust
  use dioxus::prelude::*;
  use dioxus_stores::*;

  // 1. GlobalSignal for flat AppSettings
  #[derive(Default, Clone)]
  struct AppSettings { theme: String }
  
  static APP_SETTINGS: GlobalSignal<AppSettings> = Signal::global(|| AppSettings::default());

  // 2. Store for nested/paginated lists
  #[derive(Store, Default, Clone, PartialEq)]
  struct ConversationState {
      conversations: Vec<Conversation>,
  }

  #[component]
  fn ConversationList() -> Element {
      let store = use_store(ConversationState::default);
      let mut list = store.conversations();
      
      rsx! {
          for item in list.iter() {
              // Reactively binds ONLY to the specific item
              ConversationItem { item }
          }
      }
  }
  ```

## 2. `serde_yml` vs `serde_yaml`

- **Decision**: Do not use `serde_yml`; use `serde_yaml_ng` or the frozen `serde_yaml` crate instead. For async I/O, utilize `tokio::fs` combined with `tokio::task::spawn_blocking` for the serialization step.
- **Rationale**: `serde_yml` is a fork of the unmaintained `serde_yaml` crate, but it has been flagged by the Rust community for potential memory safety vulnerabilities (e.g., unsound emitter operations leading to segmentation faults) and the repository is now archived. `serde_yaml_ng` provides a safer drop-in replacement.
  For handling file I/O in a non-blocking way, reading/writing must be done via `tokio::fs`. Since serialization/deserialization is CPU-bound, performing it inside `spawn_blocking` ensures the async executor is not starved, particularly for large YAML files.
- **Alternatives considered**: `serde_yml` (rejected due to security risks), `serde_json` (rejected because JSON is not human-friendly for manual configuration edits).
- **Code Example**:
  ```rust
  use tokio::fs;
  use tokio::task;

  async fn save_settings_async(settings: AppSettings) -> std::io::Result<()> {
      // Offload the CPU-bound serialization task
      let yaml_string = task::spawn_blocking(move || {
          serde_yaml_ng::to_string(&settings)
      })
      .await
      .expect("Task panicked")
      .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

      // Perform async write
      fs::write("settings.yml", yaml_string).await?;
      Ok(())
  }
  ```

## 3. `rmcp` Host Implementation

- **Decision**: Spawn the MCP server as a child process utilizing `tokio::process::Command` mapped to `rmcp`'s stdio transport, inside a Dioxus `use_coroutine`.
- **Rationale**: `use_coroutine` binds the lifetime of the async task to the Dioxus component. This guarantees the background process connection operates cleanly in the background and shuts down alongside the UI component. `rmcp` natively supports binding a transport to `stdio` handles spawned by a Tokio child process.
- **Alternatives considered**: Running the MCP client in a global `tokio::spawn` task (harder to manage lifecycle/teardown and channel synchronization) or using HTTP/SSE transports (requires network ports and more overhead compared to standard Unix pipes).
- **Code Example**:
  ```rust
  use dioxus::prelude::*;
  use rmcp::ServiceExt;
  use tokio::process::Command;
  // Assuming `rmcp` provides TokioChildProcess for stdio transport

  enum McpAction {
      CallTool(String),
  }

  #[component]
  fn McpClientComponent() -> Element {
      let mcp_task = use_coroutine(|mut rx: UnboundedReceiver<McpAction>| async move {
          // 1. Spawn MCP background server via tokio
          let mut cmd = Command::new("npx");
          cmd.args(["-y", "@modelcontextprotocol/server-everything"]);
          
          // 2. Wrap stdio in rmcp transport and initialize client
          // (Implementation details depend on the specific rmcp module/version)
          let client = ().serve(cmd).await.expect("Failed to start MCP client");
          
          // 3. Listen for commands from Dioxus UI
          while let Some(action) = rx.next().await {
              match action {
                  McpAction::CallTool(tool) => {
                      // client.call_tool(...).await;
                  }
              }
          }
      });

      rsx! {
          button {
              onclick: move |_| mcp_task.send(McpAction::CallTool("echo".into())),
              "Call MCP Tool"
          }
      }
  }
  ```