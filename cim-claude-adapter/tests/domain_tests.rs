/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Domain Types Tests
//!
//! Unit tests for Claude domain types and data structures.
//! Links to User Stories 4.1-4.3: "Conversation Management" from user-stories.md

use chrono::Utc;
use serde_json::json;

use cim_claude_adapter::{
    ClaudeRequest, ClaudeResponse, ClaudeMessage, MessageRole, 
    ConversationContext, ModelInfo, Usage
};

/// Test Story 4.1: Claude Message and Request Construction
#[cfg(test)]
mod message_construction_tests {
    use super::*;

    #[test]
    fn test_claude_message_creation() {
        // Given/When
        let message = ClaudeMessage {
            role: MessageRole::User,
            content: "Hello, Claude!".to_string(),
        };
        
        // Then
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, "Hello, Claude!");
    }

    #[test]
    fn test_claude_request_minimal() {
        // Given/When
        let request = ClaudeRequest {
            messages: vec![ClaudeMessage {
                role: MessageRole::User,
                content: "Test message".to_string(),
            }],
            system_prompt: None,
            metadata: None,
        };
        
        // Then
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].content, "Test message");
        assert!(request.system_prompt.is_none());
        assert!(request.metadata.is_none());
    }

    #[test]
    fn test_claude_request_complete() {
        // Given/When
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
            ],
            system_prompt: Some("You are a helpful assistant.".to_string()),
            metadata: Some(json!({"conversation_id": "test-123", "user_id": "user-456"})),
        };
        
        // Then
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.system_prompt.as_ref().unwrap(), "You are a helpful assistant.");
        assert!(request.metadata.is_some());
        assert_eq!(request.metadata.as_ref().unwrap()["conversation_id"], "test-123");
    }

    #[test]
    fn test_message_role_equality() {
        // Given/When/Then
        assert_eq!(MessageRole::User, MessageRole::User);
        assert_eq!(MessageRole::Assistant, MessageRole::Assistant);
        assert_eq!(MessageRole::System, MessageRole::System);
        
        assert_ne!(MessageRole::User, MessageRole::Assistant);
        assert_ne!(MessageRole::User, MessageRole::System);
        assert_ne!(MessageRole::Assistant, MessageRole::System);
    }
}

/// Test Story 4.1: Claude Response Handling
#[cfg(test)]
mod response_handling_tests {
    use super::*;

    #[test]
    fn test_claude_response_creation() {
        // Given/When
        let response = ClaudeResponse {
            content: "Hello! How can I help you?".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            usage: json!({
                "input_tokens": 10,
                "output_tokens": 8,
                "total_tokens": 18
            }),
            metadata: Some(json!({"finish_reason": "stop"})),
        };
        
        // Then
        assert_eq!(response.content, "Hello! How can I help you?");
        assert_eq!(response.model, "claude-3-5-sonnet-20241022");
        assert_eq!(response.usage["input_tokens"], 10);
        assert_eq!(response.usage["output_tokens"], 8);
        assert_eq!(response.metadata.as_ref().unwrap()["finish_reason"], "stop");
    }

    #[test]
    fn test_usage_from_json_value() {
        // Given
        let usage_json = json!({
            "input_tokens": 25,
            "output_tokens": 15
        });
        
        // When
        let usage: Usage = usage_json.into();
        
        // Then
        assert_eq!(usage.input_tokens, 25);
        assert_eq!(usage.output_tokens, 15);
        assert_eq!(usage.total_tokens, 40); // Should be sum of input + output
    }

    #[test]
    fn test_usage_from_incomplete_json() {
        // Given
        let usage_json = json!({
            "input_tokens": 10
            // Missing output_tokens
        });
        
        // When
        let usage: Usage = usage_json.into();
        
        // Then
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.output_tokens, 0); // Should default to 0
        assert_eq!(usage.total_tokens, 10);
    }

    #[test]
    fn test_usage_from_empty_json() {
        // Given
        let usage_json = json!({});
        
        // When
        let usage: Usage = usage_json.into();
        
        // Then
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
    }
}

/// Test Story 4.1-4.2: Conversation Context Management
#[cfg(test)]
mod conversation_context_tests {
    use super::*;

