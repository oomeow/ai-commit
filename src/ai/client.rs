use log::debug;
use reqwest::Client;
use serde::Serialize;

use crate::{ai::provider::resolve_api_config, config::AppConfig};

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

    pub async fn fetch_provider_models(&self) -> anyhow::Result<Vec<String>> {
        let api_config = resolve_api_config(&self.config.api)?;
        let models_endpoint = api_config.models_endpoint.as_deref().ok_or_else(|| {
            anyhow::anyhow!("Models endpoint not available for custom api.endpoint. Set a built-in provider.")
        })?;
        let headers = api_config.protocol.headers(api_config.api_key.as_deref())?;
        let response = self.client.get(models_endpoint).headers(headers).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Fetch models failed: {error_text}"));
        }

        Ok(api_config.protocol.parse_models_response(response.json().await?))
    }

    pub async fn send_chat_request(&self, system_msg: &Message<'_>, user_msg: &Message<'_>) -> anyhow::Result<String> {
        let api_config = resolve_api_config(&self.config.api)?;
        let body = api_config.protocol.build_chat_request_body(&api_config, system_msg, user_msg);
        debug!("{body:#?}");
        let headers = api_config.protocol.headers(api_config.api_key.as_deref())?;
        let response = self.client.post(&api_config.endpoint).headers(headers).json(&body).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("API request failed: {error_text}"));
        }

        if let Some(content) = api_config.protocol.parse_chat_response(response.json().await?) {
            Ok(content)
        } else {
            Err(anyhow::anyhow!("No response from AI"))
        }
    }

    pub async fn generate_commit_message(&self, diff: &str) -> anyhow::Result<String> {
        let user_prompt = self.config.generate_user_prompt(diff);
        // std::fs::write("user_prompt.md", &user_prompt).unwrap();
        let system_message = Message::system(self.config.prompts.system_prompt.as_str());
        let user_message = Message::user(user_prompt.as_str());
        debug!("Sending system messages: {system_message:?}");
        debug!("Sending user messages: {user_message:?}");
        self.send_chat_request(&system_message, &user_message).await
    }
}
