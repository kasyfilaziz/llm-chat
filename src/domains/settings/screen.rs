use dioxus::prelude::*;
use crate::components::{Button, Input};
use crate::domains::settings::state::SETTINGS;
use crate::domains::settings::repo::{save_settings, McpServerConfig};

#[component]
pub fn SettingsScreen() -> Element {
    let mut show_success = use_signal(|| false);

    rsx! {
        div { class: "p-8 max-w-2xl mx-auto w-full",
            h2 { class: "text-2xl font-bold mb-6 text-on-surface", "Settings" }

            div { class: "space-y-6",
                div { class: "bg-surface-container p-6 rounded-xl border border-outline-variant/20",
                    h3 { class: "text-lg font-medium mb-4 pb-2 text-on-surface border-b border-outline-variant/20", "Providers" }

                    div { class: "space-y-4",
                        div {
                            label { class: "block text-sm font-medium text-on-surface-variant mb-1", "Default LLM" }
                            select {
                                class: "w-full bg-surface-container-high border border-outline-variant/30 rounded-xl px-3 py-2 text-sm text-on-surface outline-none focus:border-primary/50",
                                value: SETTINGS.read().provider_preferences.default_llm.clone(),
                                onchange: move |evt| {
                                    SETTINGS.write().provider_preferences.default_llm = evt.value();
                                },
                                option { value: "openai", "OpenAI" }
                                option { value: "ollama", "Ollama" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-on-surface-variant mb-1", "OpenAI API Key" }
                            Input {
                                value: SETTINGS.read().provider_preferences.openai_api_key.clone(),
                                placeholder: "sk-...",
                                oninput: move |evt: Event<FormData>| {
                                    SETTINGS.write().provider_preferences.openai_api_key = evt.value();
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-on-surface-variant mb-1", "Ollama Endpoint" }
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

                div { class: "bg-surface-container p-6 rounded-xl border border-outline-variant/20",
                    h3 { class: "text-lg font-medium mb-4 pb-2 text-on-surface border-b border-outline-variant/20", "MCP Servers" }

                    div { class: "space-y-4",
                        {SETTINGS.read().mcp_servers.iter().enumerate().map(|(idx, server)| {
                            let name = server.name.clone();
                            let path = server.path.clone();
                            let args = server.args.join(" ");
                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: "flex items-center gap-2 p-3 bg-surface-container-high border border-outline-variant/20 rounded-xl",
                                    div { class: "flex-1",
                                        p { class: "font-medium text-sm text-on-surface", "{name}" }
                                        p { class: "text-xs text-secondary/60", "{path} {args}" }
                                    }
                                    button {
                                        class: "text-red-500 hover:text-red-400 p-2 transition-colors",
                                        onclick: move |_| {
                                            SETTINGS.write().mcp_servers.remove(idx);
                                        },
                                        "🗑️"
                                    }
                                }
                            }
                        })}
                        button {
                            class: "text-sm text-primary font-medium hover:text-primary-container transition-colors",
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

                div { class: "flex justify-end gap-3 pt-4 items-center",
                    if show_success() {
                        span { class: "text-tertiary text-sm flex items-center", "✓ Settings saved!" }
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
