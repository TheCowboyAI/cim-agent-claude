/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Claude API Error Types

use thiserror::Error;

/// Claude API Errors
#[derive(Error, Debug)]
pub enum ClaudeError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// API errors with status codes
    #[error("API error {status_code}: {message}")]
    Api { status_code: u16, message: String },

    /// Response parsing errors
    #[error("Parsing error: {0}")]
    Parsing(String),

    /// Client creation errors
    #[error("Client error: {0}")]
    Client(String),

    /// Rate limiting errors
    #[error("Rate limited: {0}")]
    RateLimit(String),

    /// Features not yet supported
    #[error("Not supported: {0}")]
    NotSupported(String),

    /// Timeout errors
    #[error("Timeout: {0}")]
    Timeout(String),
}

impl ClaudeError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ClaudeError::Network(_)
                | ClaudeError::RateLimit(_)
                | ClaudeError::Timeout(_)
                | ClaudeError::Api {
                    status_code: 429 | 500..=599,
                    ..
                }
        )
    }

    /// Get error category for metrics
    pub fn category(&self) -> &'static str {
        match self {
            ClaudeError::Configuration(_) => "configuration",
            ClaudeError::Authentication(_) => "authentication",
            ClaudeError::Network(_) => "network",
            ClaudeError::Api { .. } => "api",
            ClaudeError::Parsing(_) => "parsing",
            ClaudeError::Client(_) => "client",
            ClaudeError::RateLimit(_) => "rate_limit",
            ClaudeError::NotSupported(_) => "not_supported",
            ClaudeError::Timeout(_) => "timeout",
        }
    }
}