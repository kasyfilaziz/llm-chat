use dioxus::prelude::*;

#[component]
pub fn InputArea(
    value: String,
    oninput: EventHandler<String>,
    onsubmit: EventHandler<String>,
) -> Element {
    let mut sending = use_signal(|| false);

    if value.is_empty() && *sending.read() {
        sending.set(false);
    }

    let value_disabled = value.clone();
    let value_keydown = value.clone();

    rsx! {
        footer { class: "p-6 bg-surface-container border-t border-outline-variant/30",
            div { class: "max-w-4xl mx-auto flex flex-col gap-3",
                div { class: "flex gap-3",
                    div { class: "flex-1 relative",
                        input {
                            class: "w-full bg-surface-container-high border border-outline-variant/30 rounded-xl px-4 py-3 text-sm text-on-surface placeholder-secondary/60 outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-all",
                            placeholder: "Ask Terra anything...",
                            value: "{value}",
                            disabled: *sending.read(),
                            oninput: move |e| oninput.call(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter && !value_keydown.is_empty() && !*sending.read() {
                                    sending.set(true);
                                    onsubmit.call(value_keydown.clone());
                                }
                            },
                        }
                    }
                    button {
                        class: "bg-primary text-on-primary px-5 py-3 rounded-xl font-medium text-sm hover:bg-primary-container hover:text-on-primary-container transition-colors disabled:opacity-50 disabled:cursor-not-allowed shrink-0",
                        disabled: value_disabled.is_empty() || *sending.read(),
                        onclick: move |_| {
                            if !value_disabled.is_empty() && !*sending.read() {
                                sending.set(true);
                                onsubmit.call(value_disabled.clone());
                            }
                        },
                        "Send"
                    }
                }
            }
            p { class: "text-[10px] text-center text-secondary/70 mt-3", "Terra may produce inaccurate information. Please verify important details." }
        }
    }
}
