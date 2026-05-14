use dioxus::prelude::*;
use crate::domains::chat::state::Message;
use crate::components::Markdown;
use tracing::info;

#[component]
pub fn MessageBubble(message: Message) -> Element {
    info!("Rendering message bubble: ID={}, Role={}", message.id, message.role);
    let is_user = message.role == "user";
    
    // Improved Gemini/Claude-like styling
    let (align, bg_color, text_color, rounded) = if is_user {
        ("justify-end", "bg-primary", "text-white", "rounded-l-2xl rounded-tr-2xl rounded-br-sm")
    } else {
        ("justify-start", "bg-slate-100", "text-slate-900", "rounded-r-2xl rounded-tl-2xl rounded-bl-sm")
    };

    rsx! {
        div { class: "flex w-full {align} mb-6",
            div { class: "max-w-[85%] px-5 py-3 shadow-sm {bg_color} {text_color} {rounded} border border-slate-200/50",
                if is_user {
                    p { class: "text-sm leading-relaxed whitespace-pre-wrap", "{message.content}" }
                } else {
                    div { class: "text-sm leading-relaxed",
                        Markdown { content: message.content }
                    }
                }
            }
        }
    }
}
