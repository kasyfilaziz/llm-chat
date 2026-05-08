use dioxus::prelude::*;

fn main() {
    // Linux WebView compositing workaround (debug builds only)
    #[cfg(all(target_os = "linux", debug_assertions))]
    unsafe {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center min-h-screen bg-gray-950",
            h1 {
                class: "text-4xl font-bold text-white",
                "Lumina"
            }
            p {
                class: "text-gray-400 mt-2",
                "v0.1.0 — Project initialized"
            }
        }
    }
}
