use dioxus::prelude::*;

#[derive(Default, Clone, PartialEq)]
pub struct McpState {
    pub active_servers: Vec<String>,
    pub errors: Vec<String>,
}

pub static MCP_STATE: GlobalSignal<McpState> = Signal::global(|| McpState::default());

pub enum McpEvent {
    StartServers,
    #[allow(dead_code)]
    StopServers,
}