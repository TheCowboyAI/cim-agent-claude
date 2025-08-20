// ECS Systems - Asynchronous processing systems for message bus communication
//
// Systems handle all external communication and state machine driven operations.
// They are completely asynchronous and never block the TEA display layer.

use super::*;
use crate::{
    adapters::NatsAdapter,
    domain::{
        events::DomainEvent,
        commands::{ConversationCommand, ConfigurationCommand, ToolCommand},
    },
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn, error, debug};

/// Trait for all ECS systems
#[async_trait]
pub trait System: Send + Sync {
    /// Execute a command and return resulting events
    async fn execute(
        &self,
        command: EcsCommand,
        entity_manager: &mut EntityManager,
    ) -> Result<Vec<TeaEvent>, BridgeError>;
    
    /// Get system name for logging and metrics
    fn system_name(&self) -> &'static str;
    
    /// System health check
    async fn health_check(&self) -> Result<ComponentHealthStatus, BridgeError>;
}

/// NATS message system for Claude API communication
pub struct NatsMessageSystem {
    nats_adapter: Arc<NatsAdapter>,
    state_machine: MessageStateMachine,
}

impl NatsMessageSystem {
    pub fn new(nats_adapter: Arc<NatsAdapter>) -> Self {
        Self {
            nats_adapter,
            state_machine: MessageStateMachine::new(),
        }
    }
    
    /// Send message to Claude via NATS
    async fn send_to_claude(
        &self,
        conversation_id: EntityId,
        content: String,
        timestamp: DateTime<Utc>,
    ) -> Result<(), BridgeError> {
        let command = ConversationCommand::SendMessage {
            conversation_id: conversation_id.to_string(),
            content,
            timestamp,
            correlation_id: Uuid::new_v4(),
        };
        
        let subject = format!("cim.claude.conv.cmd.send.{}", conversation_id);
        
        self.nats_adapter
            .publish_command(&subject, &command)
            .await
            .map_err(|e| BridgeError::SystemError(format!("NATS publish failed: {}", e)))
    }
}

#[async_trait]
impl System for NatsMessageSystem {
    async fn execute(
        &self,
        command: EcsCommand,
        entity_manager: &mut EntityManager,
    ) -> Result<Vec<TeaEvent>, BridgeError> {
        match command {
            EcsCommand::SendMessageToNats { conversation_id, content, timestamp } => {
                debug!("Sending message to Claude for conversation {}", conversation_id);
                
                // Update state machine
                let state = self.state_machine.transition_to_sending(conversation_id).await?;
                
                // Send to NATS
                match self.send_to_claude(conversation_id, content.clone(), timestamp).await {
                    Ok(()) => {
                        // Update entity in manager
                        if let Some(mut entity) = entity_manager.get_entity(conversation_id).cloned() {
                            let message = ConversationMessage {
                                id: MessageId::new(),
                                content: content.clone(),
                                role: MessageRole::User,
                                timestamp,
                                token_count: None,
                                tool_calls: Vec::new(),
                                attachments: Vec::new(),
                            };
                            
                            entity.messages.add_message(message);
                            entity.metadata.update_activity();
                            entity_manager.update_entity(conversation_id, entity);
                        }
                        
                        // Transition state machine to sent
                        self.state_machine.transition_to_sent(conversation_id).await?;
                        
                        Ok(vec![
                            TeaEvent::MessageAdded {
                                conversation_id,
                                message_content: content,
                                timestamp,
                            }
                        ])
                    }
                    Err(e) => {
                        // Transition state machine to error
                        self.state_machine.transition_to_error(conversation_id, &e.to_string()).await?;
                        
                        Ok(vec![
                            TeaEvent::ErrorOccurred {
                                error: format!("Failed to send message: {}", e),
                                timestamp: Utc::now(),
                            }
                        ])
                    }
                }
            }
            _ => Err(BridgeError::SystemError("Invalid command for NATS message system".to_string())),
        }
    }
    
    fn system_name(&self) -> &'static str {
        "nats_messages"
    }
    
    async fn health_check(&self) -> Result<ComponentHealthStatus, BridgeError> {
        let start_time = std::time::Instant::now();
        
        match self.nats_adapter.health_check().await {
            Ok(_) => Ok(ComponentHealthStatus {
                status: HealthStatus::Healthy,
                response_time: Some(start_time.elapsed()),
                error_rate: None,
                last_error: None,
            }),
            Err(e) => Ok(ComponentHealthStatus {
                status: HealthStatus::Unhealthy,
                response_time: Some(start_time.elapsed()),
                error_rate: None,
                last_error: Some(e.to_string()),
            }),
        }
    }
}

