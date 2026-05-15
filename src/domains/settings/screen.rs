use dioxus::prelude::*;
use crate::components::{Button, Input};
use crate::domains::settings::state::SETTINGS;
use crate::domains::settings::repo::{save_settings, McpServerConfig};

#[component]
pub fn SettingsScreen() -> Element {
    let mut show_success = use_signal(|| false);

    rsx! {
        div { class: "p-8 max-w-2xl mx-auto w-full",
            h2 { class: "text-2xl font-bold mb-6", "Settings" }
            
            div { class: "space-y-6",
                div { class: "bg-white p-6 rounded-xl border border-slate-200 shadow-sm",
                    h3 { class: "text-lg font-medium mb-4 border-b pb-2", "Providers" }
                    
                    div { class: "space-y-4",
                        div {
                            label { class: "block text-sm font-medium text-slate-700 mb-1", "Default LLM" }
                            select {
                                class: "w-full border border-slate-300 rounded-md px-3 py-2 outline-none focus:border-primary",
                                value: SETTINGS.read().provider_preferences.default_llm.clone(),
                                onchange: move |evt| {
                                    SETTINGS.write().provider_preferences.default_llm = evt.value();
                                },
                                option { value: "openai", "OpenAI" }
                                option { value: "ollama", "Ollama" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-slate-700 mb-1", "OpenAI API Key" }
                            Input {
                                value: SETTINGS.read().provider_preferences.openai_api_key.clone(),
                                placeholder: "sk-...",
                                oninput: move |evt: Event<FormData>| {
                                    SETTINGS.write().provider_preferences.openai_api_key = evt.value();
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-slate-700 mb-1", "Ollama Endpoint" }
                            Input {
                                value: SETTINGS.read().provider_preferences.ollama_endpoint.clone(),
                                placeholder: "http://127.0.0.1:11434",
                                oninput: move |evt: Event<FormData>| {
                                    SETTINGS.write().provider_preferences.ollama_endpoint = evt.value();
                                }
                            }
                        }
                    }
                }
                
                div { class: "bg-white p-6 rounded-xl border border-slate-200 shadow-sm",
                    h3 { class: "text-lg font-medium mb-4 border-b pb-2", "MCP Servers" }
                    
                    div { class: "space-y-4",
                        {SETTINGS.read().mcp_servers.iter().enumerate().map(|(idx, server)| {
                            let name = server.name.clone();
                            let path = server.path.clone();
                            let args = server.args.join(" ");
                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: "flex items-center gap-2 p-3 bg-slate-50 border border-slate-200 rounded-md",
                                    div { class: "flex-1",
                                        p { class: "font-medium text-sm", "{name}" }
                                        p { class: "text-xs text-slate-500", "{path} {args}" }
                                    }
                                    button {
                                        class: "text-red-500 hover:text-red-700 p-2",
                                        onclick: move |_| {
                                            SETTINGS.write().mcp_servers.remove(idx);
                                        },
                                        "🗑️"
                                    }
                                }
                            }
                        })}
                        button {
                            class: "text-sm text-primary font-medium hover:underline",
                            onclick: move |_| {
                                SETTINGS.write().mcp_servers.push(McpServerConfig {
                                    name: "New Server".into(),
                                    path: "".into(),
                                    args: vec![],
                                });
                            },
                            "+ Add MCP Server"
                        }
                    }
                }
                
                div { class: "flex justify-end gap-3 pt-4",
                    if show_success() {
                        span { class: "text-green-600 text-sm flex items-center", "Settings saved!" }
                    }
                    Button {
                        onclick: move |_| {
                            let current = SETTINGS.read().clone();
                            spawn(async move {
                                if save_settings(current).await.is_ok() {
                                    show_success.set(true);
                                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                    show_success.set(false);
                                }
                            });
                        },
                        "Save Settings"
                    }
                }
            }
        }
    }
}