use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

#[derive(Serialize, Debug)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
pub struct ChatRequest<'a> {
    model: &'a str,
    thinking: Thinking,
    messages: Vec<Message>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Thinking {
    r#type: String,
}

impl Default for Thinking {
    fn default() -> Self {
        Thinking { r#type: "disabled".to_string() }
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

    pub async fn send_chat_request(&self, messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error>> {
        let request = ChatRequest {
            model: self.config.api.model.as_ref(),
            messages,
            max_tokens: self.config.api.max_tokens,
            temperature: self.config.api.temperature,
            thinking: Thinking::default(),
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
        let system_message = Message { role: "system".to_string(), content: self.config.prompts.system_prompt.clone() };
        let user_message = Message { role: "user".to_string(), content: self.config.generate_user_prompt(diff) };
        let messages = vec![system_message, user_message];
        debug!("Sending messages: {messages:?}");
        self.send_chat_request(messages).await
    }
}
