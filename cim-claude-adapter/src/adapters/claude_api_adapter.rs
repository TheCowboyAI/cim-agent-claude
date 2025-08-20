use async_trait::async_trait;
use chrono::{DateTime, Utc};
use governor::{Quota, RateLimiter, DefaultDirectRateLimiter};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{num::NonZeroU32, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tracing::{info, error};

use crate::{
    domain::{errors::*, value_objects::*},
    ports::{
        ClaudeApiPort, CircuitBreakerPort, 
        ClaudeApiRequest, ClaudeApiResponse, ClaudeApiHealth, RateLimitStatus,
        CircuitBreakerState, CircuitBreakerMetrics,
    },
};

/// Claude API adapter implementing the Claude API port
pub struct ClaudeApiAdapter {
    client: Client,
    api_key: String,
    base_url: String,
    rate_limiter: Arc<DefaultDirectRateLimiter>,
    circuit_breaker: Arc<SimpleCircuitBreaker>,
}

/// Simple circuit breaker implementation
pub struct SimpleCircuitBreaker {
    state: RwLock<CircuitBreakerState>,
    failure_count: RwLock<u32>,
    last_failure_time: RwLock<Option<DateTime<Utc>>>,
    metrics: RwLock<CircuitBreakerMetrics>,
    failure_threshold: u32,
    timeout_duration: Duration,
    half_open_max_calls: u32,
    half_open_success_threshold: u32,
}

impl ClaudeApiAdapter {
    /// Create new Claude API adapter
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        // Rate limiter: 50 requests per minute (Claude's typical limit)
        let rate_limiter = Arc::new(
            RateLimiter::direct(Quota::per_minute(NonZeroU32::new(50).unwrap()))
        );
        
        let circuit_breaker = Arc::new(SimpleCircuitBreaker::new(
            5,  // failure threshold
            Duration::from_secs(60), // timeout duration
            3,  // half open max calls
            2,  // half open success threshold
        ));
        
        Self {
            client,
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            rate_limiter,
            circuit_breaker,
        }
    }
}

#[async_trait]
impl ClaudeApiPort for ClaudeApiAdapter {
    async fn send_prompt(
        &self,
        request: ClaudeApiRequest,
    ) -> Result<ClaudeApiResponse, ApplicationError> {
        let start_time = std::time::Instant::now();
        
        // Rate limiting
        self.rate_limiter.until_ready().await;
        
        // Circuit breaker - for now, just call directly (can enhance with proper circuit breaker later)
        let result = self.make_claude_request(&request).await;
        
        match result {
            Ok(response) => {
                info!(
                    "Claude API request successful for conversation {} ({}ms)",
                    request.conversation_id.as_uuid(),
                    start_time.elapsed().as_millis()
                );
                Ok(response)
            }
            Err(e) => {
                error!(
                    "Claude API request failed for conversation {}: {}",
                    request.conversation_id.as_uuid(),
                    e
                );
                Err(e)
            }
        }
    }
    
    async fn health_check(&self) -> Result<ClaudeApiHealth, ApplicationError> {
        let start_time = std::time::Instant::now();
        
        // Simple health check by making a minimal request
        let test_request = ClaudeApiRequest::new(
            Prompt::new("Hello".to_string()).map_err(|e| ApplicationError::Validation(e))?,
            ConversationContext::default(),
            ConversationId::new(),
            CorrelationId::new(),
            1,
        );
        
        let is_available = match self.make_claude_request(&test_request).await {
            Ok(_) => {
                info!("Claude API health check passed");
                true
            }
            Err(e) => {
                error!("Claude API health check failed: {}", e);
                false
            }
        };
        
        let response_time_ms = start_time.elapsed().as_millis() as u64;
        let circuit_metrics = self.circuit_breaker.get_metrics().await;
        let error_rate = circuit_metrics.failure_rate();
        
        let rate_limit_status = self.get_rate_limit_status().await.unwrap_or_else(|_| {
            RateLimitStatus {
                requests_remaining: 0,
                requests_limit: 50,
                tokens_remaining: 0,
                tokens_limit: 100_000,
                reset_time: Utc::now(),
            }
        });
        
        Ok(ClaudeApiHealth {
            is_available,
            response_time_ms,
            error_rate,
            last_check: Utc::now(),
            rate_limit_status,
        })
    }
    
