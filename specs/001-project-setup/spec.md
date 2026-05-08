# Feature Specification: Project Initialization & Environment Setup

**Feature Branch**: `001-project-setup`  
**Created**: 2026-05-03  
**Status**: Draft  
**Input**: User description: "Set up the Lumina project scaffold (Rust/Dioxus 0.7, Tailwind CSS v4, SQLite stub) targeting Desktop platforms."

## Clarifications

### Session 2026-05-08

- Q: What is the platform target scope for this initial setup feature? → A: Desktop only (Windows, macOS, Linux); Mobile (iOS, Android) is out of scope for this feature.
- Q: What is the definition of success for the project setup? → A: Project compiles and opens a desktop window with a minimal placeholder UI.
- Q: Should Tailwind CSS v4 be integrated as part of this setup? → A: Yes, include Tailwind CSS v4 wiring as part of this setup using Dioxus CLI zero-config integration.
- Q: Should a database dependency be included in this setup? → A: Yes, add SQLite as a stub dependency in `Cargo.toml` (no active schema or migrations required at this stage).
- Q: How many distinct developer user stories should this setup feature cover? → A: Two stories — (1) compile and run the desktop app; (2) run the `dx serve` dev server with hot-reload.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Compile and Launch Desktop App (Priority: P1)

A developer with Rust installed clones the Lumina repository, follows the quickstart document, runs the documented build command, and sees a native desktop window displaying the Lumina placeholder UI on their machine.

**Why this priority**: Without a compiling, runnable project there is no foundation for any other Lumina development. This story is the absolute prerequisite for all subsequent features.

**Independent Test**: Can be fully tested by cloning the repository on a fresh machine (with only Rust stable and Dioxus CLI installed), running the documented command, and verifying the Lumina window appears with the app name and version visible.

**Acceptance Scenarios**:

1. **Given** a developer has Rust stable (1.80.0+) and Dioxus CLI installed, **When** they run the documented build/run command, **Then** the project compiles without errors and a native desktop window opens displaying the Lumina placeholder UI.
2. **Given** the compiled binary exists, **When** it is executed on Windows, macOS, or Linux, **Then** a native desktop window appears showing at minimum the application name and version.
3. **Given** the developer follows the quickstart document from a fresh repository clone, **When** they complete all documented steps, **Then** they reach a running desktop window without requiring undocumented steps.

---

### User Story 2 - Development Server with Hot-Reload (Priority: P2)

A developer starts the Dioxus development server (`dx serve`), edits a source file, and observes the change reflected in the running desktop window within seconds — without manually restarting the application.

**Why this priority**: Hot-reload dramatically accelerates UI development iteration. It is the core daily workflow for all future Lumina feature development and is most productive when established from day one of the project scaffold.

**Independent Test**: Can be fully tested by running `dx serve`, modifying a visible UI string in the source code, saving the file, and confirming the desktop window updates automatically within a few seconds.

**Acceptance Scenarios**:

1. **Given** the project is scaffolded, **When** the developer runs `dx serve`, **Then** the development server starts and a desktop window opens without errors.
2. **Given** the development server is running, **When** a Rust source file is modified and saved, **Then** the running desktop window reflects the change without requiring a manual restart.

---

### Edge Cases

- What happens when the developer's Rust version is below the minimum required (1.80.0)? The build MUST fail with a clear, actionable version error message.
- What happens when the Dioxus CLI is not installed? The quickstart document MUST clearly list it as a prerequisite with an installation link.
- What happens on Linux when system WebView dependencies (e.g., `libwebkit2gtk-4.1`) are missing? The quickstart document MUST list required system packages per distro.
- What happens if `dx serve` is run while another instance is already running on the same port? The dev server MUST report the port conflict and exit cleanly.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The project MUST provide a Rust dependency manifest (`Cargo.toml`) declaring all required Dioxus 0.7.x crates and their desktop feature flags.
- **FR-002**: The project MUST include a Dioxus project configuration file (`Dioxus.toml`) targeting desktop (system native WebView) as the primary platform.
- **FR-003**: The project MUST integrate Tailwind CSS v4 using the Dioxus CLI zero-config integration, with Tailwind processing active during both development and build.
- **FR-004**: The project MUST compile and produce a runnable native desktop binary on Windows, macOS, and Linux without platform-specific manual steps beyond documented prerequisites.
- **FR-005**: On launch, the desktop binary MUST open a window displaying a minimal Lumina placeholder UI (app name and version at minimum).
- **FR-006**: The project MUST include a developer quickstart document describing the exact prerequisite installation steps and commands needed to reach a running desktop window from a fresh clone.
- **FR-007**: The project MUST declare SQLite as a stub dependency in `Cargo.toml` (via `rusqlite` or `sqlx` with the SQLite feature); no active schema or migrations are required at this stage.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The project builds without compilation errors on Windows, macOS, and Linux using the standard Dioxus desktop build command.
- **SC-002**: Running the compiled binary opens a native desktop window displaying a minimal Lumina placeholder UI (e.g., app name and version visible).
- **SC-003**: A developer with Rust installed can clone the repository and reach a running desktop window in a single documented sequence of commands.
- **SC-004**: The project structure, dependency manifest, and build configuration are committed to version control and reproducible across machines.

## Assumptions


- Target developer is setting up the project for the first time on their local machine.
- **Mobile platform setup (iOS, Android) is out of scope for this feature**; it will be addressed in a separate feature branch.
- The developer's machine has internet access for downloading toolchain dependencies (Rust, Dioxus CLI).
- No pre-existing Lumina codebase exists; this is a greenfield project initialization.
