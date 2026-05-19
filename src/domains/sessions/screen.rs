use dioxus::prelude::*;
use crate::domains::sessions::state::SessionStore;
use crate::domains::sessions::widgets::directory_tree::DirectoryTree;
use crate::domains::sessions::widgets::session_grid::SessionGrid;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

#[allow(non_snake_case)]
pub fn SessionsPage() -> Element {
    let store = use_signal(SessionStore::new);
    let conn = use_context::<Arc<Mutex<Connection>>>();

    {
        let conn = conn.clone();
        let mut store = store.clone();
        use_effect(move || {
            let conn = conn.clone();
            spawn(async move {
                if let Ok(folders) = crate::domains::sessions::repo::get_all_folders(conn.clone()).await {
                    store.write().folders = folders;
                }
                if let Ok(sessions) = crate::domains::sessions::repo::get_sessions_by_folder(conn.clone(), None).await {
                    store.write().set_sessions(sessions);
                }
                store.write().loading = false;
            });
        });
    }

    let active_id = store.read().active_folder_id.clone();
    let sessions = store.read().sessions.clone();
    let loading = store.read().loading;
    let folders = store.read().folders.clone();

    rsx! {
        div { class: "flex-1 flex flex-col min-w-0 h-full bg-surface",
            div { class: "flex flex-1 overflow-hidden",
                div { class: "w-72 border-r border-outline-variant/30 flex flex-col bg-surface-container-low shrink-0",
                    DirectoryTree {
                        folders: folders,
                        active_folder_id: active_id.clone(),
                        store: store.clone(),
                    }
                }
                div { class: "flex-1 overflow-y-auto p-6",
                    if loading {
                        div { class: "flex items-center justify-center h-full",
                            div { class: "animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full" }
                        }
                    } else if sessions.is_empty() {
                        div { class: "flex flex-col items-center justify-center h-full gap-4",
                            div { class: "text-4xl", "📭" }
                            h3 { class: "text-lg font-medium text-on-surface", "No conversations yet" }
                            p { class: "text-sm text-on-surface-variant", "Start a chat to see your sessions here" }
                            Link {
                                to: crate::app::Route::ChatScreen {},
                                class: "bg-primary text-on-primary px-6 py-2 rounded-xl font-medium hover:bg-primary-container hover:text-on-primary-container transition-colors",
                                "Start a Chat"
                            }
                        }
                    } else {
                        SessionGrid {
                            sessions: sessions,
                        }
                    }
                }
            }
        }
    }
}
