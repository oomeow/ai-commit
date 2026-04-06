use anyhow::Result;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::json;

use crate::{ai::Message, config::AppConfig};

/// Trait for API providers, defining the interface for sending chat requests.
///
/// default use OpenAI protocol
pub trait ApiRequestTool {
    fn endpoint(&self) -> String;

    fn headers(&self, api_key: &Option<String>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(key) = api_key {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", key))?);
        }
        Ok(headers)
    }

    fn generate_request_body(&self, config: &AppConfig, messages: &[Message<'_>]) -> serde_json::Value {
        // json! 宏对 f32 类型进行序列化将导致精度丢失，0.7 -> 0.699999988079071
        json!({
            "model": config.provider.model,
            "messages": messages,
            "stream": false,
            "max_tokens": config.provider.max_tokens,
            "temperature": config.provider.temperature
        })
    }

    fn parse_response(&self, response: serde_json::Value) -> Option<String> {
        response["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|c| c["message"]["content"].as_str())
            .map(|s| s.to_string())
    }
}

pub struct OpenAI;
impl ApiRequestTool for OpenAI {
    fn endpoint(&self) -> String {
        "https://api.openai.com/v1/chat/completions".to_string()
    }
}

pub struct OpenRouter;
impl ApiRequestTool for OpenRouter {
    fn endpoint(&self) -> String {
        "https://openrouter.ai/api/v1/chat/completions".to_string()
    }
}

pub struct DeepSeek;
impl ApiRequestTool for DeepSeek {
    fn endpoint(&self) -> String {
        "https://api.deepseek.com/chat/completions".to_string()
    }

    fn generate_request_body(&self, config: &AppConfig, messages: &[super::Message<'_>]) -> serde_json::Value {
        json!({
            "model": config.provider.model,
            "messages": messages,
            "stream": false,
            "thinking": {
                "type": "disabled"
            },
            "max_tokens": config.provider.max_tokens,
            "temperature": config.provider.temperature
        })
    }
}

pub struct Zhipu;
impl ApiRequestTool for Zhipu {
    fn endpoint(&self) -> String {
        "https://open.bigmodel.cn/api/paas/v4/chat/completions".to_string()
    }

    fn generate_request_body(&self, config: &AppConfig, messages: &[super::Message<'_>]) -> serde_json::Value {
        json!({
            "model": config.provider.model,
            "messages": messages,
            "stream": false,
            "thinking": {
                "type": "disabled"
            },
            "max_tokens": config.provider.max_tokens,
            "temperature": config.provider.temperature
        })
    }
}

pub struct Ollama;
impl ApiRequestTool for Ollama {
    fn endpoint(&self) -> String {
        "http://localhost:11434/api/chat".to_string()
    }

    fn generate_request_body(&self, config: &AppConfig, messages: &[super::Message<'_>]) -> serde_json::Value {
        json!({
            "model": config.provider.model,
            "messages": messages,
            "think": false,
            "stream": false,
            "options": {
                "temperature": config.provider.temperature
            }
        })
    }

    fn parse_response(&self, response: serde_json::Value) -> Option<String> {
        response["message"].as_object().and_then(|c| c["content"].as_str()).map(|s| s.to_string())
    }
}
