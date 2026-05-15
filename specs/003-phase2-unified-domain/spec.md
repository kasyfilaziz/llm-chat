# Feature Specification: Phase 2 Unified Domain

**Feature Branch**: `[branch-name]`  
**Created**: 2026-05-15  
**Status**: Draft  
**Input**: User description: "implement Phase 2 architecture based on the Unified Domain accepted solution (Conversation History via SQLite, Settings via YAML, and MCP Foundation connecting to local MCP servers)"

## Clarifications
### Session 2026-05-15
- Q: How are conversation titles generated when a user creates a "New Chat"? → A: LLM Auto-generated, but user can manually rename the title.
- Q: What happens if the user configures an invalid or crashing executable path for an MCP server? → A: Explicit UI Error (e.g., "Failed to start MCP server").
- Q: How should the application handle loading a massive conversation history into the active chat view? → A: Pagination / Infinite Scroll (load chunk, then older dynamically).


## User Scenarios & Testing *(mandatory)*

### User Story 1 - Accessing and Managing Conversation History (Priority: P1)

Users need to view their past chats and resume previous conversations from a sidebar, ensuring they don't lose context between sessions.

**Why this priority**: Persisting history is the most fundamental feature expected of any chat interface. Without it, the application feels temporary and ephemeral.

**Independent Test**: Can be fully tested by creating multiple chats, closing the application, restarting, and verifying that the chats appear in the sidebar and their contents load correctly.

**Acceptance Scenarios**:

1. **Given** the application is running, **When** the user clicks "New Chat", **Then** a new empty conversation is created and saved to the database.
2. **Given** a populated conversation history, **When** the user selects a chat from the sidebar, **Then** the main chat window displays the messages associated with that chat.

---

### User Story 2 - Configuring Application Settings (Priority: P2)

Users need a central place to manage API keys, LLM provider preferences, and external MCP server paths, with changes applying globally and persisting across restarts.

**Why this priority**: Essential for integrating external tools and avoiding hardcoded secrets or `.env` modifications by end-users.

**Independent Test**: Can be fully tested by modifying a setting (like an API key) in the UI, verifying it writes to the YAML file, and seeing the change immediately applied to the chat logic without restarting.

**Acceptance Scenarios**:

1. **Given** the settings menu is open, **When** the user modifies an API key, **Then** the value is saved to the local YAML configuration file.
2. **Given** a changed setting, **When** the change is saved, **Then** the entire application reacts instantly to the new configuration without requiring a restart.

---

### User Story 3 - Connecting Local MCP Servers (Priority: P2)

Users want the AI to interact with their local environment using Model Context Protocol (MCP) servers (like a filesystem or brave-search server) running on their machine.

**Why this priority**: Extensibility and grounding via MCP is one of the core value propositions of Lumina.

**Independent Test**: Can be fully tested by starting the application, configuring a known local MCP server path, and verifying that the AI can successfully request and execute a tool from that server without freezing the UI.

**Acceptance Scenarios**:

1. **Given** an MCP server configured in settings, **When** the application initializes the MCP connection, **Then** the server process starts in the background and its tools are registered.
2. **Given** a registered tool, **When** the LLM invokes the tool, **Then** the application executes the tool via stdio and returns the result without blocking the main UI thread.

### Edge Cases

- What happens when the YAML settings file is corrupted or unreadable? (Fallback to default settings)
- How does the system handle an MCP server that crashes or hangs indefinitely? (Crashes trigger an explicit UI error; hangs need timeout handling)
- What happens when a user attempts to load a massive conversation history (e.g., thousands of messages)? (Handled via pagination/infinite scroll to prevent memory bloat)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST store and retrieve chat history from a local SQLite database without blocking the main thread.
- **FR-001.1**: The system MUST implement pagination or infinite scrolling for loading messages within a conversation to maintain UI performance on massive threads.
- **FR-002**: The system MUST display a persistent sidebar containing a list of past conversations.
- **FR-003**: The system MUST serialize and persist application settings to a local YAML configuration file.
- **FR-004**: The system MUST provide a user interface to configure API credentials and MCP server executable paths.
- **FR-005**: The system MUST instantly propagate settings changes across all active components.
- **FR-006**: The system MUST be capable of launching local MCP servers as background processes and communicating with them via standard input/output (stdio).
- **FR-007**: The system MUST gracefully terminate all child MCP processes when the main application exits.

### Key Entities

- **Conversation**: Represents a single chat thread, containing metadata like auto-generated title (user-editable), timestamps, and a list of messages.
- **Settings**: Global configuration properties including LLM provider choices, API keys, and registered MCP server paths.
- **MCP Server**: Represents an active connection to an external process providing specialized tools.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can load existing conversation history within 100 milliseconds.
- **SC-002**: Modifications to application settings are propagated and applied instantly (under 50ms) across the UI without restarting.
- **SC-003**: The main UI remains responsive (rendering at 60fps) during heavy MCP tool invocations or synchronous database saves.
- **SC-004**: Background MCP server processes are killed 100% of the time upon application shutdown, preventing process leaks.

## Assumptions

- SQLite schema changes required for `conversations` can be managed with simple SQL scripts rather than a heavy migration framework.
- The system will use a maintained YAML serialization library (`serde_yml`) to avoid security risks associated with deprecated crates.
- Users will provide the raw executable paths or commands for the local MCP servers they wish to connect to.
- Dioxus coroutines (`use_coroutine`) and global signals will be sufficient to orchestrate the separation between UI rendering and heavy I/O operations.I rendering and heavy I/O operations.separation between UI rendering and heavy I/O operations.I rendering and heavy I/O operations.tions.separation between UI rendering and heavy I/O operations.I rendering and heavy I/O operations.