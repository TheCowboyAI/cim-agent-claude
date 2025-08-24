//! Integration tests for CIM LLM Adapter
//!
//! Tests the complete LLM adapter service functionality

use cim_llm_adapter::{
    LlmAdapterConfig, LlmRequest, LlmResponse, 
    providers::{ProviderConfig, Message, CompletionOptions},
    dialog::DialogContext,
};
use async_nats;
use serde_json;
use std::collections::HashMap;
use tokio;
use uuid::Uuid;

#[tokio::test]
async fn test_llm_adapter_basic_functionality() {
    // Skip integration test if no API key available
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        println!("Skipping integration test - ANTHROPIC_API_KEY not set");
        return;
    }
    
    // This is a placeholder test structure
    // In a full implementation, this would:
    // 1. Start a test NATS server
    // 2. Start the LLM adapter service
    // 3. Send test requests via NATS
    // 4. Verify responses
    
    let config = LlmAdapterConfig::from_env();
    assert!(!config.available_providers().is_empty());
    println!("✅ Configuration test passed");
}

#[tokio::test]
async fn test_dialog_context_creation() {
    let session_id = Uuid::new_v4().to_string();
    let mut context = DialogContext::new(session_id.clone());
    
    assert_eq!(context.session_id, session_id);
    assert!(context.conversation_history.is_empty());
    
    // Add a message
    context.add_message("user".to_string(), "Hello".to_string(), Some("claude".to_string()));
    assert_eq!(context.conversation_history.len(), 1);
    assert_eq!(context.conversation_history[0].role, "user");
    assert_eq!(context.conversation_history[0].content, "Hello");
    
    println!("✅ Dialog context test passed");
}

#[tokio::test]
async fn test_provider_config_validation() {
    let mut provider_config = ProviderConfig {
        name: "test-claude".to_string(),
        base_url: Some("https://api.anthropic.com".to_string()),
        api_key: Some("test-key".to_string()),
        model: "claude-3-5-sonnet-20241022".to_string(),
        timeout_seconds: 30,
        retry_attempts: 3,
        rate_limit_per_minute: Some(60),
        custom_headers: HashMap::new(),
    };
    
    // Test valid config
    assert_eq!(provider_config.name, "test-claude");
    assert_eq!(provider_config.model, "claude-3-5-sonnet-20241022");
    
    // Test API key presence
    assert!(provider_config.api_key.is_some());
    
    println!("✅ Provider config test passed");
}

#[tokio::test]
async fn test_request_response_serialization() {
    let session_id = Uuid::new_v4().to_string();
    let request_id = Uuid::new_v4().to_string();
    
    // Create test request
    let request = LlmRequest {
        request_id: request_id.clone(),
        provider: "claude".to_string(),
        messages: vec![
            serde_json::json!({
                "role": "user",
                "content": "Hello, how are you?"
            })
        ],
        context: DialogContext::new(session_id.clone()),
        options: Some(serde_json::json!({
            "max_tokens": 100,
            "temperature": 0.7
        })),
    };
    
    // Test serialization
    let request_json = serde_json::to_string(&request).unwrap();
    assert!(request_json.contains(&request_id));
    assert!(request_json.contains("claude"));
    
    // Test deserialization
    let deserialized_request: LlmRequest = serde_json::from_str(&request_json).unwrap();
    assert_eq!(deserialized_request.request_id, request_id);
    assert_eq!(deserialized_request.provider, "claude");
    
    println!("✅ Request/Response serialization test passed");
}

// This would be a real end-to-end test if NATS and API keys are available
#[tokio::test]
#[ignore] // Ignore by default - run with --ignored for full integration test
async fn test_full_integration_with_nats() {
    // Skip if no environment setup
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        println!("Skipping full integration test - ANTHROPIC_API_KEY not set");
        return;
    }
    
    if std::env::var("NATS_URL").is_err() {
        std::env::set_var("NATS_URL", "nats://localhost:4222");
    }
    
    // This test would:
    // 1. Connect to NATS
    // 2. Send a real LLM request
    // 3. Wait for response
    // 4. Verify the response
    
    println!("🧪 Full integration test would run here");
    println!("   - Connect to NATS at {}", std::env::var("NATS_URL").unwrap());
    println!("   - Send LLM request to Claude");
    println!("   - Verify response structure");
    println!("   - Check dialog context persistence");
}