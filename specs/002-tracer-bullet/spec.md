# Feature Specification: Phase 1 Core Loop (Tracer Bullet)

**Feature Branch**: `feature/002-tracer-bullet`  
**Created**: 2026-05-09
**Status**: Draft  
**Input**: User description: "i think you should create functional this llm scope, i mean as a rust function or as prototype, don't mock anything except for UI. it should support openai compatible api ; CRUD operation; just simple tailwind with textbox to prove dioxus can work with tailwind;"

## Clarifications

### Session 2026-05-09
- Q: How should network errors (e.g., invalid API key, server unreachable) be displayed in the UI? → A: Render an inline error bubble within the chat history (e.g., in red text).
- Q: What data type should be used for the Message ID? → A: Use UUID (v4) strings.
- Q: How should API credentials and configuration be managed? → A: Load the API Key, Endpoint URL, and Model Name from a .env file.
- Q: How should missing Ollama models be handled? → A: Surface an error in the UI instructing the user to manually pull the model via CLI.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Send Message and Receive LLM Response (Priority: P1)

As a user, I want to type a prompt into a text box, hit send, and see a streaming response from an OpenAI-compatible OR Ollama LLM so that I can verify the core network and UI integration works.

**Why this priority**: This is the absolute core loop of the application. It proves that Dioxus can take user input, trigger a background task, and update the UI with streaming tokens without blocking.

**Independent Test**: Can be fully tested by entering text in the UI, observing the SQLite database for a new record, and watching the UI update with the LLM response.

**Acceptance Scenarios**:

1. **Given** the application is running, **When** the user types "Hello" and clicks "Send", **Then** the message is instantly displayed in the UI and a background process starts.
2. **Given** a message has been sent, **When** the configured LLM API (OpenAI or Ollama format) returns a stream of tokens, **Then** the UI updates asynchronously to display the incoming text.
3. **Given** the stream finishes, **Then** the full assistant message is persisted to the database.

---

### User Story 2 - Persist and Load Chat History (Priority: P2)

As a user, I want my messages to be saved to a local database and loaded when I restart the app so that I don't lose my conversation history.

**Why this priority**: Proves the SQLite integration (CRUD) works flawlessly with the UI and validates our actor-model architecture.

**Independent Test**: Can be fully tested by sending a message, closing the application, reopening it, and seeing the message still visible.

**Acceptance Scenarios**:

1. **Given** a new user message is sent, **When** the background task processes it, **Then** a new record is created in the local SQLite database.
2. **Given** the application starts up, **When** the main screen loads, **Then** it reads the existing messages from the database and renders them.
3. **Given** the user decides to clear the chat, **When** a delete action is triggered, **Then** the messages are removed from the database and UI.

### Edge Cases

- **Network/API Errors**: If the API key is invalid or the configured local/remote API server (OpenAI or Ollama) is unreachable, the system will render an inline error bubble within the chat history (e.g., in red text) to inform the user.
- **Missing Ollama Model**: If using Ollama and the requested model is not downloaded, surface an inline error instructing the user to pull the model manually via CLI.
- How does the system handle a hard crash while the LLM is streaming tokens? (Partial response might not be saved to DB).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a text input box and a "Send" button styled with basic Tailwind CSS.
- **FR-002**: System MUST capture user input and display it in a basic chat history view.
- **FR-003**: System MUST execute a real HTTP request to either an OpenAI-compatible API or the native Ollama API endpoint (no mocking).
- **FR-004**: System MUST handle streaming events (SSE for OpenAI, or NDJSON for Ollama) from the API and update the UI incrementally.
- **FR-005**: System MUST support SQLite CRUD operations (Create, Read, Update, Delete) for chat messages.
- **FR-006**: System MUST load historical messages from the SQLite database upon application startup.
- **FR-007**: System MUST NOT block the main Dioxus UI thread while performing database writes or network requests.

### Key Entities *(include if feature involves data)*

- **Message**: Represents a single chat bubble. Attributes: `id` (UUID v4), `role` (user or assistant), `content` (the text), `created_at`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: User can successfully send a prompt and receive a full streaming response from an external or local LLM.
- **SC-002**: The UI maintains responsiveness and never freezes during database I/O or network requests.
- **SC-003**: The application successfully persists messages across application restarts using SQLite.
- **SC-004**: Tailwind CSS classes successfully apply to the Dioxus UI components.

## Assumptions

- We are assuming a single, default conversation thread for this phase (no complex conversation branching or multi-chat sidebars yet).
- We assume the endpoint URL, API Key, Model Name, and API Provider Type (e.g., `openai` vs `ollama`) will be loaded from a simple `.env` file for this prototype phase.
- We accept that a hard crash during an LLM stream might result in the loss of that specific partial assistant response (prioritizing UI performance over extreme database locking).
