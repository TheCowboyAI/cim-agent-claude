# CIM Claude Adapter - Architecture Overview

This document provides comprehensive architectural diagrams for the CIM Claude Adapter, which maps 100% of the Claude API to an event-sourced domain model using Commands, Events, Queries, Value Objects, Entities, and Aggregates.

## Event Sourcing Architecture Overview

```mermaid
graph TD
    %% High contrast styling
    classDef primary fill:#FF6B6B,stroke:#2D3436,stroke-width:4px,color:#FFFFFF
    classDef secondary fill:#4ECDC4,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef decision fill:#FFE66D,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef result fill:#95E1D3,stroke:#2D3436,stroke-width:2px,color:#2D3436
    classDef start fill:#2D3436,stroke:#FFFFFF,stroke-width:2px,color:#FFFFFF

    %% External Systems
    Client[Claude API Client]:::start
    ClaudeAPI["Claude API<br/>api.anthropic.com"]:::start

    %% Command Layer
    Commands["Commands Layer<br/>14 Command Types"]:::primary
    SendMessage[SendMessage]:::secondary
    SendStreaming[SendStreamingMessage]:::secondary
    UpdatePrompt[UpdateSystemPrompt]:::secondary
    AddTools[AddTools]:::secondary

    %% Event Store
    EventStore[("Event Store<br/>NATS JetStream")]:::primary
    
    %% Event Layer  
    Events["Events Layer<br/>25+ Event Types"]:::primary
    MessageReceived[MessageResponseReceived]:::secondary
    StreamChunk[StreamingChunkReceived]:::secondary
    ApiError[ApiErrorOccurred]:::secondary
    ToolUse[ToolUseRequested]:::secondary

    %% Query Layer
    Queries["Queries Layer<br/>25+ Query Types"]:::primary
    GetConversation[GetConversation]:::secondary
    SearchMessages[SearchMessages]:::secondary
    GetUsageStats[GetUsageStatistics]:::secondary

    %% Domain Model
    Aggregates["Domain Aggregates<br/>ClaudeApiSession"]:::primary
    ValueObjects["Value Objects<br/>Model, Temperature, etc."]:::result
    Entities["Entities<br/>Request, Response, Message"]:::result

    %% Flow
    Client --> Commands
    Commands --> EventStore
    EventStore --> Events
    Events --> Aggregates
    
    Commands -.-> SendMessage
    Commands -.-> SendStreaming  
    Commands -.-> UpdatePrompt
    Commands -.-> AddTools

    Events -.-> MessageReceived
    Events -.-> StreamChunk
    Events -.-> ApiError
    Events -.-> ToolUse

    Queries -.-> GetConversation
    Queries -.-> SearchMessages
    Queries -.-> GetUsageStats

    Aggregates --> ValueObjects
    Aggregates --> Entities

    %% API Integration
    Commands --> ClaudeAPI
    ClaudeAPI --> Events
```

## Complete API Command Mapping

```mermaid
graph LR
    %% High contrast styling
    classDef primary fill:#FF6B6B,stroke:#2D3436,stroke-width:4px,color:#FFFFFF
    classDef secondary fill:#4ECDC4,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef decision fill:#FFE66D,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef result fill:#95E1D3,stroke:#2D3436,stroke-width:2px,color:#2D3436

    %% API Commands - 100% Coverage
    Commands["Claude API Commands<br/>14 Types"]:::primary
    
    %% Core API Operations
    Commands --> SendMessage["SendMessage<br/>POST /v1/messages"]:::secondary
    Commands --> SendStreaming["SendStreamingMessage<br/>POST /v1/messages?stream=true"]:::secondary
    
    %% Configuration Commands
    Commands --> UpdatePrompt[UpdateSystemPrompt]:::decision
    Commands --> UpdateModel[UpdateModelConfiguration]:::decision
    Commands --> AddTools[AddTools]:::decision
    Commands --> RemoveTools[RemoveTools]:::decision
    
    %% Tool Interaction Commands
    Commands --> HandleTool[HandleToolUse]:::result
    Commands --> SubmitResult[SubmitToolResult]:::result
    
    %% Control Commands
    Commands --> CancelRequest[CancelRequest]:::secondary
    Commands --> RetryRequest[RetryRequest]:::secondary
    Commands --> ResetConv[ResetConversation]:::secondary
    
    %% Data Management Commands
    Commands --> ExportConv[ExportConversation]:::result
    Commands --> ImportConv[ImportConversation]:::result
    Commands --> ValidateConv[ValidateConversation]:::result
```

## Event Sourcing State Machine

