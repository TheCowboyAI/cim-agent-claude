/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use crate::domain::value_objects::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Domain events for the Claude API adapter (past tense, business-focused)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomainEvent {
    /// A new conversation was initiated with Claude
    ConversationStarted {
        conversation_id: ConversationId,
        session_id: SessionId,
        initial_prompt: Prompt,
        context: ConversationContext,
    },
    /// A prompt was sent to the Claude API
    PromptSent {
        conversation_id: ConversationId,
        prompt: Prompt,
        sequence_number: u32,
        claude_request_metadata: ClaudeRequestMetadata,
    },
    /// A response was received from Claude API
    ResponseReceived {
        conversation_id: ConversationId,
        response: ClaudeResponse,
        sequence_number: u32,
        processing_duration_ms: u64,
    },
    /// A conversation was terminated
    ConversationEnded {
        conversation_id: ConversationId,
        reason: ConversationEndReason,
        total_exchanges: u32,
        total_tokens_used: TokenUsage,
    },
    /// Rate limit was exceeded for a conversation
    RateLimitExceeded {
        conversation_id: ConversationId,
        limit_type: RateLimitType,
        retry_after_seconds: u32,
    },
    /// Error occurred during Claude API communication
    ClaudeApiErrorOccurred {
        conversation_id: ConversationId,
        error_type: ClaudeApiErrorType,
        error_message: String,
        retry_count: u32,
    },
}

/// Event envelope with correlation and causation tracking (CIM standard)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub causation_id: EventId,
    pub event: DomainEvent,
    pub timestamp: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent {
    /// Create event envelope with proper correlation/causation tracking
    pub fn with_metadata(
        self,
        correlation_id: CorrelationId,
        causation_id: Option<EventId>,
    ) -> EventEnvelope {
        EventEnvelope {
            event_id: EventId::new(),
            correlation_id,
            causation_id: causation_id.unwrap_or_else(EventId::new),
            event: self,
            timestamp: Utc::now(),
            version: 1,
        }
    }

    /// Get conversation ID from any domain event
    pub fn conversation_id(&self) -> &ConversationId {
        match self {
            DomainEvent::ConversationStarted {
                conversation_id, ..
            } => conversation_id,
            DomainEvent::PromptSent {
                conversation_id, ..
            } => conversation_id,
            DomainEvent::ResponseReceived {
                conversation_id, ..
            } => conversation_id,
            DomainEvent::ConversationEnded {
                conversation_id, ..
            } => conversation_id,
            DomainEvent::RateLimitExceeded {
                conversation_id, ..
            } => conversation_id,
            DomainEvent::ClaudeApiErrorOccurred {
                conversation_id, ..
            } => conversation_id,
        }
    }
}

/// Conversation end reasons
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversationEndReason {
    UserRequested,
    Timeout,
    RateLimitExceeded,
    ApiError,
    MaxExchangesReached,
    InvalidState,
}

/// Rate limit types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RateLimitType {
    PromptsPerMinute,
    TokensPerHour,
    ConcurrentRequests,
}

/// Claude API error types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaudeApiErrorType {
    Authentication,
    RateLimit,
    Timeout,
    ServerError,
    ValidationError,
    NetworkError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_envelope_creation() {
        let event = DomainEvent::ConversationStarted {
            conversation_id: ConversationId::new(),
            session_id: SessionId::new(),
            initial_prompt: Prompt::new("Hello Claude".to_string()).unwrap(),
            context: ConversationContext::default(),
        };

        let correlation_id = CorrelationId::new();
        let envelope = event.with_metadata(correlation_id.clone(), None);

        assert_eq!(envelope.correlation_id, correlation_id);
        assert!(!envelope.event_id.is_nil());
        assert!(!envelope.causation_id.is_nil());
        assert_eq!(envelope.version, 1);
    }

    #[test]
    fn test_conversation_id_extraction() {
        let conversation_id = ConversationId::new();
        let event = DomainEvent::PromptSent {
            conversation_id: conversation_id.clone(),
            prompt: Prompt::new("Test".to_string()).unwrap(),
            sequence_number: 1,
            claude_request_metadata: ClaudeRequestMetadata::default(),
        };

        assert_eq!(event.conversation_id(), &conversation_id);
    }
}
