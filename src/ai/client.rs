use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

#[derive(Serialize, Debug)]
pub struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
pub struct ChatRequest<'a> {
    model: &'a str,
    thinking: Thinking<'a>,
    messages: &'a [Message<'a>],
    max_tokens: Option<usize>,
    temperature: Option<f32>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Thinking<'a> {
    r#type: &'a str,
}

impl<'a> Default for Thinking<'a> {
    fn default() -> Self {
        Thinking { r#type: "disabled" }
    }
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
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

    pub async fn send_chat_request(&self, messages: &[Message<'_>]) -> Result<String, Box<dyn std::error::Error>> {
        let thinking = Thinking::default();
        let request = ChatRequest {
            model: self.config.api.model.as_ref(),
            messages,
            max_tokens: self.config.api.max_tokens,
            temperature: self.config.api.temperature,
            thinking,
        };
        let response = self
            .client
            .post(&self.config.api.endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API request failed: {error_text}").into());
        }

        let chat_response: ChatResponse = response.json().await?;

        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No response from AI".into())
        }
    }

    pub async fn generate_commit_message(&self, diff: &str) -> Result<String, Box<dyn std::error::Error>> {
        let user_prompt = self.config.generate_user_prompt(diff);
        let system_message = Message { role: "system", content: self.config.prompts.system_prompt.as_str() };
        let user_message = Message { role: "user", content: user_prompt.as_str() };
        let messages = [system_message, user_message];
        debug!("Sending messages: {messages:?}");
        self.send_chat_request(&messages).await
    }
}
