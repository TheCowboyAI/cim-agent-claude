use anyhow::Result;
use async_nats::jetstream;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, error};
use uuid::Uuid;

/// Simple command structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCommand {
    pub command_id: String,
    pub correlation_id: String,
    pub prompt: String,
    pub conversation_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Simple response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub response_id: String,
    pub correlation_id: String,
    pub content: String,
    pub conversation_id: String,
    pub timestamp: DateTime<Utc>,
}

/// Simple event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeEvent {
    pub event_id: String,
    pub correlation_id: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Simple Claude NATS Adapter");
    
    // Get configuration from environment
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "localhost:4222".to_string());
    let claude_api_key = env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY environment variable is required");
    
    // Validate API key format (should start with sk-)
    if !claude_api_key.starts_with("sk-") {
        error!("Invalid Claude API key format. Should start with 'sk-'");
        error!("Your key starts with: {}", &claude_api_key[..std::cmp::min(8, claude_api_key.len())]);
        std::process::exit(1);
    }
    
    info!("Using Claude API key: {}...", &claude_api_key[..std::cmp::min(12, claude_api_key.len())]);
    
    info!("Connecting to NATS at: {}", nats_url);
    
    // Connect to NATS
    let client = async_nats::connect(&nats_url).await?;
    let jetstream = jetstream::new(client.clone());
    
    // Ensure streams exist
    setup_streams(&jetstream).await?;
    
    // Create Claude HTTP client
    let claude_client = reqwest::Client::new();
    
    // Start command consumer
    let stream = jetstream.get_stream("CLAUDE_COMMANDS").await?;
    let consumer = stream
        .get_or_create_consumer(
            "simple-claude-processor",
            jetstream::consumer::pull::Config {
                filter_subject: "claude.cmd.*".to_string(),
                ..Default::default()
            },
        )
        .await?;
    
    info!("Started consumer, listening for commands on claude.cmd.*");
    
    // Process messages
    let mut messages = consumer.messages().await?;
    
    while let Some(message) = messages.next().await {
        match message {
            Ok(msg) => {
                info!("Received message on subject: {}", msg.subject);
                
                // Parse command
                match serde_json::from_slice::<ClaudeCommand>(&msg.payload) {
                    Ok(command) => {
                        info!("Processing command: {} (correlation: {})", 
                              command.command_id, command.correlation_id);
                        
                        // Send to Claude API
                        match send_to_claude(&claude_client, &claude_api_key, &command).await {
                            Ok(response) => {
                                // Publish response event
                                if let Err(e) = publish_response_event(&jetstream, &response).await {
                                    error!("Failed to publish response event: {}", e);
                                } else {
                                    info!("Published response for correlation: {}", response.correlation_id);
                                }
                                
                                let _ = msg.ack().await;
                            }
                            Err(e) => {
                                error!("Failed to process command: {}", e);
                                let _ = msg.ack().await; // Just ack for now
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse command: {}", e);
                        let _ = msg.ack().await; // Just ack for now
                    }
                }
            }
            Err(e) => {
                error!("Error receiving message: {}", e);
            }
        }
    }
    
    Ok(())
}

async fn setup_streams(jetstream: &jetstream::Context) -> Result<()> {
    info!("Setting up NATS streams");
    
    // Commands stream
    jetstream.get_or_create_stream(jetstream::stream::Config {
        name: "CLAUDE_COMMANDS".to_string(),
        subjects: vec!["claude.cmd.*".to_string()],
        retention: jetstream::stream::RetentionPolicy::WorkQueue,
        max_messages: 1000,
        max_age: std::time::Duration::from_secs(3600), // 1 hour
        ..Default::default()
    }).await?;
    
    // Events stream
    jetstream.get_or_create_stream(jetstream::stream::Config {
        name: "CLAUDE_EVENTS".to_string(),
        subjects: vec!["claude.event.*".to_string()],
        retention: jetstream::stream::RetentionPolicy::Limits,
        max_messages: 5000,
        max_age: std::time::Duration::from_secs(24 * 3600), // 24 hours
        ..Default::default()
    }).await?;
    
    info!("NATS streams ready");
    Ok(())
}

async fn send_to_claude(
    client: &reqwest::Client,
    api_key: &str,
    command: &ClaudeCommand,
) -> Result<ClaudeResponse> {
    let conversation_id = command.conversation_id.clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Build Claude API request with proper format
    let request_body = serde_json::json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 1000,
        "messages": [
            {
                "role": "user",
                "content": command.prompt
            }
        ]
    });
    
    info!("Sending request to Claude API for correlation: {}", command.correlation_id);
    info!("Request body: {}", serde_json::to_string_pretty(&request_body).unwrap_or_default());
    
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .timeout(std::time::Duration::from_secs(30))
        .json(&request_body)
        .send()
        .await?;
    
    let status = response.status();
    info!("Claude API response status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await?;
        error!("Claude API error {}: {}", status, error_text);
        
        // Provide helpful error messages
        match status.as_u16() {
            401 => {
                error!("❌ Authentication failed - check your CLAUDE_API_KEY");
                error!("   Make sure your API key is valid and starts with 'sk-'");
            }
            429 => {
                error!("❌ Rate limit exceeded - please wait before sending more requests");
            }
            400 => {
                error!("❌ Bad request - check the message format");
            }
            500..=599 => {
                error!("❌ Claude API server error - this is usually temporary");
            }
            _ => {
                error!("❌ Unexpected error from Claude API");
            }
        }
        
        anyhow::bail!("Claude API error {}: {}", status, error_text);
    }
    
    let response_text = response.text().await?;
    info!("Claude API raw response: {}", response_text);
    
    let claude_response: serde_json::Value = serde_json::from_str(&response_text)?;
    
    // Extract content more safely
    let content = claude_response
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|text| text.as_str())
        .unwrap_or("No content found in response")
        .to_string();
    
    info!("Extracted content: {}", content);
    
    Ok(ClaudeResponse {
        response_id: Uuid::new_v4().to_string(),
        correlation_id: command.correlation_id.clone(),
        content,
        conversation_id,
        timestamp: Utc::now(),
    })
}

async fn publish_response_event(
    jetstream: &jetstream::Context,
    response: &ClaudeResponse,
) -> Result<()> {
    let event = ClaudeEvent {
        event_id: Uuid::new_v4().to_string(),
        correlation_id: response.correlation_id.clone(),
        event_type: "response_received".to_string(),
        data: serde_json::to_value(response)?,
        timestamp: Utc::now(),
    };
    
    let subject = format!("claude.event.{}.response", response.conversation_id);
    let payload = serde_json::to_vec(&event)?;
    
    jetstream.publish(subject, Bytes::from(payload)).await?;
    
    Ok(())
}