    #[test]
    fn test_new_conversation_context() {
        // Given
        let conversation_id = "conv-12345".to_string();
        let before_creation = Utc::now();
        
        // When
        let context = ConversationContext::new(conversation_id.clone());
        let after_creation = Utc::now();
        
        // Then
        assert_eq!(context.id, conversation_id);
        assert!(context.messages.is_empty());
        assert!(context.system_prompt.is_none());
        assert!(context.metadata.is_none());
        
        // Check timestamps are reasonable
        assert!(context.created_at >= before_creation);
        assert!(context.created_at <= after_creation);
        assert_eq!(context.created_at, context.updated_at);
    }

    #[test]
    fn test_add_message_to_conversation() {
        // Given
        let mut context = ConversationContext::new("test-conv".to_string());
        let original_updated_at = context.updated_at;
        
        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let message = ClaudeMessage {
            role: MessageRole::User,
            content: "Hello!".to_string(),
        };
        
        // When
        context.add_message(message);
        
        // Then
        assert_eq!(context.messages.len(), 1);
        assert_eq!(context.messages[0].content, "Hello!");
        assert_eq!(context.messages[0].role, MessageRole::User);
        assert!(context.updated_at > original_updated_at);
    }

    #[test]
    fn test_add_multiple_messages() {
        // Given
        let mut context = ConversationContext::new("test-conv".to_string());
        
        let messages = vec![
            ClaudeMessage { role: MessageRole::User, content: "Hello".to_string() },
            ClaudeMessage { role: MessageRole::Assistant, content: "Hi there!".to_string() },
            ClaudeMessage { role: MessageRole::User, content: "How are you?".to_string() },
        ];
        
        // When
        for message in messages {
            context.add_message(message);
        }
        
        // Then
        assert_eq!(context.messages.len(), 3);
        assert_eq!(context.messages[0].content, "Hello");
        assert_eq!(context.messages[1].content, "Hi there!");
        assert_eq!(context.messages[2].content, "How are you?");
        
        assert_eq!(context.messages[0].role, MessageRole::User);
        assert_eq!(context.messages[1].role, MessageRole::Assistant);
        assert_eq!(context.messages[2].role, MessageRole::User);
    }

    #[test]
    fn test_conversation_to_request() {
        // Given
        let mut context = ConversationContext::new("test-conv".to_string());
        context.system_prompt = Some("You are helpful.".to_string());
        context.metadata = Some(json!({"priority": "high"}));
        
        context.add_message(ClaudeMessage {
            role: MessageRole::User,
            content: "Test message".to_string(),
        });
        
        // When
        let request = context.to_request();
        
        // Then
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].content, "Test message");
        assert_eq!(request.system_prompt.as_ref().unwrap(), "You are helpful.");
        assert_eq!(request.metadata.as_ref().unwrap()["priority"], "high");
    }

    #[test]
    fn test_conversation_to_request_minimal() {
        // Given
        let mut context = ConversationContext::new("test-conv".to_string());
        context.add_message(ClaudeMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
        });
        
        // When
        let request = context.to_request();
        
        // Then
        assert_eq!(request.messages.len(), 1);
        assert!(request.system_prompt.is_none());
        assert!(request.metadata.is_none());
    }
}

/// Test Model Information Structures
#[cfg(test)]
mod model_info_tests {
    use super::*;

    #[test]
    fn test_model_info_creation() {
        // Given/When
        let model_info = ModelInfo {
            name: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            context_window: 200000,
            capabilities: vec![
                "text_generation".to_string(),
                "code_understanding".to_string(),
                "analysis".to_string(),
            ],
        };
        
        // Then
        assert_eq!(model_info.name, "claude-3-5-sonnet-20241022");
        assert_eq!(model_info.max_tokens, 4096);
        assert_eq!(model_info.context_window, 200000);
        assert_eq!(model_info.capabilities.len(), 3);
        assert!(model_info.capabilities.contains(&"text_generation".to_string()));
    }

    #[test]
    fn test_usage_statistics() {
        // Given/When
        let usage = Usage {
            input_tokens: 150,
            output_tokens: 75,
            total_tokens: 225,
        };
        
        // Then
        assert_eq!(usage.input_tokens, 150);
        assert_eq!(usage.output_tokens, 75);
        assert_eq!(usage.total_tokens, 225);
    }

    #[test]
    fn test_usage_calculation_consistency() {
        // Given
        let input = 100u32;
        let output = 50u32;
        
        // When
        let usage = Usage {
            input_tokens: input,
            output_tokens: output,
            total_tokens: input + output,
        };
        
        // Then
        assert_eq!(usage.total_tokens, usage.input_tokens + usage.output_tokens);
    }
}

