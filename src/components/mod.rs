use dioxus::prelude::*;

mod markdown;
pub use markdown::*;

pub mod sidebar;

#[component]
pub fn Button(
    children: Element,
    onclick: EventHandler<MouseEvent>,
    class: Option<String>,
) -> Element {
    // Modern Tailwind v4 style
    let base_class = "bg-primary hover:bg-primary/90 px-6 py-2 rounded-xl transition-all shadow-sm text-on-primary font-semibold active:scale-95";
    let final_class = format!("{} {}", base_class, class.unwrap_or_default());
    
    rsx! {
        button {
            class: "{final_class}",
            onclick: move |evt| onclick.call(evt),
            {children}
        }
    }
}

#[component]
pub fn Input(
    value: String,
    oninput: EventHandler<FormEvent>,
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    placeholder: Option<String>,
) -> Element {
    rsx! {
        input {
            class: "flex-1 bg-surface-container border border-outline-variant/30 rounded-xl px-4 py-2 text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all",
            value: "{value}",
            placeholder: "{placeholder.clone().unwrap_or_default()}",
            oninput: move |evt| oninput.call(evt),
            onkeydown: move |evt| {
                if let Some(handler) = onkeydown {
                    handler.call(evt);
                }
            }
        }
    }
}
