// Claude API Adapter: Implementation of ClaudeApiPort
// Handles HTTP communication with Claude API service

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::domain::conversation_aggregate::{
    Prompt, ClaudeResponse, ConversationId, CorrelationId, EventId
};
use crate::ports::claude_api_port::{
    ClaudeApiPort, ClaudeApiRequest, ClaudeApiResponse, ClaudeApiStreamResponse,
    ClaudeResponseStream, ResponseChunk, ApiHealth, ApiHealthStatus, 
    RateLimitStatus, ClaudeApiConfig, ClaudeApiError,
    CircuitBreakerState, CircuitBreakerConfig
};

// === Claude API HTTP Models ===

#[derive(Debug, Clone, Serialize)]
struct ClaudeHttpRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u32,
    temperature: f32,
    system: Option<String>,
    stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ClaudeHttpResponse {
    id: String,
    r#type: String,
    role: String,
    content: Vec<ClaudeContentBlock>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: ClaudeUsage,
}

#[derive(Debug, Clone, Deserialize)]
struct ClaudeContentBlock {
    r#type: String,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct ClaudeErrorResponse {
    r#type: String,
    error: ClaudeErrorDetail,
}

#[derive(Debug, Clone, Deserialize)]
struct ClaudeErrorDetail {
    r#type: String,
    message: String,
}

// === Circuit Breaker Implementation ===

struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            config,
        }
    }
    
    fn can_execute(&self) -> bool {
        match &self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open { opened_at } => {
                let now = chrono::Utc::now();
                let timeout_duration = chrono::Duration::seconds(self.config.timeout_seconds as i64);
                now.signed_duration_since(*opened_at) > timeout_duration
            },
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    fn on_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            },
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            },
            CircuitBreakerState::Open { .. } => {
                // Should not happen, but reset to half-open
                self.state = CircuitBreakerState::HalfOpen;
                self.success_count = 1;
            },
        }
    }
    
    fn on_failure(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitBreakerState::Open {
                        opened_at: chrono::Utc::now(),
                    };
                }
            },
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open {
                    opened_at: chrono::Utc::now(),
                };
                self.success_count = 0;
            },
            CircuitBreakerState::Open { .. } => {
                // Already open, update timestamp
                self.state = CircuitBreakerState::Open {
                    opened_at: chrono::Utc::now(),
                };
            },
        }
    }
}

// === Claude API Adapter Implementation ===

pub struct ClaudeApiAdapter {
    config: ClaudeApiConfig,
    client: Client,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    rate_limiter: Arc<RwLock<RateLimitTracker>>,
}