/// Test Serialization and Deserialization
#[cfg(test)]
mod serialization_tests {
    use super::*;

    #[test]
    fn test_message_role_serialization() {
        // Given
        let roles = vec![
            (MessageRole::User, "user"),
            (MessageRole::Assistant, "assistant"),
            (MessageRole::System, "system"),
        ];
        
        for (role, expected_str) in roles {
            // When
            let serialized = serde_json::to_value(&role).unwrap();
            let deserialized: MessageRole = serde_json::from_value(serialized.clone()).unwrap();
            
            // Then
            assert_eq!(serialized, expected_str);
            assert_eq!(deserialized, role);
        }
    }

    #[test]
    fn test_claude_message_serialization() {
        // Given
        let message = ClaudeMessage {
            role: MessageRole::User,
            content: "Test message".to_string(),
        };
        
        // When
        let json = serde_json::to_value(&message).unwrap();
        let deserialized: ClaudeMessage = serde_json::from_value(json.clone()).unwrap();
        
        // Then
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "Test message");
        assert_eq!(deserialized.role, MessageRole::User);
        assert_eq!(deserialized.content, "Test message");
    }

    #[test]
    fn test_claude_request_serialization() {
        // Given
        let request = ClaudeRequest {
            messages: vec![ClaudeMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
            system_prompt: Some("Be helpful".to_string()),
            metadata: Some(json!({"test": true})),
        };
        
        // When
        let json = serde_json::to_value(&request).unwrap();
        let deserialized: ClaudeRequest = serde_json::from_value(json.clone()).unwrap();
        
        // Then
        assert_eq!(json["messages"].as_array().unwrap().len(), 1);
        assert_eq!(json["system_prompt"], "Be helpful");
        assert_eq!(json["metadata"]["test"], true);
        
        assert_eq!(deserialized.messages.len(), 1);
        assert_eq!(deserialized.system_prompt.as_ref().unwrap(), "Be helpful");
        assert_eq!(deserialized.metadata.as_ref().unwrap()["test"], true);
    }

    #[test]
    fn test_claude_response_serialization() {
        // Given
        let response = ClaudeResponse {
            content: "Hello there!".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            usage: json!({"input_tokens": 5, "output_tokens": 3}),
            metadata: Some(json!({"finish_reason": "stop"})),
        };
        
        // When
        let json = serde_json::to_value(&response).unwrap();
        let deserialized: ClaudeResponse = serde_json::from_value(json.clone()).unwrap();
        
        // Then
        assert_eq!(json["content"], "Hello there!");
        assert_eq!(json["model"], "claude-3-5-sonnet-20241022");
        assert_eq!(json["usage"]["input_tokens"], 5);
        
        assert_eq!(deserialized.content, "Hello there!");
        assert_eq!(deserialized.model, "claude-3-5-sonnet-20241022");
        assert_eq!(deserialized.usage["input_tokens"], 5);
    }

    #[test]
    fn test_conversation_context_serialization() {
        // Given
        let mut context = ConversationContext::new("test-conv".to_string());
        context.add_message(ClaudeMessage {
            role: MessageRole::User,
            content: "Test".to_string(),
        });
        context.system_prompt = Some("System".to_string());
        context.metadata = Some(json!({"priority": "high"}));
        
        // When
        let json = serde_json::to_value(&context).unwrap();
        let deserialized: ConversationContext = serde_json::from_value(json.clone()).unwrap();
        
        // Then
        assert_eq!(json["id"], "test-conv");
        assert_eq!(json["messages"].as_array().unwrap().len(), 1);
        assert_eq!(json["system_prompt"], "System");
        
        assert_eq!(deserialized.id, "test-conv");
        assert_eq!(deserialized.messages.len(), 1);
        assert_eq!(deserialized.system_prompt.as_ref().unwrap(), "System");
    }

    #[test]
    fn test_datetime_serialization_in_context() {
        // Given
        let context = ConversationContext::new("test".to_string());
        
        // When
        let json = serde_json::to_value(&context).unwrap();
        let deserialized: ConversationContext = serde_json::from_value(json.clone()).unwrap();
        
        // Then
        assert!(json["created_at"].is_string());
        assert!(json["updated_at"].is_string());
        assert_eq!(deserialized.created_at, context.created_at);
        assert_eq!(deserialized.updated_at, context.updated_at);
    }
}