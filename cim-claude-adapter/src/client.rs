/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Claude API Client
//!
//! Pure Claude API integration without external dependencies.

use std::time::Duration;
use serde_json::Value;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};

use crate::{ClaudeError, ClaudeRequest, ClaudeResponse, ClaudeMessage};

/// Claude API Client Configuration
#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout: Duration,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.anthropic.com".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Pure Claude API Client
pub struct ClaudeClient {
    config: ClaudeConfig,
    client: Client,
}

impl ClaudeClient {
    /// Create a new Claude client
    pub fn new(config: ClaudeConfig) -> Result<Self, ClaudeError> {
        if config.api_key.is_empty() {
            return Err(ClaudeError::Configuration("API key is required".to_string()));
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", config.api_key))
                .map_err(|e| ClaudeError::Configuration(format!("Invalid API key format: {}", e)))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .build()
            .map_err(|e| ClaudeError::Client(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Send a message to Claude
    pub async fn send_message(&self, request: ClaudeRequest) -> Result<ClaudeResponse, ClaudeError> {
        let url = format!("{}/v1/messages", self.config.base_url);

        // Build request payload
        let payload = serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "messages": request.messages.iter().map(|msg| {
                serde_json::json!({
                    "role": match msg.role {
                        crate::MessageRole::User => "user",
                        crate::MessageRole::Assistant => "assistant",
                        crate::MessageRole::System => "system",
                    },
                    "content": msg.content
                })
            }).collect::<Vec<_>>(),
            "system": request.system_prompt.unwrap_or_default(),
            "metadata": request.metadata.unwrap_or_default()
        });

        // Send request
        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ClaudeError::Network(format!("Request failed: {}", e)))?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(ClaudeError::Api {
                status_code: status.as_u16(),
                message: error_body,
            });
        }

        // Parse response
        let response_body: Value = response
            .json()
            .await
            .map_err(|e| ClaudeError::Parsing(format!("Failed to parse response: {}", e)))?;

        // Extract Claude's response
        let content = response_body["content"]
            .as_array()
            .and_then(|arr| arr.get(0))
            .and_then(|obj| obj["text"].as_str())
            .unwrap_or("")
            .to_string();

        let usage = response_body["usage"].clone();

        Ok(ClaudeResponse {
            content,
            model: response_body["model"].as_str().unwrap_or(&self.config.model).to_string(),
            usage,
            metadata: response_body.get("metadata").cloned(),
        })
    }

    /// Stream a message to Claude (for future implementation)
    pub async fn stream_message(&self, _request: ClaudeRequest) -> Result<Box<dyn futures::Stream<Item = Result<ClaudeResponse, ClaudeError>> + Unpin + Send>, ClaudeError> {
        // TODO: Implement streaming support
        Err(ClaudeError::NotSupported("Streaming not yet implemented".to_string()))
    }

    /// Get model information
    pub async fn get_model_info(&self) -> Result<Value, ClaudeError> {
        // TODO: Implement model info endpoint when available
        Ok(serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens
        }))
    }

    /// Health check - verify API connectivity
    pub async fn health_check(&self) -> Result<bool, ClaudeError> {
        let test_request = ClaudeRequest {
            messages: vec![ClaudeMessage {
                role: crate::MessageRole::User,
                content: "Hello".to_string(),
            }],
            system_prompt: Some("Respond with 'Hello' only.".to_string()),
            metadata: None,
        };

        match self.send_message(test_request).await {
            Ok(_) => Ok(true),
            Err(ClaudeError::Api { status_code: 401, .. }) => {
                Err(ClaudeError::Authentication("Invalid API key".to_string()))
            }
            Err(e) => Err(e),
        }
    }
}