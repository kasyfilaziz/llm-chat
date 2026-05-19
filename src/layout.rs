use dioxus::prelude::*;
use crate::components::sidebar::Sidebar;

#[component]
pub fn RootLayout() -> Element {
    rsx! {
        div { class: "flex h-screen bg-surface text-on-surface font-body",
            Sidebar {}
            div { class: "flex-1 flex flex-col min-w-0",
                Outlet::<crate::app::Route> {}
            }
        }
    }
}
