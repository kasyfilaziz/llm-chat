# Tasks: Core Interface Pages — Session Management & Chat Interface

**Input**: Design documents from `specs/004-session-chat-ui/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization — configure Terra dark theme and verify toolchain

- [X] T001 Configure Terra dark theme color palette in `tailwind.css` using `@theme` block with all surface/primary/secondary/tertiary/error color tokens from the design template
- [X] T002 [P] Verify Terra design tokens compile by running `dx build --no-bundle` and check for any Tailwind class resolution errors

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Router refactor for shared layout, sidebar extraction, sessions domain scaffold, and DB migrations. **Must complete before any user story.**

- [X] T003 Refactor `src/app.rs` to use `#[layout(RootLayout)]` — create `src/layout.rs` with `RootLayout` component containing sidebar + `<Outlet/>`; move chat route under the layout; add `/sessions` route pointing to new `SessionsScreen`
- [X] T004 [P] Extract sidebar from `src/domains/chat/widgets/sidebar.rs` to `src/components/sidebar.rs` — make it a pure dumb component receiving `active_route: Route` prop; render Context Vault and Hooks as disabled items with "Coming soon" tooltip
- [X] T005 Add catch-all and routes for all screens under shared layout; update AGENTS.md
- [X] T006 [P] Create `src/domains/sessions/mod.rs` with stub `SessionsScreen` component — verify route works at `/`
- [X] T007 [P] Add `src/domains/sessions/state.rs` — define `Folder` and `SessionCard` structs with serde derive; create `FolderStore` signal
- [X] T008 [P] Run DB migrations in `src/domains/sessions/repo.rs::init_db`: add `folders` table, `ALTER TABLE conversations ADD COLUMN folder_id`, `deleted_at`, `is_pinned`, create `session_tags` table
- [X] T009 [P] Create `src/domains/sessions/repo.rs` — implement `get_all_folders`, `create_folder`, `rename_folder`, `delete_folder`, `get_sessions_by_folder`, `move_session_to_folder` functions

**Checkpoint**: Foundation ready — sidebar renders on all routes, sessions page is accessible at `/`, folders table exists in DB, sessions domain stub compiles.

---

## Phase 3: US1 — Organize Conversations in Folders (Priority: P1)

**Goal**: Users can create, rename, delete folders with subfolder nesting. Folders display in a directory tree on the left panel with session counts.

- [X] T010 [P] [US1] Create `DirectoryTree` widget in `src/domains/sessions/widgets/directory_tree.rs` — renders folder hierarchy with expandable/collapsible items, indented subfolders, session count badges, hover highlight
- [X] T011 [P] [US1] Implement "New Folder" button + inline name editor in directory tree
- [X] T012 [US1] Implement folder rename (right-click context menu or inline edit icon)
- [X] T013 [US1] Implement folder delete with confirmation
- [X] T014 [US1] Wire `DirectoryTree` into `SessionsScreen` in `screen.rs` — load folders from repo on mount, store selected folder ID, update session list on folder selection
- [X] T015 [US1] Handle subfolder creation — "New Folder" inside selected folder creates with `parent_id`

**Checkpoint**: Folder CRUD complete — users can create, rename, delete folders with nesting. Directory tree renders in the left panel with session count badges.

---

## Phase 4: US2 — View and Browse Chat Sessions (Priority: P1)

**Goal**: Sessions display as cards in a responsive bento grid within the selected folder.

- [X] T016 [P] [US2] Create `SessionCard` widget in `src/domains/sessions/widgets/session_card.rs`
- [X] T017 [P] [US2] Create `SessionGrid` widget in `src/domains/sessions/widgets/session_grid.rs`
- [X] T018 [US2] Wire `SessionGrid` into `SessionsScreen`
- [X] T019 [US2] Implement user-assignable tag
- [X] T020 [US2] Implement session pin toggle

**Checkpoint**: Session grid complete — users see session cards in a bento grid, filtered by folder.

---

## Phase 5: US3 — Chat with AI Assistant (Priority: P1)

**Goal**: Chat interface rendered in Terra dark theme with rich markdown, typing indicator, and functional input area.

- [X] T021 [P] [US3] Rewrite `MessageBubble` in `src/domains/chat/widgets/message.rs`
- [X] T022 [P] [US3] Create `TypingIndicator` widget in `src/domains/chat/widgets/typing_indicator.rs`
- [X] T023 [US3] Create `InputArea` widget in `src/domains/chat/widgets/input_area.rs`
- [X] T024 [US3] Create `PromptSuggestions` widget in `src/domains/chat/widgets/prompt_suggestions.rs`
- [X] T025 [US3] Wire new widgets into `chat/screen.rs`
- [X] T026 [US3] Add Terra dark theme header to chat screen
- [X] T027 [US3] Verify end-to-end

**Checkpoint**: Chat interface complete — Terra-themed, rich markdown rendering, typing indicator, functional input.

---

## Phase 6: US4 — Move Sessions Between Folders (Priority: P2)

- [X] T031 [US4] Add context menu "Move to folder" with folder picker overlay (right-click on session card)
- [ ] T028 [US4] Implement mouse-event drag detection in `SessionCard` (deferred — context menu fallback works)
- [ ] T029 [US4] Add folder drop-target highlighting in `DirectoryTree` (deferred — context menu fallback works)
- [ ] T030 [US4] Wire drop handler (deferred — context menu fallback works)
- [ ] T032 [US4] Handle drag cancellation (deferred — context menu fallback works)

---

## Phase 7: US5 — Persistent Sidebar Navigation (Priority: P2)

- [X] T033 [US5] Add active-route highlighting in sidebar
- [X] T034 [US5] Style Context Vault and Hooks as disabled items with "Coming soon" tooltip
- [X] T035 [US5] Add sidebar collapse/expand toggle button

---

## Phase 8: US6 — Interact with AI Response Actions (Priority: P3)

- [X] T036 [P] [US6] Add hover action bar to AI `MessageBubble`
- [X] T037 [US6] Wire Regenerate button
- [X] T038 [US6] Wire prompt suggestion chips in `PromptSuggestions`
- [X] T039 [US6] Handle copy success/failure feedback

---

## Phase 9: Polish & Cross-Cutting Concerns

- [X] T040 Implement session soft-delete from session grid (via context menu)
- [X] T041 [P] Add confirmation dialog for folder deletion (already done in DirectoryTree)
- [X] T042 [P] Apply Terra theme consistently across Settings screen
- [X] T043 Run `cargo build` and fix any compilation errors — **passes clean**
- [X] T044 [P] Run `cargo clippy` and address warnings — **0 warnings**
- [X] T045 Update AGENTS.md to reference final plan and .tot/ acceptance files if any drift occurred during implementation
