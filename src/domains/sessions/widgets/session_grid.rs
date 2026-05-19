use dioxus::prelude::*;
use crate::domains::sessions::state::SessionSummary;
use crate::domains::sessions::widgets::session_card::SessionCard;

#[component]
pub fn SessionGrid(
    sessions: Vec<SessionSummary>,
) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
            {sessions.iter().map(|session| {
                rsx! {
                    SessionCard {
                        key: "{session.id}",
                        session: session.clone(),
                    }
                }
            })}
        }
    }
}
