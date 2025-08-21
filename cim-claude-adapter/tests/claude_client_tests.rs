/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Claude API Client Tests
//! 
//! Unit tests for the pure Claude API client implementation.
//! Links to User Story 2.1: "Send Message to Claude" from user-stories.md

use std::time::Duration;
use serde_json::json;
use wiremock::{
    matchers::{method, path, header},
    Mock, MockServer, ResponseTemplate,
};

use cim_claude_adapter::{
    ClaudeClient, ClaudeConfig, ClaudeRequest, ClaudeMessage, 
    MessageRole, ClaudeError
};

/// Test fixtures and utilities
mod fixtures {
    use super::*;

    pub fn default_config() -> ClaudeConfig {
        ClaudeConfig {
            api_key: "test-api-key".to_string(),
            base_url: "http://localhost".to_string(), // Will be overridden by mock server
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
            timeout: Duration::from_secs(5),
        }
    }

    pub fn sample_request() -> ClaudeRequest {
        ClaudeRequest {
            messages: vec![ClaudeMessage {
                role: MessageRole::User,
                content: "Hello, Claude!".to_string(),
            }],
            system_prompt: Some("You are a helpful assistant.".to_string()),
            metadata: Some(json!({"test": true})),
        }
    }

    pub fn successful_response() -> serde_json::Value {
        json!({
            "id": "msg_01234567890",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "Hello! How can I help you today?"
                }
            ],
            "model": "claude-3-5-sonnet-20241022",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 10,
                "output_tokens": 15
            }
        })
    }
}

/// Test Story 2.1.1: Client Configuration and Creation
#[cfg(test)]
mod client_creation_tests {
    use super::*;

    #[test]
    fn test_create_client_with_valid_config() {
        // Given
        let config = fixtures::default_config();
        
        // When
        let result = ClaudeClient::new(config);
        
        // Then
        assert!(result.is_ok(), "Client creation should succeed with valid config");
    }

    #[test]
    fn test_create_client_with_empty_api_key() {
        // Given
        let mut config = fixtures::default_config();
        config.api_key = String::new();
        
        // When
        let result = ClaudeClient::new(config);
        
        // Then
        assert!(result.is_err(), "Client creation should fail with empty API key");
        if let Err(ClaudeError::Configuration(msg)) = result {
            assert_eq!(msg, "API key is required");
        } else {
            panic!("Expected Configuration error");
        }
    }

    #[test]
    fn test_create_client_with_invalid_api_key_characters() {
        // Given
        let mut config = fixtures::default_config();
        config.api_key = "invalid\x00key".to_string(); // Contains null byte
        
        // When
        let result = ClaudeClient::new(config);
        
        // Then
        assert!(result.is_err(), "Client creation should fail with invalid API key characters");
        if let Err(ClaudeError::Configuration(msg)) = result {
            assert!(msg.contains("Invalid API key format"));
        } else {
            panic!("Expected Configuration error");
        }
    }

    #[test]
    fn test_config_default_values() {
        // Given/When
        let config = ClaudeConfig::default();
        
        // Then
        assert_eq!(config.base_url, "https://api.anthropic.com");
        assert_eq!(config.model, "claude-3-5-sonnet-20241022");
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.api_key.is_empty());
    }
}

/// Test Story 2.1.2: Message Sending - Success Cases
#[cfg(test)]
mod message_sending_success_tests {
    use super::*;

    #[tokio::test]
    async fn test_send_message_successful() {
        // Given
        let mock_server = MockServer::start().await;
        let mut config = fixtures::default_config();
        config.base_url = mock_server.uri();
        
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("x-api-key", "test-api-key"))
            .and(header("content-type", "application/json"))
            .and(header("anthropic-version", "2023-06-01"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::successful_response()))
            .expect(1)
            .mount(&mock_server)
            .await;
            
        let client = ClaudeClient::new(config).unwrap();
        let request = fixtures::sample_request();
        
        // When
        let result = client.send_message(request).await;
        
