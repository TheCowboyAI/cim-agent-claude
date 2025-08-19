/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use crate::domain::conversation_aggregate::ConversationState;
use thiserror::Error;

/// Domain errors for the Claude adapter
#[derive(Error, Debug, Clone, PartialEq)]
pub enum DomainError {
    #[error("Conversation not found")]
    ConversationNotFound,

    #[error("Conversation has ended")]
    ConversationEnded,

    #[error("Correlation ID mismatch")]
    CorrelationMismatch,

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: ConversationState,
        to: ConversationState,
    },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Exchange limit exceeded")]
    ExchangeLimitExceeded,

    #[error("No active exchange found")]
    NoActiveExchange,

    #[error("Prompt validation failed: {0}")]
    PromptValidation(String),

    #[error("Context validation failed: {0}")]
    ContextValidation(String),

    #[error("Aggregate version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u64, actual: u64 },

    #[error("Business rule violation: {rule}")]
    BusinessRuleViolation { rule: String },
}

/// Infrastructure errors (outside domain boundary)
#[derive(Error, Debug)]
pub enum InfrastructureError {
    #[error("NATS connection error: {0}")]
    NatsConnection(String),

    #[error("NATS publish error: {0}")]
    NatsPublish(String),

    #[error("NATS subscribe error: {0}")]
    NatsSubscribe(String),

    #[error("Claude API error: {0}")]
    ClaudeApi(String),

    #[error("Claude API rate limit: retry after {retry_after_seconds}s")]
    ClaudeApiRateLimit { retry_after_seconds: u32 },

    #[error("Claude API timeout")]
    ClaudeApiTimeout,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("NATS KV store error: {0}")]
    NatsKvStore(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

/// Application service errors
#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InfrastructureError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {reason}")]
    Conflict { reason: String },

    #[error("Service unavailable: {reason}")]
    ServiceUnavailable { reason: String },

    #[error("Optimistic locking failed - resource was modified by another process")]
    OptimisticLockingFailed,
}

impl From<serde_json::Error> for InfrastructureError {
    fn from(err: serde_json::Error) -> Self {
        InfrastructureError::Serialization(err.to_string())
    }
}

impl From<reqwest::Error> for InfrastructureError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            InfrastructureError::ClaudeApiTimeout
        } else if err.is_connect() {
            InfrastructureError::Network(err.to_string())
        } else {
            InfrastructureError::ClaudeApi(err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversions() {
        let domain_err = DomainError::ConversationNotFound;
        let app_err: ApplicationError = domain_err.into();
        assert!(matches!(app_err, ApplicationError::Domain(_)));

        let infra_err = InfrastructureError::ClaudeApiTimeout;
        let app_err: ApplicationError = infra_err.into();
        assert!(matches!(app_err, ApplicationError::Infrastructure(_)));
    }

    #[test]
    fn test_error_display() {
        let err = DomainError::InvalidStateTransition {
            from: ConversationState::Draft,
            to: ConversationState::Ended,
        };
        let error_string = err.to_string();
        assert!(error_string.contains("Invalid state transition"));
        assert!(error_string.contains("Draft"));
        assert!(error_string.contains("Ended"));
    }
}
