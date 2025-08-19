use std::sync::Arc;
use tracing::{info, warn, error};

use crate::{
    domain::{
        commands::*, events::*, value_objects::*, errors::*,
        ConversationAggregate,
    },
    ports::{
        ConversationPort, ConversationStatePort, ClaudeApiPort,
        ClaudeApiRequest,
    },
    adapters::NatsAdapter,
};

/// Application service that orchestrates the conversation workflow
/// This is where the hexagonal architecture comes together
pub struct ConversationService {
    nats_adapter: Arc<NatsAdapter>,
    state_port: Arc<dyn ConversationStatePort>,
    claude_api_port: Arc<dyn ClaudeApiPort>,
}

impl ConversationService {
    pub fn new(
        nats_adapter: Arc<NatsAdapter>,
        state_port: Arc<dyn ConversationStatePort>,
        claude_api_port: Arc<dyn ClaudeApiPort>,
    ) -> Self {
        Self {
            nats_adapter,
            state_port,
            claude_api_port,
        }
    }
    
    /// Start the service and begin listening for commands
    pub async fn start(&self) -> Result<(), ApplicationError> {
        info!("Starting conversation service");
        
        // Clone references for the closure
        let state_port = self.state_port.clone();
        let claude_api_port = self.claude_api_port.clone();
        let nats_adapter = self.nats_adapter.clone();
        
        // Subscribe to commands from NATS
        self.nats_adapter
            .subscribe_to_commands(move |command_envelope| {
                let state_port = state_port.clone();
                let claude_api_port = claude_api_port.clone();
                let nats_adapter = nats_adapter.clone();
                
                tokio::spawn(async move {
                    let service = ConversationService::new(
                        nats_adapter,
                        state_port,
                        claude_api_port,
                    );
                    
                    if let Err(e) = service.handle_command_envelope(command_envelope).await {
                        error!("Failed to handle command: {}", e);
                    }
                });
                
                Ok(())
            })
            .await?;
        
        info!("Conversation service started successfully");
        Ok(())
    }
    
    /// Handle a command envelope from NATS
    async fn handle_command_envelope(
        &self,
        command_envelope: CommandEnvelope,
    ) -> Result<(), ApplicationError> {
        info!(
            "Handling command: {:?} with correlation ID: {}",
            command_envelope.command,
            command_envelope.correlation_id.as_uuid()
        );
        
        match command_envelope.command.clone() {
            Command::StartConversation { .. } => {
                self.handle_start_conversation(command_envelope).await
            }
            Command::SendPrompt { conversation_id, .. } => {
                self.handle_send_prompt(command_envelope, conversation_id).await
            }
            Command::EndConversation { conversation_id, .. } => {
                self.handle_end_conversation(command_envelope, conversation_id).await
            }
        }
    }
    
