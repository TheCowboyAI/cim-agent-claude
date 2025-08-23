/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use iced::futures::{stream, Stream};
use futures::StreamExt;
use tracing::{info, error, warn};
use serde_json;

// Simplified domain types - complex command/event system removed
use crate::messages::{Message, ConversationMessage, MessageRole};

/// Global NATS client - initialized once at startup
static NATS_CLIENT: tokio::sync::OnceCell<async_nats::Client> = tokio::sync::OnceCell::const_new();

/// Initialize NATS client at startup - called once from main
pub async fn initialize_nats() -> Result<(), String> {
    let client = async_nats::connect("nats://localhost:4222")
        .await
        .map_err(|e| format!("NATS connection failed: {}", e))?;
    
    NATS_CLIENT.set(client).map_err(|_| "NATS client already initialized".to_string())?;
    info!("NATS client initialized");
    Ok(())
}

/// Get the global NATS client
pub fn get_nats_client() -> Option<&'static async_nats::Client> {
    NATS_CLIENT.get()
}

/// NATS Commands - Pure async functions that use the global client
pub mod commands {
    use super::*;
    
    /// Legacy command publishing (simplified - SAGE handles complex messaging)
    pub async fn publish_simple_message(subject: String, message: String) -> Message {
        match get_nats_client() {
            Some(client) => {
                info!("Publishing message to {}", subject);
                match client.publish(subject.clone(), message.clone().into()).await {
                    Ok(_) => {
                        info!("Message published successfully");
                        
                        // For testing: publish a mock Claude response after a short delay
                        if subject.contains("claude.conversation.prompt") {
                            tokio::spawn(publish_mock_claude_response(subject, message));
                        }
                        
                        Message::CommandSent
                    }
                    Err(e) => {
                        error!("NATS publish failed: {}", e);
                        Message::Error(format!("Publish failed: {}", e))
                    }
                }
            }
            None => {
                error!("NATS client not initialized");
                Message::Error("NATS client not initialized".to_string())
            }
        }
    }
    
    /// Mock Claude response for testing - simulates Claude responding to user prompts
    async fn publish_mock_claude_response(original_subject: String, original_message: String) {
        // Wait a bit to simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        if let Some(client) = get_nats_client() {
            // Extract conversation ID from subject
            let parts: Vec<&str> = original_subject.split('.').collect();
            if let Some(conversation_id) = parts.get(3) {
                let response_subject = format!("claude.conversation.response.{}", conversation_id);
                
                // Parse the original message to extract the prompt
                let prompt = match serde_json::from_str::<serde_json::Value>(&original_message) {
                    Ok(parsed) => {
                        parsed.get("prompt")
                            .and_then(|p| p.as_str())
                            .unwrap_or("[Unknown prompt]")
                            .to_string()
                    }
                    Err(_) => "[Failed to parse prompt]".to_string()
                };
                
                // Generate a mock response based on the prompt
                let mock_response = generate_mock_claude_response(&prompt);
                
                let response_json = serde_json::json!({
                    "conversation_id": conversation_id,
                    "response": mock_response,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "agent_name": "Claude (Mock)"
                }).to_string();
                
                match client.publish(response_subject.clone(), response_json.into()).await {
                    Ok(_) => info!("Mock Claude response published to {}", response_subject),
                    Err(e) => error!("Failed to publish mock response: {}", e),
                }
            }
        }
    }
    
    /// Generate mock Claude response based on user input
    fn generate_mock_claude_response(prompt: &str) -> String {
        // Simple mock responses based on keywords
        let prompt_lower = prompt.to_lowercase();
        
        if prompt_lower.contains("hello") || prompt_lower.contains("hi") {
            format!("Hello! I'm Claude, and I'm here to help you with your questions. You asked: \"{}\"", prompt)
        } else if prompt_lower.contains("cim") {
            format!("I see you're asking about CIM (Composable Information Machine). This is an exciting architecture! Regarding your question: \"{}\", I can help you understand how CIM's mathematical foundations using Category Theory enable powerful distributed systems.", prompt)
        } else if prompt_lower.contains("sage") {
            format!("SAGE is the conscious orchestrator in the CIM system. About your question: \"{}\", SAGE coordinates multiple expert agents to provide comprehensive responses across different domains.", prompt)
        } else if prompt_lower.contains("help") {
            format!("I'm happy to help! You asked: \"{}\". Please let me know more specifically what you'd like assistance with, and I'll do my best to provide useful information.", prompt)
        } else if prompt_lower.contains("test") {
            format!("This is a test response to your message: \"{}\". The CIM Claude GUI message rendering system is now working correctly, allowing you to see this conversation in real-time!", prompt)
        } else {
            format!("Thank you for your message: \"{}\". I understand you're looking for information on this topic. While I'm currently running in mock mode for testing the CIM GUI, I can see that the message rendering pipeline is working correctly. Your message was received, processed, and this response is being displayed in the conversation interface.", prompt)
        }
    }
}