```mermaid
stateDiagram-v2
    %% High contrast styling applied via CSS classes
    classDef primary fill:#FF6B6B,stroke:#2D3436,stroke-width:4px,color:#FFFFFF
    classDef secondary fill:#4ECDC4,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef decision fill:#FFE66D,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef result fill:#95E1D3,stroke:#2D3436,stroke-width:2px,color:#2D3436
    classDef error fill:#E74C3C,stroke:#2D3436,stroke-width:3px,color:#FFFFFF

    [*] --> ConversationCreated
    ConversationCreated --> AwaitingMessage

    AwaitingMessage --> MessageSent : SendMessage Command
    MessageSent --> ProcessingMessage : Request to Claude API
    
    ProcessingMessage --> MessageReceived : Success Response
    ProcessingMessage --> StreamingStarted : Streaming Response
    ProcessingMessage --> ApiErrorOccurred : Error Response
    ProcessingMessage --> RequestTimeout : Timeout
    
    StreamingStarted --> StreamingChunkReceived : Chunk Received
    StreamingChunkReceived --> StreamingChunkReceived : More Chunks
    StreamingChunkReceived --> StreamingCompleted : Final Chunk
    
    MessageReceived --> ToolUseRequested : Tool Use Detected
    MessageReceived --> AwaitingMessage : Normal Response
    
    StreamingCompleted --> ToolUseRequested : Tool Use Detected
    StreamingCompleted --> AwaitingMessage : Normal Response
    
    ToolUseRequested --> ToolExecutionStarted : HandleToolUse Command
    ToolExecutionStarted --> ToolExecutionCompleted : Success
    ToolExecutionStarted --> ToolExecutionFailed : Error
    
    ToolExecutionCompleted --> ToolResultSubmitted : SubmitToolResult Command
    ToolExecutionFailed --> ToolResultSubmitted : Submit Error Result
    ToolResultSubmitted --> AwaitingMessage
    
    ApiErrorOccurred --> RetryInitiated : Retryable Error
    ApiErrorOccurred --> ConversationError : Non-retryable Error
    RequestTimeout --> RetryInitiated : Timeout Retry
    
    RetryInitiated --> ProcessingMessage : Retry Attempt
    RetryInitiated --> RetryExhausted : Max Retries Reached
    RetryExhausted --> ConversationError
    
    AwaitingMessage --> ConversationReset : ResetConversation Command
    ConversationReset --> AwaitingMessage
    
    ConversationError --> ConversationReset : Manual Reset
    ConversationError --> [*] : End Conversation
    
    class ConversationCreated primary
    class MessageSent secondary
    class ProcessingMessage decision
    class MessageReceived result
    class StreamingCompleted result
    class ToolUseRequested decision
    class ApiErrorOccurred error
    class ConversationError error
```

## NATS Message Flow Architecture

```mermaid
graph TB
    %% High contrast styling
    classDef primary fill:#FF6B6B,stroke:#2D3436,stroke-width:4px,color:#FFFFFF
    classDef secondary fill:#4ECDC4,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef decision fill:#FFE66D,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef result fill:#95E1D3,stroke:#2D3436,stroke-width:2px,color:#2D3436
    classDef start fill:#2D3436,stroke:#FFFFFF,stroke-width:2px,color:#FFFFFF

    %% External Client
    Client["CIM Client<br/>Local NATS"]:::start
    
    %% NATS Infrastructure
    LeafNode["Leaf Node<br/>NATS Server"]:::primary
    JetStream[("JetStream<br/>Event Store")]:::primary
    KVStore[("KV Store<br/>Configuration")]:::secondary
    
    %% Subject Hierarchy
    Commands["claude.commands.*"]:::decision
    Events["claude.events.*"]:::decision
    Queries["claude.queries.*"]:::decision
    
    %% Command Handlers
    CommandHandler["Command Handler<br/>Service"]:::secondary
    APIAdapter["Claude API Adapter<br/>Service"]:::secondary
    QueryHandler["Query Handler<br/>Service"]:::secondary
    
    %% Message Flow
    Client --> |"Publish Command"| LeafNode
    LeafNode --> Commands
    Commands --> CommandHandler
    CommandHandler --> APIAdapter
    APIAdapter --> |"Store Event"| JetStream
    JetStream --> Events
    Events --> |"Event Notification"| Client
    
    Client --> |"Query Request"| Queries
    Queries --> QueryHandler
    QueryHandler --> |"Read Events"| JetStream
    QueryHandler --> |"Response"| Client
    
    %% Configuration Flow
    CommandHandler --> |"Read Config"| KVStore
    APIAdapter --> |"Update Config"| KVStore
    
    %% Subject Examples
    Commands -.-> CmdSend["claude.commands.send"]:::result
    Commands -.-> CmdStream["claude.commands.stream"]:::result
    Commands -.-> CmdTool["claude.commands.tool"]:::result
    
    Events -.-> EvtResponse["claude.events.response"]:::result
    Events -.-> EvtError["claude.events.error"]:::result
    Events -.-> EvtTool["claude.events.tool"]:::result
    
    Queries -.-> QryConv["claude.queries.conversation"]:::result
    Queries -.-> QryUsage["claude.queries.usage"]:::result
    Queries -.-> QryHistory["claude.queries.history"]:::result
```

