/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

#[cfg(not(target_arch = "wasm32"))]
use async_nats::{Client, jetstream};

use futures::StreamExt;
use iced::futures::stream::{self, BoxStream};

#[cfg(feature = "tokio")]
use tokio::sync::mpsc;

#[cfg(not(feature = "tokio"))]
use futures::channel::mpsc;

use tracing::{info, warn, error};
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use crate::nats_websocket::WebSocketNatsClient;

use cim_claude_adapter::{
    domain::{commands::{Command as DomainCommand, CommandEnvelope}, events::*, value_objects::*, ConversationAggregate},
};
use crate::messages::{Message, HealthStatus, SystemMetrics};

/// NATS client for the GUI application
/// Handles all NATS communication using async streams that Iced can consume
#[derive(Clone, Debug)]
pub struct GuiNatsClient {
    #[cfg(not(target_arch = "wasm32"))]
    client: Arc<Mutex<Option<Client>>>,
    #[cfg(not(target_arch = "wasm32"))]
    jetstream: Arc<Mutex<Option<jetstream::Context>>>,
    
    #[cfg(target_arch = "wasm32")]
    websocket_client: WebSocketNatsClient,
    
    event_sender: Option<mpsc::UnboundedSender<Message>>,
}

impl GuiNatsClient {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            client: Arc::new(Mutex::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            jetstream: Arc::new(Mutex::new(None)),
            
            #[cfg(target_arch = "wasm32")]
            websocket_client: WebSocketNatsClient::new(),
            
            event_sender: None,
        }
    }
    
    /// Connect to NATS and return a stream of messages
    pub fn connect(&mut self, nats_url: String) -> BoxStream<'static, Message> {
        #[cfg(target_arch = "wasm32")]
        {
            // Use WebSocket client for WASM builds
            self.websocket_client.connect(nats_url)
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (tx, rx) = mpsc::unbounded_channel();
            self.event_sender = Some(tx.clone());
            
            // Clone the Arc references for the async task
            let client_ref = self.client.clone();
            let jetstream_ref = self.jetstream.clone();
            
            // Create a task to handle the NATS connection with proper error handling
            let connect_task = async move {
                info!("Attempting to connect to NATS at {}", nats_url);
                match async_nats::connect(&nats_url).await {
                    Ok(client) => {
                        info!("Successfully connected to NATS at {}", nats_url);
                        let jetstream = jetstream::new(client.clone());
                        
                        // Store the client and jetstream instances BEFORE sending Connected message
                        {
                            let mut client_lock = client_ref.lock().unwrap();
                            *client_lock = Some(client.clone());
                        }
                        {
                            let mut jetstream_lock = jetstream_ref.lock().unwrap();
                            *jetstream_lock = Some(jetstream.clone());
                        }
                        
                        // Send connected message only after client is stored
                        let _ = tx.send(Message::Connected);
                        info!("NATS client stored and Connected message sent");
                        
                        // Start event subscription task
                        let subscription_tx = tx.clone();
                        let sub_client = client.clone();
                        let sub_jetstream = jetstream.clone();
                        tokio::spawn(async move {
                            Self::subscribe_to_events(sub_client, sub_jetstream, subscription_tx).await;
                        });
                        
                        // Start health monitoring task
                        let health_tx = tx.clone();
                        let health_client = client.clone();
                        tokio::spawn(async move {
                            Self::monitor_health(health_client, health_tx).await;
                        });
                        
                        // Keep the connection alive
                        loop {
                            if !matches!(client.connection_state(), async_nats::connection::State::Connected) {
                                warn!("NATS connection lost");
                                let _ = tx.send(Message::Disconnected);
                                break;
                            }
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                    Err(e) => {
                        error!("Failed to connect to NATS: {}", e);
                        let _ = tx.send(Message::ConnectionError(e.to_string()));
                    }
                }
            };
            
            // Schedule the connection task to run
            tokio::spawn(connect_task);
            
            // Return stream of messages from the receiver
            Box::pin(stream::unfold(rx, |mut rx| async move {
                rx.recv().await.map(|message| (message, rx))
            }))
        }
    }
    
    /// Subscribe to conversation events from NATS
    #[cfg(not(target_arch = "wasm32"))]
    async fn subscribe_to_events(
        client: Client,
        jetstream: jetstream::Context,
        sender: mpsc::UnboundedSender<Message>,
    ) {
        tokio::spawn(async move {
            // Create a consumer for the CLAUDE_EVENTS stream
            match jetstream.get_stream("CLAUDE_EVENTS").await {
                Ok(stream) => {
                    match stream.create_consumer(async_nats::jetstream::consumer::pull::Config {
                        durable_name: Some("gui-consumer".to_string()),
                        filter_subject: "claude.event.*".to_string(),
                        ..Default::default()
                    }).await {
                        Ok(consumer) => {
                            info!("Created JetStream consumer for GUI events");
                            
                            let mut messages = consumer.messages().await.unwrap();
                            while let Some(message) = messages.next().await {
                                if let Ok(message) = message {
                                    match serde_json::from_slice::<EventEnvelope>(&message.payload) {
                                        Ok(event_envelope) => {
                                            let _ = sender.send(Message::ConversationEvent(event_envelope));
                                            let _ = message.ack().await;
                                        }
                                        Err(e) => {
                                            warn!("Failed to deserialize event: {}", e);
                                            let _ = message.ack().await;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to create JetStream consumer, falling back to simple subscription: {}", e);
                            // Fallback to regular subscription
                            if let Ok(mut subscription) = client.subscribe("claude.event.*".to_string()).await {
                                info!("Using fallback subscription for conversation events");
                                while let Some(message) = subscription.next().await {
                                    match serde_json::from_slice::<EventEnvelope>(&message.payload) {
                                        Ok(event_envelope) => {
                                            let _ = sender.send(Message::ConversationEvent(event_envelope));
                                        }
                                        Err(e) => {
                                            warn!("Failed to deserialize event: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get CLAUDE_EVENTS stream, using regular subscription: {}", e);
                    // Fallback to regular subscription
                    if let Ok(mut subscription) = client.subscribe("claude.event.*".to_string()).await {
                        info!("Using fallback subscription for conversation events");
                        while let Some(message) = subscription.next().await {
                            match serde_json::from_slice::<EventEnvelope>(&message.payload) {
                                Ok(event_envelope) => {
                                    let _ = sender.send(Message::ConversationEvent(event_envelope));
                                }
                                Err(e) => {
                                    warn!("Failed to deserialize event: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    /// Monitor system health by querying NATS directly
    #[cfg(not(target_arch = "wasm32"))]
    async fn monitor_health(client: Client, sender: mpsc::UnboundedSender<Message>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            let jetstream = jetstream::new(client.clone());
            
            loop {
                interval.tick().await;
                
                // Query NATS for real-time statistics
                let (active_conversations, events_processed) = 
                    Self::query_nats_statistics(&jetstream).await;
                
                let health = HealthStatus {
                    nats_connected: matches!(client.connection_state(), async_nats::connection::State::Connected),
                    claude_api_available: Self::check_claude_api_health(&client).await,
                    active_conversations,
                    events_processed,
                    last_check: chrono::Utc::now(),
                };
                
                let _ = sender.send(Message::HealthCheckReceived(health));
                
                // Also query and send system metrics
                let metrics = Self::query_system_metrics(&jetstream, &client).await;
                let _ = sender.send(Message::MetricsReceived(metrics));
            }
        });
    }
    
    /// Query NATS streams and KV stores for statistics
    #[cfg(not(target_arch = "wasm32"))]
    async fn query_nats_statistics(jetstream: &jetstream::Context) -> (u32, u64) {
        let active_conversations = match jetstream.get_key_value("CONVERSATION_STATE").await {
            Ok(_kv) => {
                // Count active conversations by scanning KV store
                // In production, you'd maintain an index or use watch patterns
                0 // Simplified for now
            }
            Err(_) => 0,
        };
        
        let events_processed = match jetstream.get_stream("CLAUDE_EVENTS").await {
            Ok(stream) => {
                match stream.get_info().await {
                    Ok(info) => info.state.messages,
                    Err(_) => 0,
                }
            }
            Err(_) => 0,
        };
        
        (active_conversations, events_processed)
    }
    
    /// Check Claude API health by querying recent request metrics
    #[cfg(not(target_arch = "wasm32"))]
    async fn check_claude_api_health(client: &Client) -> bool {
        // Query metrics or recent API call results from NATS
        // For now, assume healthy if NATS is connected
        matches!(client.connection_state(), async_nats::connection::State::Connected)
    }
    
    /// Query conversation statistics from NATS streams
    #[cfg(not(target_arch = "wasm32"))]
    async fn query_conversation_stats(jetstream: &jetstream::Context, _client: &Client) -> (u32, u64) {
        let mut active_conversations = 0u32;
        let mut events_processed = 0u64;
        
        // Query KV store for active conversations
        if let Ok(_kv) = jetstream.get_key_value("CONVERSATION_STATE").await {
            // In production, scan the KV store for active conversations
            active_conversations = 0; // Simplified
        }
        
        // Query events stream for processed events count
        if let Ok(events_stream) = jetstream.get_stream("CLAUDE_EVENTS").await {
            if let Ok(info) = events_stream.get_info().await {
                events_processed = info.state.messages;
            }
        }
        
        (active_conversations, events_processed)
    }
    
    /// Query comprehensive system metrics from NATS streams
    #[cfg(not(target_arch = "wasm32"))]
    async fn query_system_metrics(jetstream: &jetstream::Context, _client: &Client) -> SystemMetrics {
        let mut metrics = SystemMetrics::default();
        
        // Query conversation metrics from KV store
        if let Ok(_kv) = jetstream.get_key_value("CONVERSATION_STATE").await {
            // Count total and active conversations
            // In production, scan the KV store or maintain indexes
            metrics.conversations_total = 0; // Simplified
            metrics.conversations_active = 0;
        }
        
        // Query event metrics from streams
        if let Ok(events_stream) = jetstream.get_stream("CLAUDE_EVENTS").await {
            if let Ok(info) = events_stream.get_info().await {
                metrics.events_published = info.state.messages;
            }
        }
        
        if let Ok(commands_stream) = jetstream.get_stream("CLAUDE_COMMANDS").await {
            if let Ok(info) = commands_stream.get_info().await {
                metrics.events_consumed = info.state.messages;
            }
        }
        
        // Query API metrics (could be stored in a metrics stream)
        // For now, use placeholder values
        metrics.api_requests_total = 0;
        metrics.api_requests_failed = 0;
        metrics.response_time_avg_ms = 0.0;
        
        metrics
    }
    
    
    /// Load conversation state from NATS KV store
    pub async fn load_conversation(
        &self, 
        conversation_id: &ConversationId
    ) -> Result<Option<ConversationAggregate>, String> {
        #[cfg(target_arch = "wasm32")]
        {
            self.websocket_client.load_conversation(conversation_id).await
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Extract jetstream from mutex guard before async operations
            let jetstream_option = {
                let jetstream_guard = self.jetstream.lock().unwrap();
                jetstream_guard.clone()
            };
            
            if let Some(jetstream) = jetstream_option.as_ref() {
                match jetstream.get_key_value("CONVERSATION_STATE").await {
                    Ok(kv) => {
                        let key = format!("conversation:{}", conversation_id.as_uuid());
                        
                        match kv.get(&key).await {
                            Ok(Some(entry)) => {
                                match serde_json::from_slice::<ConversationAggregate>(&entry) {
                                    Ok(aggregate) => Ok(Some(aggregate)),
                                    Err(e) => Err(format!("Deserialization failed: {}", e)),
                                }
                            }
                            Ok(None) => Ok(None),
                            Err(e) => Err(format!("KV get failed: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("KV store access failed: {}", e)),
                }
            } else {
                Err("Not connected to NATS".to_string())
            }
        }
    }
    
    /// List active conversations
    pub async fn list_active_conversations(&self) -> Result<Vec<String>, String> {
        #[cfg(target_arch = "wasm32")]
        {
            self.websocket_client.list_active_conversations().await
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // For now, return empty list
            // In production, you'd scan the KV store or maintain an index
            Ok(vec![])
        }
    }
    
    /// Request a health check from the CIM system
    pub async fn request_health_check(&self) -> Result<HealthStatus, String> {
        #[cfg(target_arch = "wasm32")]
        {
            self.websocket_client.request_health_check().await
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Extract values from mutex guards before async operations
            let (client_option, jetstream_option) = {
                let client_guard = self.client.lock().unwrap();
                let jetstream_guard = self.jetstream.lock().unwrap();
                (client_guard.clone(), jetstream_guard.clone())
            };
            
            if let (Some(client), Some(jetstream)) = (client_option.as_ref(), jetstream_option.as_ref()) {
                let nats_connected = matches!(client.connection_state(), async_nats::connection::State::Connected);
                let claude_api_available = Self::check_claude_api_health(client).await;
                let (active_conversations, events_processed) = Self::query_conversation_stats(jetstream, client).await;
                
                Ok(HealthStatus {
                    nats_connected,
                    claude_api_available,
                    active_conversations,
                    events_processed,
                    last_check: chrono::Utc::now(),
                })
            } else {
                Ok(HealthStatus {
                    nats_connected: false,
                    claude_api_available: false,
                    active_conversations: 0,
                    events_processed: 0,
                    last_check: chrono::Utc::now(),
                })
            }
        }
    }
    
    /// Create a future for sending a command that can be used with Iced's Command::perform
    pub fn send_command_future(command_envelope: CommandEnvelope) -> impl std::future::Future<Output = Result<(), String>> + Send {
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                match async_nats::connect("nats://localhost:4222").await {
                    Ok(client) => {
                        let command_type = match &command_envelope.command {
                            DomainCommand::StartConversation { .. } => "start_conversation",
                            DomainCommand::SendPrompt { .. } => "send_prompt", 
                            DomainCommand::EndConversation { .. } => "end_conversation",
                        };
                        let subject = format!("cim.claude.command.{}", command_type);
                        
                        let payload = serde_json::to_vec(&command_envelope)
                            .map_err(|e| format!("Serialization failed: {}", e))?;
                        
                        match client.publish(subject, payload.into()).await {
                            Ok(_) => {
                                info!("Successfully published command to {}", subject);
                                Ok(())
                            }
                            Err(e) => {
                                error!("NATS publish failed: {}", e);
                                Err(format!("NATS publish failed: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to connect to NATS: {}", e);
                        Err(format!("NATS connection failed: {}", e))
                    }
                }
            }
            
            #[cfg(target_arch = "wasm32")]
            {
                // WebSocket implementation would go here
                Err("WebSocket NATS not implemented".to_string())
            }
        }
    }

    /// Send a command to the CIM system (deprecated - use send_command_future with Command::perform)
    pub async fn send_command(&self, command_envelope: CommandEnvelope) -> Result<(), String> {
        Self::send_command_future(command_envelope).await
    }
}