/// NATS Event Subscription - Stream of Events from NATS using global client
pub fn nats_event_stream() -> impl Stream<Item = Message> {
    stream::unfold((), |_| async {
        match get_nats_client() {
            Some(client) => {
                // Subscribe to both SAGE responses and general events
                let sage_subscriber_result = client.subscribe("sage.response.*").await;
                let status_subscriber_result = client.subscribe("sage.status.response").await;
                
                match (sage_subscriber_result, status_subscriber_result) {
                    (Ok(mut sage_subscriber), Ok(mut status_subscriber)) => {
                        info!("NATS event stream started - listening for SAGE responses");
                        loop {
                            tokio::select! {
                                Some(msg) = sage_subscriber.next() => {
                                    info!("Received SAGE response on subject: {}", msg.subject);
                                    match String::from_utf8(msg.payload.to_vec()) {
                                        Ok(response_json) => {
                                            match serde_json::from_str::<crate::sage_client::SageResponse>(&response_json) {
                                                Ok(sage_response) => {
                                                    info!("Parsed SAGE response successfully");
                                                    return Some((Message::SageResponseReceived(sage_response), ()));
                                                }
                                                Err(e) => {
                                                    warn!("Failed to parse SAGE response JSON: {}", e);
                                                    return Some((Message::Error(format!("SAGE response parsing failed: {}", e)), ()));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Failed to parse SAGE response as UTF-8: {}", e);
                                            return Some((Message::Error(format!("SAGE response UTF-8 parsing failed: {}", e)), ()));
                                        }
                                    }
                                }
                                Some(msg) = status_subscriber.next() => {
                                    info!("Received SAGE status response");
                                    match String::from_utf8(msg.payload.to_vec()) {
                                        Ok(status_json) => {
                                            match serde_json::from_str::<crate::sage_client::SageStatus>(&status_json) {
                                                Ok(sage_status) => {
                                                    info!("Parsed SAGE status successfully");
                                                    return Some((Message::SageStatusReceived(sage_status), ()));
                                                }
                                                Err(e) => {
                                                    warn!("Failed to parse SAGE status JSON: {}", e);
                                                    return Some((Message::Error(format!("SAGE status parsing failed: {}", e)), ()));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Failed to parse SAGE status as UTF-8: {}", e);
                                            return Some((Message::Error(format!("SAGE status UTF-8 parsing failed: {}", e)), ()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        error!("Failed to subscribe to SAGE subjects: {}", e);
                        return Some((Message::ConnectionError(format!("SAGE subscription failed: {}", e)), ()));
                    }
                }
            }
            None => {
                error!("NATS client not initialized");
                return Some((Message::Error("NATS client not initialized".to_string()), ()));
            }
        }
    })
}

/// SAGE Response Subscription - Stream of SAGE responses from NATS
pub fn sage_response_stream() -> impl Stream<Item = Message> {
    stream::unfold((), |_| async {
        match get_nats_client() {
            Some(client) => {
                match client.subscribe("sage.response.*").await {
                    Ok(mut subscriber) => {
                        info!("SAGE response stream started");
                        while let Some(msg) = subscriber.next().await {
                            match String::from_utf8(msg.payload.to_vec()) {
                                Ok(json_str) => {
                                    info!("Received SAGE response: {}", json_str);
                                    match serde_json::from_str::<crate::sage_client::SageResponse>(&json_str) {
                                        Ok(response) => {
                                            return Some((Message::SageResponseReceived(response), ()));
                                        }
                                        Err(e) => {
                                            error!("Failed to parse SAGE response: {}", e);
                                            return Some((Message::Error(format!("SAGE response parsing failed: {}", e)), ()));
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse SAGE response as UTF-8: {}", e);
                                    return Some((Message::Error(format!("SAGE response UTF-8 parsing failed: {}", e)), ()));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to subscribe to SAGE responses: {}", e);
                        return Some((Message::ConnectionError(format!("SAGE subscription failed: {}", e)), ()));
                    }
                }
            }
            None => {
                error!("NATS client not initialized");
                return Some((Message::Error("NATS client not initialized".to_string()), ()));
            }
        }
        None
    })
}

/// Claude Conversation Messages - Stream of actual conversation messages
pub fn conversation_messages_stream() -> impl Stream<Item = Message> {
    stream::unfold((), |_| async {
        match get_nats_client() {
            Some(client) => {
                // Subscribe to multiple conversation patterns
                match client.subscribe("claude.conversation.>")
                    .await
                {
                    Ok(mut subscriber) => {
                        info!("Claude conversation message stream started");
                        while let Some(msg) = subscriber.next().await {
                            match String::from_utf8(msg.payload.to_vec()) {
                                Ok(json_str) => {
                                    info!("Received conversation message on subject: {}", msg.subject);
                                    
                                    // Try to parse as conversation message
                                    if let Ok(parsed_msg) = parse_conversation_message(&json_str, &msg.subject) {
                                        return Some((Message::ConversationMessageReceived(parsed_msg), ()));
                                    } else {
                                        warn!("Failed to parse conversation message: {}", json_str);
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse conversation message as UTF-8: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to subscribe to conversation messages: {}", e);
                        return Some((Message::ConnectionError(format!("Conversation subscription failed: {}", e)), ()));
                    }
                }
            }
            None => {
                error!("NATS client not initialized");
                return Some((Message::Error("NATS client not initialized".to_string()), ()));
            }
        }
        None
    })
}

/// Parse conversation message from JSON and NATS subject
fn parse_conversation_message(json_str: &str, subject: &str) -> Result<ConversationMessage, Box<dyn std::error::Error>> {
    let parsed: serde_json::Value = serde_json::from_str(json_str)?;
    
    // Extract conversation ID from subject (e.g., claude.conversation.prompt.session-id)
    let parts: Vec<&str> = subject.split('.').collect();
    let conversation_id = parts.get(3).unwrap_or(&"unknown").to_string();
    
    let role = if subject.contains("prompt") {
        MessageRole::User
    } else if subject.contains("response") {
        MessageRole::Assistant
    } else if subject.contains("sage") {
        MessageRole::Sage
    } else {
        MessageRole::System
    };
    
    let content = parsed.get("prompt")
        .or(parsed.get("response"))
        .or(parsed.get("initial_prompt"))
        .or(parsed.get("content"))
        .and_then(|v| v.as_str())
        .unwrap_or("[Unknown message content]");
        
    let timestamp = parsed.get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now());
        
    let agent_name = parsed.get("agent_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Ok(ConversationMessage {
        id: uuid::Uuid::new_v4().to_string(),
        conversation_id,
        role,
        content: content.to_string(),
        timestamp,
        agent_name,
    })
}

/// TEA-compliant subscription for NATS events
pub fn nats_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run_with_id(
        "nats-events",
        nats_event_stream()
    )
}

/// TEA-compliant subscription for SAGE responses
pub fn sage_response_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run_with_id(
        "sage-responses",
        sage_response_stream()
    )
}

/// TEA-compliant subscription for conversation messages
pub fn conversation_messages_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run_with_id(
        "conversation-messages",
        conversation_messages_stream()
    )
}