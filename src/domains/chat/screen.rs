use dioxus::prelude::*;
use crate::domains::chat::state::{Message, ConversationStore, Conversation};
use crate::domains::chat::widgets::message::MessageBubble;
use crate::domains::chat::widgets::input_area::InputArea;
use crate::domains::chat::widgets::prompt_suggestions::PromptSuggestions;
use crate::domains::llm::client::{LlmClient, ProviderType};
use crate::domains::llm::models::{ChatMessage, ChatRequest};
use crate::domains::llm::providers::process_stream;
use futures_util::StreamExt;
use crate::domains::chat::repo::{load_messages, save_message, create_conversation, update_conversation_title, get_all_conversations};
use tracing::{info, error};
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

pub enum ChatAction {
    SendMessage(String),
    Regenerate(String),
}

#[component]
pub fn ChatScreen() -> Element {
    let mut messages = use_signal(Vec::<Message>::new);
    let mut input_text = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
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
                let provider_type = ProviderType::from(settings.provider_preferences.default_llm.as_str());

                let (url, key) = match provider_type {
                    ProviderType::Ollama => {
                        let base_url = settings.provider_preferences.ollama_endpoint.trim_end_matches('/');
                        (format!("{}/api/chat", base_url), "".to_string())
                    }
                    ProviderType::OpenAI => {
                        ("https://api.openai.com/v1/chat/completions".to_string(), settings.provider_preferences.openai_api_key.clone())
                    }
                };

                let model = if matches!(provider_type, ProviderType::Ollama) {
                    "gemma4:31b-cloud".to_string()
                } else {
                    "gpt-4-turbo".to_string()
                };

                let llm_client = LlmClient::new(
                    provider_type,
                    url,
                    key,
                    model,
                );

                match action {
                    ChatAction::Regenerate(msg_id) => {
                        error_msg.set(None);
                        info!("Regenerating response for message: {}", msg_id);

                        {
                            let mut msgs = messages.write();
                            msgs.retain(|m| m.id != msg_id);
                        }

                        let cid = store.read().active_id.clone().unwrap_or_default();
                        let mut assistant_msg = Message::new(&cid, "assistant", "");
                        let assistant_id = assistant_msg.id.clone();
                        messages.push(assistant_msg.clone());

                        let request = ChatRequest {
                            model: llm_client.model.clone(),
                            messages: messages.read().iter().map(|m| ChatMessage {
                                role: m.role.clone(),
                                content: m.content.clone(),
                            }).collect(),
                            stream: true,
                        };

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
                                info!("Regeneration completed");
                                let _ = save_message(conn.clone(), assistant_msg.clone()).await;
                            }
                            Err(e) => {
                                error!("Regeneration stream error: {}", e);
                                error_msg.set(Some(format!("Stream error: {}", e)));
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

    let input_chat = chat_service.clone();
    let regen_chat = chat_service.clone();
    let msg_count = messages.read().len();

    rsx! {
        div { class: "flex-1 flex flex-col min-w-0 bg-surface text-on-surface",
            header { class: "px-6 py-4 bg-surface-container border-b border-outline-variant/30 flex justify-between items-center z-10",
                div { class: "flex items-center gap-3",
                    div { class: "w-8 h-8 bg-primary rounded-lg flex items-center justify-center text-on-primary text-sm font-bold", "T" }
                    div { class: "flex flex-col",
                        h1 { class: "text-sm font-headline font-bold tracking-tight text-on-surface", "Terra" }
                        span { class: "text-[10px] text-secondary/60", "AI Assistant" }
                    }
                }
            }
            div { class: "flex-1 overflow-y-auto py-4",
                if msg_count == 0 {
                    div { class: "flex flex-col items-center justify-center h-full gap-6",
                        div { class: "w-16 h-16 bg-primary-container/30 rounded-2xl flex items-center justify-center text-2xl", "🤖" }
                        h2 { class: "text-lg font-medium text-on-surface", "How can I help you?" }
                        p { class: "text-sm text-on-surface-variant max-w-md text-center",
                            "Ask me anything — I can help with code, writing, research, and more."
                        }
                        PromptSuggestions {
                            onselect: move |text: String| {
                                input_chat.send(ChatAction::SendMessage(text));
                            },
                        }
                    }
                } else {
                    div { class: "max-w-4xl mx-auto w-full space-y-1",
                        for err in crate::domains::mcp::state::MCP_STATE.read().errors.iter() {
                            div { class: "flex justify-center my-2",
                                div { class: "bg-error-container border border-error-container text-error px-6 py-3 rounded-2xl text-sm shadow-sm flex items-center gap-2",
                                    span { class: "font-bold", "⚠️" }
                                    "{err}"
                                }
                            }
                        }
                        {messages.read().iter().map(|msg| {
                            let mid = msg.id.clone();
                            let regen = regen_chat.clone();
                            rsx! {
                                MessageBubble {
                                    key: "{mid}",
                                    message: msg.clone(),
                                    onregenerate: Some(EventHandler::new(move |id: String| {
                                        regen.send(ChatAction::Regenerate(id));
                                    })),
                                }
                            }
                        })}
                        if let Some(err) = error_msg.read().as_ref() {
                            div { class: "flex justify-center my-2",
                                div { class: "bg-error-container border border-error-container text-error px-6 py-3 rounded-2xl text-sm shadow-sm flex items-center gap-2",
                                    span { class: "font-bold", "⚠️" }
                                    "{err}"
                                }
                            }
                        }
                    }
                }
            }
            InputArea {
                value: input_text.read().clone(),
                oninput: move |val| input_text.set(val),
                onsubmit: move |text| {
                    chat_service.send(ChatAction::SendMessage(text));
                    input_text.set(String::new());
                },
            }
        }
    }
}