    /// Handle StartConversation command
    async fn handle_start_conversation(
        &self,
        command_envelope: CommandEnvelope,
    ) -> Result<(), ApplicationError> {
        // Create new aggregate from command
        let mut aggregate = ConversationAggregate::from_command(
            command_envelope.command.clone(),
            command_envelope.correlation_id.clone(),
        )?;
        
        // Save initial state
        self.state_port.save_conversation(&aggregate, 0).await?;
        
        // Process the initial prompt with Claude API
        if let Command::StartConversation { initial_prompt, context, .. } = command_envelope.command {
            let claude_request = ClaudeApiRequest::new(
                initial_prompt,
                context,
                aggregate.id().clone(),
                command_envelope.correlation_id.clone(),
                1,
            );
            
            match self.claude_api_port.send_prompt(claude_request).await {
                Ok(claude_response) => {
                    // Apply response to aggregate
                    let events = aggregate.apply_response(
                        claude_response.response,
                        claude_response.processing_time_ms,
                    )?;
                    
                    // Save updated aggregate
                    self.state_port.save_conversation(&aggregate, aggregate.version() - 1).await?;
                    
                    // Publish events
                    self.publish_events(events, command_envelope.correlation_id).await?;
                    
                    info!(
                        "Successfully processed initial prompt for conversation {}",
                        aggregate.id().as_uuid()
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to process initial prompt for conversation {}: {}",
                        aggregate.id().as_uuid(),
                        e
                    );
                    // Could emit a ClaudeApiErrorOccurred event here
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle SendPrompt command
    async fn handle_send_prompt(
        &self,
        command_envelope: CommandEnvelope,
        conversation_id: ConversationId,
    ) -> Result<(), ApplicationError> {
        // Load existing aggregate
        let mut aggregate = self.state_port
            .load_conversation(&conversation_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                resource: format!("Conversation {}", conversation_id.as_uuid())
            })?;
        
        let old_version = aggregate.version();
        
        // Handle the command
        let events = aggregate.handle_command(
            command_envelope.command.clone(),
            command_envelope.correlation_id.clone(),
        )?;
        
        // Save updated aggregate
        self.state_port.save_conversation(&aggregate, old_version).await?;
        
        // Publish events
        self.publish_events(events, command_envelope.correlation_id.clone()).await?;
        
        // Send prompt to Claude API
        if let Command::SendPrompt { prompt, .. } = command_envelope.command {
            let claude_request = ClaudeApiRequest::new(
                prompt,
                aggregate.context().clone(),
                conversation_id.clone(),
                command_envelope.correlation_id.clone(),
                aggregate.exchanges().len() as u32,
            );
            
            match self.claude_api_port.send_prompt(claude_request).await {
                Ok(claude_response) => {
                    // Apply response to aggregate
                    let response_events = aggregate.apply_response(
                        claude_response.response,
                        claude_response.processing_time_ms,
                    )?;
                    
                    // Save updated aggregate again
                    self.state_port.save_conversation(&aggregate, aggregate.version() - 1).await?;
                    
                    // Publish response events
                    self.publish_events(response_events, command_envelope.correlation_id).await?;
                    
                    info!(
                        "Successfully processed prompt for conversation {}",
                        conversation_id.as_uuid()
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to process prompt for conversation {}: {}",
                        conversation_id.as_uuid(),
                        e
                    );
                    // Could emit error event and continue
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle EndConversation command
    async fn handle_end_conversation(
        &self,
        command_envelope: CommandEnvelope,
        conversation_id: ConversationId,
    ) -> Result<(), ApplicationError> {
        // Load existing aggregate
        let mut aggregate = self.state_port
            .load_conversation(&conversation_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                resource: format!("Conversation {}", conversation_id.as_uuid())
            })?;
        
        let old_version = aggregate.version();
        
        // Handle the command
        let events = aggregate.handle_command(
            command_envelope.command,
            command_envelope.correlation_id.clone(),
        )?;
        
        // Save updated aggregate
        self.state_port.save_conversation(&aggregate, old_version).await?;
        
        // Publish events
        self.publish_events(events, command_envelope.correlation_id).await?;
        
        info!("Successfully ended conversation {}", conversation_id.as_uuid());
        Ok(())
    }
    
    /// Publish domain events with correlation tracking
    async fn publish_events(
        &self,
        events: Vec<DomainEvent>,
        correlation_id: CorrelationId,
    ) -> Result<(), ApplicationError> {
        if events.is_empty() {
            return Ok(());
        }
        
        let event_envelopes: Vec<EventEnvelope> = events
            .into_iter()
            .map(|event| event.with_metadata(correlation_id.clone(), None))
            .collect();
        
        self.nats_adapter.publish_events(event_envelopes).await?;
        Ok(())
    }
    
    /// Health check for the entire service
    pub async fn health_check(&self) -> Result<ServiceHealth, ApplicationError> {
        let conversation_port_health = self.nats_adapter.health_check().await?;
        let claude_api_health = self.claude_api_port.health_check().await?;
        
        let is_healthy = conversation_port_health.is_healthy && claude_api_health.is_available;
        
        Ok(ServiceHealth {
            is_healthy,
            conversation_port_healthy: conversation_port_health.is_healthy,
            claude_api_available: claude_api_health.is_available,
            claude_api_response_time_ms: claude_api_health.response_time_ms,
            error_rate: claude_api_health.error_rate,
            last_check: chrono::Utc::now(),
        })
    }
    
    /// Cleanup expired conversations (background task)
    pub async fn cleanup_expired_conversations(&self) -> Result<u32, ApplicationError> {
        info!("Starting cleanup of expired conversations");
        let count = self.state_port.cleanup_expired_conversations().await?;
        info!("Cleaned up {} expired conversations", count);
        Ok(count)
    }
}

/// Service health status
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub is_healthy: bool,
    pub conversation_port_healthy: bool,
    pub claude_api_available: bool,
    pub claude_api_response_time_ms: u64,
    pub error_rate: f64,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::MemoryStateAdapter;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tokio::sync::Mutex;
    
    // Mock implementations for testing
    struct MockConversationPort;
    
    #[async_trait]
    impl ConversationPort for MockConversationPort {
        async fn handle_command(
            &self,
            _command: Command,
            _correlation_id: CorrelationId,
        ) -> Result<Vec<DomainEvent>, ApplicationError> {
            Ok(vec![])
        }
        
        async fn publish_events(
            &self,
            _events: Vec<EventEnvelope>,
        ) -> Result<(), ApplicationError> {
            Ok(())
        }
        
        async fn subscribe_to_commands<F>(&self, _handler: F) -> Result<(), ApplicationError>
        where
            F: Fn(CommandEnvelope) -> Result<(), ApplicationError> + Send + Sync + 'static,
        {
            Ok(())
        }
        
        async fn health_check(&self) -> Result<crate::ports::PortHealth, ApplicationError> {
            Ok(crate::ports::PortHealth::healthy("Mock port healthy".to_string()))
        }
    }
    
    struct MockClaudeApiPort;
    
    #[async_trait]
    impl ClaudeApiPort for MockClaudeApiPort {
        async fn send_prompt(
            &self,
            _request: ClaudeApiRequest,
        ) -> Result<ClaudeApiResponse, ApplicationError> {
            let response = ClaudeResponse::new(
                "Hello! This is a mock response.".to_string(),
                TokenUsage::new(10, 15),
                "stop".to_string(),
                "claude-3-sonnet-mock".to_string(),
            );
            
            Ok(ClaudeApiResponse::new(
                response,
                "mock-request-id".to_string(),
                100,
            ))
        }
        
        async fn health_check(&self) -> Result<crate::ports::ClaudeApiHealth, ApplicationError> {
            Ok(crate::ports::ClaudeApiHealth {
                is_available: true,
                response_time_ms: 100,
                error_rate: 0.0,
                last_check: chrono::Utc::now(),
                rate_limit_status: crate::ports::RateLimitStatus {
                    requests_remaining: 50,
                    requests_limit: 50,
                    tokens_remaining: 100000,
                    tokens_limit: 100000,
                    reset_time: chrono::Utc::now(),
                },
            })
        }
        
        async fn get_rate_limit_status(&self) -> Result<crate::ports::RateLimitStatus, ApplicationError> {
            Ok(crate::ports::RateLimitStatus {
                requests_remaining: 50,
                requests_limit: 50,
                tokens_remaining: 100000,
                tokens_limit: 100000,
                reset_time: chrono::Utc::now(),
            })
        }
    }
    
    #[tokio::test]
    async fn test_conversation_service_health_check() {
        let conversation_port = Arc::new(MockConversationPort);
        let state_port = Arc::new(MemoryStateAdapter::new());
        let claude_api_port = Arc::new(MockClaudeApiPort);
        
        let service = ConversationService::new(
            conversation_port,
            state_port,
            claude_api_port,
        );
        
        let health = service.health_check().await.unwrap();
        assert!(health.is_healthy);
        assert!(health.conversation_port_healthy);
        assert!(health.claude_api_available);
    }
}