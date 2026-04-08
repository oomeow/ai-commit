use anyhow::Result;
use log::debug;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::{Value, json};

use crate::{ai::Message, config::AppConfig};

#[derive(Clone, Copy)]
pub enum ProtocolKind {
    OpenAiCompatible,
    Ollama,
    LMStudio,
}

impl ProtocolKind {
    pub fn headers(&self, api_key: &Option<String>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(key) = api_key {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", key))?);
        }
        Ok(headers)
    }

    pub fn build_chat_request(&self, config: &AppConfig, system_msg: &Message, user_msg: &Message) -> Value {
        match self {
            Self::OpenAiCompatible => json!({
                "model": config.api.model,
                "messages": [system_msg, user_msg],
                "stream": false,
                "thinking": {
                    "type": "disabled"
                },
                "max_tokens": config.api.max_tokens,
                "temperature": config.api.temperature
            }),
            Self::Ollama => json!({
                "model": config.api.model,
                "messages": [system_msg, user_msg],
                "think": false,
                "stream": false,
                "options": {
                    "temperature": config.api.temperature,
                    "num_predict": config.api.max_tokens
                }
            }),
            Self::LMStudio => json!({
                "model": config.api.model,
                "system_prompt": system_msg.content,
                "input": user_msg.content,
                "stream": false,
                // Will error if the model being used does not support the reasoning setting using.
                "reasoning": "off",
                "temperature": config.api.temperature,
                "max_output_tokens": config.api.max_tokens
            }),
        }
    }

    pub fn parse_chat_response(&self, response: Value) -> Option<String> {
        debug!("{:#?}", response);
        match self {
            Self::OpenAiCompatible => response["choices"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|choice| choice["message"]["content"].as_str())
                .map(str::to_owned),
            Self::Ollama => {
                response["message"].as_object().and_then(|message| message["content"].as_str()).map(str::to_owned)
            }
            Self::LMStudio => response["output"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|choice| choice["content"].as_str())
                .map(str::to_owned),
        }
    }

    pub fn parse_models_response(&self, response: Value) -> Vec<String> {
        debug!("{:#?}", response);
        match self {
            Self::OpenAiCompatible => response["data"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|model| model["id"].as_str().map(str::to_owned))
                .collect(),
            Self::Ollama => response["models"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|model| model["name"].as_str().map(str::to_owned))
                .collect(),
            Self::LMStudio => response["models"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|model| model["key"].as_str().map(str::to_owned))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ProtocolKind;

    #[test]
    fn should_parse_openai_compatible_models_response() {
        let response = json!({
            "data": [
                {"id": "gpt-4o"},
                {"id": "gpt-4.1-mini"}
            ]
        });

        assert_eq!(
            ProtocolKind::OpenAiCompatible.parse_models_response(response),
            vec!["gpt-4o".to_string(), "gpt-4.1-mini".to_string()]
        );
    }

    #[test]
    fn should_parse_ollama_models_response() {
        let response = json!({
            "models": [
                {"name": "qwen2.5:14b"},
                {"name": "deepseek-r1:14b"}
            ]
        });

        assert_eq!(
            ProtocolKind::Ollama.parse_models_response(response),
            vec!["qwen2.5:14b".to_string(), "deepseek-r1:14b".to_string()]
        );
    }

    #[test]
    fn should_build_authorization_header_for_openai_compatible_protocol() {
        let headers =
            ProtocolKind::OpenAiCompatible.headers(&Some("test-key".to_string())).expect("headers should build");

        assert_eq!(headers.get("authorization").and_then(|value| value.to_str().ok()), Some("Bearer test-key"));
    }

    #[test]
    fn should_not_add_authorization_header_without_api_key() {
        let headers = ProtocolKind::Ollama.headers(&None).expect("headers should build");

        assert!(headers.get("authorization").is_none());
    }
}
