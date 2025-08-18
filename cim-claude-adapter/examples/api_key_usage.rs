/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Example showing how to configure Claude API key authentication
//! 
//! This example demonstrates the proper way to set up the Claude API client
//! with authentication using the x-api-key header format required by Anthropic.

use cim_claude_adapter::infrastructure::claude_client::{ClaudeClient, ClaudeClientConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing_subscriber::init(); // Optional - for debugging

    // Method 1: Use environment variable (recommended)
    println!("=== Method 1: Environment Variable ===");
    std::env::set_var("CLAUDE_API_KEY", "sk-ant-api03-example-key-here");
    
    let config_from_env = ClaudeClientConfig::default();
    println!("API key loaded from environment: {}", 
        if config_from_env.api_key.is_empty() { "❌ MISSING" } else { "✅ LOADED" });

    // Method 2: Direct configuration
    println!("\n=== Method 2: Direct Configuration ===");
    let config_direct = ClaudeClientConfig {
        api_key: "sk-ant-api03-your-actual-key-here".to_string(),
        base_url: "https://api.anthropic.com".to_string(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
        retry_delay: Duration::from_secs(1),
        user_agent: "cim-claude-adapter/0.1.0".to_string(),
    };

    // Create Claude client with authentication
    match ClaudeClient::new(config_direct) {
        Ok(client) => {
            println!("✅ Claude client created successfully!");
            println!("🔑 Authentication configured with x-api-key header");
            println!("🌐 Base URL: https://api.anthropic.com");
            println!("📅 API Version: 2023-06-01");
            
            // Show client info
            let info = client.config_info();
            println!("\n📊 Client Configuration:");
            println!("  - Base URL: {}", info.base_url);
            println!("  - Timeout: {} seconds", info.timeout_seconds);
            println!("  - Max Retries: {}", info.max_retries);
            println!("  - API Key Present: {}", if info.has_api_key { "✅ Yes" } else { "❌ No" });
            println!("  - API Version: {} ({})", info.anthropic_api_version, 
                if option_env!("CIM_ANTHROPIC_API_VERSION").is_some() { "Nix-locked" } else { "fallback" });
            
            // Example API request structure (without actually calling)
            println!("\n🔄 Example Request Headers:");
            println!("  x-api-key: [your-api-key]");
            println!("  content-type: application/json");
            println!("  anthropic-version: 2023-06-01");
            println!("  user-agent: cim-claude-adapter/0.1.0");
            
        }
        Err(e) => {
            println!("❌ Failed to create Claude client: {}", e);
            return Err(e.into());
        }
    }

    // Method 3: Integration with Adapter Service
    println!("\n=== Method 3: Full Integration Example ===");
    println!("To use with the Claude Adapter Service:");
    println!();
    println!("1. Set environment variable:");
    println!("   export CLAUDE_API_KEY=sk-ant-api03-your-actual-key");
    println!();
    println!("2. Create adapter service:");
    println!("   let claude_config = ClaudeClientConfig::default();");
    println!("   let claude_client = ClaudeClient::new(claude_config)?;");
    println!("   let adapter_service = ClaudeAdapterService::new(nats_client, claude_client);");
    println!();
    println!("3. Start the service:");
    println!("   let handles = adapter_service.start().await?;");
    println!();
    println!("✅ The service will now:");
    println!("   - Receive commands via NATS");
    println!("   - Call Claude API using x-api-key authentication");
    println!("   - Publish events back to NATS");

    Ok(())
}

/// Security best practices for API key management
fn _print_security_notes() {
    println!("\n🔒 Security Best Practices:");
    println!("1. Never hardcode API keys in source code");
    println!("2. Use environment variables or secure configuration");
    println!("3. Rotate API keys regularly");
    println!("4. Monitor API usage and costs");
    println!("5. Use least-privilege access principles");
    println!("6. Store keys in encrypted configuration or secrets management");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_configuration() {
        // Test that API key is required
        let config = ClaudeClientConfig {
            api_key: "".to_string(),
            ..Default::default()
        };
        
        let result = ClaudeClient::new(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Claude API key is required"));
    }

    #[test]
    fn test_valid_api_key_configuration() {
        let config = ClaudeClientConfig {
            api_key: "sk-ant-api03-test-key".to_string(),
            ..Default::default()
        };
        
        let result = ClaudeClient::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_environment_variable_loading() {
        std::env::set_var("CLAUDE_API_KEY", "test-key-from-env");
        
        let config = ClaudeClientConfig::default();
        assert_eq!(config.api_key, "test-key-from-env");
        
        std::env::remove_var("CLAUDE_API_KEY");
    }
}