## Domain Model Visualization

```mermaid
classDiagram
    %% High contrast styling
    class ClaudeApiSession {
        <<Aggregate Root>>
        +conversation_id: ConversationId
        +model_config: ClaudeModel
        +system_prompt: ClaudeSystemPrompt
        +message_history: Vec~ClaudeMessage~
        +tool_definitions: Vec~ClaudeToolDefinition~
        +total_usage: ClaudeUsage
        +add_user_message(content)
        +add_assistant_message(response)
        +can_add_message(content) bool
        +estimated_cost_usd() f64
    }
    
    class ClaudeApiRequest {
        <<Entity>>
        +model: ClaudeModel
        +messages: Vec~ClaudeMessage~
        +max_tokens: MaxTokens
        +system: ClaudeSystemPrompt
        +temperature: Temperature
        +tools: Vec~ClaudeToolDefinition~
        +validate() Result
        +estimated_input_tokens() u32
    }
    
    class ClaudeApiResponse {
        <<Entity>>
        +id: ClaudeMessageId
        +model: ClaudeModel
        +content: Vec~ContentBlock~
        +stop_reason: StopReason
        +usage: ClaudeUsage
        +text_content() String
        +tool_uses() Vec~ContentBlock~
    }
    
    class ClaudeModel {
        <<Value Object>>
        Claude35Sonnet20241022
        Claude35Sonnet20240620
        Claude3Opus20240229
        Claude3Sonnet20240229
        Claude3Haiku20240307
        +as_str() &str
        +max_tokens() u32
        +context_window() u32
    }
    
    class Temperature {
        <<Value Object>>
        -value: f64
        +new(f64) Result~Self~
        +value() f64
    }
    
    class MaxTokens {
        <<Value Object>>
        -value: u32
        +new(u32) Result~Self~
        +value() u32
    }
    
    class ClaudeUsage {
        <<Value Object>>
        +input_tokens: u32
        +output_tokens: u32
        +total_tokens() u32
        +estimated_cost_usd(model) f64
    }
    
    class ClaudeMessage {
        <<Entity>>
        +role: MessageRole
        +content: MessageContent
        +user(content) Self
        +assistant(content) Self
    }
    
    class ContentBlock {
        <<Value Object>>
        Text
        Image
        ToolUse
        ToolResult
        +token_estimate() u32
    }
    
    class ClaudeApiError {
        <<Value Object>>
        +error_type: ClaudeErrorType
        +message: String
        +http_status: u16
        +retry_after: u32
        +is_retryable() bool
        +is_client_error() bool
    }
    
    %% Relationships
    ClaudeApiSession ||--o{ ClaudeMessage
    ClaudeApiSession ||--o{ ClaudeToolDefinition
    ClaudeApiSession ||--|| ClaudeModel
    ClaudeApiSession ||--|| ClaudeUsage
    
    ClaudeApiRequest ||--|| ClaudeModel
    ClaudeApiRequest ||--o{ ClaudeMessage
    ClaudeApiRequest ||--|| MaxTokens
    ClaudeApiRequest ||--o| Temperature
    
    ClaudeApiResponse ||--|| ClaudeModel
    ClaudeApiResponse ||--o{ ContentBlock
    ClaudeApiResponse ||--|| ClaudeUsage
    
    ClaudeMessage ||--|| MessageContent
    MessageContent ||--o{ ContentBlock
```

## Complete Event Types Mapping

```mermaid
mindmap
  root)Claude API Events - 25+ Types(
    )API Response Events(
      MessageResponseReceived
      StreamingChunkReceived
      StreamingMessageCompleted
    )Error Events(
      ApiErrorOccurred
      RequestTimeoutOccurred
      RequestRetryExhausted
    )Control Events(
      RequestCancelled
      RequestRetryInitiated
      ConversationReset
    )Configuration Events(
      SystemPromptUpdated
      ModelConfigurationUpdated
      ToolsAdded
      ToolsRemoved
    )Tool Events(
      ToolUseRequested
      ToolExecutionStarted
      ToolExecutionCompleted
      ToolExecutionFailed
      ToolResultSubmitted
    )Management Events(
      ConversationExported
      ConversationImported
      ConversationValidationCompleted
    )Monitoring Events(
      RateLimitEncountered
      UsageThresholdReached
      TokenLimitApproaching
      CostThresholdExceeded
      ApiHealthCheckCompleted
      ApiAvailabilityChanged
```

