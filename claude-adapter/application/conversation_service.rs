// Application Service: ConversationService
// Orchestrates conversation workflow between domain, ports, and adapters

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::domain::commands::{
    ConversationCommand, StartConversationCommand, SendPromptCommand, 
    EndConversationCommand, CommandResult, CommandHandler
};
use crate::domain::conversation_aggregate::{
    ConversationAggregate, DomainEvent, ConversationId, DomainError
};
use crate::ports::conversation_port::DomainEventPublisher;
use crate::ports::claude_api_port::{ClaudeApiPort, ClaudeApiRequest};
use crate::adapters::nats_adapter::{CommandProcessor, CommandProcessingResult};

// === Application Service Implementation ===

pub struct ConversationService {
    conversation_repository: Arc<dyn ConversationRepository>,
    claude_api: Arc<dyn ClaudeApiPort<Error = Box<dyn std::error::Error + Send + Sync>>>,
    event_publisher: Arc<dyn DomainEventPublisher<Error = Box<dyn std::error::Error + Send + Sync>>>,
}

impl ConversationService {
    pub fn new(
        conversation_repository: Arc<dyn ConversationRepository>,
        claude_api: Arc<dyn ClaudeApiPort<Error = Box<dyn std::error::Error + Send + Sync>>>,
        event_publisher: Arc<dyn DomainEventPublisher<Error = Box<dyn std::error::Error + Send + Sync>>>,
    ) -> Self {
        Self {
            conversation_repository,
            claude_api,
            event_publisher,
        }
    }
    
    async fn handle_start_conversation(&self, command: StartConversationCommand) -> Result<CommandResult, ConversationServiceError> {
        info!("Starting new conversation for session: {}", command.session_id.value());
        
        // Create new conversation aggregate
        let (mut conversation, events) = ConversationAggregate::start_conversation(
            command.session_id,
            command.initial_prompt.clone(),
            command.correlation_id,
        ).map_err(|e| ConversationServiceError::DomainError(e))?;
        
        // Save conversation
        self.conversation_repository
            .save(&conversation)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;
        
        // Publish domain events
        self.event_publisher
            .publish_events(events)
            .await
            .map_err(|e| ConversationServiceError::EventPublishingError(e.to_string()))?;
        
        // Send initial prompt to Claude API
        let api_request = ClaudeApiRequest::new(
            command.initial_prompt,
            conversation.id(),
            command.correlation_id,
            // Need to get the event ID from the prompt sent event
            crate::domain::conversation_aggregate::EventId::generate(), // Simplified
        );
        
        // Spawn async task to handle Claude API response
        let conversation_repo = Arc::clone(&self.conversation_repository);
        let event_pub = Arc::clone(&self.event_publisher);
        let claude_api = Arc::clone(&self.claude_api);
        let conversation_id = conversation.id();
        
        tokio::spawn(async move {
            match claude_api.send_prompt(api_request).await {
                Ok(api_response) => {
                    // Load conversation and apply response
                    if let Ok(mut conversation) = conversation_repo.load(&conversation_id).await {
                        if let Ok(events) = conversation.receive_response(
                            api_response.response,
                            api_response.event_id,
                        ) {
                            // Save updated conversation
                            if let Err(e) = conversation_repo.save(&conversation).await {
                                error!("Failed to save conversation after response: {}", e);
                                return;
                            }
                            
                            // Publish events
                            if let Err(e) = event_pub.publish_events(events).await {
                                error!("Failed to publish response events: {}", e);
                            }
                        } else {
                            error!("Failed to apply response to conversation");
                        }
                    } else {
                        error!("Failed to load conversation for response");
                    }
                },
                Err(e) => {
                    error!("Claude API request failed: {}", e);
                    // TODO: Handle API failures (retry, circuit breaker, etc.)
                }
            }
        });
        
        Ok(CommandResult::ConversationStarted {
            conversation_id: conversation.id(),
            correlation_id: command.correlation_id,
        })
    }
    