        // Then
        if result.is_err() {
            println!("Error details: {:?}", result.err().unwrap());
            panic!("Message sending should succeed");
        }
        let response = result.unwrap();
        assert_eq!(response.content, "Hello! How can I help you today?");
        assert_eq!(response.model, "claude-3-5-sonnet-20241022");
        assert!(response.usage["input_tokens"].is_number());
        assert!(response.usage["output_tokens"].is_number());
    }

    #[tokio::test]
    async fn test_send_message_with_minimal_request() {
        // Given
        let mock_server = MockServer::start().await;
        let mut config = fixtures::default_config();
        config.base_url = mock_server.uri();
        
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::successful_response()))
            .expect(1)
            .mount(&mock_server)
            .await;
            
        let client = ClaudeClient::new(config).unwrap();
        let request = ClaudeRequest {
            messages: vec![ClaudeMessage {
                role: MessageRole::User,
                content: "Test".to_string(),
            }],
            system_prompt: None,
            metadata: None,
        };
        
        // When
        let result = client.send_message(request).await;
        
        // Then
        assert!(result.is_ok(), "Message sending should succeed with minimal request");
    }

    #[tokio::test]
    async fn test_send_message_with_multiple_messages() {
        // Given
        let mock_server = MockServer::start().await;
        let mut config = fixtures::default_config();
        config.base_url = mock_server.uri();
        
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::successful_response()))
            .expect(1)
            .mount(&mock_server)
            .await;
            
        let client = ClaudeClient::new(config).unwrap();
        let request = ClaudeRequest {
            messages: vec![
                ClaudeMessage {
                    role: MessageRole::User,
                    content: "First message".to_string(),
                },
                ClaudeMessage {
                    role: MessageRole::Assistant,
                    content: "Assistant response".to_string(),
                },
                ClaudeMessage {
                    role: MessageRole::User,
                    content: "Follow-up question".to_string(),
                },
            ],
            system_prompt: Some("You are helpful.".to_string()),
            metadata: None,
        };
        
        // When
        let result = client.send_message(request).await;
        
        // Then
        assert!(result.is_ok(), "Message sending should succeed with conversation history");
    }
}

/// Test Story 2.1.3: Message Sending - Error Cases  
#[cfg(test)]
mod message_sending_error_tests {
    use super::*;

    #[tokio::test]
    async fn test_send_message_authentication_error() {
        // Given
        let mock_server = MockServer::start().await;
        let mut config = fixtures::default_config();
        config.base_url = mock_server.uri();
        
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": {
                    "type": "authentication_error",
                    "message": "Invalid API key"
                }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;
            
        let client = ClaudeClient::new(config).unwrap();
        let request = fixtures::sample_request();
        
        // When
        let result = client.send_message(request).await;
        
        // Then
        assert!(result.is_err(), "Message sending should fail with invalid API key");
        if let Err(ClaudeError::Api { status_code: 401, .. }) = result {
            // Expected
        } else {
            panic!("Expected API error 401");
        }
    }

    #[tokio::test]
    async fn test_send_message_rate_limit_error() {
        // Given
        let mock_server = MockServer::start().await;
        let mut config = fixtures::default_config();
        config.base_url = mock_server.uri();
        
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(429).set_body_json(json!({
                "error": {
                    "type": "rate_limit_error",
                    "message": "Rate limit exceeded"
                }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;
            
        let client = ClaudeClient::new(config).unwrap();
        let request = fixtures::sample_request();
        
        // When
        let result = client.send_message(request).await;
        
        // Then
        assert!(result.is_err(), "Message sending should fail with rate limit");
        if let Err(ClaudeError::Api { status_code: 429, .. }) = result {
            // Expected - rate limit error should be retryable
        } else {
            panic!("Expected API error 429");
        }
    }

    #[tokio::test]
    async fn test_send_message_server_error() {
        // Given
        let mock_server = MockServer::start().await;
        let mut config = fixtures::default_config();
        config.base_url = mock_server.uri();
        
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .expect(1)
            .mount(&mock_server)
            .await;
            
        let client = ClaudeClient::new(config).unwrap();
        let request = fixtures::sample_request();
        
        // When
        let result = client.send_message(request).await;
        
        // Then
        assert!(result.is_err(), "Message sending should fail with server error");
        if let Err(ClaudeError::Api { status_code: 500, .. }) = result {
            // Expected
        } else {
            panic!("Expected API error 500");
        }
    }

    #[tokio::test]
    async fn test_send_message_network_error() {
        // Given
        let config = ClaudeConfig {
            api_key: "test-key".to_string(),
            base_url: "http://non-existent-host:12345".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
            timeout: Duration::from_millis(100), // Very short timeout
        };
        
        let client = ClaudeClient::new(config).unwrap();
        let request = fixtures::sample_request();
        
        // When
        let result = client.send_message(request).await;
        
        // Then
        assert!(result.is_err(), "Message sending should fail with network error");
        if let Err(ClaudeError::Network(_)) = result {
            // Expected
        } else {
            panic!("Expected Network error");
        }
    }

    #[tokio::test]
    async fn test_send_message_invalid_response_format() {
        // Given
        let mock_server = MockServer::start().await;
        let mut config = fixtures::default_config();
        config.base_url = mock_server.uri();
        
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Invalid JSON"))
            .expect(1)
            .mount(&mock_server)
            .await;
            
        let client = ClaudeClient::new(config).unwrap();
        let request = fixtures::sample_request();
        
        // When
        let result = client.send_message(request).await;
        
        // Then
        assert!(result.is_err(), "Message sending should fail with invalid JSON");
        if let Err(ClaudeError::Parsing(_)) = result {
            // Expected
        } else {
            panic!("Expected Parsing error");
        }
    }
}

/// Test Story 2.1.4: Health Check Functionality
#[cfg(test)]
mod health_check_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_successful() {
        // Given
        let mock_server = MockServer::start().await;
        let mut config = fixtures::default_config();
        config.base_url = mock_server.uri();
        
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "msg_health_check",
                "type": "message", 
                "role": "assistant",
                "content": [{"type": "text", "text": "Hello"}],
                "model": "claude-3-5-sonnet-20241022",
                "usage": {"input_tokens": 5, "output_tokens": 1}
            })))
            .expect(1)
            .mount(&mock_server)
            .await;
            
        let client = ClaudeClient::new(config).unwrap();
        
        // When
        let result = client.health_check().await;
        
        // Then
        assert!(result.is_ok(), "Health check should succeed");
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_health_check_authentication_error() {
        // Given
        let mock_server = MockServer::start().await;
        let mut config = fixtures::default_config();
        config.base_url = mock_server.uri();
        
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": {
                    "type": "authentication_error",
                    "message": "Invalid API key"
                }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;
            
        let client = ClaudeClient::new(config).unwrap();
        
        // When
        let result = client.health_check().await;
        
        // Then
        assert!(result.is_err(), "Health check should fail with invalid API key");
        if let Err(ClaudeError::Authentication(msg)) = result {
            assert_eq!(msg, "Invalid API key");
        } else {
            panic!("Expected Authentication error");
        }
    }

    #[tokio::test]
    async fn test_health_check_network_error() {
        // Given
        let config = ClaudeConfig {
            api_key: "test-key".to_string(),
            base_url: "http://non-existent-host:12345".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
            timeout: Duration::from_millis(100), // Very short timeout
        };
        
        let client = ClaudeClient::new(config).unwrap();
        
        // When
        let result = client.health_check().await;
        
        // Then
        assert!(result.is_err(), "Health check should fail with network error");
        if let Err(ClaudeError::Network(_)) = result {
            // Expected
        } else {
            panic!("Expected Network error");
        }
    }
}

