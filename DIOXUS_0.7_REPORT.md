# Dioxus 0.7 Research Report

This report synthesizes the findings from the parallel research tracks on Dioxus 0.7.

## 1. Core & Reactivity
Dioxus 0.7 introduces a more ergonomic RSX syntax and a robust signal-based reactivity system.

### RSX & Components
- **Enhanced Syntax**: Supports shorthand attributes (e.g., `class` for `class: "{class}"`), direct interpolation in strings, and native Rust control flow (`if`, `else`, `for`) inside the `rsx!` macro.
- **Component Macro**: `#[component]` allows direct function arguments as props, with support for defaults via `#[props(default = ...)]`.

### Signals & Hooks
- **Signals**: `use_signal` is the primary state primitive. Signals are `Copy` by default and automatically track dependencies.
- **Global State**: `Signal::global(|| ...)` (using `GlobalSignal`) allows for application-wide state.
- **Async Data**: `use_resource` is the standard for reactive async data fetching.
- **Memoization**: `use_memo` and `use_effect` are fully integrated with the signal system for automatic tracking.

## 2. Fullstack & Routing
Unifies routing, SSR, and server functions into a cohesive framework built on Axum.

### Routing
- **Typesafe Routing**: Routes are defined using a central `Routable` enum.
- **Shared UI**: Supports `#[layout]` and `#[nest]` for complex navigation structures.
- **Navigation**: `use_navigator` provides programmatic, typesafe navigation.

### SSR & Server Functions
- **SSR & Hydration**: Renders initial HTML on the server. `use_server_future` fetches data during SSR and serializes it for the client.
- **Server Functions**: The `#[server]` macro enables frontend-backend communication, with direct access to Axum extractors (headers, cookies, state).
- **Streaming**: Supports HTML streaming and `SuspenseBoundary` for pushing components as they resolve.

## 3. Platforms & Tooling
Deepens cross-platform support and improves the development experience.

### Platform Support
- **Web**: Optimized WASM with hydration support.
- **Desktop**: WebView-based (mature) or Blitz (experimental, GPU-accelerated, pure-Rust renderer).
- **Mobile**: Native iOS/Android support with a new FFI bridge for Swift/Kotlin.

### dx CLI
- **Hot-patching**: `dx serve --hotpatch` enables sub-second logic updates (not just UI).
- **Bundling**: `dx bundle` creates native installers (.dmg, .msi, .deb, .aab).
- **Tailwind**: Zero-config support for Tailwind CSS v4.

## 4. Ecosystem & Performance
Integrated asset management and specialized optimization tools.

### Assets & Styling
- **Manganis**: The `asset!()` macro handles automatic collection, optimization, and bundling of images, CSS, and fonts.
- **Styling**: Native integration with Tailwind CSS and CSS modules.

### Performance & Testing
- **Stores API**: Designed for reactive handling of complex nested collections (Vec/HashMap).
- **Optimization**: Fine-grained reactivity ensures only necessary components re-render.
- **Testing**: `dioxus-testing` (0.7) provides tools for headless component testing and integration testing with `tokio`.

---

## Idiomatic Code Example (0.7)

```rust
use dioxus::prelude::*;

static COUNT: GlobalSignal<i32> = Signal::global(|| 0);

#[component]
fn App() -> Element {
    let mut name = use_signal(|| "Guest".to_string());
    let title = use_memo(move || format!("Hello, {name}!"));

    rsx! {
        h1 { "{title}" }
        p { "Global Count: {COUNT}" }
        button { onclick: move |_| *COUNT.write() += 1, "Increment Global" }
        input { 
            oninput: move |e| name.set(e.value()), 
            value: "{name}" 
        }
        Child { name }
    }
}

#[component]
fn Child(name: ReadOnlySignal<String>) -> Element {
    rsx! { p { "Child sees: {name}" } }
}
```
