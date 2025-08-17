// Inbound Port: ConversationPort
// Interface for receiving commands from external systems (NATS)

use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::commands::{
    ConversationCommand, StartConversationCommand, SendPromptCommand, 
    EndConversationCommand, CommandResult
};
use crate::domain::conversation_aggregate::{DomainEvent, ConversationId, SessionId};

// === Inbound Port (Interface for external systems to interact with domain) ===

#[async_trait]
pub trait ConversationPort: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    /// Receive and process a conversation command
    async fn handle_command(&self, command: ConversationCommand) -> Result<CommandResult, Self::Error>;
    
    /// Subscribe to domain events for a specific conversation
    async fn subscribe_to_conversation_events(
        &self, 
        conversation_id: ConversationId,
        event_handler: Box<dyn ConversationEventHandler>,
    ) -> Result<EventSubscription, Self::Error>;
    
    /// Subscribe to domain events for a session (multiple conversations)
    async fn subscribe_to_session_events(
        &self,
        session_id: SessionId,
        event_handler: Box<dyn ConversationEventHandler>,
    ) -> Result<EventSubscription, Self::Error>;
    
    /// Health check for the port
    async fn health_check(&self) -> Result<PortHealth, Self::Error>;
}

// === Event Handler Trait ===

#[async_trait]
pub trait ConversationEventHandler: Send + Sync {
    async fn handle_event(&self, event: DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

// === Event Subscription Management ===

pub struct EventSubscription {
    pub subscription_id: String,
    pub conversation_id: Option<ConversationId>,
    pub session_id: Option<SessionId>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl EventSubscription {
    pub fn new(subscription_id: String) -> Self {
        Self {
            subscription_id,
            conversation_id: None,
            session_id: None,
            created_at: chrono::Utc::now(),
        }
    }
    
    pub fn with_conversation_id(mut self, conversation_id: ConversationId) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }
    
    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

// === Port Health Monitoring ===

#[derive(Debug, Clone)]
pub struct PortHealth {
    pub status: HealthStatus,
    pub connected_adapters: u32,
    pub active_subscriptions: u32,
    pub last_command_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metrics: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

// === Command Processing Traits ===

#[async_trait]
pub trait StartConversationHandler: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    async fn start_conversation(&self, command: StartConversationCommand) -> Result<CommandResult, Self::Error>;
}

#[async_trait]
pub trait SendPromptHandler: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    async fn send_prompt(&self, command: SendPromptCommand) -> Result<CommandResult, Self::Error>;
}

#[async_trait]
pub trait EndConversationHandler: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    async fn end_conversation(&self, command: EndConversationCommand) -> Result<CommandResult, Self::Error>;
}

// === Event Publishing (Outbound from Domain) ===

#[async_trait]
pub trait DomainEventPublisher: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    async fn publish_event(&self, event: DomainEvent) -> Result<(), Self::Error>;
    
    async fn publish_events(&self, events: Vec<DomainEvent>) -> Result<(), Self::Error> {
        for event in events {
            self.publish_event(event).await?;
        }
        Ok(())
    }
}

// === Port Configuration ===

#[derive(Debug, Clone)]
pub struct ConversationPortConfig {
    pub max_concurrent_conversations: u32,
    pub command_timeout_seconds: u32,
    pub event_buffer_size: u32,
    pub health_check_interval_seconds: u32,
    pub retry_policy: RetryPolicy,
}

impl Default for ConversationPortConfig {
    fn default() -> Self {
        Self {
            max_concurrent_conversations: 1000,
            command_timeout_seconds: 30,
            event_buffer_size: 10000,
            health_check_interval_seconds: 30,
            retry_policy: RetryPolicy::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_factor: f32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_factor: 2.0,
        }
    }
}

// === Port Error Types ===

#[derive(Debug, thiserror::Error)]
pub enum ConversationPortError {
    #[error("Command validation failed: {0}")]
    ValidationError(String),
    
    #[error("Command processing failed: {0}")]
    ProcessingError(String),
    
    #[error("Event subscription failed: {0}")]
    SubscriptionError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Timeout occurred while processing command")]
    Timeout,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Internal error: {0}")]
    InternalError(String),
}