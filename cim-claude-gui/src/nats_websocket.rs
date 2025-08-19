/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! WebSocket-based NATS client for WASM builds
//! Connects to NATS WebSocket endpoint for browser compatibility

#[cfg(target_arch = "wasm32")]
use {
    wasm_bindgen::{prelude::*, JsCast},
    web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent, BinaryType},
    wasm_bindgen_futures::spawn_local,
    js_sys::{Uint8Array, ArrayBuffer},
};

use iced::futures::stream::{self, BoxStream};

#[cfg(feature = "tokio")]
use tokio::sync::mpsc;

#[cfg(not(feature = "tokio"))]
use futures::channel::mpsc;

use futures::StreamExt;
use tracing::{info, warn, error, debug};
use serde_json;
use std::collections::HashMap;

use cim_claude_adapter::{
    domain::{commands::*, events::*, value_objects::*, ConversationAggregate},
};
use crate::messages::{Message, HealthStatus, SystemMetrics};

/// WebSocket-based NATS client for WASM environments
#[derive(Clone, Debug)]
pub struct WebSocketNatsClient {
    #[cfg(target_arch = "wasm32")]
    websocket: Option<WebSocket>,
    
    event_sender: Option<mpsc::UnboundedSender<Message>>,
    connected: bool,
    nats_ws_url: String,
    subscription_id_counter: u64,
    subscriptions: HashMap<String, u64>,
}

impl WebSocketNatsClient {
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            websocket: None,
            
