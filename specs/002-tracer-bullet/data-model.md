# Phase 1: Data Model

## Entity: Message

Represents a single chat bubble in the conversation history.

**Fields**:
- `id` (String): A UUID v4 string. Serves as the primary key.
- `role` (String): Enum-like value representing the sender. Either `"user"` or `"assistant"`.
- `content` (String): The text payload of the message.
- `created_at` (Integer): Unix timestamp (seconds or milliseconds) of when the message was created.

**SQLite Schema Example**:
```sql
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
```

**State Transitions**:
- For streaming assistant messages, the UI state will maintain a `Message` in memory with actively mutating `content`.
- The database is only written to twice per cycle: once for the initial user message insertion, and once when the assistant stream is fully completed.