    async fn handle_send_prompt(&self, command: SendPromptCommand) -> Result<CommandResult, ConversationServiceError> {
        info!("Sending prompt to conversation: {}", command.conversation_id.value());
        
        // Load conversation
        let mut conversation = self.conversation_repository
            .load(&command.conversation_id)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;
        
        // Send prompt to aggregate
        let events = conversation
            .send_prompt(command.prompt.clone(), command.correlation_id)
            .map_err(|e| ConversationServiceError::DomainError(e))?;
        
        // Save updated conversation
        self.conversation_repository
            .save(&conversation)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;
        
        // Publish events
        self.event_publisher
            .publish_events(events.clone())
            .await
            .map_err(|e| ConversationServiceError::EventPublishingError(e.to_string()))?;
        
        // Get the event ID from the PromptSent event
        let event_id = events.iter()
            .find_map(|event| match event {
                DomainEvent::PromptSent { event_id, .. } => Some(*event_id),
                _ => None,
            })
            .unwrap_or_else(|| crate::domain::conversation_aggregate::EventId::generate());
        
        // Send to Claude API
        let api_request = ClaudeApiRequest::new(
            command.prompt,
            command.conversation_id,
            command.correlation_id,
            event_id,
        );
        
        // Spawn async task to handle Claude API response
        let conversation_repo = Arc::clone(&self.conversation_repository);
        let event_pub = Arc::clone(&self.event_publisher);
        let claude_api = Arc::clone(&self.claude_api);
        let conversation_id = command.conversation_id;
        
        tokio::spawn(async move {
            match claude_api.send_prompt(api_request).await {
                Ok(api_response) => {
                    // Load conversation and apply response
                    if let Ok(mut conversation) = conversation_repo.load(&conversation_id).await {
                        if let Ok(events) = conversation.receive_response(
                            api_response.response,
                            api_response.event_id,
                        ) {
                            // Save updated conversation
                            if let Err(e) = conversation_repo.save(&conversation).await {
                                error!("Failed to save conversation after response: {}", e);
                                return;
                            }
                            
                            // Publish events
                            if let Err(e) = event_pub.publish_events(events).await {
                                error!("Failed to publish response events: {}", e);
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Claude API request failed: {}", e);
                }
            }
        });
        
        Ok(CommandResult::PromptSent {
            conversation_id: command.conversation_id,
            correlation_id: command.correlation_id,
        })
    }
    
    async fn handle_end_conversation(&self, command: EndConversationCommand) -> Result<CommandResult, ConversationServiceError> {
        info!("Ending conversation: {}", command.conversation_id.value());
        
        // Load conversation
        let mut conversation = self.conversation_repository
            .load(&command.conversation_id)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;
        
        // End conversation
        let events = conversation
            .end_conversation(command.reason)
            .map_err(|e| ConversationServiceError::DomainError(e))?;
        
        // Save updated conversation
        self.conversation_repository
            .save(&conversation)
            .await
            .map_err(|e| ConversationServiceError::RepositoryError(e.to_string()))?;
        
        // Publish events
        self.event_publisher
            .publish_events(events)
            .await
            .map_err(|e| ConversationServiceError::EventPublishingError(e.to_string()))?;
        
        Ok(CommandResult::ConversationEnded {
            conversation_id: command.conversation_id,
            correlation_id: command.correlation_id,
        })
    }
}

#[async_trait]
impl CommandHandler<ConversationCommand> for ConversationService {
    type Result = CommandResult;
    type Error = ConversationServiceError;
    
    async fn handle(&self, command: ConversationCommand) -> Result<Self::Result, Self::Error> {
        match command {
            ConversationCommand::StartConversation(cmd) => self.handle_start_conversation(cmd).await,
            ConversationCommand::SendPrompt(cmd) => self.handle_send_prompt(cmd).await,
            ConversationCommand::EndConversation(cmd) => self.handle_end_conversation(cmd).await,
        }
    }
}

#[async_trait]
impl CommandProcessor for ConversationService {
    type Error = ConversationServiceError;
    
    async fn process_command(&self, command: ConversationCommand) -> Result<CommandProcessingResult, Self::Error> {
        let command_result = self.handle(command).await?;
        
        Ok(CommandProcessingResult {
            command_result,
            events: None, // Events are published directly in the service
        })
    }
}

// === Repository Trait ===

#[async_trait]
pub trait ConversationRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync;
    
    async fn save(&self, conversation: &ConversationAggregate) -> Result<(), Self::Error>;
    
    async fn load(&self, conversation_id: &ConversationId) -> Result<ConversationAggregate, Self::Error>;
    
    async fn exists(&self, conversation_id: &ConversationId) -> Result<bool, Self::Error>;
    
    async fn delete(&self, conversation_id: &ConversationId) -> Result<(), Self::Error>;
    
    async fn find_by_session(&self, session_id: &crate::domain::conversation_aggregate::SessionId) -> Result<Vec<ConversationAggregate>, Self::Error>;
}

// === Error Types ===

#[derive(Debug, thiserror::Error)]
pub enum ConversationServiceError {
    #[error("Domain error: {0:?}")]
    DomainError(DomainError),
    
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Claude API error: {0}")]
    ClaudeApiError(String),
    
    #[error("Event publishing error: {0}")]
    EventPublishingError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Not found: conversation {0}")]
    ConversationNotFound(ConversationId),
    
    #[error("Concurrent modification detected")]
    ConcurrencyConflict,
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

// === Configuration ===

#[derive(Debug, Clone)]
pub struct ConversationServiceConfig {
    pub max_concurrent_conversations: u32,
    pub conversation_timeout_minutes: u32,
    pub retry_policy: RetryPolicy,
    pub circuit_breaker: CircuitBreakerConfig,
}

impl Default for ConversationServiceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_conversations: 1000,
            conversation_timeout_minutes: 60,
            retry_policy: RetryPolicy::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
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

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout_seconds: u32,
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout_seconds: 60,
            success_threshold: 3,
        }
    }
}