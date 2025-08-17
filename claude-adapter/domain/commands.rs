// Domain Commands (Imperative, Intent-Focused)
// Represent user intentions and business operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::conversation_aggregate::{
    ConversationId, SessionId, CorrelationId, Prompt, EndReason
};

// === Commands (Imperative Intent) ===

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationCommand {
    StartConversation(StartConversationCommand),
    SendPrompt(SendPromptCommand),
    EndConversation(EndConversationCommand),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StartConversationCommand {
    pub session_id: SessionId,
    pub initial_prompt: Prompt,
    pub correlation_id: CorrelationId,
    pub requested_at: DateTime<Utc>,
    pub requester_metadata: HashMap<String, String>,
}

impl StartConversationCommand {
    pub fn new(
        session_id: SessionId,
        initial_prompt: Prompt,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            session_id,
            initial_prompt,
            correlation_id,
            requested_at: Utc::now(),
            requester_metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.requester_metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendPromptCommand {
    pub conversation_id: ConversationId,
    pub prompt: Prompt,
    pub correlation_id: CorrelationId,
    pub requested_at: DateTime<Utc>,
    pub requester_metadata: HashMap<String, String>,
}

impl SendPromptCommand {
    pub fn new(
        conversation_id: ConversationId,
        prompt: Prompt,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            conversation_id,
            prompt,
            correlation_id,
            requested_at: Utc::now(),
            requester_metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.requester_metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndConversationCommand {
    pub conversation_id: ConversationId,
    pub reason: EndReason,
    pub correlation_id: CorrelationId,
    pub requested_at: DateTime<Utc>,
    pub requester_metadata: HashMap<String, String>,
}

impl EndConversationCommand {
    pub fn new(
        conversation_id: ConversationId,
        reason: EndReason,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            conversation_id,
            reason,
            correlation_id,
            requested_at: Utc::now(),
            requester_metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.requester_metadata = metadata;
        self
    }
}

// === Command Validation ===

pub trait CommandValidator<T> {
    type Error;
    
    fn validate(&self, command: &T) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub struct ConversationCommandValidator;

impl CommandValidator<StartConversationCommand> for ConversationCommandValidator {
    type Error = CommandValidationError;
    
    fn validate(&self, command: &StartConversationCommand) -> Result<(), Self::Error> {
        if command.initial_prompt.content().is_empty() {
            return Err(CommandValidationError::EmptyPrompt);
        }
        
        if command.initial_prompt.content().len() > 50000 {
            return Err(CommandValidationError::PromptTooLong);
        }
        
        Ok(())
    }
}

impl CommandValidator<SendPromptCommand> for ConversationCommandValidator {
    type Error = CommandValidationError;
    
    fn validate(&self, command: &SendPromptCommand) -> Result<(), Self::Error> {
        if command.prompt.content().is_empty() {
            return Err(CommandValidationError::EmptyPrompt);
        }
        
        if command.prompt.content().len() > 50000 {
            return Err(CommandValidationError::PromptTooLong);
        }
        
        Ok(())
    }
}

impl CommandValidator<EndConversationCommand> for ConversationCommandValidator {
    type Error = CommandValidationError;
    
    fn validate(&self, _command: &EndConversationCommand) -> Result<(), Self::Error> {
        // End conversation commands are always valid
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandValidationError {
    EmptyPrompt,
    PromptTooLong,
    InvalidSessionId,
    InvalidConversationId,
    MissingCorrelationId,
}

// === Command Results ===

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandResult {
    ConversationStarted {
        conversation_id: ConversationId,
        correlation_id: CorrelationId,
    },
    PromptSent {
        conversation_id: ConversationId,
        correlation_id: CorrelationId,
    },
    ConversationEnded {
        conversation_id: ConversationId,
        correlation_id: CorrelationId,
    },
}

// === Command Handler Trait ===

pub trait CommandHandler<TCommand> {
    type Result;
    type Error;
    
    async fn handle(&self, command: TCommand) -> Result<Self::Result, Self::Error>;
}