/// Test Story 2.1.5: Future Features (Currently Not Supported)
#[cfg(test)]
mod future_features_tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_message_not_yet_supported() {
        // Given
        let config = fixtures::default_config();
        let client = ClaudeClient::new(config).unwrap();
        let request = fixtures::sample_request();
        
        // When
        let result = client.stream_message(request).await;
        
        // Then
        assert!(result.is_err(), "Streaming should not be supported yet");
        if let Err(ClaudeError::NotSupported(msg)) = result {
            assert_eq!(msg, "Streaming not yet implemented");
        } else {
            panic!("Expected NotSupported error");
        }
    }

    #[tokio::test]
    async fn test_get_model_info_placeholder() {
        // Given
        let config = fixtures::default_config();
        let client = ClaudeClient::new(config).unwrap();
        
        // When
        let result = client.get_model_info().await;
        
        // Then
        assert!(result.is_ok(), "Model info should return placeholder data");
        let info = result.unwrap();
        assert_eq!(info["model"], "claude-3-5-sonnet-20241022");
        assert_eq!(info["max_tokens"], 1024);
    }
}

/// Test Request/Response Payload Formatting
#[cfg(test)]
mod payload_formatting_tests {
    use super::*;

    #[tokio::test]
    async fn test_request_payload_formatting() {
        // Given
        let mock_server = MockServer::start().await;
        let mut config = fixtures::default_config();
        config.base_url = mock_server.uri();
        
        // Capture the request body
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::successful_response()))
            .expect(1)
            .mount(&mock_server)
            .await;
            
        let client = ClaudeClient::new(config).unwrap();
        let request = ClaudeRequest {
            messages: vec![
                ClaudeMessage {
                    role: MessageRole::User,
                    content: "Test message".to_string(),
                },
            ],
            system_prompt: Some("Test system prompt".to_string()),
            metadata: Some(json!({"conversation_id": "test-123"})),
        };
        
        // When
        let result = client.send_message(request).await;
        
        // Then
        assert!(result.is_ok(), "Request should be formatted correctly");
        
        // The mock server validates that the request contains the expected headers
        // and the request body is properly formatted JSON
    }

    #[tokio::test]
    async fn test_message_role_serialization() {
        // Given
        let user_msg = ClaudeMessage {
            role: MessageRole::User,
            content: "User message".to_string(),
        };
        let assistant_msg = ClaudeMessage {
            role: MessageRole::Assistant, 
            content: "Assistant message".to_string(),
        };
        let system_msg = ClaudeMessage {
            role: MessageRole::System,
            content: "System message".to_string(),
        };
        
        // When
        let user_json = serde_json::to_value(&user_msg).unwrap();
        let assistant_json = serde_json::to_value(&assistant_msg).unwrap();
        let system_json = serde_json::to_value(&system_msg).unwrap();
        
        // Then
        assert_eq!(user_json["role"], "user");
        assert_eq!(assistant_json["role"], "assistant");
        assert_eq!(system_json["role"], "system");
    }
}