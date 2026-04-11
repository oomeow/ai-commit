use log::debug;
use reqwest::Client;
use serde::Serialize;

use crate::{
    ai::provider::{ProviderSpec, find_provider},
    config::AppConfig,
};

#[derive(Serialize, Debug)]
pub struct Message<'a> {
    role: &'a str,
    pub content: &'a str,
}

impl<'a> Message<'a> {
    pub fn system(content: &'a str) -> Self {
        Message { role: "system", content }
    }

    pub fn user(content: &'a str) -> Self {
        Message { role: "user", content }
    }
}

#[derive(Default)]
pub struct AiClient {
    client: Client,
    pub config: AppConfig,
}

impl AiClient {
    pub fn new() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        let client = Client::new();
        AiClient { client, config }
    }

    pub fn with_config(config: AppConfig) -> Self {
        let client = Client::new();
        AiClient { client, config }
    }

    fn current_provider(&self) -> anyhow::Result<ProviderSpec> {
        find_provider(&self.config.api.provider)
            .ok_or_else(|| anyhow::anyhow!("Unsupported API provider: {}", self.config.api.provider))
    }

    pub async fn fetch_available_models(&self) -> anyhow::Result<Vec<String>> {
        let provider = self.current_provider()?;
        let headers = provider.headers(&self.config.api.api_key)?;
        let response = self.client.get(provider.models_url(&self.config)).headers(headers).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Fetch models failed: {error_text}"));
        }

        Ok(provider.parse_models_response(response.json().await?))
    }

    pub async fn send_chat_request(&self, system_msg: &Message<'_>, user_msg: &Message<'_>) -> anyhow::Result<String> {
        let provider = self.current_provider()?;
        let body = provider.generate_request_body(&self.config, system_msg, user_msg);
        let headers = provider.headers(&self.config.api.api_key)?;
        let endpoint = provider.endpoint(&self.config);
        let response = self.client.post(endpoint).headers(headers).json(&body).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("API request failed: {error_text}"));
        }

        if let Some(content) = provider.parse_response(response.json().await?) {
            Ok(content)
        } else {
            Err(anyhow::anyhow!("No response from AI"))
        }
    }

    pub async fn generate_commit_message(&self, diff: &str) -> anyhow::Result<String> {
        let user_prompt = self.config.generate_user_prompt(diff);
        let system_message = Message::system(self.config.prompts.system_prompt.as_str());
        let user_message = Message::user(user_prompt.as_str());
        debug!("Sending system messages: {system_message:?}");
        debug!("Sending user messages: {user_message:?}");
        self.send_chat_request(&system_message, &user_message).await
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde_json::json;

    use super::AiClient;
    use crate::{
        ai::provider::find_provider,
        config::{ApiConfig, AppConfig, CommitConfig, PromptConfig},
    };

    fn build_test_client(provider_name: &str, base_url: String, api_key: Option<String>) -> AiClient {
        AiClient {
            client: Client::new(),
            config: AppConfig {
                api: ApiConfig {
                    provider: provider_name.to_string(),
                    base_url: Some(base_url),
                    endpoint: None,
                    model: "test-model".to_string(),
                    api_key,
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
                prompts: PromptConfig {
                    system_prompt: "system".to_string(),
                    user_prompt_template: "{diff}".to_string(),
                },
            },
        }
    }

    #[test]
    fn should_build_models_url_and_headers_for_openai_compatible_provider() {
        let client = build_test_client("openai", "https://api.openai.com".to_string(), Some("test-key".to_string()));
        let provider = find_provider(&client.config.api.provider).expect("provider should exist");
        let models_url = provider.models_url(&client.config);
        let headers = provider.headers(&client.config.api.api_key).expect("headers should build");

        assert_eq!(provider.name(), "openai");
        assert_eq!(models_url, "https://api.openai.com/v1/models");
        assert_eq!(headers.get("authorization").and_then(|value| value.to_str().ok()), Some("Bearer test-key"));
    }

    #[test]
    fn should_parse_models_for_openai_compatible_provider() {
        let client = build_test_client("openai", "https://api.openai.com".to_string(), Some("test-key".to_string()));
        let provider = find_provider(&client.config.api.provider).expect("provider should exist");
        let response = json!({
            "data": [
                {"id": "gpt-4o"},
                {"id": "gpt-4.1-mini"}
            ]
        });

        let models = provider.parse_models_response(response);

        assert_eq!(models, vec!["gpt-4o".to_string(), "gpt-4.1-mini".to_string()]);
    }

    #[test]
    fn should_build_models_url_and_headers_for_ollama_provider() {
        let client = build_test_client("ollama", "http://localhost:11434".to_string(), None);
        let provider = find_provider(&client.config.api.provider).expect("provider should exist");
        let models_url = provider.models_url(&client.config);
        let headers = provider.headers(&client.config.api.api_key).expect("headers should build");

        assert_eq!(provider.name(), "ollama");
        assert_eq!(models_url, "http://localhost:11434/api/tags");
        assert!(headers.get("authorization").is_none());
    }

    #[test]
    fn should_parse_models_for_ollama_provider() {
        let client = build_test_client("ollama", "http://localhost:11434".to_string(), None);
        let provider = find_provider(&client.config.api.provider).expect("provider should exist");
        let response = json!({
            "models": [
                {"name": "qwen2.5:14b"},
                {"name": "deepseek-r1:14b"}
            ]
        });

        let models = provider.parse_models_response(response);

        assert_eq!(models, vec!["qwen2.5:14b".to_string(), "deepseek-r1:14b".to_string()]);
    }
}
