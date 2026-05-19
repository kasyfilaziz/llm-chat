use dioxus::prelude::*;

#[component]
pub fn TypingIndicator() -> Element {
    rsx! {
        div { class: "flex justify-start mb-4 px-4",
            div { class: "flex gap-3 items-center",
                div { class: "w-8 h-8 rounded-full bg-tertiary-container text-tertiary flex items-center justify-center text-xs font-bold shrink-0",
                    "AI"
                }
                div { class: "bg-surface-container-high border border-outline-variant/20 rounded-2xl rounded-bl-sm px-5 py-4",
                    div { class: "flex gap-1.5",
                        div { class: "w-2 h-2 bg-tertiary/60 rounded-full animate-bounce [animation-delay:0ms]" }
                        div { class: "w-2 h-2 bg-tertiary/60 rounded-full animate-bounce [animation-delay:150ms]" }
                        div { class: "w-2 h-2 bg-tertiary/60 rounded-full animate-bounce [animation-delay:300ms]" }
                    }
                }
            }
        }
    }
}
