/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Claude API HTTP Client
//! 
//! Handles the actual HTTP communication with Claude API.
//! This is the only place in the adapter that talks to external HTTP services.

use crate::domain::claude_api::*;
use anyhow::{Result, Context};
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde_json;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn, debug, error, instrument};

/// Configuration for Claude API client
#[derive(Debug, Clone)]
pub struct ClaudeClientConfig {
    /// Claude API key
    pub api_key: String,
    /// API base URL
    pub base_url: String,
    /// Request timeout
    pub timeout: Duration,
    /// Maximum retries for failed requests
    pub max_retries: u32,
    /// Retry delay
    pub retry_delay: Duration,
    /// User agent for requests
    pub user_agent: String,
}

impl Default for ClaudeClientConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("CLAUDE_API_KEY").unwrap_or_default(),
            base_url: "https://api.anthropic.com".to_string(),
            timeout: Duration::from_secs(60),  // Increased timeout for Claude API
            max_retries: 3,
            retry_delay: Duration::from_secs(2),  // Longer retry delay
            user_agent: "cim-claude-adapter/0.1.0".to_string(),
        }
    }
}

/// Production Claude API HTTP client
#[derive(Clone)]
pub struct ClaudeClient {
    client: Client,
    config: ClaudeClientConfig,
}

impl ClaudeClient {
    /// Create a new Claude API client
    #[instrument(skip(config))]
    pub fn new(config: ClaudeClientConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(anyhow::anyhow!("Claude API key is required"));
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key", 
            HeaderValue::from_str(&config.api_key)
                .context("Invalid API key format")?
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json")
        );
        // Anthropic API version - hard-locked via Nix flake, fallback for development
        let api_version = option_env!("CIM_ANTHROPIC_API_VERSION").unwrap_or("2023-06-01");
        headers.insert(
            "anthropic-version",
            HeaderValue::from_str(api_version)
                .context("Invalid Anthropic API version")?
        );

        let client = Client::builder()
            .timeout(config.timeout)
            .default_headers(headers)
            .user_agent(&config.user_agent)
            .build()
            .context("Failed to create HTTP client")?;

        info!("Created Claude API client for base URL: {}", config.base_url);

