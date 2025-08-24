//! OpenAI Provider Implementation
//!
//! Provides integration with OpenAI's GPT models (GPT-4, GPT-3.5, etc.)

use super::{LlmProvider, Message, CompletionOptions, ProviderResponse, ProviderError, ProviderConfig, ProviderHealth, HealthStatus, TokenCount};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::collections::HashMap;

/// OpenAI API base URL
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAI provider implementation
pub struct OpenAiProvider {
    config: ProviderConfig,
    client: Client,
    api_key: String,
}

impl OpenAiProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, ProviderError> {
        // Load API key from config or environment
        let api_key = if let Some(ref key) = config.api_key {
            key.clone()
        } else {
            // Try to read from secrets directory
            if let Ok(key) = std::fs::read_to_string("cim-llm-adapter/secrets/openai.api.key") {
                key.trim().to_string()
            } else {
                std::env::var("OPENAI_API_KEY")
                    .map_err(|_| ProviderError::AuthenticationError("OpenAI API key not found".to_string()))?
            }
        };
        
        let client = Client::new();
        
        Ok(Self { 
            config,
            client,
            api_key,
        })
    }
}

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    index: u32,
    message: OpenAiMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAiError {
    error: OpenAiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    code: Option<String>,
}

impl From<Message> for OpenAiMessage {
    fn from(msg: Message) -> Self {
        let role = match msg.role.as_str() {
            "system" => "system",
            "user" => "user",
            "assistant" => "assistant",
            _ => "user",
        }.to_string();
        
        OpenAiMessage {
            role,
            content: msg.content,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    async fn complete(
        &self,
        messages: Vec<Message>,
        options: Option<CompletionOptions>,
    ) -> Result<ProviderResponse, ProviderError> {
        let start = Instant::now();
        
        // Convert messages to OpenAI format
        let openai_messages: Vec<OpenAiMessage> = messages.into_iter()
            .map(|m| m.into())
            .collect();
        
        // Prepare request
        let model = options.as_ref()
            .and_then(|o| o.model.clone())
            .unwrap_or_else(|| self.config.model.clone());
        
        let request = OpenAiRequest {
            model,
            messages: openai_messages,
            max_tokens: options.as_ref().and_then(|o| o.max_tokens),
            temperature: options.as_ref().and_then(|o| o.temperature.map(|t| t as f32)),
            stream: Some(false),
        };
        
        // Make API request
        let response = self.client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::HttpError(e.to_string()))?;
        
        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| ProviderError::HttpError(e.to_string()))?;
        
        if !status.is_success() {
            // Try to parse error response
            if let Ok(error) = serde_json::from_str::<OpenAiError>(&response_text) {
                return Err(ProviderError::ApiError {
                    status_code: status.as_u16(),
                    message: error.error.message,
                });
            } else {
                return Err(ProviderError::ApiError {
                    status_code: status.as_u16(),
                    message: response_text,
                });
            }
        }
        
        // Parse successful response
        let openai_response: OpenAiResponse = serde_json::from_str(&response_text)
            .map_err(|e| ProviderError::SerializationError(format!("Failed to parse OpenAI response: {}", e)))?;
        
        // Extract content from first choice
        let content = openai_response.choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| ProviderError::InvalidRequest("No choices in OpenAI response".to_string()))?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        let mut metadata = HashMap::new();
        metadata.insert("latency_ms".to_string(), serde_json::json!(latency_ms));
        if let Some(ref finish_reason) = openai_response.choices.first().and_then(|c| c.finish_reason.clone()) {
            metadata.insert("finish_reason".to_string(), serde_json::json!(finish_reason));
        }
        
        Ok(ProviderResponse {
            content,
            model_used: openai_response.model,
            token_count: openai_response.usage.map(|u| TokenCount {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
            finish_reason: openai_response.choices.first().and_then(|c| c.finish_reason.clone()),
            metadata,
        })
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        let start = Instant::now();
        
        // Simple health check with minimal tokens
        let test_request = OpenAiRequest {
            model: "gpt-3.5-turbo".to_string(),  // Use cheaper model for health check
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            max_tokens: Some(5),
            temperature: Some(0.0),
            stream: Some(false),
        };
        
        let response = self.client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&test_request)
            .send()
            .await;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        match response {
            Ok(resp) if resp.status().is_success() => {
                Ok(ProviderHealth {
                    status: HealthStatus::Healthy,
                    latency_ms: Some(latency_ms),
                    error_message: None,
                })
            }
            Ok(resp) => {
                let status_code = resp.status();
                let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Ok(ProviderHealth {
                    status: HealthStatus::Unhealthy,
                    latency_ms: Some(latency_ms),
                    error_message: Some(format!("OpenAI API error {}: {}", status_code, error_text)),
                })
            }
            Err(e) => {
                Ok(ProviderHealth {
                    status: HealthStatus::Unhealthy,
                    latency_ms: None,
                    error_message: Some(format!("Failed to connect to OpenAI: {}", e)),
                })
            }
        }
    }
    
    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}