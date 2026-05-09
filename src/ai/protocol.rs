use anyhow::Result;
use log::debug;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::ai::{Message, ResolvedApiConfig};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolKind {
    #[default]
    #[serde(rename = "openai")]
    OpenAiCompatible,
    Ollama,
    LMStudio,
}

impl ProtocolKind {
    pub fn chat_path(&self) -> &'static str {
        match self {
            Self::OpenAiCompatible => "/chat/completions",
            Self::Ollama => "/api/chat",
            Self::LMStudio => "/api/v1/chat",
        }
    }

    pub fn models_path(&self) -> &'static str {
        match self {
            Self::OpenAiCompatible => "/models",
            Self::Ollama => "/api/tags",
            Self::LMStudio => "/api/v1/models",
        }
    }

    pub fn headers(&self, api_key: Option<&str>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(key) = api_key {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {key}"))?);
        }
        Ok(headers)
    }

    pub fn build_chat_request_body(
        &self,
        api_config: &ResolvedApiConfig,
        system_msg: &Message,
        user_msg: &Message,
    ) -> Value {
        match self {
            Self::OpenAiCompatible => json!({
                "model": api_config.model,
                "messages": [system_msg, user_msg],
                "stream": false,
                "thinking": {
                    "type": "disabled"
                },
                "max_tokens": api_config.max_tokens,
                "temperature": api_config.temperature
            }),
            Self::Ollama => json!({
                "model": api_config.model,
                "messages": [system_msg, user_msg],
                "think": false,
                "stream": false,
                "options": {
                    "temperature": api_config.temperature,
                    "num_predict": api_config.max_tokens
                }
            }),
            Self::LMStudio => json!({
                "model": api_config.model,
                "system_prompt": system_msg.content,
                "input": user_msg.content,
                "stream": false,
                // Will error if the model being used does not support the reasoning setting using.
                "reasoning": "off",
                "temperature": api_config.temperature,
                "max_output_tokens": api_config.max_tokens
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
        let headers = ProtocolKind::OpenAiCompatible.headers(Some("test-key")).expect("headers should build");

        assert_eq!(headers.get("authorization").and_then(|value| value.to_str().ok()), Some("Bearer test-key"));
    }

    #[test]
    fn should_not_add_authorization_header_without_api_key() {
        let headers = ProtocolKind::Ollama.headers(None).expect("headers should build");

        assert!(headers.get("authorization").is_none());
    }
}