## Query Pattern Architecture

```mermaid
graph TD
    %% High contrast styling
    classDef primary fill:#FF6B6B,stroke:#2D3436,stroke-width:4px,color:#FFFFFF
    classDef secondary fill:#4ECDC4,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef decision fill:#FFE66D,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef result fill:#95E1D3,stroke:#2D3436,stroke-width:2px,color:#2D3436

    %% Query Types
    QueryLayer["Query Layer<br/>25+ Query Types"]:::primary
    
    %% Conversation Queries
    ConvQueries["Conversation Queries"]:::secondary
    QueryLayer --> ConvQueries
    ConvQueries --> GetConv[GetConversation]:::decision
    ConvQueries --> GetHistory[GetConversationHistory]:::decision
    ConvQueries --> GetMsg[GetMessage]:::decision
    ConvQueries --> SearchMsg[SearchMessages]:::decision
    
    %% Analytics Queries  
    AnalyticsQueries["Analytics Queries"]:::secondary
    QueryLayer --> AnalyticsQueries
    AnalyticsQueries --> GetUsage[GetUsageStatistics]:::decision
    AnalyticsQueries --> GetCost[GetCostAnalysis]:::decision
    AnalyticsQueries --> GetPerf[GetPerformanceMetrics]:::decision
    AnalyticsQueries --> GetAnalytics[GetConversationAnalytics]:::decision
    
    %% Tool Queries
    ToolQueries["Tool Queries"]:::secondary
    QueryLayer --> ToolQueries
    ToolQueries --> GetTools[GetConversationTools]:::decision
    ToolQueries --> GetToolUsage[GetToolUsageHistory]:::decision
    ToolQueries --> GetToolExec[GetToolExecution]:::decision
    
    %% System Queries
    SystemQueries["System Queries"]:::secondary
    QueryLayer --> SystemQueries
    SystemQueries --> GetErrors[GetErrorHistory]:::decision
    SystemQueries --> GetRateLimit[GetRateLimitStatus]:::decision
    SystemQueries --> GetHealth[GetApiHealthStatus]:::decision
    SystemQueries --> GetQuota[GetQuotaUsage]:::decision
    
    %% Results
    GetConv --> ConvResult["ConversationDetails"]:::result
    GetUsage --> UsageResult["Usage Statistics"]:::result
    GetTools --> ToolResult["Tool Definitions"]:::result
    GetErrors --> ErrorResult["Error History"]:::result
```

## Command Validation Flow

```mermaid
flowchart TD
    %% High contrast styling
    classDef primary fill:#FF6B6B,stroke:#2D3436,stroke-width:4px,color:#FFFFFF
    classDef secondary fill:#4ECDC4,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef decision fill:#FFE66D,stroke:#2D3436,stroke-width:3px,color:#2D3436
    classDef result fill:#95E1D3,stroke:#2D3436,stroke-width:2px,color:#2D3436
    classDef error fill:#E74C3C,stroke:#2D3436,stroke-width:3px,color:#FFFFFF

    Start([Command Received]):::primary
    
    %% Basic Validation
    ValidateStruct{Validate Structure}:::decision
    StructError([Invalid Command Structure]):::error
    
    %% Command-Specific Validation
    ValidateContent{Validate Content}:::decision
    ContentError([Invalid Content]):::error
    
    %% Business Rules
    ValidateRules{Validate Business Rules}:::decision
    RulesError([Business Rule Violation]):::error
    
    %% API Constraints
    ValidateAPI{Validate API Constraints}:::decision
    APIError([API Constraint Violation]):::error
    
    %% Success Path
    CommandValid([Command Valid]):::result
    ProcessCommand[Process Command]:::secondary
    StoreEvent[Store Event]:::result
    
    %% Flow
    Start --> ValidateStruct
    ValidateStruct -->|Invalid| StructError
    ValidateStruct -->|Valid| ValidateContent
    
    ValidateContent -->|Invalid| ContentError
    ValidateContent -->|Valid| ValidateRules
    
    ValidateRules -->|Invalid| RulesError
    ValidateRules -->|Valid| ValidateAPI
    
    ValidateAPI -->|Invalid| APIError
    ValidateAPI -->|Valid| CommandValid
    
    CommandValid --> ProcessCommand
    ProcessCommand --> StoreEvent
    
    %% Error Handling
    StructError --> ErrorEvent["CommandValidationFailed Event"]:::error
    ContentError --> ErrorEvent
    RulesError --> ErrorEvent
    APIError --> ErrorEvent
```

This comprehensive architectural documentation provides complete visual coverage of our event-sourced Claude API adapter. All diagrams follow the high-contrast styling guide and show the complete 100% API mapping we've implemented across Commands, Events, Queries, Value Objects, Entities, and Aggregates.