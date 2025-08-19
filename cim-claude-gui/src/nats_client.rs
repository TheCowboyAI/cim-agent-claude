/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use futures::StreamExt;
use iced::futures::stream::{self, BoxStream};

use tracing::{info, error};

use cim_claude_adapter::{
    domain::{commands::{Command as DomainCommand, CommandEnvelope}, events::EventEnvelope},
};
use crate::messages::Message;

/// Simple NATS component for publishing commands
/// No persistent connections or complex state management
pub struct NatsComponent;

impl NatsComponent {
    /// Publish a command to NATS using the CLI tool
    pub async fn publish_command(command_envelope: CommandEnvelope) -> Result<(), String> {
        let command_type = match &command_envelope.command {
            DomainCommand::StartConversation { .. } => "start_conversation",
            DomainCommand::SendPrompt { .. } => "send_prompt", 
            DomainCommand::EndConversation { .. } => "end_conversation",
        };
        let subject = format!("cim.claude.command.{}", command_type);
        
        let payload = serde_json::to_string(&command_envelope)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        
        info!("Publishing command to {}", subject);
        
        // Simple, reliable NATS CLI execution
        let output = std::process::Command::new("nats")
            .arg("pub")
            .arg(&subject)
            .arg(&payload)
            .output()
            .map_err(|e| format!("Failed to execute nats command: {}", e))?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            info!("Successfully published command: {}", stdout.trim());
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("NATS publish failed: {}", stderr);
            Err(format!("NATS publish failed: {}", stderr.trim()))
        }
    }
}

/// NATS subscription for events - proper Iced subscription pattern
pub fn events_subscription() -> iced::Subscription<Message> {
    iced::subscription::unfold(
        "nats-events", 
        (), 
        |_state| async {
            // Connect to NATS and listen for events
            match async_nats::connect("nats://localhost:4222").await {
                Ok(client) => {
                    info!("Connected to NATS for event subscription");
                    
                    // Subscribe to events
                    match client.subscribe("claude.event.*").await {
                        Ok(mut subscription) => {
                            info!("Subscribed to claude.event.*");
                            
                            while let Some(message) = subscription.next().await {
                                match serde_json::from_slice::<EventEnvelope>(&message.payload) {
                                    Ok(event_envelope) => {
                                        return (Message::ConversationEvent(event_envelope), ());
                                    }
                                    Err(e) => {
                                        error!("Failed to deserialize event: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to subscribe to events: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to NATS: {}", e);
                }
            }
            
            // If we get here, something went wrong, wait and retry
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            (Message::ConnectionError("NATS connection lost".to_string()), ())
        }
    )
}

/// Legacy client struct for backward compatibility - now just a placeholder
#[derive(Clone, Debug, Default)]
pub struct GuiNatsClient;

impl GuiNatsClient {
    pub fn new() -> Self {
        Self
    }
    
    /// Returns an empty stream - connections now handled by subscription
    pub fn connect(&mut self, _nats_url: String) -> BoxStream<'static, Message> {
        Box::pin(stream::once(async { Message::Connected }))
    }
    
    /// Deprecated - use NatsComponent::publish_command with Command::perform
    pub async fn send_command(&self, _command_envelope: CommandEnvelope) -> Result<(), String> {
        Err("Use NatsComponent::publish_command with Command::perform".to_string())
    }
    
    // Legacy methods for compatibility
    pub async fn load_conversation(&self, _conversation_id: &cim_claude_adapter::domain::value_objects::ConversationId) -> Result<Option<cim_claude_adapter::ConversationAggregate>, String> {
        Ok(None)
    }
    
    pub async fn list_active_conversations(&self) -> Result<Vec<String>, String> {
        Ok(vec![])
    }
    
    pub async fn request_health_check(&self) -> Result<crate::messages::HealthStatus, String> {
        Ok(crate::messages::HealthStatus::default())
    }
}