# Feature Specification: Core Interface Pages — Session Management & Chat Interface

**Feature Branch**: `004-session-chat-interface`  
**Created**: 2026-05-18  
**Status**: Draft  
**Input**: User description: "Create a specification document based on conversation about session management page and chat interface page"

## Clarifications

### Session 2026-05-18

- Q: Should users be able to delete individual chat sessions from the session grid? → A: Yes, with reversible deletion (trash/undo mechanism).
- Q: What should users see when the session grid is empty or loading? → A: Loading spinner during load, then empty state with CTA ("No conversations yet" + "Start a Chat" button) if empty.
- Q: How should category tags on session cards originate? → A: User-assignable free-form tags, typed when creating/editing a session.
- Q: How should deferred sidebar links (Context Vault, Hooks) behave? → A: Shown in sidebar as disabled/placeholder items with "Coming soon" tooltip.

## Decision Records

Implementation approach decisions were made via the deep-tree (Think-Decide) framework. Full analysis documents are at `.tot/session_management_page/` and `.tot/chat_interface_implementation/`.

### Session Management Page

- **Accepted**: Solution 3 — Vertical-Slice Incremental (`.tot/session_management_page/accept_solution_3.md`)
- **Approach**: Build in 4 sequential vertical slices: (1) router refactor + shared layout with sidebar, (2) static UI scaffold, (3) SQLite folders + CRUD, (4) session grid + drag-and-drop. Each slice touches UI, state, and DB in one shot.
- **Rationale**: Lowest risk for a Rust learner. Each slice is independently testable and provides working progress. Follows existing domain pattern (`domains/sessions/` mirroring `domains/chat/`).

### Chat Interface

- **Accepted**: Solution 1 — Widget-by-Widget Surgery (`.tot/chat_interface_implementation/accept_solution_1.md`)
- **Approach**: Keep existing `screen.rs` orchestrator (coroutine, store, effects). Replace visual components one at a time: message bubbles, input area, hover action bar, typing indicator, prompt suggestion chips. Each swap is isolated and compilable.
- **Rationale**: Fastest path to working chat (3-5h estimate). Preserves proven async streaming/DB logic. Minimal risk of breaking the coroutine chain.

## User Scenarios & Testing

### User Story 1 — Organize Conversations in Folders (Priority: P1)

As a user, I want to organize my AI chat sessions into folders and subfolders so that I can group related conversations by project or topic and quickly find past sessions.

**Why this priority**: Folder organization is the foundational capability of the Session Management page. Without it, users have no way to navigate or structure their conversations, making the landing page a flat list with no value over the existing chat view.

**Independent Test**: Can be fully tested by creating multiple folders with subfolders, verifying each appears in the directory tree with correct session counts, and that clicking a folder navigates to show its sessions.

**Acceptance Scenarios**:

1. **Given** the user is on the Session Management landing page, **When** they click "New Folder", **Then** a new folder appears in the directory tree with an editable name.
2. **Given** a folder exists in the directory tree, **When** the user right-clicks or uses the action menu, **Then** they can rename or delete the folder.
3. **Given** a folder has child subfolders, **When** the user expands the parent, **Then** subfolders are displayed indented beneath it.
4. **Given** the user has multiple folders, **When** they click on a folder name, **Then** the right panel updates to show only sessions belonging to that folder.

---

### User Story 2 — View and Browse Chat Sessions (Priority: P1)

As a user, I want to see all my chat sessions displayed as cards in a visual grid so that I can quickly identify and open the conversation I need based on its title, description, date, and turn count.

**Why this priority**: The bento grid of session cards is the primary content view of the landing page. Without it, users cannot see or access their conversations from the home screen.

**Independent Test**: Can be fully tested by creating multiple chat sessions and verifying they appear as cards in the grid with correct metadata.

**Acceptance Scenarios**:

1. **Given** the user has existing chat sessions, **When** they land on the Session Management page, **Then** sessions are displayed as cards in a responsive grid layout with icon, category tag, title, truncated description, date, and turn count.
5. **Given** the session data is still loading, **Then** a loading spinner is displayed in place of the grid.
6. **Given** the user has no chat sessions, **When** they land on the Session Management page, **Then** an empty state message ("No conversations yet") is shown with a "Start a Chat" button that navigates to the Chat Interface.
2. **Given** the user hovers over a session card, **Then** a visual accent animation plays on the card border/top.
3. **Given** a session is marked as pinned/important, **Then** its card spans a wider area in the grid and displays a pin indicator.
4. **Given** the user clicks on a session card, **Then** the app navigates to the Chat Interface with that session loaded.

---

### User Story 3 — Chat with AI Assistant (Priority: P1)