/// Conversation management system
pub struct ConversationManagementSystem {
    state_machine: ConversationStateMachine,
}

impl ConversationManagementSystem {
    pub fn new() -> Self {
        Self {
            state_machine: ConversationStateMachine::new(),
        }
    }
}

#[async_trait]
impl System for ConversationManagementSystem {
    async fn execute(
        &self,
        command: EcsCommand,
        entity_manager: &mut EntityManager,
    ) -> Result<Vec<TeaEvent>, BridgeError> {
        match command {
            EcsCommand::CreateConversation { title } => {
                debug!("Creating new conversation with title: {}", title);
                
                let conversation_id = EntityId::new_v4();
                let entity = ConversationEntity::new(title.clone());
                
                // Add to entity manager
                entity_manager.add_global_entity(entity);
                
                // Update state machine
                self.state_machine.transition_to_active(conversation_id).await?;
                
                Ok(vec![
                    TeaEvent::ConversationCreated {
                        conversation_id,
                        timestamp: Utc::now(),
                    }
                ])
            }
            
            EcsCommand::ArchiveConversation { conversation_id, reason } => {
                debug!("Archiving conversation: {}", conversation_id);
                
                if let Some(mut entity) = entity_manager.get_entity(conversation_id).cloned() {
                    let old_status = entity.metadata.status.clone();
                    entity.metadata.status = ConversationStatus::Archived;
                    entity.metadata.mark_dirty();
                    entity_manager.update_entity(conversation_id, entity);
                    
                    self.state_machine.transition_to_archived(conversation_id, reason).await?;
                    
                    Ok(vec![
                        TeaEvent::ConversationStatusChanged {
                            conversation_id,
                            old_status,
                            new_status: ConversationStatus::Archived,
                            timestamp: Utc::now(),
                        }
                    ])
                } else {
                    Err(BridgeError::EntityNotFound(conversation_id))
                }
            }
            
            EcsCommand::PauseConversation { conversation_id } => {
                debug!("Pausing conversation: {}", conversation_id);
                
                if let Some(mut entity) = entity_manager.get_entity(conversation_id).cloned() {
                    let old_status = entity.metadata.status.clone();
                    entity.metadata.status = ConversationStatus::Paused;
                    entity.metadata.mark_dirty();
                    entity_manager.update_entity(conversation_id, entity);
                    
                    self.state_machine.transition_to_paused(conversation_id).await?;
                    
                    Ok(vec![
                        TeaEvent::ConversationStatusChanged {
                            conversation_id,
                            old_status,
                            new_status: ConversationStatus::Paused,
                            timestamp: Utc::now(),
                        }
                    ])
                } else {
                    Err(BridgeError::EntityNotFound(conversation_id))
                }
            }
            
            _ => Err(BridgeError::SystemError("Invalid command for conversation management system".to_string())),
        }
    }
    
    fn system_name(&self) -> &'static str {
        "conversations"
    }
    
    async fn health_check(&self) -> Result<ComponentHealthStatus, BridgeError> {
        Ok(ComponentHealthStatus {
            status: HealthStatus::Healthy,
            response_time: Some(std::time::Duration::from_millis(1)),
            error_rate: None,
            last_error: None,
        })
    }
}

