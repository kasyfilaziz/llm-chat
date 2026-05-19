use dioxus::prelude::*;
use crate::domains::sessions::state::{SessionSummary, SessionStore};
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

#[allow(deprecated)]
#[component]
pub fn SessionCard(
    session: SessionSummary,
) -> Element {
    let conn = use_context::<Arc<Mutex<Connection>>>();
    let store = use_context::<Signal<SessionStore>>();
    let show_menu = use_signal(|| false);
    let mut menu_pos = use_signal(|| (0i32, 0i32));
    let col_span = if session.is_pinned { "md:col-span-2" } else { "" };
    let tag_count = session.tags.len();

    let folders = store.read().folders.clone();
    let current_folder = store.read().active_folder_id.clone();
    let sid_none = session.id.clone();
    let sid_delete = session.id.clone();
    let sid_folder = session.id.clone();
    let c1 = conn.clone();
    let c2 = conn.clone();
    let c3 = conn.clone();
    let st1 = store.clone();
    let st2 = store.clone();
    let st3 = store.clone();

    rsx! {
        div {
            class: "relative {col_span}",
            oncontextmenu: move |e| {
                e.prevent_default();
                let coords = e.client_coordinates();
                menu_pos.set((coords.x as i32, coords.y as i32));
                *show_menu.write_silent() = true;
            },
            Link {
                to: crate::app::Route::ChatScreen {},
                class: "group block bg-surface-container rounded-xl border border-outline-variant/20 p-4 hover:border-primary/30 transition-all hover:shadow-[0_4px_20px_rgba(46,50,48,0.08)]",
                div { class: "absolute top-0 left-0 right-0 h-0.5 bg-gradient-to-r from-primary/10 via-primary to-primary/10 opacity-0 group-hover:opacity-100 transition-opacity rounded-t-xl" }
                div { class: "flex items-start justify-between mb-2",
                    div { class: "flex items-center gap-2",
                        div { class: "w-8 h-8 rounded-lg bg-primary/10 text-primary flex items-center justify-center text-sm", "💬" }
                        PinButton {
                            is_pinned: session.is_pinned,
                            session_id: session.id.clone(),
                        }
                    }
                    span { class: "text-[10px] text-secondary/60 px-2 py-0.5 bg-surface-container-high rounded-full",
                        "{session.turn_count} turns"
                    }
                }
                h3 { class: "text-sm font-medium text-on-surface truncate", "{session.title}" }
                p { class: "text-xs text-on-surface-variant line-clamp-2 mt-1", "{session.description}" }
                if tag_count > 0 {
                    div { class: "flex flex-wrap gap-1 mt-2",
                        {session.tags.iter().map(|tag| {
                            rsx! {
                                span {
                                    key: "{tag}",
                                    class: "text-[10px] px-1.5 py-0.5 bg-tertiary-container/30 text-tertiary rounded-full",
                                    "{tag}"
                                }
                            }
                        })}
                    }
                }
            }
            if *show_menu.read() {
                div {
                    class: "fixed inset-0 z-50",
                    onclick: move |_| *show_menu.write_silent() = false,
                    oncontextmenu: move |e| { e.prevent_default(); *show_menu.write_silent() = false; },
                    div {
                        class: "absolute bg-surface-container rounded-xl border border-outline-variant/30 shadow-xl py-1 min-w-[180px]",
                        style: "left: {menu_pos.read().0}px; top: {menu_pos.read().1}px",
                        onclick: move |e| e.stop_propagation(),
                        div { class: "px-3 py-1.5 text-[10px] text-secondary/60 uppercase tracking-wider", "Move to folder" }
                        button {
                            class: "w-full text-left px-3 py-2 text-xs text-on-surface hover:bg-surface-container-high transition-colors flex items-center gap-2",
                            onclick: move |_| {
                                let sid = sid_none.clone();
                                let conn = c1.clone();
                                let mut store = st1.clone();
                                *show_menu.write_silent() = false;
                                spawn(async move {
                                    let _ = crate::domains::sessions::repo::move_session_to_folder(conn.clone(), sid, None).await;
                                    let folder = store.read().active_folder_id.clone();
                                    if let Ok(sessions) = crate::domains::sessions::repo::get_sessions_by_folder(conn.clone(), folder).await {
                                        store.write().set_sessions(sessions);
                                    }
                                });
                            },
                            span { class: "text-xs", "📁" }
                            "No folder"
                        }
                        div { class: "border-t border-outline-variant/20 my-1" }
                        button {
                            class: "w-full text-left px-3 py-2 text-xs text-red-500 hover:bg-surface-container-high transition-colors flex items-center gap-2",
                            onclick: move |_| {
                                let sid = sid_delete.clone();
                                let conn = c3.clone();
                                let mut store = st3.clone();
                                *show_menu.write_silent() = false;
                                spawn(async move {
                                    let _ = crate::domains::sessions::repo::soft_delete_session(conn.clone(), sid).await;
                                    let folder = store.read().active_folder_id.clone();
                                    if let Ok(sessions) = crate::domains::sessions::repo::get_sessions_by_folder(conn.clone(), folder).await {
                                        store.write().set_sessions(sessions);
                                    }
                                });
                            },
                            span { class: "text-xs", "🗑️" }
                            "Delete session"
                        }
                        div { class: "border-t border-outline-variant/20 my-1" }
                        {folders.iter().map(|folder| {
                            let fid = folder.id.clone();
                            let fname = folder.name.clone();
                            let is_target = Some(&fid) == current_folder.as_ref();
                            let fc = if is_target { "bg-primary/10 text-primary" } else { "" };
                            let s_clone = sid_folder.clone();
                            let c_clone = c2.clone();
                            let st_clone = st2.clone();

                            rsx! {
                                button {
                                    key: "{fid}",
                                    class: "w-full text-left px-3 py-2 text-xs text-on-surface hover:bg-surface-container-high transition-colors flex items-center gap-2 {fc}",
                                    onclick: move |_| {
                                        let sid = s_clone.clone();
                                        let fid = fid.clone();
                                        let conn = c_clone.clone();
                                        let mut store = st_clone.clone();
                                        *show_menu.write_silent() = false;
                                        spawn(async move {
                                            let _ = crate::domains::sessions::repo::move_session_to_folder(conn.clone(), sid, Some(fid)).await;
                                            let folder = store.read().active_folder_id.clone();
                                            if let Ok(sessions) = crate::domains::sessions::repo::get_sessions_by_folder(conn.clone(), folder).await {
                                                store.write().set_sessions(sessions);
                                            }
                                        });
                                    },
                                    span { class: "text-xs", "📂" }
                                    "{fname}"
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}

#[allow(deprecated)]
#[component]
fn PinButton(
    is_pinned: bool,
    session_id: String,
) -> Element {
    let conn = use_context::<Arc<Mutex<Connection>>>();
    let pinned = use_signal(|| is_pinned);
    let label = if *pinned.read() { "📌" } else { "📍" };
    let title = if *pinned.read() { "Unpin" } else { "Pin" };

    rsx! {
        button {
            class: "text-xs opacity-0 group-hover:opacity-100 transition-opacity hover:scale-110",
            title: "{title}",
            onclick: move |e| {
                e.stop_propagation();
                let new_val = !*pinned.read();
                let sid = session_id.clone();
                let conn = conn.clone();
                *pinned.write_silent() = new_val;
                spawn(async move {
                    let _ = crate::domains::sessions::repo::toggle_pin_session(conn.clone(), sid, new_val).await;
                });
            },
            "{label}"
        }
    }
}