        Ok(Self { client, config })
    }

    /// Send a message to Claude API
    #[instrument(skip(self, request))]
    pub async fn send_message(&self, request: ClaudeApiRequest) -> Result<ClaudeApiResponse> {
        let url = format!("{}/v1/messages", self.config.base_url);
        
        // Validate request before sending
        request.validate().map_err(|e| anyhow::anyhow!("Request validation failed: {}", e))?;

        let mut attempt = 0;
        loop {
            match self.make_request(&url, &request).await {
                Ok(response) => return Ok(response),
                Err(e) if attempt < self.config.max_retries => {
                    attempt += 1;
                    warn!("Claude API request failed (attempt {}/{}): {}", attempt, self.config.max_retries, e);
                    tokio::time::sleep(self.config.retry_delay).await;
                }
                Err(e) => {
                    error!("Claude API request failed after {} attempts: {}", self.config.max_retries, e);
                    return Err(e);
                }
            }
        }
    }

    /// Send a streaming message to Claude API
    #[instrument(skip(self, request))]
    pub async fn send_streaming_message(&self, request: ClaudeApiRequest) -> Result<ClaudeApiResponse> {
        let url = format!("{}/v1/messages", self.config.base_url);
        
        // For now, we'll implement streaming by setting stream=true in the request
        // A full streaming implementation would use Server-Sent Events
        let mut streaming_request = request;
        streaming_request.stream = Some(true);
        
        streaming_request.validate().map_err(|e| anyhow::anyhow!("Streaming request validation failed: {}", e))?;

        // TODO: Implement actual streaming response handling
        // For now, return a regular response
        self.make_request(&url, &streaming_request).await
    }

    /// Make the actual HTTP request
    async fn make_request(&self, url: &str, request: &ClaudeApiRequest) -> Result<ClaudeApiResponse> {
        debug!("Sending request to Claude API: {}", url);

        let response = timeout(
            self.config.timeout,
            self.client.post(url).json(request).send()
        )
        .await
        .context("Request timeout")?
        .context("HTTP request failed")?;

        let status = response.status();
        
        if !status.is_success() {
            let error_body = response.text().await
                .context("Failed to read error response")?;
            
            let claude_error = self.parse_error_response(status.as_u16(), error_body)?;
            return Err(anyhow::anyhow!("Claude API error: {:?}", claude_error));
        }

        let response_text = response.text().await
            .context("Failed to read response body")?;

        debug!("Received Claude API response: {} chars", response_text.len());

        let claude_response: ClaudeApiResponse = serde_json::from_str(&response_text)
            .context("Failed to parse Claude API response")?;

        Ok(claude_response)
    }

    /// Parse Claude API error response
    fn parse_error_response(&self, status_code: u16, error_body: String) -> Result<ClaudeApiError> {
        // Try to parse as structured error
        if let Ok(parsed_error) = serde_json::from_str::<serde_json::Value>(&error_body) {
            if let Some(error_obj) = parsed_error.get("error") {
                let error_type = error_obj.get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown_error");

                let message = error_obj.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");

                let claude_error_type = match error_type {
                    "invalid_request_error" => ClaudeErrorType::InvalidRequestError,
                    "authentication_error" => ClaudeErrorType::AuthenticationError,
                    "permission_error" => ClaudeErrorType::PermissionError,
                    "not_found_error" => ClaudeErrorType::NotFoundError,
                    "request_too_large" => ClaudeErrorType::RequestTooLarge,
                    "rate_limit_error" => ClaudeErrorType::RateLimitError,
                    "api_error" => ClaudeErrorType::ApiError,
                    "overloaded_error" => ClaudeErrorType::OverloadedError,
                    _ => ClaudeErrorType::ApiError,
                };

                return Ok(ClaudeApiError::new(claude_error_type, message.to_string(), status_code));
            }
        }

        // Fallback to generic error
        let error_type = match status_code {
            400 => ClaudeErrorType::InvalidRequestError,
            401 => ClaudeErrorType::AuthenticationError,
            403 => ClaudeErrorType::PermissionError,
            404 => ClaudeErrorType::NotFoundError,
            413 => ClaudeErrorType::RequestTooLarge,
            429 => ClaudeErrorType::RateLimitError,
            500..=599 => ClaudeErrorType::ApiError,
            _ => ClaudeErrorType::ApiError,
        };

        Ok(ClaudeApiError::new(error_type, error_body, status_code))
    }

    /// Health check - verify API connectivity
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<()> {
        // Create a minimal request to test connectivity
        let test_request = ClaudeApiRequest::new(
            ClaudeModel::Claude3Haiku20240307, // Use cheapest model for health check
            vec![ClaudeMessage::user("ping")],
            MaxTokens::new(10).unwrap()
        );

        match timeout(Duration::from_secs(10), self.make_request(&format!("{}/v1/messages", self.config.base_url), &test_request)).await {
            Ok(Ok(_)) => {
                info!("Claude API health check passed");
                Ok(())
            }
            Ok(Err(e)) => {
                warn!("Claude API health check failed: {}", e);
                Err(e)
            }
            Err(_) => {
                error!("Claude API health check timeout");
                Err(anyhow::anyhow!("Health check timeout"))
            }
        }
    }

    /// Get client configuration info
    pub fn config_info(&self) -> ClientInfo {
        ClientInfo {
            base_url: self.config.base_url.clone(),
            timeout_seconds: self.config.timeout.as_secs(),
            max_retries: self.config.max_retries,
            has_api_key: !self.config.api_key.is_empty(),
            anthropic_api_version: Self::anthropic_api_version(),
        }
    }

    /// Get the configured Anthropic API version
    /// This version is hard-locked via Nix flake for consistency
    pub fn anthropic_api_version() -> &'static str {
        option_env!("CIM_ANTHROPIC_API_VERSION").unwrap_or("2023-06-01")
    }
}

/// Client configuration info for monitoring
#[derive(Debug, serde::Serialize)]
pub struct ClientInfo {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub has_api_key: bool,
    pub anthropic_api_version: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        // Skip if no API key in environment
        if std::env::var("CLAUDE_API_KEY").is_err() {
            return;
        }

        let config = ClaudeClientConfig::default();
        assert!(!config.api_key.is_empty());
        assert_eq!(config.base_url, "https://api.anthropic.com");
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_error_parsing() {
        let config = ClaudeClientConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let client = ClaudeClient::new(config).unwrap();

        let error = client.parse_error_response(
            400,
            r#"{"error": {"type": "invalid_request_error", "message": "Invalid request"}}"#.to_string()
        ).unwrap();

        assert_eq!(error.error_type, ClaudeErrorType::InvalidRequestError);
        assert_eq!(error.message, "Invalid request");
        assert_eq!(error.http_status, 400);
    }

    #[tokio::test]
    async fn test_client_creation_without_api_key() {
        let config = ClaudeClientConfig {
            api_key: "".to_string(),
            ..Default::default()
        };

        let result = ClaudeClient::new(config);
        assert!(result.is_err());
    }
}