/// Configuration management system
pub struct ConfigurationSystem {
    config_cache: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl ConfigurationSystem {
    pub fn new() -> Self {
        Self {
            config_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl System for ConfigurationSystem {
    async fn execute(
        &self,
        command: EcsCommand,
        _entity_manager: &mut EntityManager,
    ) -> Result<Vec<TeaEvent>, BridgeError> {
        match command {
            EcsCommand::UpdateConfiguration { config_key, value } => {
                debug!("Updating configuration: {} = {:?}", config_key, value);
                
                // Update cache
                {
                    let mut cache = self.config_cache.write().await;
                    cache.insert(config_key.clone(), value.clone());
                }
                
                Ok(vec![
                    TeaEvent::ConfigurationUpdated {
                        config_key,
                        new_value: value,
                        timestamp: Utc::now(),
                    }
                ])
            }
            _ => Err(BridgeError::SystemError("Invalid command for configuration system".to_string())),
        }
    }
    
    fn system_name(&self) -> &'static str {
        "configuration"
    }
    
    async fn health_check(&self) -> Result<ComponentHealthStatus, BridgeError> {
        Ok(ComponentHealthStatus {
            status: HealthStatus::Healthy,
            response_time: Some(std::time::Duration::from_millis(1)),
            error_rate: None,
            last_error: None,
        })
    }
}

/// Tool invocation system
pub struct ToolInvocationSystem {
    nats_adapter: Arc<NatsAdapter>,
    tool_state_machine: ToolStateMachine,
}

impl ToolInvocationSystem {
    pub fn new(nats_adapter: Arc<NatsAdapter>) -> Self {
        Self {
            nats_adapter,
            tool_state_machine: ToolStateMachine::new(),
        }
    }
    
    async fn invoke_tool_via_nats(
        &self,
        tool_id: &str,
        parameters: &serde_json::Value,
        conversation_id: EntityId,
    ) -> Result<serde_json::Value, BridgeError> {
        let command = ToolCommand::InvokeTool {
            tool_id: tool_id.to_string(),
            parameters: parameters.clone(),
            conversation_id: conversation_id.to_string(),
            correlation_id: Uuid::new_v4(),
        };
        
        let subject = format!("cim.core.tools.cmd.invoke.{}", tool_id);
        
        // Send command and wait for response
        self.nats_adapter
            .request_response(&subject, &command, std::time::Duration::from_secs(30))
            .await
            .map_err(|e| BridgeError::SystemError(format!("Tool invocation failed: {}", e)))
    }
}

#[async_trait]
impl System for ToolInvocationSystem {
    async fn execute(
        &self,
        command: EcsCommand,
        entity_manager: &mut EntityManager,
    ) -> Result<Vec<TeaEvent>, BridgeError> {
        match command {
            EcsCommand::InvokeTool { tool_id, parameters, conversation_id } => {
                debug!("Invoking tool: {} for conversation: {}", tool_id, conversation_id);
                
                // Transition state machine to invoking
                self.tool_state_machine.transition_to_invoking(&tool_id, conversation_id).await?;
                
                match self.invoke_tool_via_nats(&tool_id, &parameters, conversation_id).await {
                    Ok(result) => {
                        // Update entity if applicable
                        if let Some(mut entity) = entity_manager.get_entity(conversation_id).cloned() {
                            // Add tool call to latest message or create tool message
                            let tool_call = ToolCall {
                                id: Uuid::new_v4().to_string(),
                                name: tool_id.clone(),
                                parameters,
                                result: Some(result.clone()),
                                error: None,
                            };
                            
                            // Create tool result message
                            let tool_message = ConversationMessage {
                                id: MessageId::new(),
                                content: format!("Tool {} completed successfully", tool_id),
                                role: MessageRole::Tool,
                                timestamp: Utc::now(),
                                token_count: None,
                                tool_calls: vec![tool_call],
                                attachments: Vec::new(),
                            };
                            
                            entity.messages.add_message(tool_message);
                            entity.metadata.update_activity();
                            entity_manager.update_entity(conversation_id, entity);
                        }
                        
                        // Transition state machine to completed
                        self.tool_state_machine.transition_to_completed(&tool_id, conversation_id).await?;
                        
                        Ok(vec![
                            TeaEvent::ToolInvocationCompleted {
                                conversation_id,
                                tool_id,
                                result,
                                timestamp: Utc::now(),
                            }
                        ])
                    }
                    Err(e) => {
                        // Transition state machine to error
                        self.tool_state_machine.transition_to_error(&tool_id, conversation_id, &e.to_string()).await?;
                        
                        Ok(vec![
                            TeaEvent::ToolInvocationFailed {
                                conversation_id,
                                tool_id,
                                error: e.to_string(),
                                timestamp: Utc::now(),
                            }
                        ])
                    }
                }
            }
            _ => Err(BridgeError::SystemError("Invalid command for tool invocation system".to_string())),
        }
    }
    
    fn system_name(&self) -> &'static str {
        "tools"
    }
    
    async fn health_check(&self) -> Result<ComponentHealthStatus, BridgeError> {
        let start_time = std::time::Instant::now();
        
        match self.nats_adapter.health_check().await {
            Ok(_) => Ok(ComponentHealthStatus {
                status: HealthStatus::Healthy,
                response_time: Some(start_time.elapsed()),
                error_rate: None,
                last_error: None,
            }),
            Err(e) => Ok(ComponentHealthStatus {
                status: HealthStatus::Unhealthy,
                response_time: Some(start_time.elapsed()),
                error_rate: None,
                last_error: Some(e.to_string()),
            }),
        }
    }
}

/// State machine for message processing
pub struct MessageStateMachine {
    states: Arc<RwLock<HashMap<EntityId, MessageState>>>,
}

impl MessageStateMachine {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn transition_to_sending(&self, conversation_id: EntityId) -> Result<MessageState, BridgeError> {
        let mut states = self.states.write().await;
        let new_state = MessageState::Sending { started_at: Utc::now() };
        states.insert(conversation_id, new_state.clone());
        Ok(new_state)
    }
    
    pub async fn transition_to_sent(&self, conversation_id: EntityId) -> Result<MessageState, BridgeError> {
        let mut states = self.states.write().await;
        let new_state = MessageState::Sent { sent_at: Utc::now() };
        states.insert(conversation_id, new_state.clone());
        Ok(new_state)
    }
    
    pub async fn transition_to_error(&self, conversation_id: EntityId, error: &str) -> Result<MessageState, BridgeError> {
        let mut states = self.states.write().await;
        let new_state = MessageState::Error {
            error: error.to_string(),
            occurred_at: Utc::now(),
        };
        states.insert(conversation_id, new_state.clone());
        Ok(new_state)
    }
}

/// Message processing states
#[derive(Debug, Clone)]
pub enum MessageState {
    Idle,
    Sending { started_at: DateTime<Utc> },
    Sent { sent_at: DateTime<Utc> },
    Error { error: String, occurred_at: DateTime<Utc> },
}

/// State machine for conversation lifecycle
pub struct ConversationStateMachine {
    states: Arc<RwLock<HashMap<EntityId, ConversationLifecycleState>>>,
}

impl ConversationStateMachine {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn transition_to_active(&self, conversation_id: EntityId) -> Result<(), BridgeError> {
        let mut states = self.states.write().await;
        states.insert(conversation_id, ConversationLifecycleState::Active { created_at: Utc::now() });
        Ok(())
    }
    
    pub async fn transition_to_paused(&self, conversation_id: EntityId) -> Result<(), BridgeError> {
        let mut states = self.states.write().await;
        states.insert(conversation_id, ConversationLifecycleState::Paused { paused_at: Utc::now() });
        Ok(())
    }
    
    pub async fn transition_to_archived(&self, conversation_id: EntityId, reason: Option<String>) -> Result<(), BridgeError> {
        let mut states = self.states.write().await;
        states.insert(conversation_id, ConversationLifecycleState::Archived {
            archived_at: Utc::now(),
            reason,
        });
        Ok(())
    }
}

/// Conversation lifecycle states
#[derive(Debug, Clone)]
pub enum ConversationLifecycleState {
    Active { created_at: DateTime<Utc> },
    Paused { paused_at: DateTime<Utc> },
    Archived { archived_at: DateTime<Utc>, reason: Option<String> },
}

/// State machine for tool invocations
pub struct ToolStateMachine {
    states: Arc<RwLock<HashMap<String, ToolInvocationState>>>,
}

impl ToolStateMachine {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn transition_to_invoking(&self, tool_id: &str, conversation_id: EntityId) -> Result<(), BridgeError> {
        let mut states = self.states.write().await;
        let key = format!("{}:{}", tool_id, conversation_id);
        states.insert(key, ToolInvocationState::Invoking { started_at: Utc::now() });
        Ok(())
    }
    
    pub async fn transition_to_completed(&self, tool_id: &str, conversation_id: EntityId) -> Result<(), BridgeError> {
        let mut states = self.states.write().await;
        let key = format!("{}:{}", tool_id, conversation_id);
        states.insert(key, ToolInvocationState::Completed { completed_at: Utc::now() });
        Ok(())
    }
    
    pub async fn transition_to_error(&self, tool_id: &str, conversation_id: EntityId, error: &str) -> Result<(), BridgeError> {
        let mut states = self.states.write().await;
        let key = format!("{}:{}", tool_id, conversation_id);
        states.insert(key, ToolInvocationState::Error {
            error: error.to_string(),
            occurred_at: Utc::now(),
        });
        Ok(())
    }
}

/// Tool invocation states
#[derive(Debug, Clone)]
pub enum ToolInvocationState {
    Idle,
    Invoking { started_at: DateTime<Utc> },
    Completed { completed_at: DateTime<Utc> },
    Error { error: String, occurred_at: DateTime<Utc> },
}