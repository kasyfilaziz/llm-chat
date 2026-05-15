use dioxus::prelude::*;
use crate::domains::chat::state::ConversationStore;
use crate::domains::chat::repo::update_conversation_title;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

#[component]
pub fn Sidebar() -> Element {
    let mut store = use_context::<Signal<ConversationStore>>();
    let conn = use_context::<Arc<Mutex<Connection>>>();
    let mut editing_id = use_signal(|| None::<String>);
    let mut edit_title = use_signal(|| String::new());

    let btn_conn = conn.clone();

    rsx! {
        div { class: "w-64 bg-slate-100 border-r border-slate-200 flex flex-col h-full",
            div { class: "p-4 border-b border-slate-200",
                button {
                    class: "w-full bg-primary text-white font-medium py-2 px-4 rounded-lg shadow-sm hover:bg-primary/90 transition-colors",
                    onclick: move |_| {
                        // Create a new conversation and set active
                        let new_conv = crate::domains::chat::state::Conversation::new("New Chat");
                        let new_id = new_conv.id.clone();
                        let conn_clone = btn_conn.clone();
                        let mut store_mut = store.write();
                        store_mut.list.insert(0, new_conv.clone());
                        store_mut.active_id = Some(new_id);
                        
                        spawn(async move {
                            let _ = crate::domains::chat::repo::create_conversation(conn_clone, new_conv).await;
                        });
                    },
                    "+ New Chat"
                }
            }
            div { class: "flex-1 overflow-y-auto p-2 space-y-1",
                {store.read().list.iter().map(|conv| {
                    let id = conv.id.clone();
                    let id_for_click = id.clone();
                    let id_for_keydown = id.clone();
                    let id_for_edit_btn = id.clone();
                    let title = conv.title.clone();
                    let title_for_edit_btn = title.clone();
                    let is_active = store.read().active_id.as_ref() == Some(&id);
                    let active_class = if is_active { "bg-slate-200 text-slate-900" } else { "text-slate-600 hover:bg-slate-200/50" };
                    let is_editing = editing_id.read().as_ref() == Some(&id);
                    let conn_for_input = conn.clone();

                    rsx! {
                        div {
                            key: "{id}",
                            class: "group flex items-center justify-between p-2 rounded-md cursor-pointer transition-colors {active_class}",
                            onclick: move |_| {
                                if !is_editing {
                                    store.write().active_id = Some(id_for_click.clone());
                                }
                            },
                            if is_editing {
                                input {
                                    class: "flex-1 bg-white border border-slate-300 rounded px-2 py-1 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary",
                                    value: edit_title.read().clone(),
                                    autofocus: "true",
                                    oninput: move |evt| edit_title.set(evt.value()),
                                    onkeydown: move |evt| {
                                        if evt.key() == Key::Enter {
                                            let new_title = edit_title.read().clone();
                                            let id_clone = id_for_keydown.clone();
                                            let conn_clone = conn_for_input.clone();
                                            
                                            // Update local state
                                            if let Some(c) = store.write().list.iter_mut().find(|c| c.id == id_clone) {
                                                c.title = new_title.clone();
                                                c.updated_at = chrono::Utc::now().timestamp();
                                            }
                                            
                                            // Update DB
                                            spawn(async move {
                                                let _ = update_conversation_title(conn_clone, id_clone, new_title).await;
                                            });
                                            
                                            editing_id.set(None);
                                        } else if evt.key() == Key::Escape {
                                            editing_id.set(None);
                                        }
                                    }
                                }
                            } else {
                                span { class: "truncate text-sm font-medium", "{title}" }
                                button {
                                    class: "opacity-0 group-hover:opacity-100 text-slate-400 hover:text-slate-700 transition-opacity",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        edit_title.set(title_for_edit_btn.clone());
                                        editing_id.set(Some(id_for_edit_btn.clone()));
                                    },
                                    "✎"
                                }
                            }
                        }
                    }
                })}
            }
        }
    }
}
