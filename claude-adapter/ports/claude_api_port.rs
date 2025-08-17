// Outbound Port: ClaudeApiPort
// Interface for communicating with external Claude API service

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::conversation_aggregate::{
    Prompt, ClaudeResponse, ConversationId, CorrelationId, EventId
};

// === Outbound Port (Interface for domain to interact with external services) ===

#[async_trait]
pub trait ClaudeApiPort: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    /// Send a prompt to Claude API and receive response
    async fn send_prompt(&self, request: ClaudeApiRequest) -> Result<ClaudeApiResponse, Self::Error>;
    
    /// Send a prompt with streaming response
    async fn send_prompt_stream(&self, request: ClaudeApiRequest) -> Result<ClaudeApiStreamResponse, Self::Error>;
    
    /// Check API health and rate limits
    async fn health_check(&self) -> Result<ApiHealth, Self::Error>;
    
    /// Get current rate limit status
    async fn get_rate_limits(&self) -> Result<RateLimitStatus, Self::Error>;
}

// === API Request/Response Models ===

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeApiRequest {
    pub prompt: Prompt,
    pub conversation_id: ConversationId,
    pub correlation_id: CorrelationId,
    pub event_id: EventId,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub system_prompt: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl ClaudeApiRequest {
    pub fn new(
        prompt: Prompt,
        conversation_id: ConversationId,
        correlation_id: CorrelationId,
        event_id: EventId,
    ) -> Self {
        Self {
            prompt,
            conversation_id,
            correlation_id,
            event_id,
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            system_prompt: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
    
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }
    
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }
    
    pub fn with_system_prompt(mut self, system_prompt: String) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeApiResponse {
    pub response: ClaudeResponse,
    pub conversation_id: ConversationId,
    pub correlation_id: CorrelationId,
    pub event_id: EventId,
    pub request_id: String,
    pub processing_time_ms: u64,
    pub rate_limit_remaining: Option<u32>,
    pub metadata: HashMap<String, String>,
}

impl ClaudeApiResponse {
    pub fn new(
        response: ClaudeResponse,
        conversation_id: ConversationId,
        correlation_id: CorrelationId,
        event_id: EventId,
        request_id: String,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            response,
            conversation_id,
            correlation_id,
            event_id,
            request_id,
            processing_time_ms,
            rate_limit_remaining: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_rate_limit(mut self, remaining: u32) -> Self {
        self.rate_limit_remaining = Some(remaining);
        self
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

// === Streaming Response ===

#[derive(Debug)]
pub struct ClaudeApiStreamResponse {
    pub conversation_id: ConversationId,
    pub correlation_id: CorrelationId,
    pub event_id: EventId,
    pub request_id: String,
    pub stream: Box<dyn ClaudeResponseStream>,
}

#[async_trait]
pub trait ClaudeResponseStream: Send + Sync {
    async fn next_chunk(&mut self) -> Result<Option<ResponseChunk>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn collect_full_response(&mut self) -> Result<ClaudeResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut content = String::new();
        let mut model = String::new();
        let mut total_tokens = 0u32;
        
        while let Some(chunk) = self.next_chunk().await? {
            match chunk {
                ResponseChunk::Content { text } => content.push_str(&text),
                ResponseChunk::Metadata { model: m, tokens } => {
                    if !m.is_empty() {
                        model = m;
                    }
                    total_tokens += tokens;
                },
                ResponseChunk::Done => break,
            }
        }
        
        Ok(ClaudeResponse::new(content, model, total_tokens))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseChunk {
    Content { text: String },
    Metadata { model: String, tokens: u32 },
    Done,
}

// === API Health Monitoring ===

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiHealth {
    pub status: ApiHealthStatus,
    pub response_time_ms: u64,
    pub rate_limits: RateLimitStatus,
    pub last_successful_request: Option<chrono::DateTime<chrono::Utc>>,
    pub error_count_last_hour: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApiHealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub requests_remaining: u32,
    pub requests_reset_at: chrono::DateTime<chrono::Utc>,
    pub tokens_remaining: u32,
    pub tokens_reset_at: chrono::DateTime<chrono::Utc>,
}

// === Configuration ===

#[derive(Debug, Clone)]
pub struct ClaudeApiConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout_seconds: u32,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub rate_limit_buffer: u32,
    pub default_model: String,
    pub default_max_tokens: u32,
    pub default_temperature: f32,
}

impl Default for ClaudeApiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(), // Must be provided
            base_url: "https://api.anthropic.com".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            rate_limit_buffer: 10, // Keep 10 requests in reserve
            default_model: "claude-3-sonnet-20240229".to_string(),
            default_max_tokens: 4096,
            default_temperature: 0.7,
        }
    }
}

// === Claude API Specific Errors ===

#[derive(Debug, thiserror::Error)]
pub enum ClaudeApiError {
    #[error("Authentication failed: invalid API key")]
    AuthenticationError,
    
    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { message: String },
    
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },
    
    #[error("Model not available: {model}")]
    ModelNotAvailable { model: String },
    
    #[error("Content filtering triggered: {reason}")]
    ContentFiltered { reason: String },
    
    #[error("Token limit exceeded: used {used}, limit {limit}")]
    TokenLimitExceeded { used: u32, limit: u32 },
    
    #[error("API timeout after {seconds} seconds")]
    Timeout { seconds: u32 },
    
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    #[error("Server error: {status_code} - {message}")]
    ServerError { status_code: u16, message: String },
    
    #[error("Parsing error: {message}")]
    ParsingError { message: String },
    
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

// === Circuit Breaker Pattern for API Reliability ===

#[derive(Debug, Clone)]
pub enum CircuitBreakerState {
    Closed,
    Open { opened_at: chrono::DateTime<chrono::Utc> },
    HalfOpen,
}

pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout_seconds: u32,
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout_seconds: 60,
            success_threshold: 3,
        }
    }
}