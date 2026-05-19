use dioxus::prelude::*;
use crate::domains::sessions::state::{Folder, SessionStore};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

#[derive(Clone, PartialEq)]
struct TreeNode {
    folder: Folder,
    depth: usize,
    has_children: bool,
}

#[component]
#[allow(deprecated)]
pub fn DirectoryTree(
    folders: Vec<Folder>,
    active_folder_id: Option<String>,
    store: Signal<SessionStore>,
) -> Element {
    let conn = use_context::<Arc<Mutex<Connection>>>();
    let expanded = use_signal(HashMap::<String, bool>::new);
    let renaming_id = use_signal(|| Option::<String>::None);
    let rename_value = use_signal(|| String::new());
    let mut creating = use_signal(|| false);
    let mut new_name = use_signal(|| String::new());

    {
        let conn = conn.clone();
        use_effect(move || {
            let conn = conn.clone();
            spawn(async move {
                let _ = crate::domains::sessions::repo::get_folder_session_counts(conn.clone()).await;
            });
        });
    }

    let children_map: HashMap<Option<String>, Vec<Folder>> = {
        let mut map = HashMap::new();
        for f in &folders {
            map.entry(f.parent_id.clone()).or_insert_with(Vec::new).push(f.clone());
        }
        map
    };

    let root_folders = children_map.get(&None).cloned().unwrap_or_default();

    let tree_nodes: Vec<TreeNode> = {
        let mut nodes = Vec::new();
        for f in &root_folders {
            let has_children = children_map.contains_key(&Some(f.id.clone()));
            nodes.push(TreeNode { folder: f.clone(), depth: 0, has_children });
            if expanded.read().contains_key(&f.id) && *expanded.read().get(&f.id).unwrap_or(&false) {
                append_children(&f.id, 1, &children_map, &expanded, &mut nodes);
            }
        }
        nodes
    };

    let all_class = if active_folder_id.is_none() {
        "bg-primary/10 text-primary font-medium"
    } else {
        "text-on-surface-variant hover:bg-surface-container hover:text-on-surface"
    };

    let conn2 = conn.clone();
    let store2 = store.clone();
    let parent_for_new = active_folder_id.clone();

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "p-3 border-b border-outline-variant/20 flex items-center justify-between",
                h2 { class: "text-xs font-semibold text-on-surface-variant uppercase tracking-wider", "Folders" }
                button {
                    class: "w-6 h-6 flex items-center justify-center rounded hover:bg-surface-container text-on-surface-variant hover:text-primary transition-colors text-sm",
                    title: if parent_for_new.is_some() { "New subfolder" } else { "New folder" },
                    onclick: move |_| {
                        creating.set(true);
                        new_name.set(String::new());
                    },
                    "+"
                }
            }
            div { class: "flex-1 overflow-y-auto p-2 space-y-0.5",
                button {
                    class: "w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors cursor-pointer {all_class}",
                    onclick: move |_| {
                        let conn = conn2.clone();
                        let mut store = store2.clone();
                        spawn(async move {
                            if let Ok(sessions) = crate::domains::sessions::repo::get_sessions_by_folder(conn.clone(), None).await {
                                store.write().select_folder(None);
                                store.write().set_sessions(sessions);
                            }
                        });
                    },
                    span { class: "text-base", "📁" }
                    span { "All Conversations" }
                }
                for node in &tree_nodes {
                    TreeItem {
                        key: "{node.folder.id}",
                        node: node.clone(),
                        is_active: active_folder_id.as_deref() == Some(&node.folder.id),
                        store: store.clone(),
                        expanded: expanded.clone(),
                        renaming_id: renaming_id.clone(),
                        rename_value: rename_value.clone(),
                    }
                }
            }
            if creating() {
                div { class: "p-3 border-t border-outline-variant/20",
                    input {
                        class: "w-full px-2 py-1.5 text-sm bg-surface-container border border-outline-variant/30 rounded-lg text-on-surface placeholder-secondary/60 focus:outline-none focus:border-primary/50",
                        placeholder: "Folder name...",
                        value: "{new_name}",
                        oninput: move |e| new_name.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter && !new_name.read().is_empty() {
                                let name = new_name.clone();
                                let conn = conn.clone();
                                let mut store = store.clone();
                                let parent = active_folder_id.clone();
                                spawn(async move {
                                    let folder = Folder {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        name: name.read().clone(),
                                        parent_id: parent,
                                        created_at: chrono::Utc::now().timestamp(),
                                        sort_order: 0,
                                    };
                                    if crate::domains::sessions::repo::create_folder(conn.clone(), &folder).await.is_ok() {
                                        if let Ok(folders) = crate::domains::sessions::repo::get_all_folders(conn.clone()).await {
                                            store.write().folders = folders;
                                        }
                                    }
                                });
                                creating.set(false);
                                new_name.set(String::new());
                            } else if e.key() == Key::Escape {
                                creating.set(false);
                                new_name.set(String::new());
                            }
                        },
                    }
                }
            }
        }
    }
}

fn append_children(
    parent_id: &str,
    depth: usize,
    children_map: &HashMap<Option<String>, Vec<Folder>>,
    expanded: &Signal<HashMap<String, bool>>,
    nodes: &mut Vec<TreeNode>,
) {
    if let Some(children) = children_map.get(&Some(parent_id.to_string())) {
        for f in children {
            let has_children = children_map.contains_key(&Some(f.id.clone()));
            nodes.push(TreeNode { folder: f.clone(), depth, has_children });
            if expanded.read().contains_key(&f.id) && *expanded.read().get(&f.id).unwrap_or(&false) {
                append_children(&f.id, depth + 1, children_map, expanded, nodes);
            }
        }
    }
}

