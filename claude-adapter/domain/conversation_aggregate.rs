// Domain Aggregate: ConversationAggregate
// Manages conversation state and enforces business invariants

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// === Value Objects (Invariant Groups) ===

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationId(Uuid);

impl ConversationId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }
    
    pub fn value(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }
    
    pub fn value(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }
    
    pub fn value(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn value(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prompt {
    content: String,
    context: Vec<String>,
    metadata: HashMap<String, String>,
}

impl Prompt {
    pub fn new(content: String) -> Result<Self, DomainError> {
        if content.is_empty() {
            return Err(DomainError::EmptyPrompt);
        }
        
        if content.len() > 50000 {
            return Err(DomainError::PromptTooLong(content.len()));
        }
        
        Ok(Self {
            content: content.trim().to_string(),
            context: Vec::new(),
            metadata: HashMap::new(),
        })
    }
    
    pub fn with_context(mut self, context: Vec<String>) -> Self {
        self.context = context;
        self
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn content(&self) -> &str {
        &self.content
    }
    
    pub fn context(&self) -> &[String] {
        &self.context
    }
    
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeResponse {
    content: String,
    model: String,
    usage_tokens: u32,
    metadata: HashMap<String, String>,
}

impl ClaudeResponse {
    pub fn new(content: String, model: String, usage_tokens: u32) -> Self {
        Self {
            content,
            model,
            usage_tokens,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn content(&self) -> &str {
        &self.content
    }
    
    pub fn model(&self) -> &str {
        &self.model
    }
    
    pub fn usage_tokens(&self) -> u32 {
        self.usage_tokens
    }
}

// === Domain State Machine ===

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationState {
    Started {
        session_id: SessionId,
        created_at: DateTime<Utc>,
    },
    Processing {
        session_id: SessionId,
        prompt_sent_at: DateTime<Utc>,
        correlation_id: CorrelationId,
    },
    Responded {
        session_id: SessionId,
        last_response_at: DateTime<Utc>,
        total_exchanges: u32,
    },
    Ended {
        session_id: SessionId,
        ended_at: DateTime<Utc>,
        reason: EndReason,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EndReason {
    UserRequested,
    Timeout,
    Error(String),
    TokenLimitReached,
}

// === Exchange Value Object ===

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Exchange {
    id: EventId,
    prompt: Prompt,
    response: Option<ClaudeResponse>,
    created_at: DateTime<Utc>,
    responded_at: Option<DateTime<Utc>>,
    correlation_id: CorrelationId,
}

impl Exchange {
    pub fn new(prompt: Prompt, correlation_id: CorrelationId) -> Self {
        Self {
            id: EventId::generate(),
            prompt,
            response: None,
            created_at: Utc::now(),
            responded_at: None,
            correlation_id,
        }
    }
    
    pub fn complete_with_response(mut self, response: ClaudeResponse) -> Self {
        self.response = Some(response);
        self.responded_at = Some(Utc::now());
        self
    }
    
    pub fn is_complete(&self) -> bool {
        self.response.is_some()
    }
    
    pub fn id(&self) -> &EventId {
        &self.id
    }
    
    pub fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }
}

// === Domain Errors ===

#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    EmptyPrompt,
    PromptTooLong(usize),
    InvalidStateTransition {
        from: ConversationState,
        command: String,
    },
    ConversationEnded,
    TooManyPrompts {
        current: u32,
        limit: u32,
    },
    ContextWindowExceeded,
    ExchangeNotFound(EventId),
}

// === Business Invariants ===

#[derive(Debug, Clone)]
pub struct ConversationInvariants {
    max_prompts_per_minute: u32,
    context_retention_hours: u32,
    max_prompt_length: usize,
}

impl Default for ConversationInvariants {
    fn default() -> Self {
        Self {
            max_prompts_per_minute: 10,
            context_retention_hours: 24,
            max_prompt_length: 50000,
        }
    }
}

impl ConversationInvariants {
    pub fn validate_prompt_rate(&self, recent_prompts: u32) -> Result<(), DomainError> {
        if recent_prompts >= self.max_prompts_per_minute {
            return Err(DomainError::TooManyPrompts {
                current: recent_prompts,
                limit: self.max_prompts_per_minute,
            });
        }
        Ok(())
    }
    
    pub fn validate_prompt_length(&self, prompt: &Prompt) -> Result<(), DomainError> {
        if prompt.content().len() > self.max_prompt_length {
            return Err(DomainError::PromptTooLong(prompt.content().len()));
        }
        Ok(())
    }
}

// === Conversation Aggregate Root ===

#[derive(Debug, Clone)]
pub struct ConversationAggregate {
    id: ConversationId,
    state: ConversationState,
    exchanges: Vec<Exchange>,
    context_window: Vec<String>,
    invariants: ConversationInvariants,
    version: u64,
}

impl ConversationAggregate {
    // Factory method to start new conversation
    pub fn start_conversation(
        session_id: SessionId,
        initial_prompt: Prompt,
        correlation_id: CorrelationId,
    ) -> Result<(Self, Vec<DomainEvent>), DomainError> {
        let invariants = ConversationInvariants::default();
        invariants.validate_prompt_length(&initial_prompt)?;
        
        let conversation_id = ConversationId::generate();
        let exchange = Exchange::new(initial_prompt.clone(), correlation_id);
        
        let aggregate = Self {
            id: conversation_id,
            state: ConversationState::Started {
                session_id,
                created_at: Utc::now(),
            },
            exchanges: vec![exchange],
            context_window: vec![initial_prompt.content().to_string()],
            invariants,
            version: 1,
        };
        
        let events = vec![
            DomainEvent::ConversationStarted {
                conversation_id: aggregate.id,
                session_id,
                initial_prompt,
                correlation_id,
                started_at: Utc::now(),
            }
        ];
        
        Ok((aggregate, events))
    }
    
    // Send prompt command
    pub fn send_prompt(&mut self, prompt: Prompt, correlation_id: CorrelationId) -> Result<Vec<DomainEvent>, DomainError> {
        self.invariants.validate_prompt_length(&prompt)?;
        
        match &self.state {
            ConversationState::Started { session_id, .. } | 
            ConversationState::Responded { session_id, .. } => {
                // Transition to Processing state
                self.state = ConversationState::Processing {
                    session_id: *session_id,
                    prompt_sent_at: Utc::now(),
                    correlation_id,
                };
                
                // Create new exchange
                let exchange = Exchange::new(prompt.clone(), correlation_id);
                let event_id = *exchange.id();
                self.exchanges.push(exchange);
                self.context_window.push(prompt.content().to_string());
                self.version += 1;
                
                Ok(vec![DomainEvent::PromptSent {
                    conversation_id: self.id,
                    prompt,
                    correlation_id,
                    event_id,
                    sent_at: Utc::now(),
                }])
            },
            ConversationState::Processing { .. } => {
                Err(DomainError::InvalidStateTransition {
                    from: self.state.clone(),
                    command: "SendPrompt".to_string(),
                })
            },
            ConversationState::Ended { .. } => {
                Err(DomainError::ConversationEnded)
            },
        }
    }
    
    // Receive response event
    pub fn receive_response(&mut self, response: ClaudeResponse, causation_id: EventId) -> Result<Vec<DomainEvent>, DomainError> {
        match &self.state {
            ConversationState::Processing { session_id, correlation_id, .. } => {
                // Find the exchange to complete
                let exchange_pos = self.exchanges.iter()
                    .position(|e| e.id() == &causation_id)
                    .ok_or(DomainError::ExchangeNotFound(causation_id))?;
                
                // Complete the exchange
                let exchange = self.exchanges.remove(exchange_pos);
                let completed_exchange = exchange.complete_with_response(response.clone());
                self.exchanges.insert(exchange_pos, completed_exchange);
                
                // Transition to Responded state
                let total_exchanges = self.exchanges.len() as u32;
                self.state = ConversationState::Responded {
                    session_id: *session_id,
                    last_response_at: Utc::now(),
                    total_exchanges,
                };
                
                self.context_window.push(response.content().to_string());
                self.version += 1;
                
                Ok(vec![DomainEvent::ResponseReceived {
                    conversation_id: self.id,
                    response,
                    causation_id,
                    correlation_id: *correlation_id,
                    received_at: Utc::now(),
                }])
            },
            _ => Err(DomainError::InvalidStateTransition {
                from: self.state.clone(),
                command: "ReceiveResponse".to_string(),
            })
        }
    }
    
    // End conversation
    pub fn end_conversation(&mut self, reason: EndReason) -> Result<Vec<DomainEvent>, DomainError> {
        match &self.state {
            ConversationState::Ended { .. } => {
                Err(DomainError::ConversationEnded)
            },
            ConversationState::Started { session_id, .. } |
            ConversationState::Processing { session_id, .. } |
            ConversationState::Responded { session_id, .. } => {
                let session_id = *session_id;
                self.state = ConversationState::Ended {
                    session_id,
                    ended_at: Utc::now(),
                    reason: reason.clone(),
                };
                self.version += 1;
                
                Ok(vec![DomainEvent::ConversationEnded {
                    conversation_id: self.id,
                    session_id,
                    reason,
                    ended_at: Utc::now(),
                }])
            }
        }
    }
    
    // Getters
    pub fn id(&self) -> ConversationId {
        self.id
    }
    
    pub fn state(&self) -> &ConversationState {
        &self.state
    }
    
    pub fn exchanges(&self) -> &[Exchange] {
        &self.exchanges
    }
    
    pub fn version(&self) -> u64 {
        self.version
    }
}

// === Domain Events (Past Tense, Business-Focused) ===

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DomainEvent {
    ConversationStarted {
        conversation_id: ConversationId,
        session_id: SessionId,
        initial_prompt: Prompt,
        correlation_id: CorrelationId,
        started_at: DateTime<Utc>,
    },
    PromptSent {
        conversation_id: ConversationId,
        prompt: Prompt,
        correlation_id: CorrelationId,
        event_id: EventId,
        sent_at: DateTime<Utc>,
    },
    ResponseReceived {
        conversation_id: ConversationId,
        response: ClaudeResponse,
        causation_id: EventId,
        correlation_id: CorrelationId,
        received_at: DateTime<Utc>,
    },
    ConversationEnded {
        conversation_id: ConversationId,
        session_id: SessionId,
        reason: EndReason,
        ended_at: DateTime<Utc>,
    },
}

impl DomainEvent {
    pub fn conversation_id(&self) -> ConversationId {
        match self {
            DomainEvent::ConversationStarted { conversation_id, .. } |
            DomainEvent::PromptSent { conversation_id, .. } |
            DomainEvent::ResponseReceived { conversation_id, .. } |
            DomainEvent::ConversationEnded { conversation_id, .. } => *conversation_id,
        }
    }
    
    pub fn correlation_id(&self) -> Option<CorrelationId> {
        match self {
            DomainEvent::ConversationStarted { correlation_id, .. } |
            DomainEvent::PromptSent { correlation_id, .. } |
            DomainEvent::ResponseReceived { correlation_id, .. } => Some(*correlation_id),
            DomainEvent::ConversationEnded { .. } => None,
        }
    }
}