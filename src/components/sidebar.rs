use dioxus::prelude::*;
use crate::app::Route;

#[derive(Clone, PartialEq)]
struct NavItem {
    route: Option<Route>,
    label: &'static str,
    icon: &'static str,
    disabled: bool,
    tooltip: &'static str,
}

#[allow(deprecated)]
#[component]
pub fn Sidebar() -> Element {
    let current_route = use_route::<Route>();
    let collapsed = use_signal(|| false);

    let nav_items = vec![
        NavItem { route: Some(Route::ChatScreen {}), label: "Chat", icon: "💬", disabled: false, tooltip: "" },
        NavItem { route: Some(Route::SessionsPage {}), label: "Sessions", icon: "📁", disabled: false, tooltip: "" },
        NavItem { route: None, label: "Context Vault", icon: "🔐", disabled: true, tooltip: "Coming soon" },
        NavItem { route: None, label: "Hooks", icon: "🔌", disabled: true, tooltip: "Coming soon" },
        NavItem { route: Some(Route::SettingsScreen {}), label: "Settings", icon: "⚙️", disabled: false, tooltip: "" },
        NavItem { route: None, label: "Help", icon: "❓", disabled: true, tooltip: "Coming soon" },
        NavItem { route: None, label: "Logout", icon: "🚪", disabled: true, tooltip: "Coming soon" },
    ];

    let is_active = |item: &NavItem| -> bool {
        match &item.route {
            Some(route) => route == &current_route,
            None => false,
        }
    };

    let sidebar_width = if *collapsed.read() { "w-16" } else { "w-64" };

    let toggle_label = if *collapsed.read() { "▶" } else { "◀" };

    rsx! {
        nav { class: "{sidebar_width} bg-surface-container-low border-r border-outline-variant/30 flex flex-col h-full shrink-0 transition-all duration-200",
            if collapsed() {
                div { class: "flex flex-col items-center py-4",
                    div { class: "w-8 h-8 bg-primary rounded-lg flex items-center justify-center text-xs text-on-primary font-bold",
                        "T"
                    }
                }
            } else {
                div { class: "p-4 border-b border-outline-variant/20",
                    div { class: "flex items-center gap-2",
                        div { class: "w-8 h-8 bg-primary rounded-lg flex items-center justify-center text-xs text-on-primary font-bold",
                            "T"
                        }
                        h1 { class: "text-lg font-headline font-bold text-on-surface tracking-tight", "Terra" }
                    }
                }
            }
            div { class: "flex-1 overflow-y-auto p-3 space-y-1",
                for item in &nav_items {
                    SidebarItem { item: item.clone(), is_active: is_active(item), collapsed: *collapsed.read() }
                }
            }
            div { class: "p-3 border-t border-outline-variant/20",
                button {
                    class: "w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm text-on-surface-variant hover:bg-surface-container transition-colors",
                    title: if collapsed() { "Expand" } else { "Collapse" },
                    onclick: move |_| *collapsed.write_silent() = !*collapsed.read(),
                    "{toggle_label}"
                }
            }
        }
    }
}

#[component]
fn SidebarItem(item: NavItem, is_active: bool, collapsed: bool) -> Element {
    let mut class = "flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-colors".to_string();

    if collapsed {
        class.push_str(" justify-center");
    }

    if is_active {
        class.push_str(" bg-primary/10 text-primary font-medium");
    } else if item.disabled {
        class.push_str(" text-secondary/40 cursor-not-allowed");
    } else {
        class.push_str(" text-on-surface-variant hover:bg-surface-container hover:text-on-surface cursor-pointer");
    }

    rsx! {
        if item.disabled {
            div {
                class: "{class}",
                title: "{item.tooltip}",
                span { class: "text-base", "{item.icon}" }
                if !collapsed {
                    span { "{item.label}" }
                }
            }
        } else if let Some(route) = &item.route {
            Link {
                to: route.clone(),
                class: "{class}",
                span { class: "text-base", "{item.icon}" }
                if !collapsed {
                    span { "{item.label}" }
                }
            }
        }
    }
}
