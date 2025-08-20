/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Claude Error Tests
//!
//! Unit tests for Claude error types and error handling logic.
//! Links to User Story 2.2: "Handle Claude API Errors" and Story 6.1: "Handle Rate Limiting Gracefully"

use cim_claude_adapter::ClaudeError;

/// Test Story 2.2: Error Classification and Categorization
#[cfg(test)]
mod error_classification_tests {
    use super::*;

    #[test]
    fn test_configuration_error() {
        // Given/When
        let error = ClaudeError::Configuration("API key is required".to_string());
        
        // Then
        assert_eq!(error.to_string(), "Configuration error: API key is required");
        assert_eq!(error.category(), "configuration");
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_authentication_error() {
        // Given/When
        let error = ClaudeError::Authentication("Invalid API key".to_string());
        
        // Then
        assert_eq!(error.to_string(), "Authentication error: Invalid API key");
        assert_eq!(error.category(), "authentication");
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_network_error() {
        // Given/When
        let error = ClaudeError::Network("Connection timeout".to_string());
        
        // Then
        assert_eq!(error.to_string(), "Network error: Connection timeout");
        assert_eq!(error.category(), "network");
        assert!(error.is_retryable(), "Network errors should be retryable");
    }

    #[test]
    fn test_client_error() {
        // Given/When
        let error = ClaudeError::Client("Failed to create HTTP client".to_string());
        
        // Then
        assert_eq!(error.to_string(), "Client error: Failed to create HTTP client");
        assert_eq!(error.category(), "client");
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_parsing_error() {
        // Given/When
        let error = ClaudeError::Parsing("Invalid JSON response".to_string());
        
        // Then
        assert_eq!(error.to_string(), "Parsing error: Invalid JSON response");
        assert_eq!(error.category(), "parsing");
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_not_supported_error() {
        // Given/When
        let error = ClaudeError::NotSupported("Streaming not implemented".to_string());
        
        // Then
        assert_eq!(error.to_string(), "Not supported: Streaming not implemented");
        assert_eq!(error.category(), "not_supported");
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_timeout_error() {
        // Given/When
        let error = ClaudeError::Timeout("Request timed out after 30s".to_string());
        
        // Then
        assert_eq!(error.to_string(), "Timeout: Request timed out after 30s");
        assert_eq!(error.category(), "timeout");
        assert!(error.is_retryable(), "Timeout errors should be retryable");
    }
}

/// Test Story 6.1: Rate Limiting Error Handling
#[cfg(test)]
mod rate_limit_error_tests {
    use super::*;

    #[test]
    fn test_rate_limit_error() {
        // Given/When
        let error = ClaudeError::RateLimit("Rate limit exceeded, retry after 60s".to_string());
        
        // Then
        assert_eq!(error.to_string(), "Rate limited: Rate limit exceeded, retry after 60s");
        assert_eq!(error.category(), "rate_limit");
        assert!(error.is_retryable(), "Rate limit errors should be retryable");
    }

    #[test]
    fn test_api_error_rate_limit() {
        // Given/When
        let error = ClaudeError::Api {
            status_code: 429,
            message: "Too Many Requests".to_string(),
        };
        
        // Then
        assert_eq!(error.to_string(), "API error 429: Too Many Requests");
        assert_eq!(error.category(), "api");
        assert!(error.is_retryable(), "HTTP 429 errors should be retryable");
    }

    #[test]
    fn test_api_error_rate_limit_with_details() {
        // Given/When
        let error = ClaudeError::Api {
            status_code: 429,
            message: r#"{"error": {"type": "rate_limit_error", "message": "Rate limit exceeded"}}"#.to_string(),
        };
        
        // Then
        assert!(error.to_string().contains("429"));
        assert!(error.to_string().contains("Rate limit exceeded"));
        assert!(error.is_retryable());
    }
}

/// Test Story 2.2: API Error Types and Status Codes
#[cfg(test)]
mod api_error_tests {
    use super::*;

    #[test]
    fn test_api_error_client_error_400() {
        // Given/When
        let error = ClaudeError::Api {
            status_code: 400,
            message: "Bad Request".to_string(),
        };
        
        // Then
        assert_eq!(error.to_string(), "API error 400: Bad Request");
        assert_eq!(error.category(), "api");
        assert!(!error.is_retryable(), "4xx errors should not be retryable");
    }

    #[test]
    fn test_api_error_authentication_401() {
        // Given/When
        let error = ClaudeError::Api {
            status_code: 401,
            message: "Unauthorized".to_string(),
        };
        
        // Then
        assert_eq!(error.to_string(), "API error 401: Unauthorized");
        assert!(!error.is_retryable(), "Authentication errors should not be retryable");
    }

    #[test]
    fn test_api_error_forbidden_403() {
        // Given/When
        let error = ClaudeError::Api {
            status_code: 403,
            message: "Forbidden".to_string(),
        };
        
        // Then
        assert_eq!(error.to_string(), "API error 403: Forbidden");
        assert!(!error.is_retryable(), "Permission errors should not be retryable");
    }

    #[test]
    fn test_api_error_not_found_404() {
        // Given/When
        let error = ClaudeError::Api {
            status_code: 404,
            message: "Not Found".to_string(),
        };
        
        // Then
        assert_eq!(error.to_string(), "API error 404: Not Found");
        assert!(!error.is_retryable(), "404 errors should not be retryable");
    }

    #[test]
    fn test_api_error_server_error_500() {
        // Given/When
        let error = ClaudeError::Api {
            status_code: 500,
            message: "Internal Server Error".to_string(),
        };
        
        // Then
        assert_eq!(error.to_string(), "API error 500: Internal Server Error");
        assert!(error.is_retryable(), "5xx errors should be retryable");
    }

    #[test]
    fn test_api_error_bad_gateway_502() {
        // Given/When
        let error = ClaudeError::Api {
            status_code: 502,
            message: "Bad Gateway".to_string(),
        };
        
        // Then
        assert_eq!(error.to_string(), "API error 502: Bad Gateway");
        assert!(error.is_retryable(), "5xx errors should be retryable");
    }

    #[test]
    fn test_api_error_service_unavailable_503() {
        // Given/When
        let error = ClaudeError::Api {
            status_code: 503,
            message: "Service Unavailable".to_string(),
        };
        
        // Then
        assert_eq!(error.to_string(), "API error 503: Service Unavailable");
        assert!(error.is_retryable(), "5xx errors should be retryable");
    }

    #[test]
    fn test_api_error_gateway_timeout_504() {
        // Given/When
        let error = ClaudeError::Api {
            status_code: 504,
            message: "Gateway Timeout".to_string(),
        };
        
        // Then
        assert_eq!(error.to_string(), "API error 504: Gateway Timeout");
        assert!(error.is_retryable(), "5xx errors should be retryable");
    }
}

/// Test Retry Logic Classifications
#[cfg(test)]
mod retry_logic_tests {
    use super::*;

    #[test]
    fn test_retryable_errors() {
        let retryable_errors = vec![
            ClaudeError::Network("Connection failed".to_string()),
            ClaudeError::RateLimit("Rate limited".to_string()),
            ClaudeError::Timeout("Timed out".to_string()),
            ClaudeError::Api { status_code: 429, message: "Rate limited".to_string() },
            ClaudeError::Api { status_code: 500, message: "Server error".to_string() },
            ClaudeError::Api { status_code: 502, message: "Bad gateway".to_string() },
            ClaudeError::Api { status_code: 503, message: "Service unavailable".to_string() },
            ClaudeError::Api { status_code: 504, message: "Gateway timeout".to_string() },
            ClaudeError::Api { status_code: 599, message: "Network timeout".to_string() },
        ];

        for error in retryable_errors {
            assert!(error.is_retryable(), "Error should be retryable: {:?}", error);
        }
    }

    #[test]
    fn test_non_retryable_errors() {
        let non_retryable_errors = vec![
            ClaudeError::Configuration("Bad config".to_string()),
            ClaudeError::Authentication("Bad auth".to_string()),
            ClaudeError::Client("Client error".to_string()),
            ClaudeError::Parsing("Parse error".to_string()),
            ClaudeError::NotSupported("Not supported".to_string()),
            ClaudeError::Api { status_code: 400, message: "Bad request".to_string() },
            ClaudeError::Api { status_code: 401, message: "Unauthorized".to_string() },
            ClaudeError::Api { status_code: 403, message: "Forbidden".to_string() },
            ClaudeError::Api { status_code: 404, message: "Not found".to_string() },
            ClaudeError::Api { status_code: 422, message: "Unprocessable".to_string() },
        ];

        for error in non_retryable_errors {
            assert!(!error.is_retryable(), "Error should not be retryable: {:?}", error);
        }
    }

    #[test]
    fn test_boundary_status_codes() {
        // Test boundary cases for retry logic
        
        // 428 should not be retryable (precondition required)
        let error_428 = ClaudeError::Api { 
            status_code: 428, 
            message: "Precondition Required".to_string() 
        };
        assert!(!error_428.is_retryable());
        
        // 429 should be retryable (too many requests)  
        let error_429 = ClaudeError::Api { 
            status_code: 429, 
            message: "Too Many Requests".to_string() 
        };
        assert!(error_429.is_retryable());
        
        // 499 should not be retryable (client closed request)
        let error_499 = ClaudeError::Api { 
            status_code: 499, 
            message: "Client Closed Request".to_string() 
        };
        assert!(!error_499.is_retryable());
        
        // 500 should be retryable (internal server error)
        let error_500 = ClaudeError::Api { 
            status_code: 500, 
            message: "Internal Server Error".to_string() 
        };
        assert!(error_500.is_retryable());
    }
}

/// Test Error Message Formatting and Details
#[cfg(test)]
mod error_formatting_tests {
    use super::*;

    #[test]
    fn test_error_message_formatting() {
        let test_cases = vec![
            (
                ClaudeError::Configuration("Missing API key".to_string()),
                "Configuration error: Missing API key"
            ),
            (
                ClaudeError::Authentication("Invalid token".to_string()),
                "Authentication error: Invalid token"
            ),
            (
                ClaudeError::Network("DNS resolution failed".to_string()),
                "Network error: DNS resolution failed"
            ),
            (
                ClaudeError::Api { 
                    status_code: 422, 
                    message: "Validation failed".to_string() 
                },
                "API error 422: Validation failed"
            ),
            (
                ClaudeError::Parsing("Expected JSON, got XML".to_string()),
                "Parsing error: Expected JSON, got XML"
            ),
            (
                ClaudeError::RateLimit("Exceeded quota".to_string()),
                "Rate limited: Exceeded quota"
            ),
            (
                ClaudeError::Timeout("30 second timeout exceeded".to_string()),
                "Timeout: 30 second timeout exceeded"
            ),
        ];

        for (error, expected_message) in test_cases {
            assert_eq!(error.to_string(), expected_message);
        }
    }

    #[test]
    fn test_error_categories() {
        let test_cases = vec![
            (ClaudeError::Configuration("test".to_string()), "configuration"),
            (ClaudeError::Authentication("test".to_string()), "authentication"),
            (ClaudeError::Network("test".to_string()), "network"),
            (ClaudeError::Api { status_code: 400, message: "test".to_string() }, "api"),
            (ClaudeError::Parsing("test".to_string()), "parsing"),
            (ClaudeError::Client("test".to_string()), "client"),
            (ClaudeError::RateLimit("test".to_string()), "rate_limit"),
            (ClaudeError::NotSupported("test".to_string()), "not_supported"),
            (ClaudeError::Timeout("test".to_string()), "timeout"),
        ];

        for (error, expected_category) in test_cases {
            assert_eq!(error.category(), expected_category);
        }
    }

    #[test]
    fn test_complex_error_messages() {
        // Test errors with JSON content
        let json_error = ClaudeError::Api {
            status_code: 400,
            message: r#"{"error": {"type": "invalid_request_error", "message": "Missing required field"}}"#.to_string(),
        };
        assert!(json_error.to_string().contains("400"));
        assert!(json_error.to_string().contains("Missing required field"));
        
        // Test errors with special characters
        let special_char_error = ClaudeError::Network("Connection failed: 'timeout' after 30s".to_string());
        assert!(special_char_error.to_string().contains("'timeout'"));
        assert!(special_char_error.to_string().contains("30s"));
        
        // Test errors with newlines and formatting
        let multiline_error = ClaudeError::Parsing("JSON parse error:\nExpected '}' at line 5\nGot 'null'".to_string());
        assert!(multiline_error.to_string().contains("line 5"));
        assert!(multiline_error.to_string().contains("null"));
    }
}

/// Test Error Trait Implementations
#[cfg(test)]
mod error_trait_tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_error_trait_implementation() {
        let error = ClaudeError::Configuration("Test error".to_string());
        
        // Should implement std::error::Error
        let _: &dyn Error = &error;
        
        // Should be able to format with Display
        let display_string = format!("{}", error);
        assert_eq!(display_string, "Configuration error: Test error");
        
        // Should be able to format with Debug
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("Configuration"));
        assert!(debug_string.contains("Test error"));
    }

    #[test]
    fn test_error_send_sync() {
        // ClaudeError should be Send + Sync for use in async contexts
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<ClaudeError>();
        assert_sync::<ClaudeError>();
    }

    #[test]
    fn test_error_clone() {
        // Test that errors can be cloned (useful for retry logic)
        let original = ClaudeError::Network("Connection failed".to_string());
        let cloned = original.clone();
        
        assert_eq!(original.to_string(), cloned.to_string());
        assert_eq!(original.category(), cloned.category());
        assert_eq!(original.is_retryable(), cloned.is_retryable());
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = ClaudeError::Api {
            status_code: 429,
            message: "Rate limit exceeded".to_string(),
        };
        
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("Api"));
        assert!(debug_output.contains("status_code"));
        assert!(debug_output.contains("429"));
        assert!(debug_output.contains("Rate limit exceeded"));
    }
}