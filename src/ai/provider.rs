use serde_json::Value;

use crate::{
    ai::{Message, protocol::ProtocolKind},
    config::AppConfig,
};

#[derive(Clone, Copy)]
pub struct ProviderSpec {
    name: &'static str,
    default_base_url: &'static str,
    chat_path: &'static str,
    models_path: &'static str,
    protocol: ProtocolKind,
}

impl ProviderSpec {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn headers(&self, api_key: &Option<String>) -> anyhow::Result<reqwest::header::HeaderMap> {
        self.protocol.headers(api_key)
    }

    pub fn endpoint(&self, config: &AppConfig) -> String {
        config.api.endpoint.clone().unwrap_or_else(|| join_url(self.base_url(config), self.chat_path))
    }

    pub fn models_url(&self, config: &AppConfig) -> String {
        join_url(self.base_url(config), self.models_path)
    }

    pub fn generate_request_body(&self, config: &AppConfig, system_msg: &Message<'_>, user_msg: &Message<'_>) -> Value {
        self.protocol.build_chat_request(config, system_msg, user_msg)
    }

    pub fn parse_response(&self, response: Value) -> Option<String> {
        self.protocol.parse_chat_response(response)
    }

    pub fn parse_models_response(&self, response: Value) -> Vec<String> {
        self.protocol.parse_models_response(response)
    }

    fn base_url<'a>(&self, config: &'a AppConfig) -> &'a str {
        config.api.base_url.as_deref().unwrap_or(self.default_base_url)
    }
}

const OPENAI_SPEC: ProviderSpec = ProviderSpec {
    name: "openai",
    default_base_url: "https://api.openai.com",
    chat_path: "/v1/chat/completions",
    models_path: "/v1/models",
    protocol: ProtocolKind::OpenAiCompatible,
};

const OPENROUTER_SPEC: ProviderSpec = ProviderSpec {
    name: "openrouter",
    default_base_url: "https://openrouter.ai",
    chat_path: "/api/v1/chat/completions",
    models_path: "/api/v1/models",
    protocol: ProtocolKind::OpenAiCompatible,
};

const DEEPSEEK_SPEC: ProviderSpec = ProviderSpec {
    name: "deepseek",
    default_base_url: "https://api.deepseek.com",
    chat_path: "/chat/completions",
    models_path: "/models",
    protocol: ProtocolKind::OpenAiCompatible,
};

const ZHIPU_SPEC: ProviderSpec = ProviderSpec {
    name: "zhipu",
    default_base_url: "https://open.bigmodel.cn",
    chat_path: "/api/paas/v4/chat/completions",
    models_path: "/api/paas/v4/models",
    protocol: ProtocolKind::OpenAiCompatible,
};

const OLLAMA_SPEC: ProviderSpec = ProviderSpec {
    name: "ollama",
    default_base_url: "http://localhost:11434",
    chat_path: "/api/chat",
    models_path: "/api/tags",
    protocol: ProtocolKind::Ollama,
};

const LMSTUDIO_SPEC: ProviderSpec = ProviderSpec {
    name: "lmstudio",
    default_base_url: "http://localhost:1234",
    chat_path: "/api/v1/chat",
    models_path: "/api/v1/models",
    protocol: ProtocolKind::LMStudio,
};

static PROVIDER_SPECS: &[ProviderSpec] =
    &[OPENAI_SPEC, OPENROUTER_SPEC, DEEPSEEK_SPEC, ZHIPU_SPEC, OLLAMA_SPEC, LMSTUDIO_SPEC];

pub fn find_provider(name: &str) -> Option<ProviderSpec> {
    let normalized = name.trim().to_ascii_lowercase();
    PROVIDER_SPECS.iter().find(|spec| spec.name == normalized).copied()
}

pub fn provider_names() -> Vec<&'static str> {
    PROVIDER_SPECS.iter().map(ProviderSpec::name).collect()
}

fn join_url(base_url: &str, path: &str) -> String {
    format!("{}/{}", base_url.trim_end_matches('/'), path.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::{find_provider, provider_names};
    use crate::config::{ApiConfig, AppConfig, CommitConfig, PromptConfig};

    fn test_config() -> AppConfig {
        AppConfig {
            api: ApiConfig {
                provider: "openai".to_string(),
                base_url: None,
                endpoint: None,
                model: "gpt-4o-mini".to_string(),
                api_key: Some("test-key".to_string()),
                max_tokens: Some(1000),
                temperature: Some(0.7),
                context_limit: 200000,
            },
            commit: CommitConfig {
                auto_confirm: false,
                dry_run_by_default: false,
                gpg_sign: None,
                ignore_lock_files: true,
                custom_ignore_patterns: vec![],
            },
            prompts: PromptConfig { system_prompt: "system".to_string(), user_prompt_template: "{diff}".to_string() },
        }
    }

    #[test]
    fn should_get_provider_by_name_case_insensitively() {
        let provider = find_provider(" OpenAI ").expect("provider should exist");

        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn should_return_all_provider_names() {
        assert_eq!(provider_names(), vec!["openai", "openrouter", "deepseek", "zhipu", "ollama"]);
    }

    #[test]
    fn should_build_models_url_from_base_url() {
        let config = test_config();
        let provider = find_provider("openai").expect("provider should exist");

        assert_eq!(provider.models_url(&config), "https://api.openai.com/v1/models");
    }

    #[test]
    fn should_use_custom_base_url_for_models() {
        let mut config = test_config();
        config.api.base_url = Some("https://example.com/custom/".to_string());
        let provider = find_provider("openai").expect("provider should exist");

        assert_eq!(provider.models_url(&config), "https://example.com/custom/v1/models");
    }
}
