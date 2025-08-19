use async_trait::async_trait;
use crate::domain::{commands::*, events::*, value_objects::*, errors::*};

/// Inbound port for conversation management
/// This is the interface that external systems (NATS) use to interact with the domain
#[async_trait]
pub trait ConversationPort: Send + Sync {
    /// Handle incoming commands from NATS
    async fn handle_command(
        &self,
        command: Command,
        correlation_id: CorrelationId,
    ) -> Result<Vec<DomainEvent>, ApplicationError>;
    
    /// Publish domain events to NATS
    async fn publish_events(
        &self,
        events: Vec<EventEnvelope>,
    ) -> Result<(), ApplicationError>;
    
    /// Subscribe to incoming commands from NATS
    async fn subscribe_to_commands<F>(&self, handler: F) -> Result<(), ApplicationError>
    where
        F: Fn(CommandEnvelope) -> Result<(), ApplicationError> + Send + Sync + 'static;
    
    /// Health check for the port
    async fn health_check(&self) -> Result<PortHealth, ApplicationError>;
}

/// Outbound port for conversation state management
/// This abstracts the persistence/state management from the domain
#[async_trait]
pub trait ConversationStatePort: Send + Sync {
    /// Load conversation aggregate by ID
    async fn load_conversation(
        &self,
        id: &ConversationId,
    ) -> Result<Option<ConversationAggregate>, ApplicationError>;
    
    /// Save conversation aggregate with optimistic locking
    async fn save_conversation(
        &self,
        aggregate: &ConversationAggregate,
        expected_version: u64,
    ) -> Result<(), ApplicationError>;
    
    /// Find active conversations by session ID
    async fn find_active_conversations(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<ConversationId>, ApplicationError>;
    
    /// Clean up expired conversations
    async fn cleanup_expired_conversations(&self) -> Result<u32, ApplicationError>;
}

/// Port health status
#[derive(Debug, Clone)]
pub struct PortHealth {
    pub is_healthy: bool,
    pub message: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub metrics: PortMetrics,
}

/// Port metrics
#[derive(Debug, Clone, Default)]
pub struct PortMetrics {
    pub commands_processed: u64,
    pub events_published: u64,
    pub errors_count: u64,
    pub average_processing_time_ms: f64,
    pub active_connections: u32,
}

impl PortHealth {
    pub fn healthy(message: String) -> Self {
        Self {
            is_healthy: true,
            message,
            last_check: chrono::Utc::now(),
            metrics: PortMetrics::default(),
        }
    }
    
    pub fn unhealthy(message: String) -> Self {
        Self {
            is_healthy: false,
            message,
            last_check: chrono::Utc::now(),
            metrics: PortMetrics::default(),
        }
    }
    
    pub fn with_metrics(mut self, metrics: PortMetrics) -> Self {
        self.metrics = metrics;
        self
    }
}

// Re-export aggregate for convenience
pub use crate::domain::ConversationAggregate;