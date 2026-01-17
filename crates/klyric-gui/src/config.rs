use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ui_font: Option<String>,
    #[serde(default = "default_show_chinese_only")]
    pub show_chinese_only: bool,
}

fn default_show_chinese_only() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ui_font: None,
            show_chinese_only: true,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if let Some(config_path) = Self::config_path() {
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(config_path) = Self::config_path() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)?;
            std::fs::write(config_path, json)?;
        }
        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("klyric").join("config.json"))
    }
}
