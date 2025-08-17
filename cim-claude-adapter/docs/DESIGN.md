# CIM Claude Adapter - Design Documentation

*Copyright 2025 - Cowboy AI, LLC. All rights reserved.*

## Overview

The CIM Claude Adapter is a NATS-native service that provides a Domain-Driven Design (DDD) interface for integrating Claude AI into CIM (Composable Information Machine) ecosystems. Built following event-sourcing patterns and hexagonal architecture principles, it serves as a translation layer between CIM's event-driven infrastructure and Claude's API.

## Architecture

### High-Level Architecture

```
┌─────────────────┐    NATS     ┌─────────────────┐    HTTPS    ┌─────────────────┐
│                 │ ◄────────► │                 │ ◄────────► │                 │
│   CIM Services  │   Events   │ CIM Claude      │  REST API  │   Claude API    │
│                 │ Commands   │   Adapter       │            │                 │
└─────────────────┘            └─────────────────┘            └─────────────────┘
```

### Hexagonal Architecture

The adapter follows hexagonal (ports and adapters) architecture:

```
                    ┌─────────────────────────────────┐
                    │         Application Layer       │
                    │    ┌─────────────────────┐      │
                    │    │   Conversation      │      │
                    │    │     Service         │      │
                    │    └─────────────────────┘      │
                    └─────────────┬───────────────────┘
                                  │
           ┌──────────────────────┼──────────────────────┐
           │                      │                      │
    ┌─────────────┐        ┌─────────────┐        ┌─────────────┐
    │   Domain    │        │    Ports    │        │  Adapters   │
    │             │        │             │        │             │
    │ • Events    │        │ • NATS Port │        │ • NATS      │
    │ • Commands  │        │ • Claude    │        │ • Claude    │
    │ • Aggregates│        │   API Port  │        │   API       │
    │ • Values    │        │ • Storage   │        │ • Storage   │
    │ • Errors    │        │   Port      │        │             │
    └─────────────┘        └─────────────┘        └─────────────┘
```

### Domain Model

#### Core Domain Concepts

1. **Conversation Aggregate**: Central entity managing Claude conversation lifecycle
2. **Commands**: Intent expressions (StartConversation, SendPrompt, EndConversation)
3. **Events**: Business facts (ConversationStarted, PromptSent, ResponseReceived, etc.)
4. **Value Objects**: Domain primitives (ConversationId, Prompt, ClaudeResponse, etc.)

#### Event Sourcing Pattern

All state changes are captured as immutable domain events:

```rust
pub enum DomainEvent {
    ConversationStarted { ... },
    PromptSent { ... },
    ResponseReceived { ... },
    ConversationEnded { ... },
    RateLimitExceeded { ... },
    ClaudeApiErrorOccurred { ... },
}
```

#### Command/Query Responsibility Segregation (CQRS)

- **Commands**: Mutate state through events
- **Queries**: Read from event-sourced projections
- **Event Store**: Single source of truth

## Component Design

### 1. Domain Layer

#### Conversation Aggregate

The core business entity managing conversation lifecycle:

```rust
pub struct ConversationAggregate {
    id: ConversationId,
    state: ConversationState,
    session_id: SessionId,
    context: ConversationContext,
    exchanges: VecDeque<ConversationExchange>,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
}
```

**States**: `Active`, `Waiting`, `RateLimited`, `Failed`, `Ended`

**Invariants**:
- Conversation must be started before prompts can be sent
- Maximum exchange limits are enforced
- Rate limits are respected
- State transitions follow business rules

#### Value Objects

**Strong Type System** for domain safety:

- `ConversationId`: Unique conversation identifier
- `SessionId`: User session correlation
- `CorrelationId`: CIM event correlation
- `EventId`: Event identification
- `Prompt`: Validated user input (max 50K chars)
- `ClaudeResponse`: Structured API response
- `TokenUsage`: Token consumption tracking

### 2. Application Layer

#### Conversation Service

Main application service orchestrating conversation workflows:

```rust
pub struct ConversationService {
    event_store: Box<dyn EventStore>,
    claude_client: Box<dyn ClaudeApiPort>,
    nats_publisher: Box<dyn NatsPublisher>,
}
```

**Key Responsibilities**:
- Command validation and processing
- Event sourcing and projection
- External API coordination
- Error handling and recovery

### 3. Infrastructure Layer

#### NATS Integration

**Subject Design**:
```
cim.claude.commands.{conversation_id}    # Command ingestion
cim.claude.events.{event_type}          # Event publishing
cim.claude.queries.{query_type}         # Query responses
```

**Message Patterns**:
- Commands: Request/Reply with correlation tracking
- Events: Publish/Subscribe with guaranteed delivery
- Queries: Request/Reply for read operations

#### Claude API Integration

**API Client Features**:
- Async/await HTTP client
- Automatic retry with exponential backoff
- Rate limiting compliance
- Structured error handling
- Token usage tracking

### 4. Ports (Interfaces)

#### Primary Ports (Driving)

```rust
pub trait ConversationService {
    async fn start_conversation(&self, command: StartConversation) -> Result<ConversationId>;
    async fn send_prompt(&self, command: SendPrompt) -> Result<()>;
    async fn end_conversation(&self, command: EndConversation) -> Result<()>;
}
```

#### Secondary Ports (Driven)

