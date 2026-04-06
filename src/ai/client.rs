use log::debug;
use reqwest::Client;
use serde::Serialize;

use crate::{
    ai::{
        OpenAI, Zhipu,
        provider::{ApiRequestTool, DeepSeek, Ollama, OpenRouter},
    },
    config::{ApiProvider, AppConfig},
};

#[derive(Serialize, Debug)]
pub struct Message<'a> {
    role: &'a str,
    content: &'a str,
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

    pub fn get_api_provider(&self) -> Option<Box<dyn ApiRequestTool>> {
        match self.config.provider.name {
            ApiProvider::OpenAI => Some(Box::new(OpenAI)),
            ApiProvider::DeepSeek => Some(Box::new(DeepSeek)),
            ApiProvider::OpenRouter => Some(Box::new(OpenRouter)),
            ApiProvider::Zhipu => Some(Box::new(Zhipu)),
            ApiProvider::Ollama => Some(Box::new(Ollama)),
        }
    }

    pub async fn send_chat_request(&self, messages: &[Message<'_>]) -> Result<String, Box<dyn std::error::Error>> {
        let provider = self.get_api_provider().ok_or_else(|| anyhow::anyhow!("No API provider found"))?;
        let request = provider.generate_request_body(&self.config, messages);
        let headers = provider.headers(&self.config.provider.api_key)?;
        let endpoint = if let Some(custom) = &self.config.provider.endpoint { custom } else { &provider.endpoint() };
        let response = self.client.post(endpoint).headers(headers).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API request failed: {error_text}").into());
        }

        if let Some(content) = provider.parse_response(response.json().await?) {
            Ok(content)
        } else {
            Err("No response from AI".into())
        }
    }

    pub async fn generate_commit_message(&self, diff: &str) -> Result<String, Box<dyn std::error::Error>> {
        let user_prompt = self.config.generate_user_prompt(diff);
        let system_message = Message::system(self.config.prompts.system_prompt.as_str());
        let user_message = Message::user(user_prompt.as_str());
        let messages = [system_message, user_message];
        debug!("Sending messages: {messages:?}");
        self.send_chat_request(&messages).await
    }
}
