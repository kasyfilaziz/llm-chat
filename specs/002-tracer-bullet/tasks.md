---
description: "Task list template for feature implementation"
---

# Tasks: Phase 1 Core Loop (Tracer Bullet)

**Input**: Design documents from `specs/002-tracer-bullet/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Initialize Rust project via `cargo init`
- [X] T002 [P] Add dependencies to `Cargo.toml` (dioxus, reqwest, reqwest-eventsource, rusqlite, tokio, uuid)
- [X] T003 [P] Configure Tailwind CSS setup and `Dioxus.toml`
- [X] T004 Create the "Pragmatic Hybrid DDD" folder structure within `src/` (components, domains, utils)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T005 [P] Implement environment variable loader (`.env`) in `src/utils/env.rs`
- [X] T006 Implement global SQLite connection setup and pool in `src/db.rs`
- [X] T007 Create basic App shell layout and router setup in `src/app.rs`
- [X] T008 Setup application entry point configuring Dioxus and async runtime in `src/main.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Send Message and Receive LLM Response (Priority: P1) 🎯 MVP

**Goal**: Type a prompt, hit send, and see a streaming response from an OpenAI-compatible OR Ollama LLM API without blocking the UI.

**Independent Test**: Enter text in the UI, observe the UI update asynchronously with incoming tokens from the selected configured API.

### Implementation for User Story 1

- [X] T009 [P] [US1] Create basic shared UI components (e.g., TextInput, Button) in `src/components/mod.rs`
- [X] T010 [P] [US1] Define LLM Request/Response structs in `src/domains/llm/models.rs`
- [X] T011 [US1] Implement base LLM HTTP client logic in `src/domains/llm/client.rs`
- [X] T012 [US1] Implement OpenAI SSE and Ollama NDJSON streaming parsers in `src/domains/llm/providers.rs`
- [X] T013 [P] [US1] Create Chat State (`Message` struct and Dioxus Signals) in `src/domains/chat/state.rs`
- [X] T014 [US1] Create MessageBubble UI component in `src/domains/chat/widgets/message.rs`
- [X] T015 [US1] Implement Chat screen UI and Dioxus `use_coroutine` orchestrator in `src/domains/chat/screen.rs`
- [X] T016 [US1] Add inline error bubble for missing Ollama models / invalid API keys in `src/domains/chat/screen.rs`

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently (network calls and UI streaming work).

---

## Phase 4: User Story 2 - Persist and Load Chat History (Priority: P2)

**Goal**: Messages are saved to a local SQLite database and loaded when the app restarts.

**Independent Test**: Send a message, close the application, reopen it, and see the message still visible.

### Implementation for User Story 2

- [X] T017 [P] [US2] Define database schema and write SQLite CRUD queries in `src/domains/chat/repo.rs` wrapped in `tokio::task::spawn_blocking`
- [X] T018 [US2] Integrate `repo.rs` load queries on Chat screen initialization in `src/domains/chat/screen.rs`
- [X] T019 [US2] Integrate `repo.rs` save queries into the `use_coroutine` orchestrator (triggering upon `StreamCompleted`) in `src/domains/chat/screen.rs`

**Checkpoint**: User Stories 1 AND 2 should both work independently. The app now persists chat history.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T020 Run manual tests to verify <100MB RAM usage and non-blocking 60fps UI streaming updates
- [X] T021 [P] Validate `specs/002-tracer-bullet/quickstart.md` `.env` instructions work cleanly

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Depends on User Story 1 being substantially complete (requires the Chat State and `use_coroutine` orchestrator to plug into).

### Parallel Opportunities

- All Setup tasks marked `[P]` can run in parallel (T002, T003).
- All Foundational tasks marked `[P]` can run in parallel (T005).
- Shared components (T009), LLM structs (T010), and Chat state structs (T013) can be built in parallel.
- Database queries (T017) can be built in parallel alongside any UI refinements.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently (verify LLM connectivity and streaming).
5. Proceed to Phase 4 for Database Persistence.
