/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use crate::domain::{commands::*, errors::*, events::*, value_objects::*};
use chrono::{DateTime, Duration, Utc};
use std::collections::VecDeque;

/// Conversation aggregate - manages the lifecycle of a Claude conversation
#[derive(Debug, Clone, PartialEq)]
pub struct ConversationAggregate {
    id: ConversationId,
    session_id: SessionId,
    state: ConversationState,
    context: ConversationContext,
    exchanges: VecDeque<Exchange>,
    correlation_chains: Vec<CorrelationChain>,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    version: u64,
    pending_events: Vec<DomainEvent>,
}

/// Conversation state machine
#[derive(Debug, Clone, PartialEq)]
pub enum ConversationState {
    Draft,      // Initial state, not yet started
    Processing, // Awaiting Claude response
    Responded,  // Response received, ready for next prompt
    Ended,      // Conversation terminated
}

/// Exchange represents a prompt-response pair
#[derive(Debug, Clone, PartialEq)]
pub struct Exchange {
    sequence_number: u32,
    prompt: Prompt,
    response: Option<ClaudeResponse>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

/// Correlation chain for tracking event causation
#[derive(Debug, Clone, PartialEq)]
pub struct CorrelationChain {
    correlation_id: CorrelationId,
    events: Vec<EventId>,
}

impl ConversationAggregate {
    /// Business rules constants
    const MAX_PROMPTS_PER_MINUTE: u32 = 10;
    const MAX_CONTEXT_RETENTION_HOURS: i64 = 24;
    const MAX_EXCHANGES_PER_CONVERSATION: u32 = 100;

    /// Create new conversation aggregate from StartConversation command
    pub fn from_command(
        command: Command,
        correlation_id: CorrelationId,
    ) -> Result<Self, DomainError> {
        match command {
            Command::StartConversation {
                session_id,
                initial_prompt,
                context,
                correlation_id: cmd_correlation_id,
            } => {
                // Validate correlation IDs match
                if correlation_id != cmd_correlation_id {
                    return Err(DomainError::CorrelationMismatch);
                }

                let conversation_id = ConversationId::new();
                let now = Utc::now();

                // Create initial exchange
                let initial_exchange = Exchange {
                    sequence_number: 1,
                    prompt: initial_prompt.clone(),
                    response: None,
                    started_at: now,
                    completed_at: None,
                };

                let mut exchanges = VecDeque::new();
                exchanges.push_back(initial_exchange);

                let mut aggregate = Self {
                    id: conversation_id.clone(),
                    session_id: session_id.clone(),
                    state: ConversationState::Processing,
                    context,
                    exchanges,
                    correlation_chains: vec![CorrelationChain {
                        correlation_id: correlation_id.clone(),
                        events: vec![],
                    }],
                    created_at: now,
                    last_activity: now,
                    version: 1,
                    pending_events: vec![],
                };

                // Record ConversationStarted event
                aggregate.record_event(DomainEvent::ConversationStarted {
                    conversation_id,
                    session_id,
                    initial_prompt,
                    context: aggregate.context.clone(),
                });

                // Record PromptSent event
                aggregate.record_event(DomainEvent::PromptSent {
                    conversation_id: aggregate.id.clone(),
                    prompt: aggregate.exchanges[0].prompt.clone(),
                    sequence_number: 1,
                    claude_request_metadata: ClaudeRequestMetadata::default(),
                });

                Ok(aggregate)
            }
            _ => Err(DomainError::InvalidCommand(
                "Expected StartConversation command".to_string(),
            )),
        }
    }