    async fn get_rate_limit_status(&self) -> Result<RateLimitStatus, ApplicationError> {
        // This would typically be tracked based on response headers from Claude API
        // For now, return a mock status
        Ok(RateLimitStatus {
            requests_remaining: 45,
            requests_limit: 50,
            tokens_remaining: 90_000,
            tokens_limit: 100_000,
            reset_time: Utc::now() + chrono::Duration::minutes(1),
        })
    }
}

impl ClaudeApiAdapter {
    /// Make the actual HTTP request to Claude API
    async fn make_claude_request(
        &self,
        request: &ClaudeApiRequest,
    ) -> Result<ClaudeApiResponse, ApplicationError> {
        let start_time = std::time::Instant::now();
        
        // Build Claude API request payload
        let claude_request = ClaudeRequestPayload {
            model: request.context.metadata()
                .get("model")
                .cloned()
                .unwrap_or_else(|| "claude-3-sonnet-20240229".to_string()),
            max_tokens: request.context.max_tokens().unwrap_or(4000),
            messages: vec![
                ClaudeMessage {
                    role: "user".to_string(),
                    content: request.prompt.content().to_string(),
                }
            ],
            system: request.context.system_prompt().map(|s| s.to_string()),
            temperature: request.context.temperature(),
        };
        
        let url = format!("{}/v1/messages", self.base_url);
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("x-request-id", request.correlation_id.as_uuid().to_string())
            .json(&claude_request)
            .send()
            .await
            .map_err(InfrastructureError::from)?;
            
        let status = response.status();
        let headers = response.headers().clone();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return match status.as_u16() {
                429 => {
                    let retry_after = headers
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(60);
                    Err(InfrastructureError::ClaudeApiRateLimit { 
                        retry_after_seconds: retry_after 
                    }.into())
                }
                _ => Err(InfrastructureError::ClaudeApi(
                    format!("HTTP {}: {}", status, error_text)
                ).into())
            };
        }
        
        let claude_response: ClaudeResponsePayload = response.json().await
            .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;
        
        let processing_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Extract rate limit info from headers
        let rate_limit_remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());
            
        let rate_limit_reset = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now()));
        
        // Convert to domain objects
        let content = claude_response.content
            .first()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text.clone())
            .unwrap_or_default();
            
        let usage = TokenUsage::new(
            claude_response.usage.input_tokens,
            claude_response.usage.output_tokens,
        );
        
        let claude_response = ClaudeResponse::new(
            content,
            usage,
            claude_response.stop_reason,
            claude_response.model,
        );
        
        let mut api_response = ClaudeApiResponse::new(
            claude_response,
            request.correlation_id.as_uuid().to_string(),
            processing_time_ms,
        );
        
        if let (Some(remaining), Some(reset)) = (rate_limit_remaining, rate_limit_reset) {
            api_response = api_response.with_rate_limit_info(remaining, reset);
        }
        
        Ok(api_response)
    }
}