As a user, I want to send messages and receive streaming AI responses in a real-time chat interface so that I can have natural, fluid conversations with the language model.

**Why this priority**: The chat interface is the core functionality of the entire application. Without it, the app provides no value.

**Independent Test**: Can be fully tested by typing a message, sending it, and observing the AI response stream in real-time with proper message bubble formatting.

**Acceptance Scenarios**:

1. **Given** the user is on the Chat Interface page, **When** they type a message and press Enter or click Send, **Then** the message appears as a user bubble with avatar and the AI begins streaming a response.
2. **Given** the AI is generating a response, **When** tokens arrive, **Then** they appear incrementally in the AI message bubble in real-time.
3. **Given** the AI message bubble contains structured content, **Then** headings, lists, bold text, and block quotes render with proper formatting.
4. **Given** the user opens an existing conversation, **Then** all prior messages are loaded and displayed in the chat canvas.

---

### User Story 4 — Move Sessions Between Folders (Priority: P2)

As a user, I want to drag a session card from one folder to another so that I can reorganize my conversations as my projects evolve.

**Why this priority**: Folder organization is incomplete without the ability to move existing sessions between folders. This enables long-term workspace maintenance.

**Independent Test**: Can be fully tested by dragging a session card from one folder view to another folder and verifying the session's folder association updates.

**Acceptance Scenarios**:

1. **Given** the user is viewing sessions in a folder, **When** they drag a session card to a destination folder in the directory tree, **Then** the session moves to the destination folder and disappears from the source.
2. **Given** a drag operation fails, **Then** the session remains in its original folder with no data loss.

---

### User Story 5 — Navigate Using Persistent Sidebar (Priority: P2)

As a user, I want a persistent sidebar available on every page so that I can navigate between Chat, Sessions, Context Vault, Hooks, and Settings without returning to a home screen.

**Why this priority**: The sidebar provides global navigation. While each page works independently, the sidebar is essential for the overall app experience.

**Independent Test**: Can be fully tested by verifying the sidebar renders on both the Session Management page and the Chat Interface page with consistent navigation items.

**Acceptance Scenarios**:

1. **Given** the user is on any page, **Then** the sidebar is visible on the left with navigation items: Chat, Sessions (highlighted as active), Context Vault (disabled), Hooks (disabled), Settings, Help, Logout.
2. **Given** the user clicks "Sessions" in the sidebar, **Then** they are taken to the Session Management landing page.
3. **Given** the user clicks "Chat" in the sidebar, **Then** they are taken to the Chat Interface page.
4. **Given** the user clicks "Context Vault" or "Hooks" in the sidebar, **Then** nothing happens or a "Coming soon" tooltip is shown.

---

### User Story 6 — Interact with AI Response Actions (Priority: P3)

As a user, I want to copy, regenerate, or give feedback on AI responses so that I can refine the output and reuse content.

**Why this priority**: These actions enhance the chat experience but are not required for the core chat function to work.

**Independent Test**: Can be fully tested by hovering over an AI message, observing the action bar appear, and testing each action button.

**Acceptance Scenarios**:

1. **Given** the user hovers over an AI message, **Then** a toolbar with Copy, Regenerate, and Thumbs Up icons appears below the message.
2. **Given** the user clicks Copy, **Then** the message content is copied to the clipboard.
3. **Given** the user clicks Regenerate, **Then** the AI generates a new response for the same prompt.

---

### Edge Cases

- What happens when a user tries to delete a folder that still contains sessions? — Sessions should be moved to an "Uncategorized" default folder or the user should be prompted to confirm and choose a destination.
- What happens when a user deletes a session? — The session is soft-deleted and placed in a trash state. An undo option is shown temporarily (e.g., 5 seconds). After the undo window expires, the session is permanently deleted.
- How does the system handle a very large number of sessions (1000+) in a single folder? — The grid should handle empty, single, and many session states gracefully without performance degradation.
- What happens when the LLM stream is interrupted mid-response (network failure, API error)? — The partial response should be preserved and the user should see an error indicator.
- How does the system behave when the user sends a message while the AI is still streaming a previous response? — The streaming response should be canceled and the new request should begin.
- What happens when a folder name is left empty during creation? — A default name like "New Folder" should be assigned.
- How does the drag-and-drop interaction work on devices without a mouse? — Desktop-first; the drag interaction assumes a pointing device.

## Requirements

### Functional Requirements

