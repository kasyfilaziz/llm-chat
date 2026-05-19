# Data Model: Core Interface Pages

**Feature**: Session Management & Chat Interface
**Branch**: `004-session-chat-interface`
**Date**: 2026-05-18

## Entity-Relationship Diagram

```
SidebarNavItem (static config, not persisted)
└── label: String
└── route: String
└── icon: String
└── disabled: bool
└── tooltip: Option<String>

Folder
├── id: String (UUID v4)
├── name: String
├── parent_id: Option<String>          → Folder.id (self-referential)
├── sort_order: i64
├── created_at: String (ISO 8601)
├── deleted_at: Option<String>         → soft-delete marker
│
├── children: Vec<Folder>              → resolved via parent_id
└── sessions: Vec<Session>             → resolved via folder_id

Session (maps to existing conversations table)
├── id: String (UUID v4)
├── title: String
├── folder_id: Option<String>          → Folder.id (nullable = uncategorized)
├── created_at: i64 (unix timestamp)
├── updated_at: i64 (unix timestamp)
├── deleted_at: Option<i64>            → soft-delete marker (null = active)
├── is_pinned: bool (default false)
│
├── tags: Vec<SessionTag>              → resolved via session_tags table
├── messages: Vec<Message>             → resolved via messages.conversation_id
└── turn_count: usize                  → computed: COUNT(messages) / 2

Message
├── id: String (UUID v4)
├── conversation_id: String            → Session.id
├── role: String ("user" | "assistant" | "system")
├── content: String
└── created_at: i64 (unix timestamp)

SessionTag (join table)
├── session_id: String                 → Session.id
└── tag: String                        → free-form text
```

## SQLite Schema

### New: `folders` table

```sql
CREATE TABLE IF NOT EXISTS folders (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    parent_id  TEXT REFERENCES folders(id) ON DELETE SET NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX idx_folders_parent_id ON folders(parent_id);
CREATE INDEX idx_folders_sort_order ON folders(sort_order);
```

### Migration: add `folder_id` to `conversations`

```sql
ALTER TABLE conversations ADD COLUMN folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL;
ALTER TABLE conversations ADD COLUMN deleted_at INTEGER;
ALTER TABLE conversations ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0;

CREATE INDEX idx_conversations_folder_id ON conversations(folder_id);
CREATE INDEX idx_conversations_deleted_at ON conversations(deleted_at);
```

### New: `session_tags` table

```sql
CREATE TABLE IF NOT EXISTS session_tags (
    session_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    tag        TEXT NOT NULL,
    PRIMARY KEY (session_id, tag)
);

CREATE INDEX idx_session_tags_session_id ON session_tags(session_id);
```

### View: `session_turn_counts`

```sql
CREATE VIEW IF NOT EXISTS session_turn_counts AS
SELECT
    conversation_id AS session_id,
    COUNT(*) / 2    AS turn_count
FROM messages
GROUP BY conversation_id;
```

## Key Relationships

| From         | Relationship | To               | Via                   | Cardinality |
|--------------|-------------|------------------|-----------------------|-------------|
| Folder       | parent      | Folder           | folders.parent_id     | 1→N         |
| Folder       | contains    | Session          | conversations.folder_id | 1→N       |
| Session      | belongs to  | Folder           | conversations.folder_id | N→1       |
| Session      | has         | Message          | messages.conversation_id | 1→N       |
| Session      | tagged as   | SessionTag       | session_tags          | 1→N         |

**Integrity rules:**

- Deleting a folder with `ON DELETE SET NULL` moves child sessions to uncategorized (folder_id = NULL).
- Deleting a session cascades to its messages and tags.
- Folders with `parent_id = NULL` are root-level folders.
- A session with `folder_id = NULL` is "Uncategorized" — the UI displays it under a virtual "Uncategorized" heading.

## Validation Rules

### Folder

| Rule | Description | Enforcement |
|------|-------------|-------------|
| Non-empty name | `name.trim().is_empty()` must be false | Application layer before INSERT/UPDATE |
| No self-parent | `parent_id != id` | Application layer; also impossible at DB due to FK referencing different row |
| Acyclic parent chain | No folder can be its own ancestor | Application layer (traverse parent chain before UPDATE) |
| Unique sort within parent | Sort order should be unique per parent scope | Application layer (re-index on reorder) |

### Session

| Rule | Description | Enforcement |
|------|-------------|-------------|
| Single folder | `folder_id` can only reference one folder | Column constraint (single value) |
| Turn count derivation | Turn count never directly stored — always computed | SQL view or query-time COUNT/2 |
| Title fallback | If title is empty, display first 50 chars of first user message | Application layer in screen.rs |
| Soft-delete only | `deleted_at` set on delete, row never physically removed by user action | Application layer (DELETE queries forbidden for sessions) |

### Message

| Rule | Description | Enforcement |
|------|-------------|-------------|
| Valid role | Role must be one of `user`, `assistant`, `system` | CHECK constraint |
| Non-empty content | Content must not be NULL or empty string | Application layer |

## State Transitions

### Folder Lifecycle

```
[active] ──────────────────────────────────────────────► [deleted]
    │                                                         │
    │ user deletes folder                                     │ deleted_at IS NOT NULL
    │                                                         │
    │ side effects:                                           │ side effects:
    │                                                         │
    │   SET NULL on child folder.parent_id                    │   cascade sessions to
    │   (promotes subfolders to root)                         │   uncategorized
    │                                                         │   (folder_id = NULL)
    │   SET NULL on child session.folder_id                   │
    │   (moves sessions to "Uncategorized")                   │
    │                                                         │
    └─────────────────────────────────────────────────────────┘
```

- **Active**: Folder is visible in the directory tree. `deleted_at IS NULL`.
- **Deleted**: Folder is hidden from the directory tree. `deleted_at IS NOT NULL`.
- `ON DELETE SET NULL` on both `parent_id` and `folder_id` foreign keys ensures no orphan constraints when a referenced folder is deleted.
- Re-parenting a folder's children to the grandparent (rather than SET NULL) is handled at the application layer before the DELETE.

### Session Lifecycle

```
[active] ──► [deleted (soft)] ──5s timer──► [permanently deleted]
                 │
                 │ user clicks "Undo"
                 ▼
             [active] (restore)
```

- **Active**: Session is visible in the grid. `deleted_at IS NULL`.
- **Deleted (soft)**: Session hidden from grid, appears in trash. `deleted_at` set to current timestamp. User sees undo toast for 5 seconds.
- **Permanently deleted**: After undo window expires (5s) or on explicit trash-emptying. Row physically removed from `conversations` (cascade deletes messages and tags).
- **Restore**: On undo, `deleted_at` set back to NULL. Session returns to its original folder.

### Tag Lifecycle

```
Tags are ephemeral — created and destroyed by user editing a session's tags.
No soft-delete. Tags are INSERTed/DELETEd in the session_tags table atomically
when the user saves a session's tag list.
```
