use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderPreferences {
    pub default_llm: String,
    pub openai_api_key: String,
    pub ollama_endpoint: String,
}

impl Default for ProviderPreferences {
    fn default() -> Self {
        Self {
            default_llm: "openai".to_string(),
            openai_api_key: "".to_string(),
            ollama_endpoint: "http://127.0.0.1:11434".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServerConfig {
    pub name: String,
    pub path: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppSettings {
    pub provider_preferences: ProviderPreferences,
    pub mcp_servers: Vec<McpServerConfig>,
}

pub fn get_config_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("dev", "lumina", "app").expect("Failed to get project dirs");
    let config_dir = proj_dirs.config_dir();
    std::fs::create_dir_all(config_dir).unwrap_or_default();
    config_dir.join("settings.yaml")
}

pub async fn load_settings() -> AppSettings {
    let path = get_config_path();
    if !path.exists() {
        let default_settings = AppSettings::default();
        let _ = save_settings(default_settings.clone()).await;
        return default_settings;
    }
    match tokio::fs::read_to_string(&path).await {
        Ok(contents) => {
            tokio::task::spawn_blocking(move || {
                serde_yaml_ng::from_str(&contents).unwrap_or_else(|_| AppSettings::default())
            }).await.unwrap_or_else(|_| AppSettings::default())
        }
        Err(_) => AppSettings::default(),
    }
}

pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = get_config_path();
    let yaml_string = tokio::task::spawn_blocking(move || {
        serde_yaml_ng::to_string(&settings)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    tokio::fs::write(&path, yaml_string).await.map_err(|e| e.to_string())?;
    Ok(())
}