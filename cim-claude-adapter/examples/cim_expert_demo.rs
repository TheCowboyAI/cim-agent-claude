/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Expert Demonstration
//! 
//! This example demonstrates a CIM Expert agent that can answer comprehensive questions
//! about what a CIM (Composable Information Machine) is and what it does.
//! Shows real Claude API conversations with NATS event publishing.

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
    println!("🤖 CIM Expert - Comprehensive CIM Architecture Assistant");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Check for API key
    let api_key = std::env::var("CLAUDE_API_KEY")
        .context("Please set CLAUDE_API_KEY environment variable")?;
    
    if api_key.is_empty() || api_key.contains("your-api-key") || api_key.contains("placeholder") {
        show_demo_without_api_key();
        return Ok(());
    }

    println!("✅ Found Claude API key");

    // Connect to NATS
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
    
    // Initialize Claude client as CIM Expert
    let config = ClaudeClientConfig {
        api_key,
        base_url: "https://api.anthropic.com".to_string(),
        timeout: Duration::from_secs(60),
        max_retries: 3,
        retry_delay: Duration::from_secs(2),
        user_agent: "cim-expert-agent/0.1.0".to_string(),
    };

    let client = ClaudeClient::new(config)
        .context("Failed to create Claude client")?;

    println!("🧠 CIM Expert agent initialized");
    println!("🔒 Hard-locked API Version: {}", ClaudeClient::anthropic_api_version());
    println!();

    // Generate conversation IDs
    let session_id = SessionId::new();
    let conversation_id = ConversationId::new();
    
    println!("🤖 Ready to answer questions about CIM architecture!");
    println!("   Ask me anything about what a CIM is and what it does.");
    println!();

    // Run the CIM expert conversation
    let mut published_events = Vec::new();
    run_cim_expert_session(&client, nats_client.as_ref(), &session_id, &conversation_id, &mut published_events).await?;
    
    // Display the actual JetStream events as evidence
    display_jetstream_events(&published_events);

    println!();
    println!("🎉 CIM Expert session completed successfully!");
    println!("   This demonstrates the CIM Expert's ability to:");
    println!("   - Explain CIM architecture comprehensively");
    println!("   - Reference mathematical foundations");
    println!("   - Provide practical implementation guidance");
    println!("   - Maintain context across complex technical questions");

    Ok(())
}

