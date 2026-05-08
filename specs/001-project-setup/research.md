# Research: Project Initialization & Environment Setup

**Branch**: `001-project-setup` | **Date**: 2026-05-08  
**Input**: Technical unknowns extracted from spec + Technical Context

---

## Finding 1: Dioxus 0.7 Cargo.toml Dependencies

**Decision**: Use `dioxus = { version = "0.7", features = ["desktop"] }` as the sole Dioxus dependency.

**Rationale**: In Dioxus 0.7, all platform-specific runtime components (WebView via `wry`/`tao`, tokio runtime, devtools) are gated behind the `desktop` feature flag on the single `dioxus` crate. No need to list `dioxus-desktop` separately unless fine-grained control over renderer internals is required. This keeps the manifest clean and matches the recommended 0.7 setup pattern.

**Alternatives considered**:
- `dioxus-desktop` as a standalone dependency: redundant when `features = ["desktop"]` is sufficient; adds maintenance overhead.
- Pinning to `"0.7.7"`: unnecessary for a greenfield project; use `"0.7"` for compatible patch updates.

---

## Finding 2: Dioxus.toml Configuration

**Decision**: Use the following minimal `Dioxus.toml` structure:

```toml
[application]
name = "lumina"
default_platform = "desktop"

[bundle]
identifier = "dev.lumina.app"
icon = ["assets/icon.png"]
```

**Rationale**: `default_platform = "desktop"` makes `dx serve` and `dx build` default to desktop mode without flags. The `[bundle]` section is required for future `dx bundle` usage (installers). Icon path is set to a stub asset.

**Alternatives considered**:
- Omitting `Dioxus.toml`: forces developers to pass `--platform desktop` on every CLI command — poor DX.
- Complex bundle config now: premature; should be expanded in a dedicated packaging feature.

---

## Finding 3: Tailwind CSS v4 Integration

**Decision**: No manual Tailwind configuration required. Use Dioxus CLI 0.7's built-in zero-config Tailwind v4 support by writing Tailwind classes directly in `rsx!` macros.

**Rationale**: The `dx` CLI (0.7+) bundles a standalone Tailwind binary internally. It automatically scans Rust source files for class names, generates minimal CSS, and bundles it. No `tailwind.config.js`, `postcss.config.js`, or manual CLI installation needed. This satisfies FR-003 with zero additional configuration files.

**Alternatives considered**:
- Manual Tailwind CLI setup: adds unnecessary complexity and a separate tool dependency.
- CSS modules: diverges from Constitution mandate for Tailwind v4.
- Custom CSS input file: viable for overrides but not required at scaffold stage.

---

## Finding 4: SQLite Stub Dependency

**Decision**: Use `rusqlite = { version = "0.31", features = ["bundled"] }` as the SQLite stub.

**Rationale**: 
- `rusqlite` with `bundled` feature compiles the SQLite C library alongside the app, eliminating any system SQLite dependency. This is critical for cross-platform consistency (Windows has no system SQLite).
- Synchronous access is appropriate for a desktop app where DB calls happen on background threads.
- `sqlx` requires async setup (`tokio`, compile-time query validation against a live DB) which is overkill for a stub with no schema.
- The `bundled` feature aligns with the Constitution's local-first, self-contained binary principle.

**Alternatives considered**:
- `sqlx` with sqlite + runtime-tokio: deferred to when async DB access patterns are needed (future persistence feature).
- System SQLite (no `bundled`): unreliable on Windows; breaks cross-platform build guarantee.

---

## Finding 5: Linux System WebView Prerequisites

**Decision**: Document the following per-distro prerequisites in `quickstart.md`:

| Distro family | Install command |
|---|---|
| Ubuntu/Debian | `sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libssl-dev pkg-config build-essential` |
| Fedora | `sudo dnf install gtk3-devel webkit2gtk4.1-devel openssl-devel pkgconf-pkg-config gcc-c++` |
| Arch Linux | `sudo pacman -S gtk3 webkit2gtk-4.1 pkgconf base-devel` |

**Known gotcha**: On some Linux systems the WebView renders a black window due to a compositing bug. Workaround: set `std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1")` in `main()` (dev-only, guarded by `#[cfg(debug_assertions)]`).

**Rationale**: The `wry` crate (used by Dioxus desktop) requires `libwebkit2gtk-4.1` — specifically the **4.1** version, not 4.0. Missing these packages causes linker errors. Windows and macOS use their bundled system WebView (Edge WebView2 and WKWebView respectively) with no extra installation.

**Alternatives considered**:
- Blitz renderer (experimental GPU renderer): not stable enough for a baseline scaffold; contradicts Constitution constraint on experimental renderers.

---

## Finding 6: Hot-Reload / Dev Server

**Decision**: Use `dx serve` (default) for development. `dx serve --hotpatch` enables sub-second logic updates (not just RSX/CSS).

**Rationale**: Standard `dx serve` provides hot-reload for RSX templates and CSS. `--hotpatch` extends this to Rust logic changes. Both are zero-config. The distinction is worth documenting in quickstart for developer awareness.

**Alternatives considered**:
- `cargo run` + manual restart: eliminates hot-reload entirely; breaks US2.
- Third-party watch tools (`cargo-watch`): unnecessary given `dx serve` covers the use case natively.

---

## All NEEDS CLARIFICATION items: Resolved ✅

No outstanding unknowns. Proceeding to Phase 1.
