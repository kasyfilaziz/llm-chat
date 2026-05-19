#![allow(deprecated)]
use dioxus::prelude::*;
use crate::domains::chat::state::Message;
use crate::components::Markdown;

#[component]
pub fn MessageBubble(
    message: Message,
    onregenerate: Option<EventHandler<String>>,
) -> Element {
    let is_user = message.role == "user";

    let (align, container_class, avatar) = if is_user {
        ("justify-end",
         "bg-primary text-on-primary rounded-2xl rounded-br-sm",
         None)
    } else {
        ("justify-start",
         "bg-surface-container-high text-on-surface rounded-2xl rounded-bl-sm border border-outline-variant/20",
         Some("AI"))
    };

    let msg_content = message.content.clone();
    let mid = message.id.clone();
    let copied = use_signal(|| false);

    rsx! {
        div { class: "flex w-full {align} mb-4 px-4 group/message",
            div { class: "flex gap-3 max-w-[85%] {align}",
                if let Some(label) = avatar {
                    div { class: "w-8 h-8 rounded-full bg-tertiary-container text-tertiary flex items-center justify-center text-xs font-bold shrink-0 mt-1",
                        "{label}"
                    }
                }
                div { class: "flex flex-col",
                    div { class: "px-4 py-3 shadow-sm {container_class}",
                        if is_user {
                            p { class: "text-sm leading-relaxed whitespace-pre-wrap", "{msg_content}" }
                        } else {
                            div { class: "text-sm leading-relaxed [&_pre]:bg-surface-container [&_code]:bg-surface-container [&_pre]:p-3 [&_pre]:rounded-xl",
                                Markdown { content: msg_content.clone() }
                            }
                        }
                    }
                    if !is_user {
                        div { class: "flex items-center gap-1 mt-1 opacity-0 group-hover/message:opacity-100 transition-opacity px-2",
                            button {
                                class: "flex items-center gap-1 px-2 py-1 rounded-md text-[10px] text-secondary/60 hover:text-primary hover:bg-surface-container transition-colors",
                                title: "Copy message",
                                onclick: move |_| {
                                    let text = msg_content.clone();
                                    let _ = std::process::Command::new("xclip")
                                        .args(["-selection", "clipboard"])
                                        .stdin(std::process::Stdio::piped())
                                        .spawn()
                                        .and_then(|mut child| {
                                            use std::io::Write;
                                            child.stdin.take().unwrap().write_all(text.as_bytes())
                                        });
                                    *copied.write_silent() = true;
                                    spawn(async move {
                                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                        *copied.write_silent() = false;
                                    });
                                },
                                if *copied.read() {
                                    span { class: "text-tertiary font-medium", "✓ Copied" }
                                } else {
                                    "📋 Copy"
                                }
                            }
                            {
                                let handler = onregenerate.clone();
                                match handler {
                                    Some(h) => rsx! {
                                        button {
                                            class: "flex items-center gap-1 px-2 py-1 rounded-md text-[10px] text-secondary/60 hover:text-primary hover:bg-surface-container transition-colors",
                                            title: "Regenerate response",
                                            onclick: move |_| {
                                                h.call(mid.clone());
                                            },
                                            "🔄 Regenerate"
                                        }
                                    },
                                    None => rsx! {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
