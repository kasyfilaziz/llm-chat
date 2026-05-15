use crate::domains::settings::state::SETTINGS;
use crate::domains::mcp::state::MCP_STATE;
use tokio::process::Command;
use tracing::{info, error};
use dioxus::prelude::*;

pub async fn start_mcp_servers() {
    let settings = SETTINGS.read().clone();
    
    // Clear previous errors/servers
    MCP_STATE.write().errors.clear();
    MCP_STATE.write().active_servers.clear();

    for server in settings.mcp_servers {
        info!("Starting MCP server: {}", server.name);
        
        let mut cmd = Command::new(&server.path);
        cmd.args(&server.args);
        
        // Ensure stdio is piped if we want to hook it into rmcp
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        
        match cmd.spawn() {
            Ok(mut _child) => {
                info!("Successfully spawned {}", server.name);
                MCP_STATE.write().active_servers.push(server.name.clone());
                
                // Keep the child process alive in the background
                // and kill it if the app drops (handled by dropping Child or explicit kill)
                tokio::spawn(async move {
                    let _ = _child.wait().await;
                    error!("MCP server {} exited", server.name);
                });
            }
            Err(e) => {
                error!("Failed to start MCP server {}: {}", server.name, e);
                MCP_STATE.write().errors.push(format!("Failed to start MCP server '{}': {}", server.name, e));
            }
        }
    }
}