use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::dirs::get_config_file_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub commit: CommitConfig,
    // pub hooks: HookConfig,
    pub prompts: PromptConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub provider: String,
    pub base_url: Option<String>,
    pub endpoint: Option<String>,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f64>,
    pub context_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitConfig {
    pub auto_confirm: bool,
    pub dry_run_by_default: bool,
    pub gpg_sign: Option<bool>,
    pub ignore_lock_files: bool,
    pub custom_ignore_patterns: Vec<String>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct HookConfig {
//     pub enabled: bool,
//     pub hook_types: Vec<String>,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    pub system_prompt: String,
    pub user_prompt_template: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig::load().unwrap_or_else(|e| {
            debug!("failed to load configuration, {e}");
            eprintln!("Failed to load configuration, using default settings.");
            std::process::exit(1);
        })
    }
}

impl AppConfig {
    pub fn default_config() -> Result<Self> {
        let default_config = include_str!("../../config.sample.toml");
        Ok(toml::from_str(default_config)?)
    }

    pub fn init() -> Result<()> {
        let config = Self::default_config()?;
        config.save()?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let config_path = get_config_file_path()?;
        debug!("Loading configuration from: {}", config_path.display());
        if config_path.exists() {
            let config_content = fs::read_to_string(&config_path)?;
            let config: AppConfig = toml::from_str(&config_content)?;
            Ok(config)
        } else {
            let default_config = include_str!("../../config.sample.toml");
            fs::write(&config_path, default_config)?;
            let config = Self::default_config()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_file_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config_content = toml::to_string_pretty(self)?;
        fs::write(&config_path, config_content)?;
        Ok(())
    }

    pub fn generate_user_prompt(&self, diff: &str) -> String {
        self.prompts.user_prompt_template.replace("{diff}", diff)
    }
}