async fn run_cim_expert_session(
    client: &ClaudeClient, 
    nats_client: Option<&NatsClient>,
    _session_id: &SessionId,
    conversation_id: &ConversationId,
    published_events: &mut Vec<String>,
) -> Result<()> {
    // CIM Expert system prompt - comprehensive CIM architecture knowledge
    let cim_expert_prompt = r#"
You are a CIM (Composable Information Machine) Expert with deep knowledge of the mathematical foundations and architectural patterns of distributed systems. You specialize in:

**Mathematical Foundations:**
- Category Theory: Domains as Categories, Objects as Entities, Arrows as Systems
- Graph Theory: Nodes and Edges, traversal algorithms, distributed graph operations  
- Content-Addressed Storage (IPLD): CIDs, Merkle DAGs, deduplication, referential integrity
- Structure-Preserving Propagation: How mathematical properties maintain across boundaries

**CIM Architecture:**
- Domain-Driven Design with mathematical rigor
- Event Sourcing with sequential events and CID references
- CQRS Patterns with write models and future read model projections
- NATS JetStream: Subject algebra, stream patterns, command/subscribe flows
- Object Store: Smart file system analogies, automatic deduplication, claims-based security

**Core CIM Principles:**
1. CIMs are distributed systems where a client runs NATS locally and communicates with Leaf Nodes
2. We ASSEMBLE existing cim-* modules rather than creating everything from scratch
3. Each CIM targets ONE specific business domain (mortgage, manufacturing, healthcare, etc.)
4. NATS-First Architecture with subject-based routing and pub-sub patterns
5. Event-driven with no CRUD operations - everything is immutable events

**Communication Style:**
- Use familiar technology analogies (like network file systems) to explain complex concepts
- Provide both mathematical rigor and practical examples
- Include the "why" behind CIM design decisions
- Break down abstract mathematical concepts into understandable terms
- Be comprehensive but accessible

You help users understand what a CIM is, how it works, and why it's designed this way.
"#;

    let questions = vec![
        "What is a CIM (Composable Information Machine) and how does it work?",
        "What are the key architectural components of a CIM system?",
        "How does the NATS-first architecture work in CIM systems?",
        "What role do mathematical foundations like Category Theory play in CIM design?",
        "How do you actually build and deploy a CIM for a specific business domain?",
    ];

    for (i, question) in questions.iter().enumerate() {
        let start_time = Instant::now();
        let correlation_id = CorrelationId::new();
        let _command_id = ClaudeCommandId::new();
        
        println!("❓ Question {}: {}", i + 1, question);
        println!();

        // Create request payload using the Claude API format
        let request_body = json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 800,
            "temperature": 0.3,
            "system": cim_expert_prompt,
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
                println!("🧠 CIM Expert:");
                println!();
                
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
                                let response_text_content = text.to_string();
                                
                                // Show usage statistics and publish event
                                if let Some(usage) = response["usage"].as_object() {
                                    if let (Some(input), Some(output)) = 
                                        (usage["input_tokens"].as_u64(), usage["output_tokens"].as_u64()) {
                                        println!();
                                        println!("📊 Tokens: {} in, {} out | Duration: {}ms", input, output, start_time.elapsed().as_millis());
                                        
                                        // Publish event to NATS if connected
                                        if let Some(nats) = nats_client {
                                            let event_json = json!({
                                                "event_type": "CimExpertResponse", 
                                                "conversation_id": conversation_id.to_string(),
                                                "correlation_id": correlation_id.to_string(),
                                                "duration_ms": start_time.elapsed().as_millis(),
                                                "input_tokens": input,
                                                "output_tokens": output,
                                                "timestamp": chrono::Utc::now().to_rfc3339(),
                                                "question": question,
                                                "question_topic": get_question_topic(i),
                                                "response_preview": response_text_content.chars().take(100).collect::<String>() + "...",
                                                "expert_type": "cim_architecture"
                                            });
                                            
                                            let subject = format!("cim.expert.responses.architecture.{}", &conversation_id.to_string()[..8]);
                                            
                                            if let Err(e) = nats.publish_raw(&subject, serde_json::to_vec(&event_json).unwrap_or_default()).await {
                                                println!("⚠️  Failed to publish event: {}", e);
                                            } else {
                                                published_events.push(format!(
                                                    "📨 Subject: {}\n📋 CIM Expert Event:\n{}\n",
                                                    subject,
                                                    serde_json::to_string_pretty(&event_json).unwrap_or_default()
                                                ));
                                                
                                                println!("📡 Published to NATS: {}", subject);
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
                println!("─────────────────────────────────────────────────────────────────");
                println!();
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                println!();
            }
        }

        // Small delay between questions
        if i < questions.len() - 1 {
            tokio::time::sleep(Duration::from_millis(2000)).await;
        }
    }

    Ok(())
}

fn get_question_topic(index: usize) -> &'static str {
    match index {
        0 => "cim_overview",
        1 => "architectural_components", 
        2 => "nats_architecture",
        3 => "mathematical_foundations",
        4 => "practical_implementation",
        _ => "general_cim_question"
    }
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
        .header("user-agent", "cim-expert-agent/0.1.0")
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
    println!("🔑 To run with real CIM Expert conversations:");
    println!("   1. Get an API key from https://console.anthropic.com/");
    println!("   2. Export it: export CLAUDE_API_KEY=sk-ant-api03-your-actual-key");
    println!("   3. Run again: cargo run --example cim_expert_demo");
    println!();
    
    println!("📝 What this CIM Expert would demonstrate:");
    println!();
    
    show_expected_cim_expert_output();
    
    println!();
    println!("✨ Key CIM Expert Features:");
    println!("   🧠 Comprehensive CIM architecture explanations");
    println!("   📐 Mathematical foundations (Category Theory, Graph Theory)");
    println!("   🏗️ Practical implementation guidance");
    println!("   📡 NATS-first architecture expertise");
    println!("   🔗 Event-sourced system design");
    println!("   📊 Real-time expert consultation with audit trails");
}

fn show_expected_cim_expert_output() {
    println!("❓ Question 1: What is a CIM (Composable Information Machine) and how does it work?");
    println!();
    println!("🧠 CIM Expert:");
    println!("   A CIM (Composable Information Machine) is a distributed system architecture that");
    println!("   combines mathematical rigor with practical software engineering. Think of it as a");
    println!("   'smart network file system' where information flows through mathematically-defined");
    println!("   channels rather than traditional request-response patterns...");
    println!("📊 Tokens: 245 in, 687 out");
    println!();
    
    println!("❓ Question 2: What are the key architectural components of a CIM system?");
    println!();
    println!("🧠 CIM Expert:");
    println!("   CIM architecture follows a hierarchical pattern: Client → Leaf Node → Cluster →");
    println!("   Super-cluster. Each level provides specific capabilities while maintaining the");
    println!("   mathematical properties of the system. The core components include...");
    println!("📊 Tokens: 198 in, 542 out");
}

fn display_jetstream_events(published_events: &[String]) {
    println!();
    println!("📡 NATS JetStream Events Published by CIM Expert");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if published_events.is_empty() {
        println!("⚠️  No events published (NATS server not running)");
        println!("   To see events: start NATS server with 'nats-server -js'");
        println!("   Then run: nats sub 'cim.expert.responses.>' --raw");
    } else {
        println!("✅ Successfully published {} CIM Expert events:", published_events.len());
        for (i, event) in published_events.iter().enumerate() {
            println!("{}. {}", i + 1, event);
        }
        
        println!();
        println!("🔍 To monitor CIM Expert events in real-time:");
        println!("   nats sub 'cim.expert.responses.>' --raw");
        println!();
        println!("📊 CIM Expert Event Features:");
        println!("   • Complete architectural consultation audit trail");
        println!("   • Question topic classification for analysis");
        println!("   • Expert response correlation and tracing");
        println!("   • Performance metrics for complex explanations");
        println!("   • Structured events for knowledge management");
    }
}