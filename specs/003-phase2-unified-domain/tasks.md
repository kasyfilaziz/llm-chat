# Tasks: Phase 2 Unified Domain

**Input**: Design documents from `/specs/003-phase2-unified-domain/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure for Phase 2

- [x] T001 [P] Add dependencies to `Cargo.toml`: `dioxus-stores`, `serde_yaml_ng`, `directories`, `rmcp`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

- [x] T002 Update database schema initialization logic in `src/db.rs` to support `conversations` table

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Accessing and Managing Conversation History (Priority: P1) 🎯 MVP

**Goal**: Users need to view their past chats and resume previous conversations from a sidebar, ensuring they don't lose context between sessions.

**Independent Test**: Can be fully tested by creating multiple chats, closing the application, restarting, and verifying that the chats appear in the sidebar and their contents load correctly.

### Implementation for User Story 1

- [x] T003 [P] [US1] Create SQLite `conversations` table initialization script in `src/domains/chat/repo.rs`
- [x] T004 [P] [US1] Update `messages` table schema in `src/domains/chat/repo.rs` to include `conversation_id`
- [x] T005 [US1] Implement DB query functions for conversations (`get_all_conversations`, `create_conversation`, `update_conversation_title`) in `src/domains/chat/repo.rs`
- [x] T006 [US1] Implement DB query functions for messages with pagination (OFFSET/LIMIT) in `src/domains/chat/repo.rs`
- [x] T007 [US1] Create `ConversationStore` using `#[derive(Store)]` in `src/domains/chat/state.rs`
- [x] T008 [US1] Create Sidebar component in `src/domains/chat/widgets/sidebar.rs`
- [x] T009 [US1] Integrate Sidebar into `ChatScreen` in `src/domains/chat/screen.rs`
- [x] T010 [US1] Implement LLM auto-generated title logic in `src/domains/chat/screen.rs`
- [x] T011 [US1] Implement manual title rename functionality in `src/domains/chat/widgets/sidebar.rs`

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Configuring Application Settings (Priority: P2)

**Goal**: Users need a central place to manage API keys, LLM provider preferences, and external MCP server paths, with changes applying globally and persisting across restarts.

**Independent Test**: Can be fully tested by modifying a setting (like an API key) in the UI, verifying it writes to the YAML file, and seeing the change immediately applied to the chat logic without restarting.

### Implementation for User Story 2

- [x] T012 [P] [US2] Create settings domain module structure in `src/domains/settings/mod.rs`
- [x] T013 [P] [US2] Implement `AppSettings` struct and YAML default fallback logic in `src/domains/settings/repo.rs`
- [x] T014 [US2] Implement async `load` and `save` methods using `tokio::fs` and `serde_yaml_ng` in `src/domains/settings/repo.rs`
- [x] T015 [US2] Setup `GlobalSignal<AppSettings>` in `src/domains/settings/state.rs`
- [x] T016 [US2] Create Settings UI Component/Page in `src/domains/settings/screen.rs`
- [x] T017 [US2] Update LLM client to read keys and model info from `GlobalSignal<AppSettings>` instead of env vars in `src/domains/llm/client.rs`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Connecting Local MCP Servers (Priority: P2)

**Goal**: Users want the AI to interact with their local environment using Model Context Protocol (MCP) servers running on their machine.

**Independent Test**: Can be fully tested by starting the application, configuring a known local MCP server path, and verifying that the AI can successfully request and execute a tool from that server without freezing the UI.

### Implementation for User Story 3

- [x] T018 [P] [US3] Create MCP domain module structure in `src/domains/mcp/mod.rs`
- [x] T019 [P] [US3] Define `McpEvent` and local connection state in `src/domains/mcp/state.rs`
- [x] T020 [US3] Implement `rmcp` Stdio server spawner and lifecycle wrapper in `src/domains/mcp/client.rs`
- [x] T021 [US3] Setup MCP orchestrator coroutine (`use_coroutine`) in `src/app.rs`
- [x] T022 [US3] Integrate explicit UI error alerts for crashed MCP connections in `src/domains/mcp/client.rs` and `src/app.rs`
- [x] T023 [US3] Connect active MCP tools to the Chat LLM context in `src/domains/chat/screen.rs`

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T024 [P] Update documentation in `README.md` reflecting Phase 2 completion
- [x] T025 Run cleanup and refactoring across domains
- [x] T026 Add error handling fallbacks for empty `settings.yaml`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 3 (P2)**: Depends slightly on User Story 2 (for configuring the MCP server path), but mock paths can be used during standalone US3 testing.

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- Models and Repo layers within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories
