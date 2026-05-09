---

description: "Task list for Project Initialization & Environment Setup"
---

# Tasks: Project Initialization & Environment Setup

**Input**: Design documents from `/specs/001-project-setup/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, quickstart.md ✅

**Tests**: Not requested in specification — no test tasks generated.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)

## Path Conventions

- Single project: all source at repository root (`Cargo.toml`, `Dioxus.toml`, `src/`, `assets/`)

---

## Phase 1: Setup (Project Files)

**Purpose**: Create all project configuration and asset files. These are independent of each other and can be created in any order (T002–T004 in parallel after T001).

- [X] T001 Create `Cargo.toml` at repository root with package metadata (`name = "lumina"`, `version = "0.1.0"`, `edition = "2021"`) and dependencies: `dioxus = { version = "0.7", features = ["desktop"] }` and `rusqlite = { version = "0.31", features = ["bundled"] }`
- [X] T002 [P] Create `Dioxus.toml` at repository root with `[application]` section (`name = "lumina"`, `default_platform = "desktop"`) and `[bundle]` section (`identifier = "dev.lumina.app"`, `icon = ["assets/icon.png"]`)
- [X] T003 [P] Create `assets/` directory and add `assets/icon.png` as a minimal placeholder PNG (any valid 1x1 PNG or simple icon — exact design is not required at scaffold stage)
- [X] T004 [P] Create `assets/main.css` as an empty stub file (Tailwind CSS entry point; the `dx` CLI manages this file's content automatically during `dx serve` / `dx build`)

**Checkpoint**: All configuration and asset files exist — source code phase can begin.

---

## Phase 2: Foundational (Source Code — Blocks Both User Stories)

**Purpose**: Create the Rust application entry point and root component. Both user stories depend on this phase completing first.

**⚠️ CRITICAL**: No user story validation can begin until this phase is complete.

- [X] T005 Create `src/main.rs` with the `main()` entry point: include the Linux WebView compositing workaround (`#[cfg(all(target_os = "linux", debug_assertions))]` block setting `WEBKIT_DISABLE_COMPOSITING_MODE=1`) followed by the `dioxus::launch(App)` call, and `use dioxus::prelude::*;` import
- [X] T006 Add the `App` root component to `src/main.rs`: annotate with `#[component]`, return `Element`, and render a Tailwind-styled placeholder UI — a dark full-screen div (`bg-gray-950`, `flex`, `items-center`, `justify-center`, `min-h-screen`) containing an `h1` with `"Lumina"` and a `p` with `"v0.1.0 — Project initialized"` in muted text
- [X] T007 Run `cargo check --all-targets` from repository root and confirm zero compilation errors; fix any errors before proceeding

**Checkpoint**: Foundation ready — both user stories can now be validated independently.

---

## Phase 3: User Story 1 - Compile and Launch Desktop App (Priority: P1) 🎯 MVP

**Goal**: A developer with Rust + Dioxus CLI installed can produce and run a Lumina desktop binary that opens a native window with the placeholder UI.

**Independent Test**: Run `dx build` → launch binary → confirm native window shows "Lumina" and "v0.1.0 — Project initialized".

### Implementation for User Story 1

- [X] T008 [US1] Run `dx build` (or `dx serve` then immediately close) from repository root and confirm the project builds without compilation errors, satisfying SC-001 and FR-004
- [X] T009 [US1] Launch the compiled desktop application and verify a native window opens on the current platform displaying at minimum: the text `"Lumina"` as a heading and `"v0.1.0"` visible — satisfying SC-002 and FR-005
- [X] T010 [US1] Review `specs/001-project-setup/quickstart.md` against the actual steps performed in T001–T009; update any step that is missing, inaccurate, or requires an undocumented action — satisfying FR-006 and SC-003

**Checkpoint**: User Story 1 complete — a developer can independently clone the repo and reach a running desktop window using only `quickstart.md`.

---

## Phase 4: User Story 2 - Development Server with Hot-Reload (Priority: P2)

**Goal**: A developer can run `dx serve` and see source-file changes reflected in the running desktop window without restarting the app.

**Independent Test**: Run `dx serve` → edit a string in `src/main.rs` → save → confirm window updates without restart.

### Implementation for User Story 2

- [X] T011 [US2] Run `dx serve` from repository root and confirm: (a) the development server starts without errors, (b) a native desktop window opens automatically
- [X] T012 [US2] While `dx serve` is running, edit a visible UI string in `src/main.rs` (e.g., change `"v0.1.0 — Project initialized"` to `"Hot-reload works!"`), save the file, and confirm the running window updates to reflect the change without requiring a manual restart — satisfying US2 acceptance scenarios
- [X] T013 [US2] Revert the test change in `src/main.rs` back to `"v0.1.0 — Project initialized"`, save, and confirm the window returns to the original state

**Checkpoint**: User Story 2 complete — hot-reload development workflow is verified and functional.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Repository hygiene, documentation, and version control.

- [X] T014 [P] Update `README.md` at repository root with: project name (Lumina), one-sentence description, tech stack summary (Rust, Dioxus 0.7, Tailwind CSS v4), and a link to `specs/001-project-setup/quickstart.md` for setup instructions
- [X] T015 [P] Verify `.gitignore` at repository root includes entries for: `target/`, `dist/`, `*.pdb`, and `*.rs.bk` — add any missing entries
- [X] T016 Commit all scaffold files to version control with a descriptive commit message (e.g., `feat: initialize Lumina project scaffold (Dioxus 0.7, Tailwind v4, SQLite stub)`) — satisfying SC-004

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — T001 first, then T002/T003/T004 in parallel
- **Foundational (Phase 2)**: Depends on Phase 1 completion — **BLOCKS** both user stories
- **User Story 1 (Phase 3)**: Depends on Phase 2 completion — no dependency on US2
- **User Story 2 (Phase 4)**: Depends on Phase 2 completion — no dependency on US1; can run in parallel with Phase 3 if desired
- **Polish (Phase 5)**: Depends on Phase 3 + Phase 4 completion

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — no dependency on US2
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) — no dependency on US1

### Within Each User Story

- Foundational source creation (T005 → T006 → T007) must complete in order
- US1 tasks (T008 → T009 → T010) must complete in order
- US2 tasks (T011 → T012 → T013) must complete in order
- T014 and T015 (polish) can run in parallel with each other

### Parallel Opportunities

```bash
# Phase 1: After T001, launch T002, T003, T004 together
Task: "Create Dioxus.toml"               # T002
Task: "Create assets/icon.png"           # T003
Task: "Create assets/main.css stub"      # T004

# Phase 2: Sequential (each depends on previous)
T005 → T006 → T007

# After Phase 2: US1 and US2 can proceed in parallel
[Developer A] T008 → T009 → T010   # User Story 1
[Developer B] T011 → T012 → T013   # User Story 2

# Polish: T014 and T015 in parallel
Task: "Update README.md"    # T014
Task: "Verify .gitignore"   # T015
# Then: T016 (commit — must be last)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T004)
2. Complete Phase 2: Foundational (T005–T007)
3. Complete Phase 3: User Story 1 (T008–T010)
4. **STOP and VALIDATE**: Confirm a native window opens from a fresh clone using only `quickstart.md`
5. Share / demo MVP scaffold

### Incremental Delivery

1. Setup + Foundational → scaffold compiles ✅
2. Add User Story 1 → binary launches, window visible → **MVP complete**
3. Add User Story 2 → hot-reload verified → full feature complete
4. Polish → repo is clean, documented, committed

---

## Notes

- `[P]` tasks = different files, no dependencies between them
- `[US1]`/`[US2]` maps task to specific user story for traceability
- No test tasks — tests not requested in specification
- `dx serve` handles Tailwind CSS v4 automatically — no manual CSS build step
- The `rusqlite` `bundled` feature means no system SQLite required; cross-platform build is self-contained
- Linux only: if window appears black/blank, the `WEBKIT_DISABLE_COMPOSITING_MODE` workaround in `src/main.rs` (T005) handles this automatically in debug builds
- Commit after each phase or logical group (T016 is the final commit for the full scaffold)