    /// Handle SendPrompt command
    pub fn handle_command(
        &mut self,
        command: Command,
        correlation_id: CorrelationId,
    ) -> Result<Vec<DomainEvent>, DomainError> {
        // Validate aggregate is not ended
        if self.state == ConversationState::Ended {
            return Err(DomainError::ConversationEnded);
        }

        match command {
            Command::SendPrompt {
                conversation_id,
                prompt,
                correlation_id: cmd_correlation_id,
            } => {
                // Validate IDs match
                if conversation_id != self.id {
                    return Err(DomainError::ConversationNotFound);
                }
                if correlation_id != cmd_correlation_id {
                    return Err(DomainError::CorrelationMismatch);
                }

                // Validate business rules
                self.validate_rate_limit()?;
                self.validate_exchange_limit()?;

                // Validate state transition
                if self.state != ConversationState::Responded {
                    return Err(DomainError::InvalidStateTransition {
                        from: self.state.clone(),
                        to: ConversationState::Processing,
                    });
                }

                let sequence_number = self.exchanges.len() as u32 + 1;
                let now = Utc::now();

                // Add new exchange
                let exchange = Exchange {
                    sequence_number,
                    prompt: prompt.clone(),
                    response: None,
                    started_at: now,
                    completed_at: None,
                };
                self.exchanges.push_back(exchange);

                // Update state and activity
                self.state = ConversationState::Processing;
                self.last_activity = now;
                self.version += 1;

                // Record event
                self.record_event(DomainEvent::PromptSent {
                    conversation_id: self.id.clone(),
                    prompt,
                    sequence_number,
                    claude_request_metadata: ClaudeRequestMetadata::default(),
                });

                Ok(self.drain_events())
            }
            Command::EndConversation {
                conversation_id,
                reason,
                correlation_id: cmd_correlation_id,
            } => {
                // Validate IDs match
                if conversation_id != self.id {
                    return Err(DomainError::ConversationNotFound);
                }
                if correlation_id != cmd_correlation_id {
                    return Err(DomainError::CorrelationMismatch);
                }

                self.state = ConversationState::Ended;
                self.last_activity = Utc::now();
                self.version += 1;

                // Calculate total usage
                let total_tokens_used = self
                    .exchanges
                    .iter()
                    .filter_map(|e| e.response.as_ref())
                    .fold(TokenUsage::default(), |acc, resp| {
                        TokenUsage::new(
                            acc.input_tokens() + resp.usage().input_tokens(),
                            acc.output_tokens() + resp.usage().output_tokens(),
                        )
                    });

                self.record_event(DomainEvent::ConversationEnded {
                    conversation_id: self.id.clone(),
                    reason,
                    total_exchanges: self.exchanges.len() as u32,
                    total_tokens_used,
                });

                Ok(self.drain_events())
            }
            _ => Err(DomainError::InvalidCommand(
                "Unsupported command for existing conversation".to_string(),
            )),
        }
    }

    /// Handle response received from Claude API
    pub fn apply_response(
        &mut self,
        response: ClaudeResponse,
        processing_duration_ms: u64,
    ) -> Result<Vec<DomainEvent>, DomainError> {
        // Validate state
        if self.state != ConversationState::Processing {
            return Err(DomainError::InvalidStateTransition {
                from: self.state.clone(),
                to: ConversationState::Responded,
            });
        }

        // Update current exchange with response
        if let Some(current_exchange) = self.exchanges.back_mut() {
            current_exchange.response = Some(response.clone());
            current_exchange.completed_at = Some(Utc::now());
        } else {
            return Err(DomainError::NoActiveExchange);
        }

        // Update state and activity
        self.state = ConversationState::Responded;
        self.last_activity = Utc::now();
        self.version += 1;

        // Record event
        self.record_event(DomainEvent::ResponseReceived {
            conversation_id: self.id.clone(),
            response,
            sequence_number: self.exchanges.len() as u32,
            processing_duration_ms,
        });

        Ok(self.drain_events())
    }

    /// Validate rate limiting business rule
    fn validate_rate_limit(&self) -> Result<(), DomainError> {
        let one_minute_ago = Utc::now() - Duration::minutes(1);
        let recent_prompts = self
            .exchanges
            .iter()
            .filter(|e| e.started_at > one_minute_ago)
            .count() as u32;

        if recent_prompts >= Self::MAX_PROMPTS_PER_MINUTE {
            return Err(DomainError::RateLimitExceeded);
        }

        Ok(())
    }

    /// Validate exchange limit business rule
    fn validate_exchange_limit(&self) -> Result<(), DomainError> {
        if self.exchanges.len() as u32 >= Self::MAX_EXCHANGES_PER_CONVERSATION {
            return Err(DomainError::ExchangeLimitExceeded);
        }

        Ok(())
    }

    /// Record domain event
    fn record_event(&mut self, event: DomainEvent) {
        self.pending_events.push(event);
    }