#[async_trait]
impl CircuitBreakerPort for SimpleCircuitBreaker {
    async fn execute<F, R>(&self, operation: F) -> Result<R, ApplicationError>
    where
        F: Fn() -> Result<R, ApplicationError> + Send + 'static,
        R: Send + 'static,
    {
        let state = self.get_state().await;
        
        match state {
            CircuitBreakerState::Open => {
                let last_failure = *self.last_failure_time.read().await;
                if let Some(failure_time) = last_failure {
                    if Utc::now() - failure_time > chrono::Duration::from_std(self.timeout_duration).unwrap() {
                        // Transition to half-open
                        *self.state.write().await = CircuitBreakerState::HalfOpen;
                        info!("Circuit breaker transitioning from Open to HalfOpen");
                    } else {
                        // Still in open state, fail fast
                        return Err(ApplicationError::ServiceUnavailable {
                            reason: "Circuit breaker is open".to_string()
                        });
                    }
                } else {
                    return Err(ApplicationError::ServiceUnavailable {
                        reason: "Circuit breaker is open".to_string()
                    });
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Allow limited calls in half-open state
                let metrics = self.metrics.read().await;
                if metrics.total_requests % (self.half_open_max_calls as u64 + 1) != 0 {
                    return Err(ApplicationError::ServiceUnavailable {
                        reason: "Circuit breaker is half-open, limiting requests".to_string()
                    });
                }
            }
            CircuitBreakerState::Closed => {
                // Normal operation
            }
        }
        
        // Execute the operation
        let start_time = Utc::now();
        let result = operation();
        
        // Update metrics and state based on result
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        
        match &result {
            Ok(_) => {
                metrics.successful_requests += 1;
                metrics.last_success_time = Some(start_time);
                
                // Reset failure count on success
                *self.failure_count.write().await = 0;
                
                // If in half-open and enough successes, close the circuit
                if state == CircuitBreakerState::HalfOpen {
                    if metrics.successful_requests >= self.half_open_success_threshold as u64 {
                        *self.state.write().await = CircuitBreakerState::Closed;
                        info!("Circuit breaker transitioning from HalfOpen to Closed");
                    }
                }
            }
            Err(_) => {
                metrics.failed_requests += 1;
                metrics.last_failure_time = Some(start_time);
                
                let mut failure_count = self.failure_count.write().await;
                *failure_count += 1;
                
                // Open circuit if failure threshold reached
                if *failure_count >= self.failure_threshold {
                    *self.state.write().await = CircuitBreakerState::Open;
                    *self.last_failure_time.write().await = Some(start_time);
                    metrics.circuit_breaker_open_count += 1;
                    info!("Circuit breaker opening due to failure threshold");
                }
            }
        }
        
        result
    }
    
    async fn get_state(&self) -> CircuitBreakerState {
        *self.state.read().await
    }
    
    async fn get_metrics(&self) -> CircuitBreakerMetrics {
        let metrics = self.metrics.read().await;
        let mut result = metrics.clone();
        result.current_state = *self.state.read().await;
        result
    }
    
    async fn reset(&self) -> Result<(), ApplicationError> {
        *self.state.write().await = CircuitBreakerState::Closed;
        *self.failure_count.write().await = 0;
        *self.last_failure_time.write().await = None;
        
        let mut metrics = self.metrics.write().await;
        *metrics = CircuitBreakerMetrics::default();
        
        info!("Circuit breaker manually reset");
        Ok(())
    }
}

impl SimpleCircuitBreaker {
    pub fn new(
        failure_threshold: u32,
        timeout_duration: Duration,
        half_open_max_calls: u32,
        half_open_success_threshold: u32,
    ) -> Self {
        Self {
            state: RwLock::new(CircuitBreakerState::Closed),
            failure_count: RwLock::new(0),
            last_failure_time: RwLock::new(None),
            metrics: RwLock::new(CircuitBreakerMetrics::default()),
            failure_threshold,
            timeout_duration,
            half_open_max_calls,
            half_open_success_threshold,
        }
    }
}

// Claude API request/response payloads
#[derive(Debug, Serialize)]
struct ClaudeRequestPayload {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponsePayload {
    content: Vec<ClaudeContentBlock>,
    usage: ClaudeUsage,
    stop_reason: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeContentBlock {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_circuit_breaker_states() {
        let cb = SimpleCircuitBreaker::new(2, Duration::from_millis(100), 1, 1);
        
        // Initial state should be Closed
        assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);
        
        // Simulate failures
        let _: Result<(), _> = cb.execute(|| Err(ApplicationError::ServiceUnavailable { 
            reason: "test".to_string() 
        })).await;
        let _: Result<(), _> = cb.execute(|| Err(ApplicationError::ServiceUnavailable { 
            reason: "test".to_string() 
        })).await;
        
        // Should now be Open
        assert_eq!(cb.get_state().await, CircuitBreakerState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Next call should transition to HalfOpen
        let _ = cb.execute(|| Ok(())).await;
        let state = cb.get_state().await;
        assert!(state == CircuitBreakerState::HalfOpen || state == CircuitBreakerState::Closed);
    }
    
    #[test]
    fn test_rate_limit_status() {
        let status = RateLimitStatus {
            requests_remaining: 4,
            requests_limit: 50,
            tokens_remaining: 900,
            tokens_limit: 10000,
            reset_time: Utc::now(),
        };
        
        assert!(status.is_near_limit());
        assert!(!status.is_exhausted());
        
        let exhausted_status = RateLimitStatus {
            requests_remaining: 0,
            requests_limit: 50,
            tokens_remaining: 1000,
            tokens_limit: 10000,
            reset_time: Utc::now(),
        };
        
        assert!(exhausted_status.is_exhausted());
    }
}