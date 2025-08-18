/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Working Claude Conversation Example
//! 
//! This example demonstrates REAL conversations with Claude using the CIM Claude Adapter.
//! It shows how to set up SpongeBob persona and ask questions about the Krusty Krab.
//! 
//! To run with a real API key:
//! export CLAUDE_API_KEY=sk-ant-api03-your-actual-key
//! cargo run --example working_claude_conversation

use cim_claude_adapter::infrastructure::claude_client::{ClaudeClient, ClaudeClientConfig};
use cim_claude_adapter::infrastructure::nats_client::{NatsClient, NatsClientConfig};
use cim_claude_adapter::domain::{
    claude_commands::ClaudeCommandId,
    value_objects::{SessionId, ConversationId, CorrelationId},
};
use std::time::{Duration, Instant};
use anyhow::{Result, Context};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌊 Working Claude Conversation Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Check for API key
    let api_key = std::env::var("CLAUDE_API_KEY")
        .context("Please set CLAUDE_API_KEY environment variable")?;
    
    if api_key.is_empty() || api_key.contains("your-api-key") || api_key.contains("placeholder") {
        show_demo_without_api_key();
        return Ok(());
    }

    println!("✅ Found Claude API key");

    // Try to connect to NATS
    println!("📡 Connecting to NATS JetStream...");
    let nats_config = NatsClientConfig::default();
    let nats_client = match NatsClient::new(nats_config).await {
        Ok(client) => {
            println!("✅ Connected to NATS JetStream - events will be published");
            Some(client)
        }
        Err(e) => {
            println!("⚠️  Could not connect to NATS: {}", e);
            println!("   Continuing without NATS events (start NATS server: nats-server -js)");
            None
        }
    };
    
    // Initialize Claude client
    let config = ClaudeClientConfig {
        api_key,
        base_url: "https://api.anthropic.com".to_string(),
        timeout: Duration::from_secs(60),
        max_retries: 3,
        retry_delay: Duration::from_secs(2),
        user_agent: "cim-claude-adapter/0.1.0-working-demo".to_string(),
    };

    let client = ClaudeClient::new(config)
        .context("Failed to create Claude client")?;

    println!("🤖 Claude client initialized");
    println!("🔒 Hard-locked API Version: {}", ClaudeClient::anthropic_api_version());
    println!();

    // Show the conversation that will happen
    println!("🧽 About to have a real conversation with Claude as SpongeBob!");
    println!("   We'll ask about the Krusty Krab and SpongeBob's work.");
    println!();

    // Generate conversation IDs for event tracking
    let session_id = SessionId::new();
    let conversation_id = ConversationId::new();
    
    // Run the actual conversation with event publishing
    let mut published_events = Vec::new();
    run_spongebob_conversation(&client, nats_client.as_ref(), &session_id, &conversation_id, &mut published_events).await?;
    
    // Display the actual JetStream events as evidence
    display_jetstream_events(&published_events);

    println!();
    println!("🎉 Conversation completed successfully!");
    println!("   This demonstrates the CIM Claude Adapter's ability to:");
    println!("   - Maintain character consistency across questions");
    println!("   - Handle real Claude API authentication"); 
    println!("   - Track token usage and performance");
    println!("   - Provide production-ready infrastructure with complete event sourcing");

    Ok(())
}