            event_sender: None,
            connected: false,
            nats_ws_url: String::new(),
            subscription_id_counter: 0,
            subscriptions: HashMap::new(),
        }
    }
    
    /// Connect to NATS WebSocket endpoint and return a stream of messages
    pub fn connect(&mut self, nats_url: String) -> BoxStream<'static, Message> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.event_sender = Some(tx.clone());
        
        // Convert NATS URL to WebSocket URL via nginx proxy
        // nats://localhost:4222 -> ws://localhost:8081/nats-ws (proxied)
        let ws_url = if nats_url.starts_with("nats://") {
            // Use the web GUI port with /nats-ws path for proxied WebSocket
            let host_part = nats_url.replace("nats://", "").replace(":4222", "");
            if host_part == "localhost" || host_part == "127.0.0.1" {
                "ws://localhost:8081/nats-ws".to_string()
            } else {
                format!("ws://{}:8081/nats-ws", host_part)
            }
        } else {
            "ws://localhost:8081/nats-ws".to_string()
        };
        
        self.nats_ws_url = ws_url.clone();
        
        info!("Connecting to NATS WebSocket at {}", ws_url);
        
        #[cfg(target_arch = "wasm32")]
        {
            self.connect_websocket(ws_url, tx.clone());
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Fallback for non-WASM builds - use regular NATS client
            let _ = tx.send(Message::ConnectionError("WebSocket client not supported on native builds".to_string()));
        }
        
        // Return stream of messages from the receiver
        Box::pin(stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|message| (message, rx))
        }))
    }
    
    #[cfg(target_arch = "wasm32")]
    fn connect_websocket(&mut self, ws_url: String, sender: mpsc::UnboundedSender<Message>) {
        let websocket = match WebSocket::new(&ws_url) {
            Ok(ws) => ws,
            Err(e) => {
                error!("Failed to create WebSocket: {:?}", e);
                let _ = sender.send(Message::ConnectionError("Failed to create WebSocket connection".to_string()));
                return;
            }
        };
        
        // Set binary type for NATS protocol
        websocket.set_binary_type(BinaryType::Arraybuffer);
        
        let sender_clone = sender.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            info!("WebSocket connected to NATS");
            let _ = sender_clone.send(Message::Connected);
        }) as Box<dyn FnMut(_)>);
        websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        
        let sender_clone = sender.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(array_buffer) = e.data().dyn_into::<ArrayBuffer>() {
                let uint8_array = Uint8Array::new(&array_buffer);
                let data = uint8_array.to_vec();
                
                // Parse NATS message
                if let Some(nats_msg) = Self::parse_nats_message(&data) {
                    Self::handle_nats_message(nats_msg, &sender_clone);
                }
            } else if let Some(text) = e.data().as_string() {
                debug!("Received text message: {}", text);
                // Handle NATS protocol messages (INFO, PING, PONG, etc.)
                Self::handle_protocol_message(&text, &sender_clone);
            }
        }) as Box<dyn FnMut(_)>);
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        
        let sender_clone = sender.clone();
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            error!("WebSocket error: {:?}", e);
            let _ = sender_clone.send(Message::ConnectionError("WebSocket connection error".to_string()));
        }) as Box<dyn FnMut(_)>);
        websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
        
        let sender_clone = sender.clone();
        let onclose_callback = Closure::wrap(Box::new(move |_: CloseEvent| {
            info!("WebSocket connection closed");
            let _ = sender_clone.send(Message::Disconnected);
        }) as Box<dyn FnMut(_)>);
        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();
        
        self.websocket = Some(websocket);
        self.connected = true;
        
        // Start subscription to conversation events after connection
        spawn_local(Self::setup_subscriptions(sender));
    }
    
    #[cfg(target_arch = "wasm32")]
    async fn setup_subscriptions(sender: mpsc::UnboundedSender<Message>) {
        // Wait a moment for connection to stabilize
        gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
        
        // Send subscription requests for conversation events
        let subscribe_msg = Self::create_subscribe_message("claude.event.*");
        // TODO: Send through WebSocket
        info!("Setting up NATS subscriptions for events");
    }
    
    #[cfg(target_arch = "wasm32")]
    fn parse_nats_message(data: &[u8]) -> Option<NatsMessage> {
        // Parse NATS wire protocol message
        // This is a simplified parser - real implementation would be more robust
        if let Ok(msg_str) = std::str::from_utf8(data) {
            if msg_str.starts_with("MSG ") {
                // MSG <subject> <sid> [reply-to] <#bytes>\r\n[payload]\r\n
                let lines: Vec<&str> = msg_str.split("\r\n").collect();
                if lines.len() >= 2 {
                    let header_parts: Vec<&str> = lines[0].split_whitespace().collect();
                    if header_parts.len() >= 4 {
                        let subject = header_parts[1].to_string();
                        let payload = lines[1].as_bytes().to_vec();
                        
                        return Some(NatsMessage {
                            subject,
                            payload,
                            reply: None,
                        });
                    }
                }
            }
        }
        None
    }
    
    fn handle_nats_message(msg: NatsMessage, sender: &mpsc::UnboundedSender<Message>) {
        // Handle different types of NATS messages
        if msg.subject.starts_with("claude.event.") {
            // Parse as event envelope
            match serde_json::from_slice::<EventEnvelope>(&msg.payload) {
                Ok(event_envelope) => {
                    let _ = sender.send(Message::ConversationEvent(event_envelope));
                }
                Err(e) => {
                    warn!("Failed to deserialize event: {}", e);
                }
            }
        } else if msg.subject.starts_with("claude.health.") {
            // Parse as health status
            match serde_json::from_slice::<HealthStatus>(&msg.payload) {
                Ok(health) => {
                    let _ = sender.send(Message::HealthCheckReceived(health));
                }
                Err(e) => {
                    warn!("Failed to deserialize health status: {}", e);
                }
            }
        } else if msg.subject.starts_with("claude.metrics.") {
            // Parse as system metrics
            match serde_json::from_slice::<SystemMetrics>(&msg.payload) {
                Ok(metrics) => {
                    let _ = sender.send(Message::MetricsReceived(metrics));
                }
                Err(e) => {
                    warn!("Failed to deserialize metrics: {}", e);
                }
            }
        }
    }
    
    fn handle_protocol_message(msg: &str, sender: &mpsc::UnboundedSender<Message>) {
        if msg.starts_with("INFO ") {
            debug!("Received NATS server info");
        } else if msg.trim() == "PING" {
            debug!("Received PING, should send PONG");
            // TODO: Send PONG response
        } else if msg.trim() == "PONG" {
            debug!("Received PONG");
        } else if msg.starts_with("+OK") {
            debug!("Operation acknowledged");
        } else if msg.starts_with("-ERR ") {
            warn!("NATS error: {}", msg);
            let _ = sender.send(Message::ConnectionError(format!("NATS error: {}", msg)));
        }
    }
    
    fn create_subscribe_message(subject: &str) -> String {
        // SUB <subject> [queue group] <sid>\r\n
        format!("SUB {} 1\r\n", subject)
    }
    
    fn create_publish_message(subject: &str, payload: &[u8]) -> Vec<u8> {
        // PUB <subject> [reply-to] <#bytes>\r\n[payload]\r\n
        let header = format!("PUB {} {}\r\n", subject, payload.len());
        let mut msg = header.into_bytes();
        msg.extend_from_slice(payload);
        msg.extend_from_slice(b"\r\n");
        msg
    }
    
    /// Send a command to NATS via WebSocket
    pub async fn send_command(&self, command_envelope: CommandEnvelope) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(ref websocket) = self.websocket {
                let subject = match &command_envelope.command {
                    Command::StartConversation { session_id, .. } => {
                        format!("claude.cmd.{}.start", session_id.as_uuid())
                    }
                    Command::SendPrompt { conversation_id, .. } => {
                        format!("claude.cmd.{}.prompt", conversation_id.as_uuid())
                    }
                    Command::EndConversation { conversation_id, .. } => {
                        format!("claude.cmd.{}.end", conversation_id.as_uuid())
                    }
                };
                
                let payload = serde_json::to_vec(&command_envelope)
                    .map_err(|e| format!("Serialization failed: {}", e))?;
                
                let nats_msg = Self::create_publish_message(&subject, &payload);
                let uint8_array = Uint8Array::from(&nats_msg[..]);
                
                websocket
                    .send_with_u8_array(&uint8_array)
                    .map_err(|e| format!("WebSocket send failed: {:?}", e))?;
                
                Ok(())
            } else {
                Err("WebSocket not connected".to_string())
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        Err("WebSocket client not supported on native builds".to_string())
    }
    
    /// Load conversation state (placeholder for WebSocket implementation)
    pub async fn load_conversation(
        &self, 
        _conversation_id: &ConversationId
    ) -> Result<Option<ConversationAggregate>, String> {
        // For WebSocket implementation, we'd need to implement request-response pattern
        // This could be done via reply subjects or a separate HTTP API
        Ok(None)
    }
    
    /// List active conversations (placeholder)
    pub async fn list_active_conversations(&self) -> Result<Vec<String>, String> {
        Ok(vec![])
    }
}

#[derive(Debug)]
struct NatsMessage {
    subject: String,
    payload: Vec<u8>,
    reply: Option<String>,
}