#[component]
fn TreeItem(
    node: TreeNode,
    is_active: bool,
    store: Signal<SessionStore>,
    expanded: Signal<HashMap<String, bool>>,
    renaming_id: Signal<Option<String>>,
    rename_value: Signal<String>,
) -> Element {
    let conn = use_context::<Arc<Mutex<Connection>>>();
    let mut showing_delete_confirm = use_signal(|| false);
    let depth_style = format!("padding-left: {}px", 8 + node.depth * 16);
    let active_bg = if is_active { "bg-primary/10 text-primary" } else { "text-on-surface-variant hover:bg-surface-container hover:text-on-surface" };
    let fid = node.folder.id.clone();
    let fid2 = node.folder.id.clone();
    let fid3 = node.folder.id.clone();
    let fid4 = node.folder.id.clone();
    let fid5 = node.folder.id.clone();
    let fid6 = node.folder.id.clone();
    let fname = node.folder.name.clone();
    let fname2 = node.folder.name.clone();
    let c1 = conn.clone();
    let c2 = conn.clone();
    let c3 = conn.clone();
    let _c4 = conn.clone();

    rsx! {
        div {
            class: "group flex items-center gap-1 px-3 py-2 rounded-lg cursor-pointer text-sm transition-colors {active_bg}",
            style: "{depth_style}",
            onclick: move |_| {
                let conn = c1.clone();
                let id = fid.clone();
                let mut store = store.clone();
                spawn(async move {
                    if let Ok(sessions) = crate::domains::sessions::repo::get_sessions_by_folder(conn.clone(), Some(id.clone())).await {
                        store.write().select_folder(Some(id));
                        store.write().set_sessions(sessions);
                    }
                });
            },
            if node.has_children {
                div {
                    class: "w-4 h-4 flex items-center justify-center text-xs text-secondary/60 transition-transform",
                    onclick: move |e| {
                        e.stop_propagation();
                        let current = expanded.read().get(&fid2).copied().unwrap_or(true);
                        expanded.write().insert(fid2.clone(), !current);
                    },
                    "▸"
                }
            } else {
                div { class: "w-4" }
            }
            span { class: "text-xs", "📁" }
            if renaming_id.read().as_deref() == Some(&fid3) {
                input {
                    class: "flex-1 px-1 py-0.5 text-sm bg-surface-container border border-primary/50 rounded text-on-surface outline-none",
                    value: "{rename_value}",
                    oninput: move |e| rename_value.set(e.value()),
                    onkeydown: move |e| {
                        let id = fid6.clone();
                        let conn = c2.clone();
                        let mut store = store.clone();
                        if e.key() == Key::Enter && !rename_value.read().is_empty() {
                            let new_name = rename_value.read().clone();
                            spawn(async move {
                                let _ = crate::domains::sessions::repo::rename_folder(conn.clone(), id, new_name).await;
                                if let Ok(folders) = crate::domains::sessions::repo::get_all_folders(conn.clone()).await {
                                    store.write().folders = folders;
                                }
                            });
                            renaming_id.set(None);
                        } else if e.key() == Key::Escape {
                            renaming_id.set(None);
                        }
                    },
                }
            } else {
                span { class: "flex-1 truncate", "{fname}" }
            }
            div { class: "hidden group-hover:flex items-center gap-0.5 ml-auto",
                button {
                    class: "w-5 h-5 flex items-center justify-center rounded text-xs text-secondary/60 hover:text-primary hover:bg-surface-container transition-colors",
                    title: "Rename",
                    onclick: move |e| {
                        e.stop_propagation();
                        renaming_id.set(Some(fid4.clone()));
                        rename_value.set(fname2.clone());
                    },
                    "✏️"
                }
                button {
                    class: "w-5 h-5 flex items-center justify-center rounded text-xs text-secondary/60 hover:text-red-500 hover:bg-surface-container transition-colors",
                    title: "Delete",
                    onclick: move |e| {
                        e.stop_propagation();
                        showing_delete_confirm.set(true);
                    },
                    "🗑️"
                }
            }
        }
        if showing_delete_confirm() {
            div {
                class: "fixed inset-0 bg-black/40 flex items-center justify-center z-50",
                onclick: move |_| showing_delete_confirm.set(false),
                div {
                    class: "bg-surface-container rounded-xl border border-outline-variant/30 p-5 shadow-xl max-w-xs",
                    onclick: move |e| e.stop_propagation(),
                    h3 { class: "text-sm font-medium text-on-surface mb-2", "Delete folder?" }
                    p { class: "text-xs text-on-surface-variant mb-4",
                        "Sessions in this folder won't be deleted. They'll be moved to 'All Conversations'."
                    }
                    div { class: "flex justify-end gap-2",
                        button {
                            class: "px-3 py-1.5 text-xs rounded-lg text-on-surface-variant hover:bg-surface-container-high transition-colors",
                            onclick: move |_| showing_delete_confirm.set(false),
                            "Cancel"
                        }
                        button {
                            class: "px-3 py-1.5 text-xs rounded-lg bg-red-600 text-white hover:bg-red-500 transition-colors",
                            onclick: move |_| {
                                let id = fid5.clone();
                                let conn = c3.clone();
                                let mut store = store.clone();
                                spawn(async move {
                                    let _ = crate::domains::sessions::repo::delete_folder(conn.clone(), id).await;
                                    if let Ok(folders) = crate::domains::sessions::repo::get_all_folders(conn.clone()).await {
                                        store.write().folders = folders;
                                    }
                                });
                                showing_delete_confirm.set(false);
                            },
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
