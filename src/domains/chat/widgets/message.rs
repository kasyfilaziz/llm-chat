use dioxus::prelude::*;
use crate::domains::chat::state::Message;

#[component]
pub fn MessageBubble(message: Message) -> Element {
    let is_user = message.role == "user";
    let bg_color = if is_user { "bg-blue-600" } else { "bg-slate-700" };
    let align = if is_user { "justify-end" } else { "justify-start" };

    rsx! {
        div { class: "flex w-full {align} mb-4",
            div { class: "max-w-[80%] px-4 py-2 rounded-2xl {bg_color} text-white shadow-lg",
                p { class: "text-sm", "{message.content}" }
            }
        }
    }
}
