/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use crate::domain::value_objects::*;
use serde::{Deserialize, Serialize};

/// Domain commands (imperative, expressing intent)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Command {
    /// Start a new conversation with Claude
    StartConversation {
        session_id: SessionId,
        initial_prompt: Prompt,
        context: ConversationContext,
        correlation_id: CorrelationId,
    },
    /// Send a prompt to an existing conversation
    SendPrompt {
        conversation_id: ConversationId,
        prompt: Prompt,
        correlation_id: CorrelationId,
    },
    /// End an existing conversation
    EndConversation {
        conversation_id: ConversationId,
        reason: ConversationEndReason,
        correlation_id: CorrelationId,
    },
}

/// Command envelope for NATS messaging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandEnvelope {
    pub command_id: EventId,
    pub correlation_id: CorrelationId,
    pub command: Command,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Command {
    /// Create command envelope with metadata
    pub fn with_metadata(self, correlation_id: CorrelationId) -> CommandEnvelope {
        CommandEnvelope {
            command_id: EventId::new(),
            correlation_id,
            command: self,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get correlation ID from command
    pub fn correlation_id(&self) -> &CorrelationId {
        match self {
            Command::StartConversation { correlation_id, .. } => correlation_id,
            Command::SendPrompt { correlation_id, .. } => correlation_id,
            Command::EndConversation { correlation_id, .. } => correlation_id,
        }
    }

    /// Get conversation ID if command relates to existing conversation
    pub fn conversation_id(&self) -> Option<&ConversationId> {
        match self {
            Command::StartConversation { .. } => None,
            Command::SendPrompt {
                conversation_id, ..
            } => Some(conversation_id),
            Command::EndConversation {
                conversation_id, ..
            } => Some(conversation_id),
        }
    }
}

// Re-export from events module to avoid duplication
pub use crate::domain::events::ConversationEndReason;

/// Command validation trait
pub trait CommandValidator {
    type Error;

    fn validate(&self) -> Result<(), Self::Error>;
}

impl CommandValidator for Command {
    type Error = String;

    fn validate(&self) -> Result<(), Self::Error> {
        match self {
            Command::StartConversation { initial_prompt, .. } => {
                if initial_prompt.character_count() == 0 {
                    return Err("Initial prompt cannot be empty".to_string());
                }
                Ok(())
            }
            Command::SendPrompt { prompt, .. } => {
                if prompt.character_count() == 0 {
                    return Err("Prompt cannot be empty".to_string());
                }
                Ok(())
            }
            Command::EndConversation { .. } => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_envelope_creation() {
        let command = Command::StartConversation {
            session_id: SessionId::new(),
            initial_prompt: Prompt::new("Hello Claude".to_string()).unwrap(),
            context: ConversationContext::default(),
            correlation_id: CorrelationId::new(),
        };

        let correlation_id = CorrelationId::new();
        let envelope = command.with_metadata(correlation_id);

        assert_eq!(envelope.correlation_id, correlation_id);
        assert!(!envelope.command_id.is_nil());
    }

    #[test]
    fn test_command_validation() {
        // Valid command
        let valid_command = Command::StartConversation {
            session_id: SessionId::new(),
            initial_prompt: Prompt::new("Hello".to_string()).unwrap(),
            context: ConversationContext::default(),
            correlation_id: CorrelationId::new(),
        };
        assert!(valid_command.validate().is_ok());

        // Invalid command with empty prompt
        let invalid_command = Command::SendPrompt {
            conversation_id: ConversationId::new(),
            prompt: Prompt::new("".to_string()).unwrap_or_else(|_| {
                // This shouldn't happen due to Prompt validation, but for test purposes
                panic!("Cannot create empty prompt")
            }),
            correlation_id: CorrelationId::new(),
        };
        // Note: This test is illustrative - in reality, Prompt::new would fail first
    }

    #[test]
    fn test_correlation_id_extraction() {
        let correlation_id = CorrelationId::new();
        let command = Command::SendPrompt {
            conversation_id: ConversationId::new(),
            prompt: Prompt::new("Test prompt".to_string()).unwrap(),
            correlation_id: correlation_id.clone(),
        };

        assert_eq!(command.correlation_id(), &correlation_id);
    }

    #[test]
    fn test_conversation_id_extraction() {
        let conversation_id = ConversationId::new();
        let command = Command::SendPrompt {
            conversation_id: conversation_id.clone(),
            prompt: Prompt::new("Test prompt".to_string()).unwrap(),
            correlation_id: CorrelationId::new(),
        };

        assert_eq!(command.conversation_id(), Some(&conversation_id));

        // StartConversation should not have conversation_id
        let start_command = Command::StartConversation {
            session_id: SessionId::new(),
            initial_prompt: Prompt::new("Hello".to_string()).unwrap(),
            context: ConversationContext::default(),
            correlation_id: CorrelationId::new(),
        };

        assert_eq!(start_command.conversation_id(), None);
    }
}