impl ClaudeApiAdapter {
    pub fn new(config: ClaudeApiConfig) -> Result<Self, ClaudeApiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds as u64))
            .user_agent("claude-adapter/1.0.0")
            .build()
            .map_err(|e| ClaudeApiError::InternalError { 
                message: format!("Failed to create HTTP client: {}", e)
            })?;
        
        let circuit_breaker = Arc::new(RwLock::new(
            CircuitBreaker::new(CircuitBreakerConfig::default())
        ));
        
        let rate_limiter = Arc::new(RwLock::new(RateLimitTracker::new()));
        
        Ok(Self {
            config,
            client,
            circuit_breaker,
            rate_limiter,
        })
    }
    
    async fn check_circuit_breaker(&self) -> Result<(), ClaudeApiError> {
        let breaker = self.circuit_breaker.read().await;
        if !breaker.can_execute() {
            return Err(ClaudeApiError::InternalError {
                message: "Circuit breaker is open".to_string(),
            });
        }
        Ok(())
    }
    
    async fn check_rate_limits(&self) -> Result<(), ClaudeApiError> {
        let rate_limiter = self.rate_limiter.read().await;
        if !rate_limiter.can_make_request() {
            return Err(ClaudeApiError::RateLimitExceeded {
                message: "Rate limit exceeded".to_string(),
            });
        }
        Ok(())
    }
    
    fn convert_to_claude_request(&self, request: &ClaudeApiRequest) -> ClaudeHttpRequest {
        let messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: request.prompt.content().to_string(),
        }];
        
        ClaudeHttpRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            system: request.system_prompt.clone(),
            stream: false,
        }
    }
    
    fn convert_from_claude_response(
        &self,
        claude_response: ClaudeHttpResponse,
        request: &ClaudeApiRequest,
        request_id: String,
        processing_time: Duration,
    ) -> Result<ClaudeApiResponse, ClaudeApiError> {
        // Extract text content from response
        let content = claude_response.content
            .into_iter()
            .filter(|block| block.r#type == "text")
            .map(|block| block.text)
            .collect::<Vec<_>>()
            .join("");
        
        let total_tokens = claude_response.usage.input_tokens + claude_response.usage.output_tokens;
        
        let claude_response_obj = ClaudeResponse::new(
            content,
            claude_response.model,
            total_tokens,
        );
        
        Ok(ClaudeApiResponse::new(
            claude_response_obj,
            request.conversation_id,
            request.correlation_id,
            request.event_id,
            request_id,
            processing_time.as_millis() as u64,
        ))
    }
    
    async fn make_api_request(&self, request: &ClaudeApiRequest) -> Result<ClaudeApiResponse, ClaudeApiError> {
        let start_time = Instant::now();
        
        // Pre-flight checks
        self.check_circuit_breaker().await?;
        self.check_rate_limits().await?;
        
        // Convert request
        let claude_request = self.convert_to_claude_request(request);
        
        // Make HTTP request
        let url = format!("{}/v1/messages", self.config.base_url);
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&claude_request)
            .send()
            .await;
        
        let response = match response {
            Ok(resp) => resp,
            Err(e) => {
                // Record failure in circuit breaker
                {
                    let mut breaker = self.circuit_breaker.write().await;
                    breaker.on_failure();
                }
                
                if e.is_timeout() {
                    return Err(ClaudeApiError::Timeout { 
                        seconds: self.config.timeout_seconds 
                    });
                } else {
                    return Err(ClaudeApiError::NetworkError { 
                        message: e.to_string() 
                    });
                }
            }
        };
        
        let processing_time = start_time.elapsed();
        let status = response.status();
        
        // Update rate limiter from headers
        self.update_rate_limiter_from_headers(response.headers()).await;
        
        match status {
            StatusCode::OK => {
                let claude_response: ClaudeHttpResponse = response
                    .json()
                    .await
                    .map_err(|e| ClaudeApiError::ParsingError { 
                        message: e.to_string() 
                    })?;
                
                // Record success
                {
                    let mut breaker = self.circuit_breaker.write().await;
                    breaker.on_success();
                }
                
                let request_id = "generated-request-id".to_string(); // Extract from headers if available
                
                self.convert_from_claude_response(claude_response, request, request_id, processing_time)
            },
            StatusCode::TOO_MANY_REQUESTS => {
                {
                    let mut breaker = self.circuit_breaker.write().await;
                    breaker.on_failure();
                }
                
                Err(ClaudeApiError::RateLimitExceeded {
                    message: "API rate limit exceeded".to_string(),
                })
            },
            StatusCode::UNAUTHORIZED => {
                Err(ClaudeApiError::AuthenticationError)
            },
            StatusCode::BAD_REQUEST => {
                let error_response: ClaudeErrorResponse = response
                    .json()
                    .await
                    .map_err(|e| ClaudeApiError::ParsingError { 
                        message: e.to_string() 
                    })?;
                
                Err(ClaudeApiError::InvalidRequest {
                    message: error_response.error.message,
                })
            },
            _ => {
                {
                    let mut breaker = self.circuit_breaker.write().await;
                    breaker.on_failure();
                }
                
                let body = response.text().await.unwrap_or_default();
                Err(ClaudeApiError::ServerError {
                    status_code: status.as_u16(),
                    message: body,
                })
            }
        }
    }
    
    async fn update_rate_limiter_from_headers(&self, headers: &reqwest::header::HeaderMap) {
        let mut rate_limiter = self.rate_limiter.write().await;
        
        if let Some(remaining) = headers.get("anthropic-ratelimit-requests-remaining") {
            if let Ok(remaining_str) = remaining.to_str() {
                if let Ok(remaining_count) = remaining_str.parse::<u32>() {
                    rate_limiter.update_requests_remaining(remaining_count);
                }
            }
        }
        
        if let Some(remaining) = headers.get("anthropic-ratelimit-tokens-remaining") {
            if let Ok(remaining_str) = remaining.to_str() {
                if let Ok(remaining_count) = remaining_str.parse::<u32>() {
                    rate_limiter.update_tokens_remaining(remaining_count);
                }
            }
        }
    }
}

#[async_trait]
impl ClaudeApiPort for ClaudeApiAdapter {
    type Error = ClaudeApiError;
    