- **FR-001**: System MUST display the Session Management page as the default landing page when the application starts.
- **FR-002**: System MUST provide a persistent sidebar navigation visible on all pages with links to Chat, Sessions, Context Vault (disabled/coming soon), Hooks (disabled/coming soon), Settings, Help, and Logout.
- **FR-003**: Users MUST be able to create new folders with a user-defined name in the directory explorer.
- **FR-004**: Users MUST be able to rename existing folders.
- **FR-005**: Users MUST be able to delete folders.
- **FR-006**: Users MUST be able to create nested subfolders within a folder (at least one level of nesting).
- **FR-007**: System MUST display session cards in a responsive grid layout showing: icon, free-form user-assignable tag, title, truncated description (2-line max), date, and turn count.
- **FR-008**: System MUST support a pinned/featured session card that spans a wider grid column with a pin indicator.
- **FR-009**: Users MUST be able to click a session card to navigate to the Chat Interface with that session loaded.
- **FR-010**: Users MUST be able to drag a session card to a different folder in the directory tree to reassign it.
- **FR-011**: System MUST persist all folders, folder hierarchy, and session-folder associations to local storage.
- **FR-012**: System MUST display a real-time chat canvas with user messages (avatar + bubble) and AI messages (icon + formatted content).
- **FR-013**: System MUST render AI responses with rich formatting including headings, lists, bold text, and block quotes.
- **FR-014**: System MUST show a typing indicator (animated dots) while the AI is generating a response.
- **FR-015**: System MUST provide a fixed input area at the bottom of the chat canvas with: text input, send button, model selector, and attachment button.
- **FR-016**: System MUST display prompt suggestion chips above the input area for quick actions.
- **FR-017**: System MUST show a hover action bar on AI messages with Copy, Regenerate, and Thumbs Up options.
- **FR-018**: System MUST apply the Terra dark theme design system consistently across both pages.
- **FR-019**: System MUST stream AI responses token-by-token in real-time, updating the message bubble incrementally.
- **FR-020**: Users MUST be able to delete individual chat sessions from the session card grid. Deletion MUST include an undo mechanism (e.g., soft-delete with trash or timed undo prompt).
- **FR-021**: System MUST display a confirmation prompt before destructive actions (folder deletion, session deletion) to prevent accidental data loss.

### Key Entities

- **Folder**: A container for organizing chat sessions. Has a name, an optional parent folder (for nesting), and an ordered position within its parent. Contains zero or more sessions.
- **Session**: A single conversation with the AI. Contains messages, belongs to a folder (or is uncategorized), has a title, creation date, last-updated date, turn count, and one free-form user-assignable tag. Can be pinned for prominence.
- **Message**: An individual exchange within a session. Has a role (user or assistant), content (text with optional formatting), timestamp, and an ordering position within the session.
- **Sidebar Navigation**: A persistent UI element listing available application sections. Highlights the currently active section.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can create a folder, rename it, create a subfolder, and delete a folder — all within 10 seconds each, with no data loss.
- **SC-002**: Users can view all their chat sessions organized in the folder grid within 2 seconds of landing on the page.
- **SC-003**: Users can send a message and see the first AI response token appear within 3 seconds on a standard broadband connection.
- **SC-004**: Users can navigate between Session Management and Chat Interface via the sidebar in under 1 second with no page reload.
- **SC-005**: Users can complete the core loop (open app → browse sessions → open session → chat → return to sessions) without encountering errors or confusion.
- **SC-006**: All interactive elements (buttons, links, drag handles) provide visual feedback within 100ms of user action.
- **SC-007**: The chat interface maintains 60fps responsiveness during token streaming, with no input blocking or visual jank.

## Assumptions

- The app targets desktop platforms (Windows, macOS, Linux) with a mouse/trackpad; mobile drag-and-drop is out of scope for v1.
- Users have existing chat sessions stored in the local database that can be surfaced on the Session Management page.
- The existing LLM provider configuration (API keys, endpoints, model selection) is already functional and does not need to be re-implemented.
- The Terra dark theme color palette, typography (Plus Jakarta Sans, Nunito Sans), and Material Symbols icon library are available or will be configured as part of implementation.
- Folder hierarchy supports at least one level of nesting (parent → child); deeper nesting is a future enhancement.
- The sidebar navigation structure is defined at build time and does not need dynamic customization by the user.
- Chat sessions created before this feature retain backward compatibility — they appear under an "Uncategorized" or "General" default folder.
- The existing `conversations` table in the database maps to "sessions" in this spec. A `folder_id` column will be added to associate sessions with folders.
- The Core UI Pages will be implemented under a new `src/domains/sessions/` domain following the same `repo.rs` / `state.rs` / `screen.rs` / `widgets/` pattern as the existing `domains/chat/`.
- The chat interface refactoring preserves the existing `use_coroutine` actor, `ConversationStore`, and `spawn_blocking` DB pattern — only the RSX rendering changes.
- Icons may use Unicode/emoji fallbacks instead of Google Material Symbols font, since desktop webview CDN loading is unreliable.
