// NATS Adapter: Implementation of ConversationPort
// Handles NATS messaging for commands and events following CIM event-driven patterns

use async_trait::async_trait;
use futures::StreamExt;
use nats::jetstream::{self, consumer::PullConsumer, stream::Stream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

use crate::domain::commands::{
    ConversationCommand, StartConversationCommand, SendPromptCommand, 
    EndConversationCommand, CommandResult
};
use crate::domain::conversation_aggregate::{
    DomainEvent, ConversationId, SessionId, CorrelationId
};
use crate::ports::conversation_port::{
    ConversationPort, ConversationPortError, ConversationEventHandler, 
    EventSubscription, PortHealth, HealthStatus, DomainEventPublisher
};

// === NATS Message Wrappers ===

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NatsCommandMessage {
    pub command: ConversationCommand,
    pub correlation_id: CorrelationId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NatsEventMessage {
    pub event: DomainEvent,
    pub correlation_id: Option<CorrelationId>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

// === NATS Subject Patterns (Following CIM Graph Specification) ===

struct NatsSubjects;

impl NatsSubjects {
    // Command subjects (inbound)
    pub fn command_start_conversation(session_id: &SessionId) -> String {
        format!("claude.cmd.{}.start", session_id.value())
    }
    
    pub fn command_send_prompt(session_id: &SessionId) -> String {
        format!("claude.cmd.{}.prompt", session_id.value())
    }
    
    pub fn command_end_conversation(session_id: &SessionId) -> String {
        format!("claude.cmd.{}.end", session_id.value())
    }
    
    // Event subjects (outbound)
    pub fn event_conversation_started(session_id: &SessionId) -> String {
        format!("claude.event.{}.started", session_id.value())
    }
    
    pub fn event_prompt_sent(session_id: &SessionId) -> String {
        format!("claude.event.{}.prompt_sent", session_id.value())
    }
    
    pub fn event_response_received(session_id: &SessionId) -> String {
        format!("claude.resp.{}.content", session_id.value())
    }
    
    pub fn event_conversation_ended(session_id: &SessionId) -> String {
        format!("claude.event.{}.ended", session_id.value())
    }
    
    // Wildcard patterns for consumers
    pub const COMMANDS_PATTERN: &'static str = "claude.cmd.*.>";
    pub const EVENTS_PATTERN: &'static str = "claude.event.*.>";
    pub const RESPONSES_PATTERN: &'static str = "claude.resp.*.>";
}

// === NATS Adapter Configuration ===

#[derive(Debug, Clone)]
pub struct NatsAdapterConfig {
    pub nats_url: String,
    pub credentials_file: Option<String>,
    pub max_reconnects: u32,
    pub reconnect_wait_ms: u64,
    pub command_stream: String,
    pub event_stream: String,
    pub response_stream: String,
    pub consumer_name: String,
    pub max_ack_pending: u32,
    pub ack_wait_seconds: u32,
}

impl Default for NatsAdapterConfig {
    fn default() -> Self {
        Self {
            nats_url: "nats://localhost:4222".to_string(),
            credentials_file: None,
            max_reconnects: 10,
            reconnect_wait_ms: 2000,
            command_stream: "CLAUDE_COMMANDS".to_string(),
            event_stream: "CLAUDE_EVENTS".to_string(),
            response_stream: "CLAUDE_RESPONSES".to_string(),
            consumer_name: "claude-adapter".to_string(),
            max_ack_pending: 1000,
            ack_wait_seconds: 30,
        }
    }
}

// === NATS Adapter Implementation ===

pub struct NatsAdapter {
    config: NatsAdapterConfig,
    connection: Arc<nats::Connection>,
    jetstream: Arc<jetstream::Context>,
    command_consumer: Arc<RwLock<Option<PullConsumer>>>,
    event_publisher: Arc<dyn EventPublisher>,
    subscriptions: Arc<RwLock<HashMap<String, EventSubscription>>>,
    command_handler: Arc<dyn CommandProcessor>,
}

impl NatsAdapter {
    pub async fn new(
        config: NatsAdapterConfig,
        command_handler: Arc<dyn CommandProcessor>,
    ) -> Result<Self, NatsAdapterError> {
        // Connect to NATS
        let connection = Self::connect(&config).await?;
        
        // Create JetStream context
        let jetstream = jetstream::new(connection.clone());
        
        // Setup streams
        Self::setup_streams(&jetstream, &config).await?;
        
        // Create event publisher
        let event_publisher = Arc::new(NatsEventPublisher::new(
            jetstream.clone(),
            config.event_stream.clone(),
            config.response_stream.clone(),
        ));
        
        Ok(Self {
            config,
            connection: Arc::new(connection),
            jetstream: Arc::new(jetstream),
            command_consumer: Arc::new(RwLock::new(None)),
            event_publisher,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            command_handler,
        })
    }
    
    async fn connect(config: &NatsAdapterConfig) -> Result<nats::Connection, NatsAdapterError> {
        let mut options = nats::Options::new()
            .with_name("claude-adapter")
            .with_max_reconnects(Some(config.max_reconnects as usize))
            .with_reconnect_delay_callback(|attempts| {
                std::time::Duration::from_millis(config.reconnect_wait_ms * attempts as u64)
            });
            
        if let Some(creds_file) = &config.credentials_file {
            options = options.with_credentials(creds_file);
        }
        
        let connection = options.connect(&config.nats_url)
            .map_err(|e| NatsAdapterError::ConnectionError(e.to_string()))?;
            
        info!("Connected to NATS at {}", config.nats_url);
        Ok(connection)
    }
    
    async fn setup_streams(
        js: &jetstream::Context,
        config: &NatsAdapterConfig,
    ) -> Result<(), NatsAdapterError> {
        // Setup command stream
        let _command_stream = js.get_or_create_stream(jetstream::stream::Config {
            name: config.command_stream.clone(),
            subjects: vec![NatsSubjects::COMMANDS_PATTERN.to_string()],
            storage: jetstream::stream::StorageType::File,
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            max_age: std::time::Duration::from_secs(24 * 3600), // 24 hours
            duplicate_window: std::time::Duration::from_secs(120), // 2 minutes
            ..Default::default()
        }).await.map_err(|e| NatsAdapterError::StreamSetupError(e.to_string()))?;
        
        // Setup event stream
        let _event_stream = js.get_or_create_stream(jetstream::stream::Config {
            name: config.event_stream.clone(),
            subjects: vec![NatsSubjects::EVENTS_PATTERN.to_string()],
            storage: jetstream::stream::StorageType::File,
            retention: jetstream::stream::RetentionPolicy::Limits,
            max_age: std::time::Duration::from_secs(30 * 24 * 3600), // 30 days
            ..Default::default()
        }).await.map_err(|e| NatsAdapterError::StreamSetupError(e.to_string()))?;
        
        // Setup response stream
        let _response_stream = js.get_or_create_stream(jetstream::stream::Config {
            name: config.response_stream.clone(),
            subjects: vec![NatsSubjects::RESPONSES_PATTERN.to_string()],
            storage: jetstream::stream::StorageType::File,
            retention: jetstream::stream::RetentionPolicy::Interest,
            max_age: std::time::Duration::from_secs(3600), // 1 hour
            ..Default::default()
        }).await.map_err(|e| NatsAdapterError::StreamSetupError(e.to_string()))?;
        
        info!("NATS streams configured successfully");
        Ok(())
    }
    
    pub async fn start_command_processing(&self) -> Result<(), NatsAdapterError> {
        let consumer = self.jetstream.get_or_create_consumer(
            &self.config.command_stream,
            jetstream::consumer::pull::Config {
                durable_name: Some(self.config.consumer_name.clone()),
                deliver_policy: jetstream::consumer::DeliverPolicy::New,
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ack_wait: std::time::Duration::from_secs(self.config.ack_wait_seconds as u64),
                max_deliver: Some(3),
                max_ack_pending: Some(self.config.max_ack_pending as i64),
                ..Default::default()
            },
        ).await.map_err(|e| NatsAdapterError::ConsumerSetupError(e.to_string()))?;
        
        // Store consumer for cleanup
        {
            let mut guard = self.command_consumer.write().await;
            *guard = Some(consumer.clone());
        }
        
        // Start processing messages
        let handler = Arc::clone(&self.command_handler);
        let event_publisher = Arc::clone(&self.event_publisher);
        
        tokio::spawn(async move {
            let mut messages = consumer.messages().await.unwrap();
            
            while let Some(message) = messages.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(e) = Self::process_command_message(
                            &msg,
                            Arc::clone(&handler),
                            Arc::clone(&event_publisher),
                        ).await {
                            error!("Failed to process command message: {}", e);
                            // NACK the message for retry
                            if let Err(nack_err) = msg.ack_with(jetstream::AckKind::Nak(None)).await {
                                error!("Failed to NACK message: {}", nack_err);
                            }
                        } else {
                            // ACK successful processing
                            if let Err(ack_err) = msg.ack().await {
                                error!("Failed to ACK message: {}", ack_err);
                            }
                        }
                    },
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                    }
                }
            }
        });
        
        info!("Started NATS command processing");
        Ok(())
    }
    
    async fn process_command_message(
        msg: &jetstream::Message,
        handler: Arc<dyn CommandProcessor>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Result<(), NatsAdapterError> {
        // Deserialize command message
        let command_msg: NatsCommandMessage = serde_json::from_slice(&msg.data)
            .map_err(|e| NatsAdapterError::DeserializationError(e.to_string()))?;
        
        // Process command
        let result = handler.process_command(command_msg.command).await
            .map_err(|e| NatsAdapterError::CommandProcessingError(e.to_string()))?;
        
        // Publish result events if any
        if let Some(events) = result.events {
            event_publisher.publish_events(events).await
                .map_err(|e| NatsAdapterError::EventPublishingError(e.to_string()))?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl ConversationPort for NatsAdapter {
    type Error = ConversationPortError;
    
    async fn handle_command(&self, command: ConversationCommand) -> Result<CommandResult, Self::Error> {
        self.command_handler.process_command(command).await
            .map_err(|e| ConversationPortError::ProcessingError(e.to_string()))
            .map(|result| result.command_result)
    }
    
    async fn subscribe_to_conversation_events(
        &self,
        conversation_id: ConversationId,
        event_handler: Box<dyn ConversationEventHandler>,
    ) -> Result<EventSubscription, Self::Error> {
        let subscription_id = format!("conv-{}", conversation_id.value());
        
        // Create NATS subscription for conversation events
        let subject = format!("claude.event.{}.>", conversation_id.value());
        let subscription = self.connection.subscribe(&subject)
            .map_err(|e| ConversationPortError::SubscriptionError(e.to_string()))?;
        
        // Spawn task to handle events
        tokio::spawn(async move {
            while let Some(msg) = subscription.next().await {
                if let Ok(event_msg) = serde_json::from_slice::<NatsEventMessage>(&msg.data) {
                    if let Err(e) = event_handler.handle_event(event_msg.event).await {
                        error!("Event handler error: {}", e);
                    }
                }
            }
        });
        
        let event_subscription = EventSubscription::new(subscription_id.clone())
            .with_conversation_id(conversation_id);
        
        // Store subscription
        {
            let mut guard = self.subscriptions.write().await;
            guard.insert(subscription_id, event_subscription.clone());
        }
        
        Ok(event_subscription)
    }
    
    async fn subscribe_to_session_events(
        &self,
        session_id: SessionId,
        event_handler: Box<dyn ConversationEventHandler>,
    ) -> Result<EventSubscription, Self::Error> {
        let subscription_id = format!("session-{}", session_id.value());
        
        // Create NATS subscription for session events  
        let subject = format!("claude.event.{}.>", session_id.value());
        let subscription = self.connection.subscribe(&subject)
            .map_err(|e| ConversationPortError::SubscriptionError(e.to_string()))?;
        
        // Spawn task to handle events
        tokio::spawn(async move {
            while let Some(msg) = subscription.next().await {
                if let Ok(event_msg) = serde_json::from_slice::<NatsEventMessage>(&msg.data) {
                    if let Err(e) = event_handler.handle_event(event_msg.event).await {
                        error!("Event handler error: {}", e);
                    }
                }
            }
        });
        
        let event_subscription = EventSubscription::new(subscription_id.clone())
            .with_session_id(session_id);
        
        // Store subscription
        {
            let mut guard = self.subscriptions.write().await;
            guard.insert(subscription_id, event_subscription.clone());
        }
        
        Ok(event_subscription)
    }
    
    async fn health_check(&self) -> Result<PortHealth, Self::Error> {
        let status = if self.connection.is_connected() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: "NATS connection lost".to_string(),
            }
        };
        
        let subscriptions_count = {
            let guard = self.subscriptions.read().await;
            guard.len() as u32
        };
        
        Ok(PortHealth {
            status,
            connected_adapters: 1,
            active_subscriptions: subscriptions_count,
            last_command_at: None, // TODO: Track this
            metrics: HashMap::new(),
        })
    }
}

// === Event Publisher Implementation ===

pub struct NatsEventPublisher {
    jetstream: jetstream::Context,
    event_stream: String,
    response_stream: String,
}

impl NatsEventPublisher {
    pub fn new(
        jetstream: jetstream::Context,
        event_stream: String,
        response_stream: String,
    ) -> Self {
        Self {
            jetstream,
            event_stream,
            response_stream,
        }
    }
    
    fn get_subject_for_event(&self, event: &DomainEvent) -> String {
        match event {
            DomainEvent::ConversationStarted { session_id, .. } => 
                NatsSubjects::event_conversation_started(session_id),
            DomainEvent::PromptSent { .. } => {
                // Extract session_id from context or use default pattern
                "claude.event.prompt_sent".to_string() // Simplified for example
            },
            DomainEvent::ResponseReceived { .. } => {
                "claude.resp.content".to_string() // Simplified for example  
            },
            DomainEvent::ConversationEnded { session_id, .. } => 
                NatsSubjects::event_conversation_ended(session_id),
        }
    }
}

#[async_trait]
impl DomainEventPublisher for NatsEventPublisher {
    type Error = NatsAdapterError;
    
    async fn publish_event(&self, event: DomainEvent) -> Result<(), Self::Error> {
        let subject = self.get_subject_for_event(&event);
        let correlation_id = event.correlation_id();
        
        let event_msg = NatsEventMessage {
            event,
            correlation_id,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        let data = serde_json::to_vec(&event_msg)
            .map_err(|e| NatsAdapterError::SerializationError(e.to_string()))?;
        
        self.jetstream.publish(subject, data.into()).await
            .map_err(|e| NatsAdapterError::PublishError(e.to_string()))?;
        
        Ok(())
    }
}

// === Command Processing Trait ===

#[async_trait]
pub trait CommandProcessor: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    async fn process_command(&self, command: ConversationCommand) -> Result<CommandProcessingResult, Self::Error>;
}

pub struct CommandProcessingResult {
    pub command_result: CommandResult,
    pub events: Option<Vec<DomainEvent>>,
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    async fn publish_events(&self, events: Vec<DomainEvent>) -> Result<(), Self::Error>;
}

// === Error Types ===

#[derive(Debug, thiserror::Error)]
pub enum NatsAdapterError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Stream setup error: {0}")]
    StreamSetupError(String),
    
    #[error("Consumer setup error: {0}")]
    ConsumerSetupError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Publish error: {0}")]
    PublishError(String),
    
    #[error("Command processing error: {0}")]
    CommandProcessingError(String),
    
    #[error("Event publishing error: {0}")]
    EventPublishingError(String),
}