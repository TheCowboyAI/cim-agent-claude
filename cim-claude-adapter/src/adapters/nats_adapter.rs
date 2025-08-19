use async_nats::{Client, jetstream::{self, consumer::PullConsumer, kv}};
use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::{
    domain::{commands::*, events::*, value_objects::*, errors::*, ConversationAggregate},
    ports::{ConversationPort, ConversationStatePort, PortHealth, PortMetrics},
};

/// NATS adapter implementing the conversation port
pub struct NatsAdapter {
    client: Client,
    jetstream: jetstream::Context,
    metrics: Arc<RwLock<PortMetrics>>,
}

impl NatsAdapter {
    /// Create new NATS adapter
    pub async fn new(nats_url: &str) -> Result<Self, InfrastructureError> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;
            
        let jetstream = jetstream::new(client.clone());
        
        let adapter = Self {
            client,
            jetstream,
            metrics: Arc::new(RwLock::new(PortMetrics::default())),
        };
        
        // Ensure streams exist
        adapter.ensure_streams().await?;
        
        Ok(adapter)
    }
    
    /// Ensure required JetStream streams exist
    async fn ensure_streams(&self) -> Result<(), InfrastructureError> {
        // Commands stream
        match self.jetstream.get_or_create_stream(jetstream::stream::Config {
            name: "CLAUDE_COMMANDS".to_string(),
            subjects: vec!["claude.cmd.*".to_string()],
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            max_messages: 10_000,
            max_age: std::time::Duration::from_secs(24 * 60 * 60), // 24 hours
            ..Default::default()
        }).await {
            Ok(_) => info!("Commands stream ready"),
            Err(e) => return Err(InfrastructureError::NatsConnection(
                format!("Failed to create commands stream: {}", e)
            )),
        }
        
        // Events stream
        match self.jetstream.get_or_create_stream(jetstream::stream::Config {
            name: "CLAUDE_EVENTS".to_string(),
            subjects: vec!["claude.event.*".to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            max_messages: 50_000,
            max_age: std::time::Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            ..Default::default()
        }).await {
            Ok(_) => info!("Events stream ready"),
            Err(e) => return Err(InfrastructureError::NatsConnection(
                format!("Failed to create events stream: {}", e)
            )),
        }
        
        // Responses stream
        match self.jetstream.get_or_create_stream(jetstream::stream::Config {
            name: "CLAUDE_RESPONSES".to_string(),
            subjects: vec!["claude.resp.*".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            max_messages: 10_000,
            max_age: std::time::Duration::from_secs(60 * 60), // 1 hour
            ..Default::default()
        }).await {
            Ok(_) => info!("Responses stream ready"),
            Err(e) => return Err(InfrastructureError::NatsConnection(
                format!("Failed to create responses stream: {}", e)
            )),
        }
        
        Ok(())
    }
    
    /// Generate NATS subject for command
    fn command_subject(session_id: &SessionId, operation: &str) -> String {
        format!("claude.cmd.{}.{}", session_id.as_uuid(), operation)
    }
    
    /// Generate NATS subject for event
    fn event_subject(conversation_id: &ConversationId, event_type: &str) -> String {
        format!("claude.event.{}.{}", conversation_id.as_uuid(), event_type)
    }
    
    /// Generate NATS subject for response
    pub fn response_subject(conversation_id: &ConversationId) -> String {
        format!("claude.resp.{}.content", conversation_id.as_uuid())
    }
    
    /// Extract event type from domain event
    fn event_type(event: &DomainEvent) -> &'static str {
        match event {
            DomainEvent::ConversationStarted { .. } => "conversation_started",
            DomainEvent::PromptSent { .. } => "prompt_sent",
            DomainEvent::ResponseReceived { .. } => "response_received",
            DomainEvent::ConversationEnded { .. } => "conversation_ended",
            DomainEvent::RateLimitExceeded { .. } => "rate_limit_exceeded",
            DomainEvent::ClaudeApiErrorOccurred { .. } => "claude_api_error_occurred",
        }
    }
    
    /// Update metrics
    async fn update_metrics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut PortMetrics),
    {
        let mut metrics = self.metrics.write().await;
        update_fn(&mut *metrics);
    }
}

#[async_trait]
impl ConversationPort for NatsAdapter {
    async fn handle_command(
        &self,
        command: Command,
        correlation_id: CorrelationId,
    ) -> Result<Vec<DomainEvent>, ApplicationError> {
        // This would typically delegate to an application service
        // For now, we'll return an empty result as this is just the adapter
        info!(
            "Received command: {:?} with correlation_id: {}", 
            command, 
            correlation_id.as_uuid()
        );
        
        self.update_metrics(|m| m.commands_processed += 1).await;
        
        // The actual command handling would be done by the application service
        // This adapter is just responsible for the transport layer
        // The correlation_id is essential for tracking causation chains
        Ok(vec![])
    }
    
