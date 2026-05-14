use dioxus::prelude::*;
use crate::db::Db;
use crate::domains::chat::repo::init_db;
use crate::domains::chat::screen::ChatScreen;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    ChatScreen {},
}

#[allow(non_snake_case)]
pub fn App() -> Element {
    let conn = use_hook(|| {
        let db = Db::new("lumina.db").expect("Failed to open database");
        let conn = db.get_conn();
        {
            let lock = conn.lock().expect("Failed to lock DB");
            init_db(&lock).expect("Failed to init DB schema");
        }
        conn
    });
    
    provide_context(conn);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        Router::<Route> {}
    }
}
