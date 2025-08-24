//! Claude AI Provider Implementation
//!
//! Anthropic Claude API integration following the universal LLM provider pattern

use super::{LlmProvider, Message, CompletionOptions, ProviderResponse, ProviderError, ProviderConfig, ProviderHealth, HealthStatus, TokenCount};
use async_trait::async_trait;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, error, warn};

/// Claude provider implementation
pub struct ClaudeProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl ClaudeProvider {
    /// Create new Claude provider
    pub fn new(config: ProviderConfig) -> Result<Self, ProviderError> {
        // Validate configuration
        if config.api_key.is_none() {
            return Err(ProviderError::InvalidRequest(
                "Claude provider requires API key".to_string()
            ));
        }
        
        // Build HTTP client with proper headers
        let mut headers = reqwest::header::HeaderMap::new();
        
        // Add Claude-specific headers
        if let Some(ref api_key) = config.api_key {
            headers.insert(
                "x-api-key",
                reqwest::header::HeaderValue::from_str(api_key)
                    .map_err(|e| ProviderError::InvalidRequest(format!("Invalid API key: {}", e)))?
            );
        }
        
        headers.insert(
            "content-type",
            reqwest::header::HeaderValue::from_static("application/json")
        );
        
        headers.insert(
            "anthropic-version",
            reqwest::header::HeaderValue::from_static("2023-06-01")
        );
        
        // Add custom headers
        for (key, value) in &config.custom_headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes())
                    .map_err(|e| ProviderError::InvalidRequest(format!("Invalid header name: {}", e)))?,
                reqwest::header::HeaderValue::from_str(value)
                    .map_err(|e| ProviderError::InvalidRequest(format!("Invalid header value: {}", e)))?
            );
        }
        
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| ProviderError::HttpError(format!("Failed to create HTTP client: {}", e)))?;
        
        info!("🤖 Claude provider initialized - model: {}", config.model);
        
        Ok(Self { config, client })
    }
    
    /// Convert universal messages to Claude format
    fn convert_messages(&self, messages: &[Message]) -> Vec<ClaudeMessage> {
        messages.iter().map(|msg| ClaudeMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        }).collect()
    }
    
    /// Extract system prompt from messages
    fn extract_system_prompt(&self, messages: &[Message]) -> Option<String> {
        messages.iter()
            .find(|msg| msg.role == "system")
            .map(|msg| msg.content.clone())
    }
    
    /// Filter out system messages (Claude handles them separately)
    fn filter_conversation_messages(&self, messages: &[Message]) -> Vec<Message> {
        messages.iter()
            .filter(|msg| msg.role != "system")
            .cloned()
            .collect()
    }
    
    /// Get base URL for Claude API
    fn base_url(&self) -> &str {
        self.config.base_url.as_deref().unwrap_or("https://api.anthropic.com")
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    async fn complete(
        &self,
        messages: Vec<Message>,
        options: Option<CompletionOptions>,
    ) -> Result<ProviderResponse, ProviderError> {
        let start_time = Instant::now();
        let opts = options.unwrap_or_default();
        
        // Extract system prompt
        let system_prompt = opts.system_prompt
            .or_else(|| self.extract_system_prompt(&messages));
        
        // Filter conversation messages
        let conversation_messages = self.filter_conversation_messages(&messages);
        let claude_messages = self.convert_messages(&conversation_messages);
        
        // Build request payload
        let mut payload = serde_json::json!({
            "model": opts.model.as_ref().unwrap_or(&self.config.model),
            "max_tokens": opts.max_tokens.unwrap_or(4096),
            "messages": claude_messages
        });
        
        // Add optional parameters
        if let Some(temp) = opts.temperature {
            payload["temperature"] = serde_json::Value::from(temp);
        }
        
        if let Some(top_p) = opts.top_p {
            payload["top_p"] = serde_json::Value::from(top_p);
        }
        
        if let Some(stop_sequences) = opts.stop_sequences {
            payload["stop_sequences"] = serde_json::Value::from(stop_sequences);
        }
        
        if let Some(system) = system_prompt {
            payload["system"] = serde_json::Value::from(system);
        }
        
        // Send request
        let url = format!("{}/v1/messages", self.base_url());
        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await?;
        
        let status = response.status();
        let response_text = response.text().await?;
        
        if !status.is_success() {
            return Err(ProviderError::ApiError {
                status_code: status.as_u16(),
                message: response_text,
            });
        }
        
        // Parse response
        let claude_response: ClaudeResponse = serde_json::from_str(&response_text)?;
        
        let elapsed = start_time.elapsed();
        info!("✅ Claude completion completed in {}ms", elapsed.as_millis());
        
        // Extract content from response
        let content = claude_response.content
            .into_iter()
            .find(|c| c.content_type == "text")
            .map(|c| c.text)
            .unwrap_or_else(|| "No text content found".to_string());
        
        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("model".to_string(), serde_json::Value::String(claude_response.model));
        metadata.insert("stop_reason".to_string(), serde_json::Value::String(claude_response.stop_reason.clone()));
        metadata.insert("response_time_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(elapsed.as_millis() as u64)));
        
        // Build token count
        let token_count = Some(TokenCount {
            input_tokens: claude_response.usage.input_tokens,
            output_tokens: claude_response.usage.output_tokens,
            total_tokens: claude_response.usage.input_tokens + claude_response.usage.output_tokens,
        });
        
        Ok(ProviderResponse {
            content,
            model_used: self.config.model.clone(),
            token_count,
            finish_reason: Some(claude_response.stop_reason),
            metadata,
        })
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        let start_time = Instant::now();
        
        // Simple health check with a minimal request
        let test_message = vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            metadata: None,
        }];
        
        let options = Some(CompletionOptions {
            max_tokens: Some(10),
            temperature: Some(0.1),
            ..Default::default()
        });
        
        match self.complete(test_message, options).await {
            Ok(_) => Ok(ProviderHealth {
                status: HealthStatus::Healthy,
                latency_ms: Some(start_time.elapsed().as_millis() as u64),
                error_message: None,
            }),
            Err(e) => {
                warn!("Claude health check failed: {}", e);
                Ok(ProviderHealth {
                    status: HealthStatus::Unhealthy,
                    latency_ms: Some(start_time.elapsed().as_millis() as u64),
                    error_message: Some(e.to_string()),
                })
            }
        }
    }
    
    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}

/// Claude API message format
#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude API response format
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ClaudeContent>,
    model: String,
    stop_reason: String,
    stop_sequence: Option<String>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}