# Problem
Implement Tailwind CSS v4 and Markdown formatting in the current Tracer Bullet phase (Dioxus 0.7, Rust). The goal is to render LLM-generated messages with rich formatting (headers, code blocks, lists) while adhering to the project's modern, CSS-first architecture and performance goals.

# User Profile
| Technical Level | Role | Output Style |
| :--- | :--- | :--- |
| Expert | Developer/Architect | Full technical detail, architectural rationale |

# Assumptions
- **Tailwind CSS v4** is the primary styling engine, utilizing its CSS-first `@import "tailwindcss";` architecture.
- **Dioxus 0.7** is the UI framework, capable of rendering raw HTML safely for trusted local LLM output.
- **pulldown-cmark** is the preferred Rust crate for high-performance, CommonMark-compliant parsing.
- The UI should remain responsive (60fps) during message streaming, necessitating efficient parsing.

# Solution Summary
Use Tailwind v4's CSS-first architecture to create a centralized styling bridge between Markdown-generated HTML and Tailwind's utility system. A `.markdown-content` class is defined in the main CSS entry point, using standard CSS nesting and the `@apply` directive to style raw HTML tags (e.g., `<h1>`, `<code>`, `<ul>`). This approach keeps the Rust UI code clean and leverages the browser's native CSS engine for styling.

# Detailed Implementation

### 1. Dependency Management
Add the `pulldown-cmark` crate to `Cargo.toml` to handle the conversion of Markdown strings to HTML.
```bash
cargo add pulldown_cmark
```

### 2. CSS-First Styling (assets/main.css)
Leverage Tailwind v4's native CSS nesting to style the output of the Markdown parser. By scoping styles under `.markdown-content`, we avoid global tag pollution while ensuring rich formatting.

```css
@import "tailwindcss";

@theme {
  /* Theme overrides if necessary */
}

/* Markdown Styling Bridge */
.markdown-content {
  @apply text-slate-200 leading-relaxed break-words;

  /* Typography */
  h1 { @apply text-2xl font-bold mt-6 mb-4 border-b border-slate-700 pb-2; }
  h2 { @apply text-xl font-semibold mt-5 mb-3; }
  h3 { @apply text-lg font-medium mt-4 mb-2; }
  p { @apply mb-4 last:mb-0; }

  /* Lists */
  ul { @apply list-disc ml-6 mb-4; }
  ol { @apply list-decimal ml-6 mb-4; }
  li { @apply mb-1; }

  /* Code Blocks */
  code {
    @apply bg-slate-800 px-1.5 py-0.5 rounded font-mono text-sm text-pink-400;
  }

  pre {
    @apply bg-slate-900 p-4 rounded-lg overflow-x-auto mb-4 border border-slate-800;
    code {
      @apply bg-transparent p-0 text-slate-300 block;
    }
  }

  /* Blockquotes & Links */
  blockquote {
    @apply border-l-4 border-slate-600 pl-4 italic my-4 text-slate-400;
  }
  a {
    @apply text-blue-400 underline hover:text-blue-300;
  }
}
```

### 3. Markdown Rendering Logic (src/utils/markdown.rs)
Create a helper function to encapsulate the parsing logic.

```rust
use pulldown_cmark::{Parser, Options, html};

pub fn render_markdown(source: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(source, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
```

### 4. Component Integration (src/domains/chat/widgets/message.rs)
Update the `MessageBubble` to render the parsed HTML within a `div` carrying the `.markdown-content` class.

```rust
#[component]
pub fn MessageBubble(message: Message) -> Element {
    let is_user = message.role == "user";
    let bg_color = if is_user { "bg-blue-600" } else { "bg-slate-700" };
    let align = if is_user { "justify-end" } else { "justify-start" };

    // Parse markdown to HTML
    let html_content = crate::utils::markdown::render_markdown(&message.content);

    rsx! {
        div { class: "flex w-full {align} mb-4",
            div { 
                class: "max-w-[80%] px-4 py-2 rounded-2xl {bg_color} text-white shadow-lg",
                div { 
                    class: "markdown-content text-sm",
                    dangerous_inner_html: "{html_content}"
                }
            }
        }
    }
}
```

# Trade-offs
| Metric | Rating | Rationale |
| :--- | :--- | :--- |
| **Complexity** | Low | Leverages existing CSS standards and a mature Rust library. |
| **Time** | Low | Rapid implementation with minimal boilerplate. |
| **Reversibility** | High | Styles are contained in one CSS class; parser can be easily swapped. |
| **Risk** | Low | `dangerous_inner_html` is safe for local, controlled LLM inputs. |
| **Scalability** | High | CSS-first approach handles large documents efficiently via browser native rendering. |
| **Maintainability** | High | Styling is centralized in CSS, not scattered in Rust components. |

# Consequences
- **Positive**: Clean separation of concerns; the Rust code only handles data flow while CSS handles presentation. High performance due to native browser styling.
- **Risks**: Use of `dangerous_inner_html` requires vigilance if external (untrusted) Markdown sources are ever introduced.
- **Second-Order**: Empowers designers to tweak Markdown aesthetics directly in the CSS file without recompiling the entire Rust application (using Tailwind's JIT/CLI).

# Verdict
Best for architectural purity and performance in a Rust/Dioxus environment, as it minimizes "Utility Class Soup" in the component logic while maximizing the power of Tailwind v4's CSS-native capabilities.