    async fn publish_events(
        &self,
        events: Vec<EventEnvelope>,
    ) -> Result<(), ApplicationError> {
        for event_envelope in events {
            let conversation_id = event_envelope.event.conversation_id();
            let event_type = Self::event_type(&event_envelope.event);
            let subject = Self::event_subject(conversation_id, event_type);
            
            let payload = serde_json::to_vec(&event_envelope)
                .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;
                
            // Add NATS headers for correlation tracking
            let mut headers = async_nats::HeaderMap::new();
            headers.insert(
                "correlation-id", 
                event_envelope.correlation_id.as_uuid().to_string().as_str()
            );
            headers.insert(
                "event-id",
                event_envelope.event_id.as_uuid().to_string().as_str()
            );
            headers.insert(
                "causation-id",
                event_envelope.causation_id.as_uuid().to_string().as_str()
            );
            
            match self.jetstream.publish_with_headers(
                subject.clone(),
                headers,
                Bytes::from(payload),
            ).await {
                Ok(_) => {
                    info!("Published event {} to {}", event_type, subject);
                    self.update_metrics(|m| m.events_published += 1).await;
                }
                Err(e) => {
                    error!("Failed to publish event {}: {}", event_type, e);
                    self.update_metrics(|m| m.errors_count += 1).await;
                    return Err(InfrastructureError::NatsPublish(e.to_string()).into());
                }
            }
        }
        
        Ok(())
    }
    
