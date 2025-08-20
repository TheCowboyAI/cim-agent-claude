/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use std::sync::Arc;
use tokio::sync::Mutex;
use iced::futures::{stream, Stream};
use futures::StreamExt;
use serde_json;
use tracing::{info, error, warn};

use cim_claude_adapter::domain::{
    commands::CommandEnvelope, 
    events::EventEnvelope,
};
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
fn get_nats_client() -> Option<&'static async_nats::Client> {
    NATS_CLIENT.get()
}

/// NATS Commands - Pure async functions that use the global client
pub mod commands {
    use super::*;
    
    /// Publish command to NATS - uses global client
    pub async fn publish_command(command_envelope: CommandEnvelope) -> Message {
        match get_nats_client() {
            Some(client) => {
                match serde_json::to_string(&command_envelope) {
                    Ok(json) => {
                        info!("Publishing command to cim.claude.commands");
                        match client.publish("cim.claude.commands", json.into()).await {
                            Ok(_) => {
                                info!("Command published successfully");
                                Message::CommandSent
                            }
                            Err(e) => {
                                error!("NATS publish failed: {}", e);
                                Message::Error(format!("Publish failed: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        error!("Command serialization failed: {}", e);
                        Message::Error(format!("Serialization failed: {}", e))
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
                            match serde_json::from_slice::<EventEnvelope>(&msg.payload) {
                                Ok(event) => {
                                    info!("Received event: {:?}", event);
                                    return Some((Message::EventReceived(event), ()));
                                }
                                Err(e) => {
                                    warn!("Failed to deserialize event: {}", e);
                                    return Some((Message::Error(format!("Event deserialization failed: {}", e)), ()));
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