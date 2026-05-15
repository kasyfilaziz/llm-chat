use dioxus::prelude::*;
use crate::domains::chat::state::{Message, ConversationStore, Conversation};
use crate::domains::chat::widgets::message::MessageBubble;
use crate::domains::chat::widgets::sidebar::Sidebar;
use crate::domains::llm::client::{LlmClient, ProviderType};
use crate::domains::llm::models::{ChatMessage, ChatRequest};
use crate::domains::llm::providers::process_stream;
use futures_util::StreamExt;
use crate::components::{Button, Input};
use crate::domains::chat::repo::{load_messages, save_message, clear_messages, create_conversation, update_conversation_title, get_all_conversations};
use tracing::{info, error};
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

pub enum ChatAction {
    SendMessage(String),
    ClearChat,
}

#[component]
pub fn ChatScreen() -> Element {
    let mut messages = use_signal(Vec::<Message>::new);
    let mut input_text = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut show_confirm = use_signal(|| false);
    let mut store = use_context::<Signal<ConversationStore>>();
    let conn = use_context::<Arc<Mutex<Connection>>>();

    // Load initial conversations
    let init_conn = conn.clone();
    use_effect(move || {
        let conn = init_conn.clone();
        spawn(async move {
            if let Ok(convos) = get_all_conversations(conn).await {
                let mut store_mut = store.write();
                store_mut.list = convos;
                if store_mut.active_id.is_none() && !store_mut.list.is_empty() {
                    store_mut.active_id = Some(store_mut.list[0].id.clone());
                }
            }
        });
    });

    // Load messages when active_id changes
    let active_id = store.read().active_id.clone();
    let load_conn = conn.clone();
    use_effect(move || {
        if let Some(id) = active_id.clone() {
            let conn = load_conn.clone();
            spawn(async move {
                // Pagination: hardcoded limit for now, infinite scroll later
                if let Ok(history) = load_messages(conn, id, 50, 0).await {
                    messages.set(history);
                }
            });
        } else {
            messages.set(Vec::new());
        }
    });

    let coroutine_conn = conn.clone();
    let chat_service = use_coroutine(move |mut rx: UnboundedReceiver<ChatAction>| {
        let conn = coroutine_conn.clone();
        async move {
            while let Some(action) = rx.next().await {
                // Instantly grab latest settings
                let settings = crate::domains::settings::state::SETTINGS.read().clone();
                let provider = settings.provider_preferences.default_llm.clone();
                let (url, key) = if provider == "ollama" {
                    (format!("{}/api/chat", settings.provider_preferences.ollama_endpoint), "".to_string())
                } else {
                    ("https://api.openai.com/v1/chat/completions".to_string(), settings.provider_preferences.openai_api_key.clone())
                };
                let model = if provider == "ollama" { "llama3".to_string() } else { "gpt-4-turbo".to_string() };

                let llm_client = LlmClient::new(
                    ProviderType::from(provider.as_str()),
                    url,
                    key,
                    model,
                );

                match action {
                    ChatAction::ClearChat => {
                        let active_id = store.read().active_id.clone();
                        if let Some(id) = active_id {
                            info!("Clearing all messages for conversation...");
                            if let Err(e) = clear_messages(conn.clone(), id).await {
                                error!("Failed to clear messages: {}", e);
                                error_msg.set(Some(format!("Database error: {}", e)));
                            } else {
                                messages.set(Vec::new());
                            }
                        }
                    }
                    ChatAction::SendMessage(text) => {
                        error_msg.set(None);
                        info!("Sending message: {}", text);

                        let mut active_id = store.read().active_id.clone();
                        let is_new_conversation = active_id.is_none();

                        if is_new_conversation {
                            let new_conv = Conversation::new("New Chat");
                            active_id = Some(new_conv.id.clone());
                            let _ = create_conversation(conn.clone(), new_conv.clone()).await;
                            
                            let mut store_mut = store.write();
                            store_mut.list.insert(0, new_conv);
                            store_mut.active_id = active_id.clone();
                        }

                        let cid = active_id.unwrap();
                        let user_msg = Message::new(&cid, "user", &text);
                        messages.push(user_msg.clone());
                        
                        let _ = save_message(conn.clone(), user_msg).await;

                        let request = ChatRequest {
                            model: llm_client.model.clone(),
                            messages: messages.read().iter().map(|m| ChatMessage {
                                role: m.role.clone(),
                                content: m.content.clone(),
                            }).collect(),
                            stream: true,
                        };

                        let mut assistant_msg = Message::new(&cid, "assistant", "");
                        let assistant_id = assistant_msg.id.clone();
                        messages.push(assistant_msg.clone());

                        match process_stream(&llm_client, request).await {
                            Ok(mut stream) => {
                                while let Some(chunk) = stream.next().await {
                                    match chunk {
                                        Ok(token) => {
                                            let mut current_messages = messages.write();
                                            if let Some(msg) = current_messages.iter_mut().find(|m| m.id == assistant_id) {
                                                msg.content.push_str(&token);
                                                assistant_msg.content.push_str(&token);
                                            }
                                        }
                                        Err(e) => {
                                            error!(e);
                                            error_msg.set(Some(e));
                                            break;
                                        }
                                    }
                                }
                                info!("Stream completed successfully");
                                let _ = save_message(conn.clone(), assistant_msg.clone()).await;

                                // LLM Auto-title logic for new conversations
                                if is_new_conversation {
                                    let title_req = ChatRequest {
                                        model: llm_client.model.clone(),
                                        messages: vec![
                                            ChatMessage { role: "system".into(), content: "Generate a very short 3-5 word title for this conversation. Return ONLY the title.".into() },
                                            ChatMessage { role: "user".into(), content: text.clone() },
                                            ChatMessage { role: "assistant".into(), content: assistant_msg.content.clone() }
                                        ],
                                        stream: false,
                                    };
                                    
                                    // Normally we would use non-streaming for title, but we can reuse stream for now and collect
                                    if let Ok(mut title_stream) = process_stream(&llm_client, title_req).await {
                                        let mut new_title = String::new();
                                        while let Some(Ok(token)) = title_stream.next().await {
                                            new_title.push_str(&token);
                                        }
                                        new_title = new_title.replace("\"", "").trim().to_string();
                                        if !new_title.is_empty() {
                                            let _ = update_conversation_title(conn.clone(), cid.clone(), new_title.clone()).await;
                                            if let Some(c) = store.write().list.iter_mut().find(|c| c.id == cid) {
                                                c.title = new_title;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Stream error: {}", e);
                                error_msg.set(Some(format!("Stream error: {}", e)));
                            }
                        }
                    }
                }
            }
        }
    });

    rsx! {
        div { class: "flex h-screen bg-slate-50 text-slate-900 font-sans",
            Sidebar {}
            div { class: "flex-1 flex flex-col min-w-0",
                // Header
                header { class: "px-6 py-4 bg-white border-b border-slate-200 flex justify-between items-center shadow-sm z-10",
                    div { class: "flex items-center gap-2",
                        div { class: "w-8 h-8 bg-primary rounded-lg flex items-center justify-center text-white font-bold", "L" }
                        h1 { class: "text-xl font-bold tracking-tight text-slate-800", "Lumina" }
                    }
                    span { class: "text-xs font-medium px-2 py-1 bg-slate-100 text-slate-500 rounded-full", "Phase 2" }
                }

                // Chat History
                div { class: "flex-1 overflow-y-auto px-4 py-8 max-w-4xl mx-auto w-full space-y-2",
                    for err in crate::domains::mcp::state::MCP_STATE.read().errors.iter() {
                        div { class: "flex justify-center my-4",
                            div { class: "bg-red-50 border border-red-200 text-red-600 px-6 py-3 rounded-2xl text-sm shadow-sm flex items-center gap-2",
                                span { class: "font-bold", "⚠️" }
                                "{err}"
                            }
                        }
                    }

                    for msg in messages.read().iter() {
                        MessageBubble { message: msg.clone() }
                    }
                    
                    if let Some(err) = error_msg.read().as_ref() {
                        div { class: "flex justify-center my-4",
                            div { class: "bg-red-50 border border-red-200 text-red-600 px-6 py-3 rounded-2xl text-sm shadow-sm flex items-center gap-2",
                                span { class: "font-bold", "⚠️" }
                                "{err}"
                            }
                        }
                    }
                }

                // Input Area
                footer { class: "p-6 bg-white border-t border-slate-200 shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.05)]",
                    div { class: "max-w-4xl mx-auto flex flex-col gap-3",
                        // Confirmation Dialog
                        if show_confirm() {
                            div { class: "bg-red-50 p-4 rounded-xl border border-red-200 flex justify-between items-center mb-2 animate-in fade-in slide-in-from-bottom-2",
                                span { class: "text-red-700 text-sm font-medium", "Permanently delete all messages?" }
                                div { class: "flex gap-2",
                                    Button { 
                                        class: "bg-red-600 hover:bg-red-700 !py-1 !px-4 text-xs",
                                        onclick: move |_| {
                                            chat_service.send(ChatAction::ClearChat);
                                            show_confirm.set(false);
                                        },
                                        "Yes, Clear"
                                    }
                                    Button { 
                                        class: "bg-slate-200 !text-slate-700 hover:bg-slate-300 !py-1 !px-4 text-xs",
                                        onclick: move |_| show_confirm.set(false),
                                        "Cancel"
                                    }
                                }
                            }
                        }

                        div { class: "flex gap-3",
                            Button {
                                class: "bg-slate-100 !text-slate-500 hover:bg-red-50 hover:!text-red-600 !px-3 shadow-none",
                                onclick: move |_| show_confirm.toggle(),
                                "🗑️"
                            }
                            Input {
                                value: input_text.read().clone(),
                                placeholder: "Ask Lumina anything...",
                                oninput: move |evt: Event<FormData>| input_text.set(evt.value().clone()),
                                onkeydown: move |evt: Event<KeyboardData>| {
                                    if evt.key() == Key::Enter && !input_text.read().is_empty() {
                                        chat_service.send(ChatAction::SendMessage(input_text.read().clone()));
                                        input_text.set(String::new());
                                    }
                                }
                            }
                            Button {
                                onclick: move |_| {
                                    if !input_text.read().is_empty() {
                                        chat_service.send(ChatAction::SendMessage(input_text.read().clone()));
                                        input_text.set(String::new());
                                    }
                                },
                                "Send"
                            }
                        }
                    }
                    p { class: "text-[10px] text-center text-slate-400 mt-3", "Lumina can make mistakes. Verify important information." }
                }
            }
        }
    }
}