    async fn subscribe_to_commands<F>(&self, handler: F) -> Result<(), ApplicationError>
    where
        F: Fn(CommandEnvelope) -> Result<(), ApplicationError> + Send + Sync + 'static,
    {
        let consumer: PullConsumer = self.jetstream
            .create_consumer_on_stream(
                jetstream::consumer::pull::Config {
                    durable_name: Some("claude-command-processor".to_string()),
                    filter_subject: "claude.cmd.*".to_string(),
                    ..Default::default()
                },
                "CLAUDE_COMMANDS",
            )
            .await
            .map_err(|e| InfrastructureError::NatsSubscribe(e.to_string()))?;
        
        let handler = Arc::new(handler);
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            let mut messages = consumer.messages().await.unwrap();
            
            while let Some(message) = messages.next().await {
                match message {
                    Ok(msg) => {
                        let start_time = std::time::Instant::now();
                        
                        match serde_json::from_slice::<CommandEnvelope>(&msg.payload) {
                            Ok(command_envelope) => {
                                info!(
                                    "Received command: {:?} with correlation ID: {}",
                                    command_envelope.command,
                                    command_envelope.correlation_id.as_uuid()
                                );
                                
                                match handler(command_envelope) {
                                    Ok(_) => {
                                        let _ = msg.ack().await;
                                        let mut m = metrics.write().await;
                                        m.commands_processed += 1;
                                        m.average_processing_time_ms = 
                                            (m.average_processing_time_ms * (m.commands_processed - 1) as f64 + 
                                             start_time.elapsed().as_millis() as f64) / m.commands_processed as f64;
                                    }
                                    Err(e) => {
                                        warn!("Failed to handle command: {}", e);
                                        // In async-nats 0.40+, message handling is different
                                        // For now, we'll just log the error
                                        let mut m = metrics.write().await;
                                        m.errors_count += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize command: {}", e);
                                // In async-nats 0.40+, message handling is different
                                // For now, we'll just log the error
                                let mut m = metrics.write().await;
                                m.errors_count += 1;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                        let mut m = metrics.write().await;
                        m.errors_count += 1;
                    }
                }
            }
        });
        
        info!("Started command subscription");
        Ok(())
    }
    
    async fn health_check(&self) -> Result<PortHealth, ApplicationError> {
        // Check NATS connection
        match self.client.connection_state() {
            async_nats::connection::State::Connected => {
                let metrics = self.metrics.read().await.clone();
                Ok(PortHealth::healthy(
                    "NATS connection healthy".to_string()
                ).with_metrics(metrics))
            }
            state => Ok(PortHealth::unhealthy(
                format!("NATS connection state: {:?}", state)
            )),
        }
    }
}

#[async_trait]
impl ConversationStatePort for NatsAdapter {
    async fn load_conversation(
        &self,
        id: &ConversationId,
    ) -> Result<Option<ConversationAggregate>, ApplicationError> {
        // Use NATS KV store to retrieve conversation state
        let kv = self.jetstream
            .get_key_value("CONVERSATION_STATE")
            .await
            .map_err(|e| InfrastructureError::NatsKvStore(e.to_string()))?;
        
        let key = format!("conversation:{}", id.as_uuid());
        
        match kv.get(&key).await {
            Ok(Some(entry)) => {
                let aggregate = serde_json::from_slice(&entry)
                    .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;
                Ok(Some(aggregate))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(InfrastructureError::NatsKvStore(e.to_string()).into()),
        }
    }
    
    async fn save_conversation(
        &self,
        aggregate: &ConversationAggregate,
        expected_version: u64,
    ) -> Result<(), ApplicationError> {
        // Use NATS KV store with optimistic locking
        let kv = match self.jetstream.get_key_value("CONVERSATION_STATE").await {
            Ok(kv) => kv,
            Err(_) => {
                // Create the KV store if it doesn't exist
                self.jetstream
                    .create_key_value(kv::Config {
                        bucket: "CONVERSATION_STATE".to_string(),
                        description: "Conversation aggregate state storage".to_string(),
                        max_value_size: 1024 * 1024, // 1MB per conversation
                        history: 10,
                        ..Default::default()
                    })
                    .await
                    .map_err(|e| InfrastructureError::NatsKvStore(e.to_string()))?
            }
        };
        
        let key = format!("conversation:{}", aggregate.id().as_uuid());
        let value = serde_json::to_vec(aggregate)
            .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;
        
        // For now, just do a simple put - optimistic locking can be enhanced later
        // In production, you'd use the expected_version for proper optimistic concurrency control
        info!(
            "Saving conversation {} at version {} (expected: {})", 
            aggregate.id().as_uuid(), 
            aggregate.version(),
            expected_version
        );
        
        match kv.put(&key, value.into()).await {
            Ok(_) => {
                info!("Saved conversation {} successfully", aggregate.id().as_uuid());
                Ok(())
            }
            Err(e) => Err(InfrastructureError::NatsKvStore(e.to_string()).into()),
        }
    }
    
    async fn find_active_conversations(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<ConversationId>, ApplicationError> {
        // Use NATS KV store to find conversations by session
        let _kv = self.jetstream
            .get_key_value("CONVERSATION_STATE")
            .await
            .map_err(|e| InfrastructureError::NatsKvStore(e.to_string()))?;
        
        let active_conversations = Vec::new();
        
        // For now, we'll use a simple approach - in production you'd want to use watch/scan patterns
        // This is a simplified implementation for getting the service running
        info!("Finding active conversations for session: {}", session_id.as_uuid());
        
        // In a real implementation, you might maintain an index or use NATS streams for queries
        // For now, we'll return an empty list and rely on direct lookups
        
        Ok(active_conversations)
    }
    
    async fn cleanup_expired_conversations(&self) -> Result<u32, ApplicationError> {
        // Clean up expired conversations from KV store
        let _kv = self.jetstream
            .get_key_value("CONVERSATION_STATE")
            .await
            .map_err(|e| InfrastructureError::NatsKvStore(e.to_string()))?;
        
        let cleaned_count = 0u32;
        
        // Simplified cleanup for initial implementation
        info!("Background cleanup check performed - would scan for expired conversations");
        
        // In production, you'd implement proper key scanning/watching here
        
        if cleaned_count > 0 {
            info!("Cleaned up {} expired conversations", cleaned_count);
        }
        
        Ok(cleaned_count)
    }
}

/// Helper for testing
impl NatsAdapter {
    pub async fn publish_command(
        &self,
        command: CommandEnvelope,
        session_id: &SessionId,
    ) -> Result<(), InfrastructureError> {
        let operation = match &command.command {
            Command::StartConversation { .. } => "start",
            Command::SendPrompt { .. } => "prompt",
            Command::EndConversation { .. } => "end",
        };
        
        let subject = Self::command_subject(session_id, operation);
        let payload = serde_json::to_vec(&command)
            .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;
            
        let mut headers = async_nats::HeaderMap::new();
        headers.insert(
            "correlation-id",
            command.correlation_id.as_uuid().to_string().as_str()
        );
        
        self.jetstream
            .publish_with_headers(subject, headers, Bytes::from(payload))
            .await
            .map_err(|e| InfrastructureError::NatsPublish(e.to_string()))?;
            
        Ok(())
    }
    
    pub async fn get_metrics(&self) -> PortMetrics {
        self.metrics.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_subject_generation() {
        let session_id = SessionId::new();
        let conversation_id = ConversationId::new();
        
        let cmd_subject = NatsAdapter::command_subject(&session_id, "start");
        assert!(cmd_subject.starts_with("claude.cmd."));
        assert!(cmd_subject.ends_with(".start"));
        
        let event_subject = NatsAdapter::event_subject(&conversation_id, "conversation_started");
        assert!(event_subject.starts_with("claude.event."));
        assert!(event_subject.ends_with(".conversation_started"));
        
        let resp_subject = NatsAdapter::response_subject(&conversation_id);
        assert!(resp_subject.starts_with("claude.resp."));
        assert!(resp_subject.ends_with(".content"));
    }
    
    #[test]
    fn test_event_type_mapping() {
        let event = DomainEvent::ConversationStarted {
            conversation_id: ConversationId::new(),
            session_id: SessionId::new(),
            initial_prompt: crate::domain::Prompt::new("test".to_string()).unwrap(),
            context: ConversationContext::default(),
        };
        
        assert_eq!(NatsAdapter::event_type(&event), "conversation_started");
    }
}