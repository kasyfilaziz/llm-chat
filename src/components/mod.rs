use dioxus::prelude::*;

#[component]
pub fn Button(
    children: Element,
    onclick: EventHandler<MouseEvent>,
    class: Option<String>,
) -> Element {
    let base_class = "bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded-lg transition-colors text-white font-medium";
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
            class: "flex-1 bg-slate-800 border border-slate-700 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-blue-500",
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
