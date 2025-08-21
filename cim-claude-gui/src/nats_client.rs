/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use iced::futures::{stream, Stream};
use futures::StreamExt;
use tracing::{info, error, warn};

// Simplified domain types - complex command/event system removed
use crate::messages::Message;

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
                match client.publish(subject, message.into()).await {
                    Ok(_) => {
                        info!("Message published successfully");
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
}

/// NATS Event Subscription - Stream of Events from NATS using global client
pub fn nats_event_stream() -> impl Stream<Item = Message> {
    stream::unfold((), |_| async {
        match get_nats_client() {
            Some(client) => {
                match client.subscribe("cim.claude.events").await {
                    Ok(mut subscriber) => {
                        info!("NATS event stream started");
                        while let Some(msg) = subscriber.next().await {
                            // Simplified event handling - just log the message
                            match String::from_utf8(msg.payload.to_vec()) {
                                Ok(message) => {
                                    info!("Received raw message: {}", message);
                                    // For now, just ignore events since SAGE handles messaging
                                    continue;
                                }
                                Err(e) => {
                                    warn!("Failed to parse message as UTF-8: {}", e);
                                    return Some((Message::Error(format!("Message parsing failed: {}", e)), ()));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to subscribe to NATS events: {}", e);
                        return Some((Message::ConnectionError(format!("Subscription failed: {}", e)), ()));
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

/// TEA-compliant subscription for NATS events
pub fn nats_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run_with_id(
        "nats-events",
        nats_event_stream()
    )
}