use dioxus::prelude::*;
use crate::domains::chat::state::Message;
use crate::domains::chat::widgets::message::MessageBubble;
use crate::domains::llm::client::{LlmClient, ProviderType};
use crate::domains::llm::models::{ChatMessage, ChatRequest};
use crate::domains::llm::providers::process_stream;
use crate::utils::env;
use futures_util::StreamExt;
use crate::components::{Button, Input};
use crate::domains::chat::repo::{load_messages, save_message};
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

pub enum ChatAction {
    SendMessage(String),
}

#[component]
pub fn ChatScreen() -> Element {
    let mut messages = use_signal(Vec::<Message>::new);
    let mut input_text = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let conn = use_context::<Arc<Mutex<Connection>>>();

    // Load initial messages
    let load_conn = conn.clone();
    use_effect(move || {
        let conn = load_conn.clone();
        spawn(async move {
            if let Ok(history) = load_messages(conn).await {
                messages.set(history);
            }
        });
    });

    let coroutine_conn = conn.clone();
    let chat_service = use_coroutine(move |mut rx: UnboundedReceiver<ChatAction>| {
        let conn = coroutine_conn.clone();
        async move {
            let provider = env::get_var("API_PROVIDER_TYPE");
            let url = env::get_var("API_ENDPOINT_URL");
            let key = env::get_var("API_KEY");
            let model = env::get_var("API_MODEL_NAME");

            let llm_client = LlmClient::new(
                ProviderType::from(provider.as_str()),
                url,
                key,
                model,
            );

            while let Some(action) = rx.next().await {
                match action {
                    ChatAction::SendMessage(text) => {
                        error_msg.set(None);

                        let user_msg = Message::new("user", &text);
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

                        let assistant_id = Message::new("assistant", "").id;
                        let mut assistant_msg = Message {
                            id: assistant_id.clone(),
                            role: "assistant".to_string(),
                            content: "".to_string(),
                            created_at: chrono::Utc::now().timestamp(),
                        };
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
                                            error_msg.set(Some(e));
                                            break;
                                        }
                                    }
                                }
                                let _ = save_message(conn.clone(), assistant_msg).await;
                            }
                            Err(e) => {
                                error_msg.set(Some(format!("Stream error: {}", e)));
                            }
                        }
                    }
                }
            }
        }
    });

    rsx! {
        div { class: "flex flex-col h-screen bg-slate-900 text-white",
            // Header
            header { class: "p-4 border-b border-slate-800 flex justify-between items-center",
                h1 { class: "text-xl font-bold text-blue-400", "Lumina" }
                span { class: "text-xs text-slate-500", "Phase 1: Tracer Bullet" }
            }

            // Chat History
            div { class: "flex-1 overflow-y-auto p-4 space-y-4",
                for msg in messages.read().iter() {
                    MessageBubble { message: msg.clone() }
                }
                
                if let Some(err) = error_msg.read().as_ref() {
                    div { class: "flex justify-center",
                        div { class: "bg-red-900/50 border border-red-500 text-red-200 px-4 py-2 rounded-lg text-sm",
                            "{err}"
                        }
                    }
                }
            }

            // Input Area
            footer { class: "p-4 border-t border-slate-800",
                div { class: "flex gap-2",
                    Input {
                        value: input_text.read().clone(),
                        placeholder: "Type a message...",
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
        }
    }
}
