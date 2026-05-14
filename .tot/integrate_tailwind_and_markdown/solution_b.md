# Problem
Implement Tailwind CSS and Markdown formatting within the Dioxus 0.7 Tracer Bullet chat interface. The solution must provide full styling control via Tailwind CSS and strictly avoid the use of raw HTML or `dangerously_set_inner_html` to ensure security and native UI performance.

# User Profile
| Technical Level | Role | Output Style |
| :--- | :--- | :--- |
| Expert | Developer / Architect | Full Technical Detail, Concise |

# Assumptions
- The project is using Dioxus 0.7 and Tailwind CSS v4.
- `pulldown-cmark` is the preferred Markdown parsing library due to its event-based architecture.
- Performance is a priority; the rendering should happen efficiently during the Dioxus component lifecycle.
- Complex Markdown features like nested lists and code blocks are required.

# Solution Summary
Create a native Dioxus component that transforms a Markdown string into a Dioxus element tree by mapping `pulldown-cmark` events directly to RSX. By recursively consuming the event stream, we can generate a nested structure of Dioxus nodes where every element (headings, paragraphs, lists, etc.) is assigned specific Tailwind CSS classes.

# Detailed Implementation

### 1. Dependency Updates
Add `pulldown-cmark` to `Cargo.toml`:
```toml
[dependencies]
pulldown-cmark = "0.12"
```

### 2. Markdown Component Structure
Create `src/components/markdown.rs` to house the renderer. The core logic uses a recursive function to consume the flat event iterator and produce a nested RSX tree.

```rust
use dioxus::prelude::*;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

#[component]
pub fn Markdown(content: String) -> Element {
    let mut parser = Parser::new(&content);
    
    rsx! {
        div { class: "prose prose-invert max-w-none space-y-2",
            {render_events(&mut parser)}
        }
    }
}

fn render_events<'a>(parser: &mut impl Iterator<Item = Event<'a>>) -> Element {
    let mut elements = Vec::new();

    while let Some(event) = parser.next() {
        match event {
            Event::Start(tag) => elements.push(render_tag(tag, parser)),
            Event::End(_) => break, // Base case for recursion
            Event::Text(text) => elements.push(rsx! { "{text}" }),
            Event::Code(code) => elements.push(rsx! { 
                code { class: "bg-neutral-800 px-1 rounded text-sm font-mono", "{code}" } 
            }),
            Event::SoftBreak | Event::HardBreak => elements.push(rsx! { br {} }),
            _ => {}
        }
    }

    rsx! { {elements.into_iter()} }
}

fn render_tag<'a>(tag: Tag<'a>, parser: &mut impl Iterator<Item = Event<'a>>) -> Element {
    match tag {
        Tag::Heading { level, .. } => {
            let class = match level {
                pulldown_cmark::HeadingLevel::H1 => "text-3xl font-bold mb-4",
                pulldown_cmark::HeadingLevel::H2 => "text-2xl font-semibold mb-3",
                _ => "text-xl font-medium mb-2",
            };
            let tag_name = format!("{}", level); // e.g., "h1"
            // Dynamic tag rendering in Dioxus 0.7
            rsx! { 
                h1 { class: "{class}", {render_events(parser)} } 
            }
        }
        Tag::Paragraph => rsx! { p { class: "leading-relaxed", {render_events(parser)} } },
        Tag::List(None) => rsx! { ul { class: "list-disc ml-6 space-y-1", {render_events(parser)} } },
        Tag::List(Some(start)) => rsx! { ol { class: "list-decimal ml-6 space-y-1", start: "{start}", {render_events(parser)} } },
        Tag::Item => rsx! { li { {render_events(parser)} } },
        Tag::BlockQuote(..) => rsx! { blockquote { class: "border-l-4 border-neutral-600 pl-4 italic", {render_events(parser)} } },
        Tag::CodeBlock(..) => {
            // Special handling: CodeBlocks usually have a Text event followed by an End event
            let mut code = String::new();
            while let Some(next_event) = parser.next() {
                if let Event::Text(t) = next_event { code.push_str(&t); }
                if matches!(next_event, Event::End(TagEnd::CodeBlock)) { break; }
            }
            rsx! {
                pre { class: "bg-neutral-900 p-4 rounded-lg overflow-x-auto my-4",
                    code { class: "text-sm font-mono text-neutral-300", "{code}" }
                }
            }
        }
        _ => rsx! { span { {render_events(parser)} } },
    }
}
```

### 3. Integration into the Chat Widget
Update the message display logic in `src/domains/chat/widgets/message.rs`:
```rust
rsx! {
    div { class: "message-content",
        Markdown { content: message.text.clone() }
    }
}
```

# Trade-offs
| Metric | Rating | Notes |
| :--- | :--- | :--- |
| Complexity | Medium | Requires understanding of `pulldown-cmark` event stream and Dioxus recursion. |
| Time | Short | Fast to implement for standard Markdown; custom syntax takes longer. |
| Reversibility | High | The component can be swapped for a different renderer with zero impact on the data layer. |
| Risk | Low | No raw HTML injection risks. Type-safe rendering. |
| Scalability | High | Easily extended to support syntax highlighting (via `syntect`) or custom shortcodes. |
| Maintainability | High | Styles are co-located with elements using standard Tailwind classes. |

# Consequences
- **Positive**: Complete control over styling; every Markdown element is a first-class Dioxus node.
- **Positive**: Zero XSS risk as it bypasses HTML parsing entirely.
- **Risks**: Deeply nested Markdown could theoretically hit recursion limits (unlikely in chat).
- **Second-Order**: Allows for "interactive" Markdown elements (e.g., a button inside a Markdown block) since they are just RSX.

# Verdict
Best for projects requiring **deep Tailwind integration and high security**, where the complexity of a recursive mapper is outweighed by the benefits of a "Dioxus-native" rendering pipeline.
