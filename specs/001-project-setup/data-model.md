# Data Model: Project Initialization & Environment Setup

**Branch**: `001-project-setup` | **Date**: 2026-05-08

---

## Overview

This feature establishes the project scaffold — it does not introduce persistent data entities, database schemas, or runtime state models. The SQLite dependency is declared as a **stub** with no schema or migrations at this stage.

The "data" of this feature is the project's **file system structure and configuration files**, documented below as the canonical reference for all subsequent features.

---

## Configuration File Entities

### 1. `Cargo.toml` — Rust Project Manifest

| Field | Value | Notes |
|---|---|---|
| `[package].name` | `"lumina"` | Canonical app name per Constitution |
| `[package].version` | `"0.1.0"` | SemVer, starts at pre-release |
| `[package].edition` | `"2021"` | Rust 2021 edition (compatible with 1.80+) |
| `[dependencies].dioxus` | `{ version = "0.7", features = ["desktop"] }` | Core UI framework + desktop WebView |
| `[dependencies].rusqlite` | `{ version = "0.31", features = ["bundled"] }` | SQLite stub, self-contained binary |

### 2. `Dioxus.toml` — Dioxus CLI Configuration

| Field | Value | Notes |
|---|---|---|
| `[application].name` | `"lumina"` | Must match Cargo.toml package name |
| `[application].default_platform` | `"desktop"` | Eliminates need for `--platform` flag |
| `[bundle].identifier` | `"dev.lumina.app"` | Reverse-DNS bundle ID for future packaging |
| `[bundle].icon` | `["assets/icon.png"]` | Stub path; icon asset created as placeholder |

---

## Source File Structure

```text
lumina/                         # repository root
├── Cargo.toml                  # Rust manifest
├── Dioxus.toml                 # Dioxus CLI config
├── assets/
│   ├── icon.png                # Placeholder app icon (stub)
│   └── main.css                # Tailwind CSS entry point (auto-managed by dx CLI)
└── src/
    └── main.rs                 # Application entry point + root component
```

### `src/main.rs` Structure

The entry point follows the idiomatic Dioxus 0.7 pattern:

```rust
use dioxus::prelude::*;

fn main() {
    // Linux WebView compositing workaround (dev builds only)
    #[cfg(all(target_os = "linux", debug_assertions))]
    unsafe { std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1"); }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center min-h-screen bg-gray-950",
            h1 { class: "text-4xl font-bold text-white", "Lumina" }
            p  { class: "text-gray-400 mt-2", "v0.1.0 — Project initialized" }
        }
    }
}
```

---

## State Model

None at this stage. Dioxus Signals state management is established as the pattern (per Constitution) but no signals are instantiated in the scaffold beyond the root component.

---

## Future Data Model Notes

- **Session persistence** (SQLite): to be designed in a future `002-session-storage` feature.
- **API key storage**: local keyring integration, future security feature.
- **MCP tool registry**: future extensibility feature.