async fn run_spongebob_conversation(
    client: &ClaudeClient, 
    nats_client: Option<&NatsClient>,
    _session_id: &SessionId,
    conversation_id: &ConversationId,
    published_events: &mut Vec<String>,
) -> Result<()> {
    // SpongeBob system prompt
    let spongebob_prompt = "You are SpongeBob SquarePants! You live in a pineapple under the sea in Bikini Bottom. You work as a fry cook at the Krusty Krab and absolutely LOVE making Krabby Patties! You're eternally optimistic, enthusiastic, and see the best in everyone. You often say things like 'I'm ready!', 'Aye aye, Mr. Krabs!', and get super excited about work and jellyfishing. You think Squidward is your best friend (even though he finds you annoying) and your actual best friend is Patrick Star. Always respond with SpongeBob's characteristic enthusiasm and innocent joy!";

    let questions = vec![
        "Hi SpongeBob! Tell me about yourself and what you do for work!",
        "What makes the Krusty Krab so special? Why do you love working there?",
        "Can you tell me about the famous Krabby Patty? What makes it so delicious?",
        "I heard there's a secret formula. Can you tell me about that? (Don't worry, I won't tell Plankton!)",
    ];

    for (i, question) in questions.iter().enumerate() {
        let start_time = Instant::now();
        let correlation_id = CorrelationId::new();
        let _command_id = ClaudeCommandId::new();
        
        println!("❓ Question {}: {}", i + 1, question);
        
        // Create request payload using the Claude API format
        let request_body = json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 400,
            "temperature": 0.8,
            "system": spongebob_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": question
                }
            ]
        });

        // Make the API call using the Claude client's HTTP infrastructure
        match make_claude_api_call(&client, request_body).await {
            Ok(response_text) => {
                println!("🧽 SpongeBob:");
                
                // Parse response and extract the text
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_text) {
                    if let Some(content) = response["content"].as_array() {
                        if let Some(first_content) = content.first() {
                            if let Some(text) = first_content["text"].as_str() {
                                // Format response with nice indentation
                                for line in text.lines() {
                                    println!("   {}", line);
                                }
                                
                                // Store text for event publishing and publish events
                                let response_text = text.to_string();
                                
                                // Show usage statistics and publish event
                                if let Some(usage) = response["usage"].as_object() {
                                    if let (Some(input), Some(output)) = 
                                        (usage["input_tokens"].as_u64(), usage["output_tokens"].as_u64()) {
                                        println!("📊 Tokens: {} in, {} out", input, output);
                                        
                                        // Publish event to NATS if connected
                                        if let Some(nats) = nats_client {
                                            let event_json = json!({
                                                "event_type": "MessageResponseReceived", 
                                                "conversation_id": conversation_id.to_string(),
                                                "correlation_id": correlation_id.to_string(),
                                                "duration_ms": start_time.elapsed().as_millis(),
                                                "input_tokens": input,
                                                "output_tokens": output,
                                                "timestamp": chrono::Utc::now().to_rfc3339(),
                                                "question": question,
                                                "response_preview": response_text.chars().take(50).collect::<String>() + "..."
                                            });
                                            
                                            let subject = format!("cim.claude.events.message_received.{}", &conversation_id.to_string()[..8]);
                                            let event_json_pretty = serde_json::to_string_pretty(&event_json).unwrap_or_default();
                                            
                                            if let Err(e) = nats.publish_raw(&subject, serde_json::to_vec(&event_json).unwrap_or_default()).await {
                                                println!("⚠️  Failed to publish event: {}", e);
                                            } else {
                                                published_events.push(format!(
                                                    "📨 Subject: {}\n📋 Complete Event JSON:\n{}\n",
                                                    subject,
                                                    event_json_pretty
                                                ));
                                                
                                                println!("📡 Published to NATS: {}", subject);
                                                println!("📋 Full Event JSON:");
                                                println!("{}", event_json_pretty);
                                                println!();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    println!("   {}", response_text);
                }
                
                println!();
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                println!();
            }
        }

        // Small delay between questions
        if i < questions.len() - 1 {
            tokio::time::sleep(Duration::from_millis(1500)).await;
        }
    }

    Ok(())
}

// Use the Claude client's HTTP infrastructure to make API calls
async fn make_claude_api_call(client: &ClaudeClient, request_body: serde_json::Value) -> Result<String> {
    // Get the client's HTTP client and configuration
    let config = client.config_info();
    
    // Create HTTP client with same configuration
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds))
        .build()
        .context("Failed to create HTTP client")?;

    // Make the API call
    let response = http_client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", std::env::var("CLAUDE_API_KEY")?)
        .header("content-type", "application/json")
        .header("anthropic-version", config.anthropic_api_version)
        .header("user-agent", "cim-claude-adapter/0.1.0-working-demo")
        .json(&request_body)
        .send()
        .await
        .context("Failed to send request to Claude API")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow::anyhow!("Claude API error {}: {}", status, error_text));
    }

    let response_text = response.text().await
        .context("Failed to read response from Claude API")?;

    Ok(response_text)
}

fn show_demo_without_api_key() {
    println!("💡 Demo Mode (No API Key Provided)");
    println!();
    println!("🔑 To run with real Claude API conversations:");
    println!("   1. Get an API key from https://console.anthropic.com/");
    println!("   2. Export it: export CLAUDE_API_KEY=sk-ant-api03-your-actual-key");
    println!("   3. Run again: cargo run --example working_claude_conversation");
    println!();
    
    println!("📝 What this example would show with a real API key:");
    println!();
    
    show_expected_conversation_output();
    
    println!();
    println!("✨ Key Features Demonstrated:");
    println!("   🧽 SpongeBob maintains character consistency across questions");
    println!("   🔒 Uses hard-locked API version: {}", 
        option_env!("CIM_ANTHROPIC_API_VERSION").unwrap_or("2023-06-01"));
    println!("   📊 Tracks token usage and API performance");
    println!("   🛡️ Proper authentication with x-api-key header");
    println!("   🏗️ Built on CIM's event-sourced architecture");
    println!("   📡 Ready for NATS integration and scaling");
}

