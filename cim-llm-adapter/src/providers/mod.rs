//! LLM Provider Abstractions
//!
//! Unified interface for multiple LLM providers with consistent API

pub mod claude;
pub mod openai;
pub mod ollama;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Universal LLM Provider trait
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;
    
    /// Send completion request to provider
    async fn complete(
        &self,
        messages: Vec<Message>,
        options: Option<CompletionOptions>,
    ) -> Result<ProviderResponse, ProviderError>;
    
    /// Check if provider is available
    async fn health_check(&self) -> Result<ProviderHealth, ProviderError>;
    
    /// Get provider configuration
    fn config(&self) -> &ProviderConfig;
}

/// Universal message format across providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String, // "user", "assistant", "system"
    pub content: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Completion options that can be passed to any provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionOptions {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: Option<bool>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
}

impl Default for CompletionOptions {
    fn default() -> Self {
        Self {
            max_tokens: Some(4096),
            temperature: Some(0.7),
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            model: None,
            system_prompt: None,
        }
    }
}

/// Provider response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub content: String,
    pub model_used: String,
    pub token_count: Option<TokenCount>,
    pub finish_reason: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token counting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCount {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

/// Provider health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub rate_limit_per_minute: Option<u32>,
    pub custom_headers: HashMap<String, String>,
}

/// Provider-specific errors
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    
    #[error("API error: {status_code} - {message}")]
    ApiError { status_code: u16, message: String },
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Provider timeout: {0}")]
    Timeout(String),
    
    #[error("Provider unavailable: {0}")]
    Unavailable(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ProviderError::Timeout(err.to_string())
        } else if err.is_connect() {
            ProviderError::Unavailable(err.to_string())
        } else {
            ProviderError::HttpError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        ProviderError::SerializationError(err.to_string())
    }
}