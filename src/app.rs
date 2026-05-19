use dioxus::prelude::*;
use crate::db::Db;
use crate::domains::chat::repo::init_db;
use crate::domains::chat::screen::ChatScreen;
use crate::domains::sessions::screen::SessionsPage;
use crate::domains::settings::screen::SettingsScreen;
use crate::domains::mcp::state::McpEvent;
use crate::domains::mcp::client::start_mcp_servers;
use futures_util::StreamExt;

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(crate::layout::RootLayout)]
        #[route("/")]
        SessionsPage {},
        #[route("/chat")]
        ChatScreen {},
        #[route("/settings")]
        SettingsScreen {},
}

#[allow(non_snake_case)]
pub fn App() -> Element {
    let conn = use_hook(|| {
        let db = Db::new("lumina.db").expect("Failed to open database");
        let conn = db.get_conn();
        {
            let lock = conn.lock().expect("Failed to lock DB");
            init_db(&lock).expect("Failed to init DB schema");
            let _ = crate::domains::sessions::repo::init_db(&lock);
        }
        conn
    });
    
    provide_context(conn);
    use_context_provider(|| Signal::new(crate::domains::chat::state::ConversationStore::default()));

    // Load settings from YAML on startup
    use_effect(move || {
        spawn(async move {
            let settings = crate::domains::settings::repo::load_settings().await;
            *crate::domains::settings::state::SETTINGS.write() = settings;
        });
    });

    // MCP Orchestrator Coroutine
    let mcp_orchestrator = use_coroutine(move |mut rx: UnboundedReceiver<McpEvent>| async move {
        while let Some(event) = rx.next().await {
            match event {
                McpEvent::StartServers => {
                    start_mcp_servers().await;
                }
                McpEvent::StopServers => {
                    // Logic to stop servers gracefully
                }
            }
        }
    });

    use_effect(move || {
        mcp_orchestrator.send(McpEvent::StartServers);
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        Router::<Route> {}
    }
}
