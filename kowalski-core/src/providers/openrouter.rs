use crate::agent::types::StreamResponse;
use crate::error::KowalskiError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
    #[serde(default)]
    delta: Option<DeltaMessage>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: Option<String>,
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeltaMessage {
    content: Option<String>,
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: DeltaMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    base_url: String,
    site_url: String,
    site_name: String,
    default_model: String,
}

impl OpenRouterClient {
    pub fn new(api_key: String, base_url: Option<String>, default_model: Option<String>) -> Result<Self, KowalskiError> {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(300))
            .build()
            .map_err(KowalskiError::Request)?;

        Ok(Self {
            client,
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
            site_url: "https://github.com/yarenty/kowalski".to_string(),
            site_name: "Kowalski".to_string(),
            default_model: default_model.unwrap_or_else(|| "anthropic/claude-sonnet-4".to_string()),
        })
    }

    pub fn set_site_info(&mut self, site_url: String, site_name: String) {
        self.site_url = site_url;
        self.site_name = site_name;
    }

    pub async fn chat(
        &self,
        messages: Vec<Message>,
        model: Option<&str>,
        temperature: Option<f32>,
    ) -> Result<String, KowalskiError> {
        let request = ChatRequest {
            model: model.unwrap_or(&self.default_model).to_string(),
            messages,
            stream: None,
            temperature,
            max_tokens: None,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", &self.site_url)
            .header("X-Title", &self.site_name)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(KowalskiError::Server(format!("OpenRouter error: {}", error_text)));
        }

        let chat_response: ChatResponse = response.json().await?;
        
        chat_response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or_else(|| KowalskiError::Server("No response from OpenRouter".to_string()))
    }

    pub async fn chat_stream(
        &self,
        messages: Vec<Message>,
        model: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<usize>,
    ) -> Result<reqwest::Response, KowalskiError> {
        let request = ChatRequest {
            model: model.unwrap_or(&self.default_model).to_string(),
            messages,
            stream: Some(true),
            temperature,
            max_tokens,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", &self.site_url)
            .header("X-Title", &self.site_name)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(KowalskiError::Server(format!("OpenRouter error: {}", error_text)));
        }

        Ok(response)
    }

    pub fn parse_stream_chunk(&self, chunk: &[u8]) -> Result<Option<StreamResponse>, KowalskiError> {
        let text = String::from_utf8(chunk.to_vec())
            .map_err(|e| KowalskiError::Server(format!("Invalid UTF-8: {}", e)))?;

        let text = text.trim();
        if text.is_empty() || text == "data: [DONE]" {
            return Ok(None);
        }

        let json_str = text.strip_prefix("data: ").unwrap_or(text);
        if json_str.is_empty() {
            return Ok(None);
        }

        let stream_chunk: StreamChunk = serde_json::from_str(json_str)
            .map_err(|e| KowalskiError::Json(e))?;

        if let Some(choice) = stream_chunk.choices.into_iter().next() {
            let content = choice.delta.content.unwrap_or_default();
            let done = choice.finish_reason.is_some();
            
            return Ok(Some(StreamResponse {
                message: crate::conversation::Message {
                    role: choice.delta.role.unwrap_or_else(|| "assistant".to_string()),
                    content,
                    tool_calls: None,
                },
                done,
            }));
        }

        Ok(None)
    }
}
