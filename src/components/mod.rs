use dioxus::prelude::*;

mod markdown;
pub use markdown::*;

#[component]
pub fn Button(
    children: Element,
    onclick: EventHandler<MouseEvent>,
    class: Option<String>,
) -> Element {
    // Modern Tailwind v4 style
    let base_class = "bg-primary hover:bg-blue-500 px-6 py-2 rounded-xl transition-all shadow-sm text-white font-semibold active:scale-95";
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
            class: "flex-1 bg-white border border-slate-200 rounded-xl px-4 py-2 text-slate-900 focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all shadow-inner",
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
