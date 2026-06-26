use anyhow::bail;

use crate::{ai::protocol::ProtocolKind, config::ApiConfig};

pub struct ProviderSpec {
    pub name: &'static str,
    pub display_name: &'static str,
    pub base_url: &'static str,
    // pub default_model: &'static str,
    pub protocol: ProtocolKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedApiConfig {
    pub endpoint: String,
    pub models_endpoint: Option<String>,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f64>,
    // pub context_limit: usize,
    pub protocol: ProtocolKind,
}

const BUILTIN_PROVIDERS: &[ProviderSpec] = &[
    ProviderSpec {
        name: "openai",
        display_name: "OpenAI",
        base_url: "https://api.openai.com/v1",
        // default_model: "gpt-5.5",
        protocol: ProtocolKind::OpenAiCompatible,
    },
    ProviderSpec {
        name: "openrouter",
        display_name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        // default_model: "z-ai/glm-4.5-air:free",
        protocol: ProtocolKind::OpenAiCompatible,
    },
    ProviderSpec {
        name: "deepseek",
        display_name: "DeepSeek",
        base_url: "https://api.deepseek.com/v1",
        // default_model: "deepseek-v4-flash",
        protocol: ProtocolKind::OpenAiCompatible,
    },
    ProviderSpec {
        name: "zhipu",
        display_name: "Zhipu AI",
        base_url: "https://open.bigmodel.cn/api/paas/v4",
        // default_model: "glm-4-flash",
        protocol: ProtocolKind::OpenAiCompatible,
    },
    ProviderSpec {
        name: "ollama",
        display_name: "Ollama (local)",
        base_url: "http://localhost:11434",
        // default_model: "local-model",
        protocol: ProtocolKind::Ollama,
    },
    ProviderSpec {
        name: "lmstudio",
        display_name: "LM Studio (local)",
        base_url: "http://localhost:1234",
        // default_model: "local-model",
        protocol: ProtocolKind::LMStudio,
    },
];

pub fn find_provider(name: &str) -> Option<&'static ProviderSpec> {
    let normalized = name.trim().to_ascii_lowercase();
    BUILTIN_PROVIDERS.iter().find(|provider| provider.name == normalized)
}

pub fn provider_specs() -> &'static [ProviderSpec] {
    BUILTIN_PROVIDERS
}

pub fn resolve_api_config(config: &ApiConfig) -> anyhow::Result<ResolvedApiConfig> {
    let provider = config.provider.as_deref().and_then(find_provider);
    let endpoint = config
        .endpoint
        .as_deref()
        .filter(|endpoint| !endpoint.trim().is_empty())
        .map(str::to_owned)
        .or_else(|| provider.map(|provider| join_url(provider.base_url, provider.protocol.chat_path())))
        .ok_or_else(|| anyhow::anyhow!("API endpoint not found. Set api.endpoint or choose a built-in provider."))?;
    let models_endpoint = provider.map(|provider| join_url(provider.base_url, provider.protocol.models_path()));

    if config.model.trim().is_empty() {
        bail!("API model not found. Set api.model.")
    }

    let protocol = config.protocol.or_else(|| provider.as_ref().map(|provider| provider.protocol)).unwrap_or_default();

    Ok(ResolvedApiConfig {
        endpoint,
        models_endpoint,
        model: config.model.clone(),
        api_key: config.api_key.clone(),
        max_tokens: config.max_tokens,
        temperature: config.temperature,
        // context_limit: config.context_limit,
        protocol,
    })
}

fn join_url(base_url: &str, path: &str) -> String {
    format!("{}/{}", base_url.trim_end_matches('/'), path.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::{find_provider, provider_specs, resolve_api_config};
    use crate::{ai::ProtocolKind, config::ApiConfig};

    #[test]
    fn should_get_provider_by_name_case_insensitively() {
        let provider = find_provider(" OpenAI ").expect("provider should exist");
        assert_eq!(provider.name, "openai");
    }

    #[test]
    fn should_return_builtin_provider_names() {
        let names: Vec<&str> = provider_specs().iter().map(|provider| provider.name).collect();

        assert_eq!(names, vec!["openai", "openrouter", "deepseek", "zhipu", "ollama", "lmstudio"]);
    }

    #[test]
    fn should_resolve_builtin_provider_defaults_with_config_overrides() {
        let resolved = resolve_api_config(&ApiConfig {
            name: None,
            provider: Some("openai".to_string()),
            endpoint: Some("https://proxy.example.com/chat".to_string()),
            model: "gpt-5.4".to_string(),
            api_key: Some("test-key".to_string()),
            max_tokens: Some(1000),
            temperature: Some(0.2),
            // context_limit: 200000,
            protocol: None,
        })
        .expect("config should resolve");

        assert_eq!(resolved.endpoint, "https://proxy.example.com/chat");
        assert_eq!(resolved.models_endpoint.as_deref(), Some("https://api.openai.com/v1/models"));
        assert_eq!(resolved.model, "gpt-5.4");
        assert_eq!(resolved.protocol, ProtocolKind::OpenAiCompatible);
    }

    #[test]
    fn should_resolve_custom_api_config_without_provider() {
        let resolved = resolve_api_config(&ApiConfig {
            name: None,
            provider: None,
            endpoint: Some("http://localhost:11434".to_string()),
            model: "qwen2.5:14b".to_string(),
            api_key: None,
            max_tokens: None,
            temperature: None,
            // context_limit: 200000,
            protocol: Some(ProtocolKind::Ollama),
        })
        .expect("custom config should resolve");

        assert_eq!(resolved.models_endpoint, None);
        assert_eq!(resolved.model, "qwen2.5:14b");
        assert_eq!(resolved.protocol, ProtocolKind::Ollama);
    }
}
