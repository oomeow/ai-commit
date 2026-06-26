use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{ai::ProtocolKind, dirs::get_config_file_path, git::DiffContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Name of the provider entry to use by default. References `ApiConfig::name`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,
    /// List of configured providers. The active one is chosen by `default_provider`
    /// (or a runtime override).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<ApiConfig>,
    /// Legacy single-provider section. Kept for backward compatibility; migrated
    /// into `providers` on load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api: Option<ApiConfig>,
    pub commit: CommitConfig,
    // pub hooks: HookConfig,
    pub prompts: PromptConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// User-defined identifier for this provider entry (e.g. "work", "local").
    /// Referenced by `AppConfig::default_provider` and the `--provider` flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub provider: Option<String>,
    pub endpoint: Option<String>,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f64>,
    // pub context_limit: usize,
    #[serde(alias = "protocol_type")]
    pub protocol: Option<ProtocolKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitConfig {
    pub auto_confirm: bool,
    pub dry_run_by_default: bool,
    pub gpg_sign: Option<bool>,
    pub ignore_lock_files: bool,
    pub custom_ignore_patterns: Vec<String>,
    pub cache_expiry_days: u64,
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
        let default_config = include_str!("../../config.sample.toml");
        let mut config: AppConfig = toml::from_str(default_config).unwrap_or_else(|e| {
            debug!("failed to load configuration, {e}");
            std::process::exit(1);
        });
        // remove carriage returns from prompts
        config.prompts.system_prompt = normalize_newlines(&config.prompts.system_prompt);
        config.prompts.user_prompt_template = normalize_newlines(&config.prompts.user_prompt_template);
        config.normalize();
        config
    }
}

impl ApiConfig {
    /// A stable identifier for this entry, falling back to the built-in provider
    /// name and finally to "default" when nothing is set.
    pub fn entry_name(&self) -> String {
        self.name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .or(self.provider.as_deref())
            .map(str::to_owned)
            .unwrap_or_else(|| "default".to_string())
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = get_config_file_path()?;
        debug!("Loading configuration from: {}", config_path.display());
        if !config_path.exists() {
            eprintln!("❌ Configuration file not found, please run `ai-commit config init` to initialize one");
            std::process::exit(0);
        }
        let config_content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;
        config.normalize();
        Ok(config)
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;
        config.normalize();
        Ok(config)
    }

    pub fn save(&self, config_path: &PathBuf) -> Result<()> {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let config_content = toml::to_string_pretty(self)?;
        fs::write(config_path, config_content)?;
        Ok(())
    }

    /// Migrate a legacy single `[api]` section into the `providers` list and
    /// ensure every entry has a usable name.
    fn normalize(&mut self) {
        if let Some(mut api) = self.api.take()
            && self.providers.is_empty()
        {
            if api.name.is_none() {
                api.name = Some(api.entry_name());
            }
            self.providers.push(api);
        }

        for provider in &mut self.providers {
            if provider.name.is_none() {
                provider.name = Some(provider.entry_name());
            }
        }

        if self.default_provider.is_none() {
            self.default_provider = self.providers.first().and_then(|provider| provider.name.clone());
        }
    }

    /// Resolve the provider to use, preferring an explicit override, then the
    /// configured default, then the first entry.
    pub fn active_provider(&self, override_name: Option<&str>) -> Result<&ApiConfig> {
        if self.providers.is_empty() {
            anyhow::bail!("No providers configured. Run `ai-commit config init` to add one.");
        }

        if let Some(name) = override_name.map(str::trim).filter(|name| !name.is_empty()) {
            return self.find_provider(name).ok_or_else(|| {
                anyhow::anyhow!("Provider '{name}' not found. Available: {}", self.provider_names().join(", "))
            });
        }

        if let Some(name) = self.default_provider.as_deref()
            && let Some(provider) = self.find_provider(name)
        {
            return Ok(provider);
        }

        Ok(&self.providers[0])
    }

    pub fn find_provider(&self, name: &str) -> Option<&ApiConfig> {
        let target = name.trim();
        self.providers.iter().find(|provider| provider.entry_name() == target)
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.providers.iter().map(ApiConfig::entry_name).collect()
    }

    pub fn generate_user_prompt(&self, diff: &DiffContext) -> String {
        self.prompts
            .user_prompt_template
            .replace("{diff_stats}", &diff.stats)
            .replace("{diff_code_block}", &diff.diff_code_block)
    }
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    const COMMIT_AND_PROMPTS: &str = r#"
[commit]
auto_confirm = false
dry_run_by_default = false
ignore_lock_files = true
custom_ignore_patterns = []
cache_expiry_days = 7

[prompts]
system_prompt = "sys"
user_prompt_template = "user"
"#;

    fn parse(extra: &str) -> AppConfig {
        let content = format!("{extra}{COMMIT_AND_PROMPTS}");
        let mut config: AppConfig = toml::from_str(&content).expect("config should parse");
        config.normalize();
        config
    }

    #[test]
    fn should_migrate_legacy_api_section_into_providers() {
        let config = parse(
            r#"
[api]
provider = "openrouter"
model = "glm-4.5"
api_key = "key"
"#,
        );

        assert!(config.api.is_none());
        assert_eq!(config.providers.len(), 1);
        assert_eq!(config.providers[0].entry_name(), "openrouter");
        assert_eq!(config.default_provider.as_deref(), Some("openrouter"));
        // legacy api is dropped from serialization
        let serialized = toml::to_string_pretty(&config).expect("serialize");
        assert!(!serialized.contains("[api]"));
        assert!(serialized.contains("[[providers]]"));
    }

    #[test]
    fn should_select_default_provider() {
        let config = parse(
            r#"
default_provider = "local"

[[providers]]
name = "work"
provider = "openrouter"
model = "glm-4.5"

[[providers]]
name = "local"
provider = "ollama"
model = "qwen2.5"
"#,
        );

        assert_eq!(config.active_provider(None).expect("active provider").entry_name(), "local");
    }

    #[test]
    fn should_prefer_override_over_default() {
        let config = parse(
            r#"
default_provider = "local"

[[providers]]
name = "work"
provider = "openrouter"
model = "glm-4.5"

[[providers]]
name = "local"
provider = "ollama"
model = "qwen2.5"
"#,
        );

        assert_eq!(config.active_provider(Some("work")).expect("active provider").entry_name(), "work");
    }

    #[test]
    fn should_error_on_unknown_provider_name() {
        let config = parse(
            r#"
[[providers]]
name = "work"
provider = "openrouter"
model = "glm-4.5"
"#,
        );

        assert!(config.active_provider(Some("missing")).is_err());
    }

    #[test]
    fn should_fall_back_to_first_provider_when_default_missing() {
        let config = parse(
            r#"
default_provider = "gone"

[[providers]]
name = "work"
provider = "openrouter"
model = "glm-4.5"
"#,
        );

        assert_eq!(config.active_provider(None).expect("active provider").entry_name(), "work");
    }
}