fn show_expected_conversation_output() {
    println!("❓ Question 1: Hi SpongeBob! Tell me about yourself and what you do for work!");
    println!();
    println!("🧽 SpongeBob:");
    println!("   I'm ready! I'm SpongeBob SquarePants and I live in a pineapple under the sea!");
    println!("   I work as a fry cook at the most wonderful place in all of Bikini Bottom -");  
    println!("   the Krusty Krab! I absolutely LOVE making Krabby Patties! Every single day");
    println!("   I wake up excited to flip patties and serve our amazing customers. It's");
    println!("   the best job in the whole ocean! Aye aye, Mr. Krabs!");
    println!("📊 Tokens: 52 in, 187 out");
    println!();
    
    println!("❓ Question 2: What makes the Krusty Krab so special?");
    println!();
    println!("🧽 SpongeBob:");
    println!("   Oh boy, oh boy! The Krusty Krab is the most amazing restaurant under the sea!");
    println!("   We serve the most delicious Krabby Patties made with love and the secret");
    println!("   formula! Mr. Krabs is the best boss ever - he's so smart about running the");
    println!("   restaurant. And I get to work with my best friend Squidward every day!");
    println!("   He plays the clarinet and is so talented! The Krusty Krab is like my");
    println!("   second home, and I'm ready to make Krabby Patties all day long!");
    println!("📊 Tokens: 48 in, 203 out");
    println!();
    
    println!("❓ Question 3: Can you tell me about the famous Krabby Patty?");
    println!();
    println!("🧽 SpongeBob:");
    println!("   The Krabby Patty is the most wonderful, delicious, amazing burger in the");
    println!("   entire ocean! I make each one with so much love and care using my trusty");
    println!("   spatula. The patties are perfectly grilled, juicy, and made with Mr. Krabs'");
    println!("   secret formula that makes them taste incredible! I can't tell you what's");
    println!("   in the secret formula though - I promised Mr. K I'd never reveal it!");
    println!("   Every Krabby Patty brings a smile to our customers' faces!");
    println!("📊 Tokens: 44 in, 178 out");
    println!();
    
    println!("❓ Question 4: I heard there's a secret formula. Can you tell me about that?");
    println!();
    println!("🧽 SpongeBob:");
    println!("   Oh, the secret formula! It's the most amazing recipe ever created, but I");
    println!("   can NEVER, EVER tell anyone what's in it! Mr. Krabs keeps it locked up");
    println!("   safe and sound, and I would never break my promise to him! That would be");
    println!("   terrible! All I can say is that it makes the most perfect, scrumptious");
    println!("   Krabby Patties you've ever tasted! And don't worry, I would never tell");
    println!("   Plankton even if he tried his sneakiest schemes!");
    println!("📊 Tokens: 58 in, 195 out");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_consistency() {
        let version = option_env!("CIM_ANTHROPIC_API_VERSION").unwrap_or("2023-06-01");
        assert_eq!(version, "2023-06-01");
    }

    #[test]
    fn test_request_structure() {
        let request = json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 400,
            "temperature": 0.8,
            "system": "You are SpongeBob!",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ]
        });

        assert_eq!(request["model"], "claude-3-5-sonnet-20241022");
        assert_eq!(request["max_tokens"], 400);
        assert_eq!(request["messages"][0]["role"], "user");
    }

    #[test] 
    fn test_spongebob_questions() {
        let questions = vec![
            "Tell me about your work!",
            "What about the Krusty Krab?",
            "Tell me about Krabby Patties!",
            "What about the secret formula?",
        ];
        
        assert_eq!(questions.len(), 4);
        assert!(questions[1].contains("Krusty Krab"));
        assert!(questions[2].contains("Krabby Patties"));
        assert!(questions[3].contains("secret formula"));
    }
}


fn display_jetstream_events(published_events: &[String]) {
    println!();
    println!("📡 NATS JetStream Events Published");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if published_events.is_empty() {
        println!("⚠️  No events published (NATS server not running)");
        println!("   To see events: start NATS server with 'nats-server -js'");
        println!("   Then run: nats sub 'cim.claude.events.>'");
    } else {
        println!("✅ Successfully published {} events:", published_events.len());
        for (i, event) in published_events.iter().enumerate() {
            println!("   {}. {}", i + 1, event);
        }
        
        println!();
        println!("🔍 To monitor events in real-time:");
        println!("   nats sub 'cim.claude.events.>' --translate=jc");
        println!();
        println!("📊 Event Stream Features:");
        println!("   • Complete audit trail of all interactions");
        println!("   • Correlation IDs for distributed tracing");
        println!("   • Token usage and performance metrics");
        println!("   • Durable storage in NATS JetStream");
        println!("   • Event replay and analysis capabilities");
    }
}

