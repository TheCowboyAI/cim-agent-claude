/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use async_nats::{Client, jetstream};
use futures::StreamExt;
use iced::futures::stream::{self, BoxStream};
// HashMap not needed for current implementation
use tokio::sync::mpsc;
use tracing::{info, warn, error};

use crate::{
    domain::{commands::*, events::*, value_objects::*, ConversationAggregate},
    gui::messages::{Message, HealthStatus, SystemMetrics},
};

/// NATS client for the GUI application
/// Handles all NATS communication using async streams that Iced can consume
#[derive(Clone, Debug)]
pub struct GuiNatsClient {
    client: Option<Client>,
    jetstream: Option<jetstream::Context>,
    event_sender: Option<mpsc::UnboundedSender<Message>>,
}

impl GuiNatsClient {
    pub fn new() -> Self {
        Self {
            client: None,
            jetstream: None,
            event_sender: None,
        }
    }
    
    /// Connect to NATS and return a stream of messages
    pub fn connect(&mut self, nats_url: String) -> BoxStream<'static, Message> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.event_sender = Some(tx.clone());
        
        // Spawn connection task
        tokio::spawn(async move {
            match async_nats::connect(&nats_url).await {
                Ok(client) => {
                    info!("Connected to NATS at {}", nats_url);
                    let _ = tx.send(Message::Connected);
                    
                    let jetstream = jetstream::new(client.clone());
                    
                    // Spawn event subscription task
                    Self::subscribe_to_events(client.clone(), jetstream.clone(), tx.clone()).await;
                    
                    // Spawn health monitoring task
                    Self::monitor_health(client, tx.clone()).await;
                }
                Err(e) => {
                    error!("Failed to connect to NATS: {}", e);
                    let _ = tx.send(Message::ConnectionError(e.to_string()));
                }
            }
        });
        
        // Return stream of messages from the receiver
        Box::pin(stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|message| (message, rx))
        }))
    }
    
    /// Subscribe to conversation events from NATS
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
                        Ok(mut consumer) => {
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
    async fn check_claude_api_health(client: &Client) -> bool {
        // Query metrics or recent API call results from NATS
        // For now, assume healthy if NATS is connected
        matches!(client.connection_state(), async_nats::connection::State::Connected)
    }
    
    /// Query comprehensive system metrics from NATS streams
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
    
    /// Send a command to NATS
    pub async fn send_command(&self, command_envelope: CommandEnvelope) -> Result<(), String> {
        if let Some(client) = &self.client {
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
            
            client
                .publish(subject, payload.into())
                .await
                .map_err(|e| format!("NATS publish failed: {}", e))?;
                
            Ok(())
        } else {
            Err("Not connected to NATS".to_string())
        }
    }
    
    /// Load conversation state from NATS KV store
    pub async fn load_conversation(
        &self, 
        conversation_id: &ConversationId
    ) -> Result<Option<ConversationAggregate>, String> {
        if let Some(jetstream) = &self.jetstream {
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
    
    /// List active conversations
    pub async fn list_active_conversations(&self) -> Result<Vec<String>, String> {
        // For now, return empty list
        // In production, you'd scan the KV store or maintain an index
        Ok(vec![])
    }
}