```rust
pub trait ClaudeApiPort {
    async fn send_prompt(&self, request: ClaudeRequest) -> Result<ClaudeResponse>;
    async fn get_usage_stats(&self) -> Result<UsageStats>;
}

pub trait EventStore {
    async fn save_events(&self, events: Vec<EventEnvelope>) -> Result<()>;
    async fn load_events(&self, aggregate_id: ConversationId) -> Result<Vec<EventEnvelope>>;
}
```

## Data Flow

### Command Processing Flow

```
1. NATS Command → Command Handler
2. Load Aggregate from Event Store
3. Business Logic Processing
4. Event Generation
5. Event Store Persistence
6. Event Publishing to NATS
7. Claude API Interaction (if needed)
8. Response Event Generation
9. Final State Update
```

### Event Processing Flow

```
1. Domain Event Generated
2. Event Envelope Creation (correlation/causation)
3. Event Store Append
4. NATS Event Publishing
5. Projection Updates (if applicable)
6. External Notifications
```

## Error Handling Strategy

### Error Taxonomy

```rust
pub enum DomainError {
    ConversationNotFound(ConversationId),
    InvalidState { expected: ConversationState, actual: ConversationState },
    PromptTooLong { length: usize, max: usize },
    RateLimitExceeded { retry_after: Duration },
    ClaudeApiError(ClaudeApiError),
    ValidationError(String),
}
```

### Recovery Patterns

1. **Transient Errors**: Automatic retry with exponential backoff
2. **Rate Limits**: Graceful degradation with retry-after compliance
3. **Permanent Errors**: Error event generation and circuit breaking
4. **Validation Errors**: Immediate rejection with detailed feedback

## Scalability Design

### Horizontal Scaling

- **Stateless Services**: All state in events, services are stateless
- **NATS Clustering**: Built-in NATS high availability
- **Event Partitioning**: Conversation-level partitioning via NATS subjects
- **Load Balancing**: NATS queue groups for automatic load distribution

### Performance Characteristics

- **Command Processing**: < 10ms (excluding Claude API)
- **Event Publishing**: < 5ms
- **Claude API Integration**: 500ms - 3s (API dependent)
- **Memory Usage**: O(1) per service instance (event-sourced)

### Monitoring & Observability

#### Metrics

- **Business Metrics**: Conversations/hour, tokens/day, success rates
- **Technical Metrics**: Latency, throughput, error rates, queue depths
- **Resource Metrics**: CPU, memory, network, API quotas

#### Tracing

- **Correlation IDs**: Full request tracing across services
- **Event Causation**: Complete event lineage tracking
- **API Call Tracing**: Claude API interaction monitoring

## Security Design

### Authentication & Authorization

- **NATS Security**: NKey-based authentication
- **Claude API**: API key management with rotation
- **TLS**: All external communications encrypted

### Data Protection

- **Conversation Privacy**: No persistent conversation content storage
- **Audit Logging**: Complete event trail for compliance
- **Secret Management**: Environment-based secret injection

### Rate Limiting

- **Claude API Compliance**: Respect API rate limits
- **Internal Protection**: Per-session rate limiting
- **Circuit Breakers**: Automatic degradation under load

## Testing Strategy

### Unit Testing

- **Domain Logic**: 100% coverage of business rules
- **Value Objects**: Validation logic testing
- **Event Sourcing**: Aggregate reconstruction testing

### Integration Testing

- **NATS Integration**: End-to-end message flow testing
- **Claude API**: Mock and contract testing
- **Event Store**: Persistence and retrieval testing

### End-to-End Testing

- **Complete Workflows**: Full conversation lifecycle testing
- **Error Scenarios**: Failure mode validation
- **Performance Testing**: Load and scalability testing

## Deployment Architecture

### Container Design

```dockerfile
# Multi-stage build for optimized production image
# Health checks and graceful shutdown
# Configuration via environment variables
```

### Infrastructure Requirements

- **NATS Server**: JetStream enabled, clustered
- **Storage**: Event store (PostgreSQL/NATS KV)
- **Monitoring**: Prometheus/Grafana stack
- **Secrets**: Kubernetes secrets or equivalent

### Configuration Management

```yaml
# Environment-specific configuration
# Feature flags for gradual rollouts
# Circuit breaker thresholds
# Rate limiting parameters
```

## Future Enhancements

### Phase 2 Features

1. **Multi-Model Support**: Integration with different Claude models
2. **Conversation Persistence**: Optional long-term conversation storage
3. **Advanced Analytics**: Usage pattern analysis and optimization
4. **Webhook Integration**: External system notification support

### Phase 3 Features

1. **GraphQL Interface**: Advanced query capabilities
2. **Real-time Streaming**: WebSocket support for live conversations
3. **Conversation Templates**: Pre-configured conversation patterns
4. **Advanced Rate Limiting**: Intelligent request queuing

## Conclusion

The CIM Claude Adapter provides a robust, scalable, and maintainable integration between CIM ecosystems and Claude AI. By following DDD principles, event sourcing patterns, and hexagonal architecture, it ensures clean separation of concerns, testability, and adaptability to changing requirements.

The design prioritizes:
- **Domain-Centric**: Business logic separated from technical concerns
- **Event-Driven**: Complete audit trail and loose coupling
- **Scalable**: Horizontal scaling through NATS clustering
- **Reliable**: Comprehensive error handling and recovery
- **Observable**: Full monitoring and tracing capabilities
- **Secure**: Enterprise-grade security patterns

This foundation supports the current Claude integration needs while providing flexibility for future enhancements and additional AI service integrations.