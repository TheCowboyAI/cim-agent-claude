# Claude API to NATS Hexagonal Architecture

## Overview

This design implements a clean hexagonal architecture following DDD and CIM principles for translating between Claude API and NATS messaging. The system is designed with proper domain boundaries, event-driven patterns, and correlation/causation tracking.

## Architecture Components

### Domain Layer (Core Business Logic)

#### 1. Conversation Aggregate (`domain/conversation_aggregate.rs`)
**State Machine Design:**
```
Draft → Processing → Responded → [Ended]
  ↑         ↓         ↓
  └─────────┴─────────┘
```

**Key Components:**
- **ConversationAggregate**: Root entity managing conversation lifecycle
- **Value Objects**: `Prompt`, `ClaudeResponse`, `Exchange` (invariant groups)
- **State Machine**: `ConversationState` with valid business transitions
- **Business Rules**: Rate limiting, context windows, prompt validation
- **Domain Events**: Past-tense, business-focused events

**Business Invariants:**
- Maximum 10 prompts per minute per conversation
- Context retention for 24 hours
- Maximum prompt length of 50,000 characters
- Prompt-response pairing integrity

#### 2. Commands (`domain/commands.rs`)
**Command Types (Imperative Intent):**
- `StartConversationCommand`: Begin new conversation with initial prompt
- `SendPromptCommand`: Add prompt to existing conversation
- `EndConversationCommand`: Terminate conversation with reason

**Command Validation:**
- Prompt length and content validation
- Session and conversation ID validation
- Correlation ID presence verification

### Ports (Interfaces/Boundaries)

#### 3. Conversation Port (`ports/conversation_port.rs`)
**Inbound Port** - Interface for external systems to interact with domain

**Key Interfaces:**
```rust
#[async_trait]
pub trait ConversationPort {
    async fn handle_command(&self, command: ConversationCommand) -> Result<CommandResult, Error>;
    async fn subscribe_to_conversation_events(&self, conversation_id: ConversationId, handler: EventHandler) -> Result<EventSubscription, Error>;
    async fn health_check(&self) -> Result<PortHealth, Error>;
}
```

**Features:**
- Command processing with proper error handling
- Event subscription management (conversation and session level)
- Health monitoring and metrics
- Configurable retry policies

#### 4. Claude API Port (`ports/claude_api_port.rs`)
**Outbound Port** - Interface for domain to communicate with external Claude API

**Key Interfaces:**
```rust
#[async_trait]
pub trait ClaudeApiPort {
    async fn send_prompt(&self, request: ClaudeApiRequest) -> Result<ClaudeApiResponse, Error>;
    async fn send_prompt_stream(&self, request: ClaudeApiRequest) -> Result<ClaudeApiStreamResponse, Error>;
    async fn health_check(&self) -> Result<ApiHealth, Error>;
    async fn get_rate_limits(&self) -> Result<RateLimitStatus, Error>;
}
```

**Features:**
- Request/response correlation tracking
- Rate limit monitoring and enforcement
- Circuit breaker pattern for reliability
- Streaming response support (future)

### Adapters (Technology Implementations)

#### 5. NATS Adapter (`adapters/nats_adapter.rs`)
**Implementation of ConversationPort**

**NATS Subject Patterns (CIM Compliant):**
```
Commands (Inbound):
- claude.cmd.{session_id}.start
- claude.cmd.{session_id}.prompt  
- claude.cmd.{session_id}.end

Events (Outbound):
- claude.event.{session_id}.started
- claude.event.{session_id}.prompt_sent
- claude.resp.{session_id}.content
- claude.event.{session_id}.ended
```

**JetStream Configuration:**
- **CLAUDE_COMMANDS**: WorkQueue retention, 24h max age
- **CLAUDE_EVENTS**: Limits retention, 30 days max age  
- **CLAUDE_RESPONSES**: Interest retention, 1h max age

**Features:**
- Pull consumer with explicit acknowledgment
- Durable subscriptions with proper error handling
- Correlation ID tracking throughout message flow
- Automatic retry with exponential backoff

#### 6. Claude API Adapter (`adapters/claude_api_adapter.rs`)
**Implementation of ClaudeApiPort**

**HTTP Client Features:**
- Authenticated requests with API key
- Circuit breaker for reliability (Open/Closed/Half-Open states)
- Rate limit tracking from response headers
- Retry logic with exponential backoff
- Proper error mapping and classification

**Rate Limiting:**
- Tracks both request and token limits
- Respects server-side rate limit headers
- Maintains buffer to prevent limit breaches
- Circuit breaker opens on repeated failures

### Application Layer

#### 7. Conversation Service (`application/conversation_service.rs`)
**Application Service orchestrating the workflow**

**Workflow Pattern:**
1. **Receive Command** → Validate and route to aggregate
2. **Domain Processing** → Apply business rules and generate events  
3. **Persistence** → Save aggregate state to repository
4. **Event Publishing** → Publish domain events to NATS
5. **External API Call** → Send to Claude API asynchronously
6. **Response Handling** → Process API response and update aggregate

**Key Features:**
- Async processing with proper error handling
- Correlation tracking throughout the workflow
- Event sourcing compatibility
- Circuit breaker and retry patterns

## Event-Driven Design (NO CRUD)

### Domain Events (Past Tense, Business-Focused)
```rust
pub enum DomainEvent {
    ConversationStarted { conversation_id, session_id, initial_prompt, correlation_id, started_at },
    PromptSent { conversation_id, prompt, correlation_id, event_id, sent_at },
    ResponseReceived { conversation_id, response, causation_id, correlation_id, received_at },
    ConversationEnded { conversation_id, session_id, reason, ended_at },
}
```

### Correlation and Causation Tracking
- **Correlation ID**: Links related events across the entire conversation
- **Causation ID**: Links cause-effect relationships (PromptSent → ResponseReceived)
- **Event ID**: Unique identifier for each domain event

### Message Flow
```
NATS Command → Domain Processing → Domain Events → NATS Publication → Claude API → Response → Domain Events → NATS
```

## CIM Framework Compliance

### 1. Business Domain Drives Architecture
- Domain model reflects actual conversation flow
- Business invariants encoded as aggregate rules
- State machine matches real conversation states

### 2. Self-Documenting Code
- Explicit value objects for business concepts
- Clear naming that business experts understand
- Domain events describe what actually happened

### 3. Natural Boundaries
- Conversation aggregate boundary around consistency
- Clear separation between ports and adapters
- Domain isolated from infrastructure concerns

### 4. Compositional Design
- Ports as composable interfaces
- Adapters can be swapped without domain changes
- Event-driven decoupling enables system evolution

## Configuration and Deployment

### NATS Configuration
- Streams configured for different retention policies
- Consumers with proper durability and acknowledgment
- Security with account-based permissions

### Claude API Configuration
- API key management and authentication
- Timeout and retry policies
- Rate limit buffer configuration

### Monitoring and Observability
- Health checks for all components
- Correlation tracking in logs
- Circuit breaker state monitoring
- Rate limit metrics

## File Structure
```
claude-adapter/
├── domain/
│   ├── conversation_aggregate.rs  # Core domain model
│   └── commands.rs                # Command definitions
├── ports/
│   ├── conversation_port.rs       # Inbound port interface
│   └── claude_api_port.rs         # Outbound port interface  
├── adapters/
│   ├── nats_adapter.rs           # NATS implementation
│   └── claude_api_adapter.rs     # Claude API implementation
└── application/
    └── conversation_service.rs   # Application orchestration
```

This design provides a clean, testable, and maintainable architecture that properly separates concerns while maintaining strong domain boundaries and enabling system evolution through compositional design.