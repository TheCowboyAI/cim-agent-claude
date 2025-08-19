use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::domain::{value_objects::*, errors::*};

/// Outbound port for Claude API communication
/// This abstracts the Claude API client from the domain logic
#[async_trait]
pub trait ClaudeApiPort: Send + Sync {
    /// Send a prompt to Claude API and receive response
    async fn send_prompt(
        &self,
        request: ClaudeApiRequest,
    ) -> Result<ClaudeApiResponse, ApplicationError>;
    
    /// Check Claude API health and rate limits
    async fn health_check(&self) -> Result<ClaudeApiHealth, ApplicationError>;
    
    /// Get current rate limit status
    async fn get_rate_limit_status(&self) -> Result<RateLimitStatus, ApplicationError>;
}

/// Claude API request
#[derive(Debug, Clone)]
pub struct ClaudeApiRequest {
    pub prompt: Prompt,
    pub context: ConversationContext,
    pub conversation_id: ConversationId,
    pub correlation_id: CorrelationId,
    pub sequence_number: u32,
}

impl ClaudeApiRequest {
    pub fn new(
        prompt: Prompt,
        context: ConversationContext,
        conversation_id: ConversationId,
        correlation_id: CorrelationId,
        sequence_number: u32,
    ) -> Self {
        Self {
            prompt,
            context,
            conversation_id,
            correlation_id,
            sequence_number,
        }
    }
}

/// Claude API response
#[derive(Debug, Clone)]
pub struct ClaudeApiResponse {
    pub response: ClaudeResponse,
    pub request_id: String,
    pub processing_time_ms: u64,
    pub rate_limit_remaining: Option<u32>,
    pub rate_limit_reset: Option<DateTime<Utc>>,
}

impl ClaudeApiResponse {
    pub fn new(
        response: ClaudeResponse,
        request_id: String,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            response,
            request_id,
            processing_time_ms,
            rate_limit_remaining: None,
            rate_limit_reset: None,
        }
    }
    
    pub fn with_rate_limit_info(
        mut self,
        remaining: u32,
        reset: DateTime<Utc>,
    ) -> Self {
        self.rate_limit_remaining = Some(remaining);
        self.rate_limit_reset = Some(reset);
        self
    }
}

/// Claude API health status
#[derive(Debug, Clone)]
pub struct ClaudeApiHealth {
    pub is_available: bool,
    pub response_time_ms: u64,
    pub error_rate: f64,
    pub last_check: DateTime<Utc>,
    pub rate_limit_status: RateLimitStatus,
}

/// Rate limit status from Claude API
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub requests_remaining: u32,
    pub requests_limit: u32,
    pub tokens_remaining: u32,
    pub tokens_limit: u32,
    pub reset_time: DateTime<Utc>,
}

impl RateLimitStatus {
    pub fn is_near_limit(&self) -> bool {
        let request_ratio = self.requests_remaining as f64 / self.requests_limit as f64;
        let token_ratio = self.tokens_remaining as f64 / self.tokens_limit as f64;
        request_ratio < 0.1 || token_ratio < 0.1
    }
    
    pub fn is_exhausted(&self) -> bool {
        self.requests_remaining == 0 || self.tokens_remaining == 0
    }
}

/// Circuit breaker state for Claude API
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Circuit is open, failing fast
    HalfOpen, // Testing if service is back up
}

/// Circuit breaker port for managing Claude API resilience
#[async_trait]
pub trait CircuitBreakerPort: Send + Sync {
    /// Execute a request through the circuit breaker
    async fn execute<F, R>(&self, operation: F) -> Result<R, ApplicationError>
    where
        F: Fn() -> Result<R, ApplicationError> + Send + 'static,
        R: Send + 'static;
    
    /// Get current circuit breaker state
    async fn get_state(&self) -> CircuitBreakerState;
    
    /// Get circuit breaker metrics
    async fn get_metrics(&self) -> CircuitBreakerMetrics;
    
    /// Reset circuit breaker (admin operation)
    async fn reset(&self) -> Result<(), ApplicationError>;
}

/// Circuit breaker metrics
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerMetrics {
    pub total_requests: u64,
    pub failed_requests: u64,
    pub successful_requests: u64,
    pub timeout_requests: u64,
    pub circuit_breaker_open_count: u64,
    pub current_state: CircuitBreakerState,
    pub last_failure_time: Option<DateTime<Utc>>,
    pub last_success_time: Option<DateTime<Utc>>,
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        CircuitBreakerState::Closed
    }
}

impl CircuitBreakerMetrics {
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.failed_requests as f64 / self.total_requests as f64
        }
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }
}