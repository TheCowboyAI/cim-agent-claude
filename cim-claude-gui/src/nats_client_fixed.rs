/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Fixed NATS Client with Proper Request-Response Correlation
//! 
//! This implementation ensures proper timing and correlation between
//! SAGE requests and responses by:
//! 1. Maintaining persistent subscriptions
//! 2. Using request-response correlation with request_id
//! 3. Handling timing issues with proper state management

use iced::futures::{stream, Stream};
use futures::StreamExt;
use tracing::{info, error, warn};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tokio::sync::oneshot;
use std::time::Duration;
use tokio::time::timeout;

use crate::messages::Message;
use crate::sage_client::{SageRequest, SageResponse, SageStatus};

/// Global NATS client - initialized once at startup
static NATS_CLIENT: tokio::sync::OnceCell<async_nats::Client> = tokio::sync::OnceCell::const_new();

/// Global response correlation map for request-response pattern
static RESPONSE_MAP: tokio::sync::OnceCell<Arc<RwLock<HashMap<String, oneshot::Sender<SageResponse>>>>> = 
    tokio::sync::OnceCell::const_new();

/// Initialize NATS client and response handler at startup
pub async fn initialize_nats() -> Result<(), String> {
    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222")
        .await
        .map_err(|e| format!("NATS connection failed: {}", e))?;
    
    NATS_CLIENT.set(client.clone())
        .map_err(|_| "NATS client already initialized".to_string())?;
    
    // Initialize response map
    let response_map = Arc::new(RwLock::new(HashMap::new()));
    RESPONSE_MAP.set(response_map.clone())
        .map_err(|_| "Response map already initialized".to_string())?;
    
    // Start background response handler
    tokio::spawn(async move {
        if let Err(e) = sage_response_handler(client, response_map).await {
            error!("SAGE response handler error: {}", e);
        }
    });
    
    info!("✅ NATS client initialized with response correlation");
    Ok(())
}

/// Background task that handles SAGE responses and correlates them with requests
async fn sage_response_handler(
    client: async_nats::Client,
    response_map: Arc<RwLock<HashMap<String, oneshot::Sender<SageResponse>>>>
) -> Result<(), Box<dyn std::error::Error>> {
    // Subscribe to SAGE responses
    let mut sage_subscriber = client.subscribe("sage.response.*").await?;
    info!("📡 SAGE response handler subscribed to sage.response.*");
    
    // Also subscribe to status responses
    let mut status_subscriber = client.subscribe("sage.status.response").await?;
    info!("📡 SAGE status handler subscribed to sage.status.response");
    
    loop {
        tokio::select! {
            Some(msg) = sage_subscriber.next() => {
                info!("📥 Received SAGE response on: {}", msg.subject);
                
                match serde_json::from_slice::<SageResponse>(&msg.payload) {
                    Ok(response) => {
                        let request_id = response.request_id.clone();
                        info!("✅ Parsed SAGE response for request: {}", request_id);
                        
                        // Check if someone is waiting for this response
                        let mut map = response_map.write().await;
                        if let Some(sender) = map.remove(&request_id) {
                            if sender.send(response).is_ok() {
                                info!("✅ Delivered response to waiting request: {}", request_id);
                            } else {
                                warn!("⚠️ Failed to deliver response (receiver dropped): {}", request_id);
                            }
                        } else {
                            warn!("⚠️ Received response but no one waiting: {}", request_id);
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to parse SAGE response: {}", e);
                    }
                }
            }
            Some(msg) = status_subscriber.next() => {
                info!("📥 Received SAGE status response");
                // Status responses don't use correlation, they're broadcast
                // This would be handled by the event stream
            }
            else => {
                warn!("⚠️ All subscriptions closed");
                break;
            }
        }
    }
    
    Ok(())
}

/// Get the global NATS client
pub fn get_nats_client() -> Option<&'static async_nats::Client> {
    NATS_CLIENT.get()
}

