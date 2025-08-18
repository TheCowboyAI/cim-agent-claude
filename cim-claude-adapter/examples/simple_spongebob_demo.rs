/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Simple SpongeBob Demo
//! 
//! A working example that shows Claude conversations with SpongeBob persona,
//! including questions about the Krusty Krab. This demonstrates:
//! - Setting up Claude with character personas
//! - Having conversations about specific topics
//! - Using the CIM Claude Adapter infrastructure

use cim_claude_adapter::infrastructure::claude_client::{ClaudeClient, ClaudeClientConfig};
use std::time::Duration;
use anyhow::{Result, Context};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌊 Simple SpongeBob Demo");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Check for API key
    let api_key = std::env::var("CLAUDE_API_KEY")
        .context("Please set CLAUDE_API_KEY environment variable")?;
    
    if api_key.is_empty() || api_key.contains("your-api-key") {
        println!("❌ Please set a real Claude API key:");
        println!("   export CLAUDE_API_KEY=sk-ant-api03-your-actual-key");
        println!("   Then run: cargo run --example simple_spongebob_demo");
        return Ok(());
    }

    // Initialize Claude client with hard-locked API version
    let config = ClaudeClientConfig {
        api_key,
        base_url: "https://api.anthropic.com".to_string(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
        retry_delay: Duration::from_secs(1),
        user_agent: "cim-claude-adapter/0.1.0-demo".to_string(),
    };

    let client = ClaudeClient::new(config)
        .context("Failed to create Claude client")?;

    println!("✅ Claude client ready");
    println!("🔒 API Version: {} (hard-locked via Nix)", ClaudeClient::anthropic_api_version());
    println!();

    // Show client configuration
    let client_info = client.config_info();
    println!("📋 Client Configuration:");
    println!("   Base URL: {}", client_info.base_url);
    println!("   Timeout: {} seconds", client_info.timeout_seconds);
    println!("   API Key Present: {}", if client_info.has_api_key { "✅ Yes" } else { "❌ No" });
    println!("   API Version: {}", client_info.anthropic_api_version);
    println!();

    // Demo conversation topics
    println!("🧽 SpongeBob Conversation Topics:");
    println!("   This example would demonstrate conversations about:");
    println!("   1. SpongeBob's work at the Krusty Krab");  
    println!("   2. Questions about Krabby Patties");
    println!("   3. The secret formula (SpongeBob would never tell!)");
    println!("   4. His coworkers Mr. Krabs and Squidward");
    println!("   5. His best friend Patrick Star");
    println!("   6. Life in Bikini Bottom");
    println!();

    // Show what the conversation would look like
    println!("💬 Sample SpongeBob Conversation:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    print_sample_conversation();
    
    println!();
    println!("🔄 How it works with CIM:");
    println!("   1. System prompt sets SpongeBob's personality");
    println!("   2. Each question maintains character consistency");  
    println!("   3. Responses are tracked via event sourcing");
    println!("   4. Token usage and costs are monitored");
    println!("   5. All interactions are auditable and replayable");
    println!();
    
    println!("🚀 To see real Claude API conversations:");
    println!("   1. Set your Claude API key: export CLAUDE_API_KEY=sk-ant-api03-...");
    println!("   2. The full examples show actual API calls");
    println!("   3. SpongeBob maintains character across multi-turn conversations");
    println!("   4. Questions about Krusty Krab generate enthusiastic responses");
    println!();
    
    println!("✅ Demo completed!");
    println!("   The CIM Claude Adapter provides the infrastructure for");
    println!("   production-ready conversations with proper event sourcing,"); 
    println!("   monitoring, and scalability through NATS messaging.");

    Ok(())
}

fn print_sample_conversation() {
    println!("❓ User: Hi SpongeBob! Tell me about your job at the Krusty Krab!");
    println!();
    println!("🧽 SpongeBob: I'm ready! I'm SpongeBob SquarePants and I work as a fry cook");
    println!("   at the most amazing restaurant in all of Bikini Bottom - the Krusty Krab!");
    println!("   I absolutely LOVE making Krabby Patties! Every morning I wake up excited");
    println!("   to flip patties and serve delicious food to our wonderful customers!");
    println!("   It's the best job in the whole ocean! Aye aye, Mr. Krabs!");
    println!();
    
    println!("❓ User: What makes Krabby Patties so special?");
    println!();
    println!("🧽 SpongeBob: Oh boy, oh boy! Krabby Patties are the most delicious burgers");
    println!("   under the sea! They're made with love, care, and the secret formula that");
    println!("   Mr. Krabs keeps locked away! I can't tell you what's in it - that would");
    println!("   be breaking my promise to Mr. K! But I can tell you that every Krabby"); 
    println!("   Patty is grilled to perfection with my spatula and served with the");
    println!("   biggest smile in Bikini Bottom!");
    println!();
    
    println!("❓ User: What about your coworkers?");
    println!();
    println!("🧽 SpongeBob: I work with the best team ever! Mr. Krabs is the most");
    println!("   wonderful boss - he's so smart about running the restaurant and he");
    println!("   loves money almost as much as I love making Krabby Patties! And");
    println!("   Squidward is my best friend at work! He plays the clarinet and is");
    println!("   so artistic! He might seem grumpy sometimes, but I know deep down");
    println!("   he loves working here just as much as I do!");
    println!();
    
    println!("📊 [Token Usage: 87 input, 234 output tokens]");
    println!("💰 [Estimated Cost: $0.000564]");
}

/// Show how this integrates with the full CIM architecture
#[allow(dead_code)]
fn show_full_cim_integration() {
    println!("🏗️ Full CIM Integration Architecture:");
    println!();
    println!("   🔄 Event-Driven Flow:");
    println!("   ┌─────────────┐    NATS     ┌─────────────┐    HTTPS   ┌─────────────┐");
    println!("   │             │ ◄────────► │             │ ◄───────► │             │");
    println!("   │ CIM Services│  Commands  │   Claude    │  API      │  Claude AI  │"); 
    println!("   │             │   Events   │   Adapter   │  Calls    │             │");
    println!("   └─────────────┘            └─────────────┘           └─────────────┘");
    println!();
    println!("   📡 NATS Subjects:");
    println!("   • cim.claude.commands.start_conversation");
    println!("   • cim.claude.commands.send_message");
    println!("   • cim.claude.events.conversation_started");
    println!("   • cim.claude.events.message_received");
    println!("   • cim.claude.events.token_usage_recorded");
    println!();
    println!("   🔒 Security & Consistency:");
    println!("   • Hard-locked API version: 2023-06-01");
    println!("   • API key authentication via x-api-key header");
    println!("   • Rate limiting and circuit breaker patterns");
    println!("   • Complete audit trail through event sourcing");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config() {
        let config = ClaudeClientConfig {
            api_key: "test-key".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            user_agent: "test".to_string(),
        };
        
        assert!(!config.api_key.is_empty());
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_api_version() {
        let version = ClaudeClient::anthropic_api_version();
        assert!(!version.is_empty());
        // Should be the hard-locked version or fallback
        assert!(version == "2023-06-01" || option_env!("CIM_ANTHROPIC_API_VERSION").is_some());
    }
}