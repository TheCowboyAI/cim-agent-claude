/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Simple Claude API Client Example
//!
//! Demonstrates the pure Claude adapter functionality without any CIM infrastructure.
//! Links to User Story 2.1: "Send Message to Claude"

use std::env;
use cim_claude_adapter::{ClaudeClient, ClaudeConfig, ClaudeRequest, ClaudeMessage, MessageRole};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get Claude API key from environment
    let api_key = env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY environment variable must be set");

    println!("🔧 Creating Claude client configuration...");
    
    // Create client configuration
    let config = ClaudeConfig {
        api_key,
        base_url: "https://api.anthropic.com".to_string(),
        model: "claude-3-5-sonnet-20241022".to_string(),
        max_tokens: 1024,
        temperature: 0.7,
        timeout: std::time::Duration::from_secs(30),
    };

    // Create the pure Claude API client
    let client = ClaudeClient::new(config)?;
    println!("✅ Claude client created successfully");

    // Test health check
    println!("\n🏥 Performing health check...");
    match client.health_check().await {
        Ok(true) => println!("✅ Health check passed - Claude API is accessible"),
        Ok(false) => println!("❌ Health check failed - unexpected response"),
        Err(e) => {
            println!("❌ Health check failed: {}", e);
            return Err(e.into());
        }
    }

    // Create a simple request
    println!("\n💬 Sending message to Claude...");
    let request = ClaudeRequest {
        messages: vec![ClaudeMessage {
            role: MessageRole::User,
            content: "Hello! Can you tell me what you are in one sentence?".to_string(),
        }],
        system_prompt: Some("You are Claude, a helpful AI assistant created by Anthropic. Be concise.".to_string()),
        metadata: None,
    };

    // Send the message
    match client.send_message(request).await {
        Ok(response) => {
            println!("✅ Received response from Claude:");
            println!("📝 Content: {}", response.content);
            println!("🤖 Model: {}", response.model);
            println!("📊 Usage: {}", serde_json::to_string_pretty(&response.usage)?);
            
            if let Some(metadata) = response.metadata {
                println!("🔍 Metadata: {}", serde_json::to_string_pretty(&metadata)?);
            }
        }
        Err(e) => {
            println!("❌ Failed to send message: {}", e);
            println!("🔄 Error category: {}", e.category());
            println!("🔁 Is retryable: {}", e.is_retryable());
            return Err(e.into());
        }
    }

    println!("\n🎉 Claude adapter example completed successfully!");
    Ok(())
}