/// Send SAGE request with guaranteed response correlation
pub async fn send_sage_request_correlated(request: SageRequest) -> Result<SageResponse, String> {
    // Get NATS client
    let client = get_nats_client()
        .ok_or_else(|| "NATS client not initialized".to_string())?;
    
    // Get response map
    let response_map = RESPONSE_MAP.get()
        .ok_or_else(|| "Response map not initialized".to_string())?;
    
    // Create oneshot channel for this specific response
    let (tx, rx) = oneshot::channel();
    let request_id = request.request_id.clone();
    
    // Register the response handler BEFORE sending request
    {
        let mut map = response_map.write().await;
        map.insert(request_id.clone(), tx);
        info!("📝 Registered response handler for: {}", request_id);
    }
    
    // Serialize and send request
    let request_json = serde_json::to_vec(&request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;
    
    client.publish("sage.request", request_json.into()).await
        .map_err(|e| format!("Failed to publish request: {}", e))?;
    
    info!("📤 Published SAGE request: {}", request_id);
    
    // Wait for correlated response with timeout
    match timeout(Duration::from_secs(10), rx).await {
        Ok(Ok(response)) => {
            info!("✅ Received correlated response for: {}", request_id);
            Ok(response)
        }
        Ok(Err(_)) => {
            error!("❌ Response channel closed for: {}", request_id);
            // Clean up
            let mut map = response_map.write().await;
            map.remove(&request_id);
            Err("Response channel closed".to_string())
        }
        Err(_) => {
            error!("❌ Timeout waiting for response: {}", request_id);
            // Clean up on timeout
            let mut map = response_map.write().await;
            map.remove(&request_id);
            Err("Timeout waiting for SAGE response (10s)".to_string())
        }
    }
}

/// NATS Commands - Updated to use correlation
pub mod commands {
    use super::*;
    
    /// Send SAGE request with correlation
    pub async fn send_sage_request(request: SageRequest) -> Message {
        let request_id = request.request_id.clone();
        
        match send_sage_request_correlated(request).await {
            Ok(response) => {
                info!("✅ SAGE request {} completed successfully", request_id);
                Message::SageResponseReceived(response)
            }
            Err(e) => {
                error!("❌ SAGE request {} failed: {}", request_id, e);
                Message::Error(format!("SAGE request failed: {}", e))
            }
        }
    }
    
    /// Legacy simple message publishing (for non-correlated messages)
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
    
    /// Request SAGE status (no correlation needed - broadcast)
    pub async fn request_sage_status() -> Message {
        match get_nats_client() {
            Some(client) => {
                info!("Requesting SAGE status");
                match client.publish("sage.status", "{}".into()).await {
                    Ok(_) => {
                        info!("SAGE status request sent");
                        Message::SageStatusRequested
                    }
                    Err(e) => {
                        error!("Failed to request SAGE status: {}", e);
                        Message::Error(format!("Status request failed: {}", e))
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

/// NATS Event Stream - For broadcast messages and status updates
pub fn nats_event_stream() -> impl Stream<Item = Message> {
    stream::unfold(None, |state| async move {
        match get_nats_client() {
            Some(client) => {
                // Maintain status subscriber across calls
                let mut status_subscriber = match state {
                    Some(sub) => sub,
                    None => {
                        match client.subscribe("sage.status.response").await {
                            Ok(sub) => {
                                info!("📡 Status event stream subscribed");
                                sub
                            }
                            Err(e) => {
                                error!("Failed to subscribe to status: {}", e);
                                return Some((Message::ConnectionError(format!("Status subscription failed: {}", e)), None));
                            }
                        }
                    }
                };
                
                // Wait for status messages
                match status_subscriber.next().await {
                    Some(msg) => {
                        info!("📥 Status event received");
                        match serde_json::from_slice::<SageStatus>(&msg.payload) {
                            Ok(status) => {
                                Some((Message::SageStatusReceived(status), Some(status_subscriber)))
                            }
                            Err(e) => {
                                warn!("Failed to parse status: {}", e);
                                Some((Message::Error(format!("Status parse failed: {}", e)), Some(status_subscriber)))
                            }
                        }
                    }
                    None => {
                        warn!("Status subscription ended");
                        Some((Message::Error("Status subscription ended".to_string()), None))
                    }
                }
            }
            None => {
                Some((Message::Error("NATS not initialized".to_string()), None))
            }
        }
    })
}

/// TEA-compliant subscription for NATS events
pub fn nats_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run_with_id(
        "nats-events",
        nats_event_stream()
    )
}