    async fn send_prompt(&self, request: ClaudeApiRequest) -> Result<ClaudeApiResponse, Self::Error> {
        let mut retries = 0;
        let mut last_error = None;
        
        while retries < self.config.max_retries {
            match self.make_api_request(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    // Don't retry certain errors
                    match e {
                        ClaudeApiError::AuthenticationError |
                        ClaudeApiError::InvalidRequest { .. } |
                        ClaudeApiError::ContentFiltered { .. } => {
                            return Err(e);
                        },
                        _ => {
                            retries += 1;
                            if retries < self.config.max_retries {
                                tokio::time::sleep(Duration::from_millis(
                                    self.config.retry_delay_ms * retries as u64
                                )).await;
                            }
                        }
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or(ClaudeApiError::InternalError {
            message: "Max retries exceeded".to_string(),
        }))
    }
    
    async fn send_prompt_stream(&self, _request: ClaudeApiRequest) -> Result<ClaudeApiStreamResponse, Self::Error> {
        // Streaming implementation would go here
        // For now, return an error indicating it's not implemented
        Err(ClaudeApiError::InternalError {
            message: "Streaming not implemented yet".to_string(),
        })
    }
    
    async fn health_check(&self) -> Result<ApiHealth, Self::Error> {
        let start_time = Instant::now();
        
        // Make a simple request to check health
        let test_prompt = Prompt::new("Hello".to_string())
            .map_err(|_| ClaudeApiError::InternalError {
                message: "Failed to create test prompt".to_string(),
            })?;
        
        let test_request = ClaudeApiRequest::new(
            test_prompt,
            ConversationId::generate(),
            CorrelationId::generate(),
            EventId::generate(),
        ).with_max_tokens(1);
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        match self.make_api_request(&test_request).await {
            Ok(_) => {
                let rate_limits = self.get_rate_limits().await?;
                
                Ok(ApiHealth {
                    status: ApiHealthStatus::Healthy,
                    response_time_ms: response_time,
                    rate_limits,
                    last_successful_request: Some(chrono::Utc::now()),
                    error_count_last_hour: 0, // TODO: Track this
                })
            },
            Err(e) => {
                let status = match e {
                    ClaudeApiError::RateLimitExceeded { .. } => {
                        ApiHealthStatus::Degraded {
                            reason: "Rate limited".to_string(),
                        }
                    },
                    _ => {
                        ApiHealthStatus::Unhealthy {
                            reason: e.to_string(),
                        }
                    }
                };
                
                Ok(ApiHealth {
                    status,
                    response_time_ms: response_time,
                    rate_limits: self.get_rate_limits().await.unwrap_or_default(),
                    last_successful_request: None,
                    error_count_last_hour: 1,
                })
            }
        }
    }
    
    async fn get_rate_limits(&self) -> Result<RateLimitStatus, Self::Error> {
        let rate_limiter = self.rate_limiter.read().await;
        Ok(rate_limiter.get_status())
    }
}

// === Rate Limiter Implementation ===

struct RateLimitTracker {
    requests_remaining: u32,
    tokens_remaining: u32,
    last_updated: Instant,
}

impl RateLimitTracker {
    fn new() -> Self {
        Self {
            requests_remaining: 1000, // Default values
            tokens_remaining: 100000,
            last_updated: Instant::now(),
        }
    }
    
    fn can_make_request(&self) -> bool {
        self.requests_remaining > 0 && self.tokens_remaining > 1000 // Keep some buffer
    }
    
    fn update_requests_remaining(&mut self, remaining: u32) {
        self.requests_remaining = remaining;
        self.last_updated = Instant::now();
    }
    
    fn update_tokens_remaining(&mut self, remaining: u32) {
        self.tokens_remaining = remaining;
        self.last_updated = Instant::now();
    }
    
    fn get_status(&self) -> RateLimitStatus {
        let now = chrono::Utc::now();
        let reset_time = now + chrono::Duration::hours(1); // Typically reset every hour
        
        RateLimitStatus {
            requests_remaining: self.requests_remaining,
            requests_reset_at: reset_time,
            tokens_remaining: self.tokens_remaining,
            tokens_reset_at: reset_time,
        }
    }
}

impl Default for RateLimitStatus {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            requests_remaining: 0,
            requests_reset_at: now,
            tokens_remaining: 0,
            tokens_reset_at: now,
        }
    }
}