    /// Drain pending events (for event publishing)
    pub fn drain_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.pending_events)
    }

    // Getters
    pub fn id(&self) -> &ConversationId {
        &self.id
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn state(&self) -> &ConversationState {
        &self.state
    }

    pub fn context(&self) -> &ConversationContext {
        &self.context
    }

    pub fn exchanges(&self) -> &VecDeque<Exchange> {
        &self.exchanges
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn last_activity(&self) -> DateTime<Utc> {
        self.last_activity
    }

    /// Check if conversation has expired
    pub fn is_expired(&self) -> bool {
        let expiry_time = self.last_activity + Duration::hours(Self::MAX_CONTEXT_RETENTION_HOURS);
        Utc::now() > expiry_time
    }
}

impl CorrelationChain {
    pub fn new(correlation_id: CorrelationId) -> Self {
        Self {
            correlation_id,
            events: vec![],
        }
    }

    pub fn add_event(&mut self, event_id: EventId) {
        self.events.push(event_id);
    }

    pub fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }

    pub fn events(&self) -> &[EventId] {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_start_command() -> (Command, CorrelationId) {
        let correlation_id = CorrelationId::new();
        let command = Command::StartConversation {
            session_id: SessionId::new(),
            initial_prompt: Prompt::new("Hello Claude".to_string()).unwrap(),
            context: ConversationContext::default(),
            correlation_id: correlation_id.clone(),
        };
        (command, correlation_id)
    }

    #[test]
    fn test_conversation_creation() {
        let (command, correlation_id) = create_start_command();
        let aggregate = ConversationAggregate::from_command(command, correlation_id).unwrap();

        assert_eq!(aggregate.state, ConversationState::Processing);
        assert_eq!(aggregate.exchanges.len(), 1);
        assert_eq!(aggregate.version, 1);
        assert!(!aggregate.pending_events.is_empty());
    }

    #[test]
    fn test_send_prompt_command() {
        let (command, correlation_id) = create_start_command();
        let mut aggregate = ConversationAggregate::from_command(command, correlation_id).unwrap();

        // First, apply a response to get to Responded state
        let response = ClaudeResponse::new(
            "Hello! How can I help?".to_string(),
            TokenUsage::new(10, 15),
            "stop".to_string(),
            "claude-3-sonnet".to_string(),
        );
        aggregate.apply_response(response, 1000).unwrap();

        // Now send another prompt
        let prompt_correlation_id = CorrelationId::new();
        let send_prompt_cmd = Command::SendPrompt {
            conversation_id: aggregate.id().clone(),
            prompt: Prompt::new("What's the weather?".to_string()).unwrap(),
            correlation_id: prompt_correlation_id.clone(),
        };

        let events = aggregate
            .handle_command(send_prompt_cmd, prompt_correlation_id)
            .unwrap();

        assert_eq!(aggregate.state, ConversationState::Processing);
        assert_eq!(aggregate.exchanges.len(), 2);
        assert!(!events.is_empty());
    }

    #[test]
    fn test_rate_limiting() {
        let (command, correlation_id) = create_start_command();
        let mut aggregate = ConversationAggregate::from_command(command, correlation_id).unwrap();

        // Set up exchanges to exceed rate limit
        for i in 0..ConversationAggregate::MAX_PROMPTS_PER_MINUTE {
            let exchange = Exchange {
                sequence_number: i + 1,
                prompt: Prompt::new(format!("Prompt {}", i)).unwrap(),
                response: None,
                started_at: Utc::now(),
                completed_at: None,
            };
            aggregate.exchanges.push_back(exchange);
        }

        // Try to send another prompt
        let fail_correlation_id = CorrelationId::new();
        let send_prompt_cmd = Command::SendPrompt {
            conversation_id: aggregate.id().clone(),
            prompt: Prompt::new("This should fail".to_string()).unwrap(),
            correlation_id: fail_correlation_id.clone(),
        };

        let result = aggregate.handle_command(send_prompt_cmd, fail_correlation_id);
        assert!(matches!(result, Err(DomainError::RateLimitExceeded)));
    }

    #[test]
    fn test_conversation_expiry() {
        let (command, correlation_id) = create_start_command();
        let mut aggregate = ConversationAggregate::from_command(command, correlation_id).unwrap();

        // Set last activity to more than 24 hours ago
        aggregate.last_activity = Utc::now() - Duration::hours(25);

        assert!(aggregate.is_expired());
    }
}
