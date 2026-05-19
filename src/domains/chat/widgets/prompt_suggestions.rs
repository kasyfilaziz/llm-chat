use dioxus::prelude::*;

const SUGGESTIONS: &[&str] = &[
    "What's new in Rust 2026?",
    "Explain monads like I'm 10",
    "Write a Python script to batch rename files",
    "Compare AWS Lambda vs Cloudflare Workers",
    "How does async Rust work under the hood?",
];

#[component]
pub fn PromptSuggestions(
    onselect: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "flex flex-wrap gap-2 justify-center px-4 pb-4",
            for suggestion in SUGGESTIONS {
                button {
                    class: "px-3 py-1.5 text-xs bg-surface-container-high text-on-surface-variant rounded-full border border-outline-variant/20 hover:border-primary/30 hover:text-primary hover:bg-primary/5 transition-colors whitespace-nowrap",
                    onclick: move |_| onselect.call(suggestion.to_string()),
                    "{suggestion}"
                }
            }
        }
    }
}
