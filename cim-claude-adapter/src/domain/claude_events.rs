/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Complete Claude API Event Mapping
//! 
//! Every Claude API response, error, timeout, and state change generates an Event.
//! This provides 100% audit trail of all Claude API interactions.

use crate::domain::claude_api::*;
use crate::domain::claude_commands::*;
use crate::domain::value_objects::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Complete Claude API Events - Every API response and error maps to one of these
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClaudeApiEvent {
    /// Successful message response from Claude API
    MessageResponseReceived {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        request: ClaudeApiRequest,
        response: ClaudeApiResponse,
        request_duration_ms: u64,
        request_id: Option<String>,
        received_at: DateTime<Utc>,
    },
    
    /// Streaming message chunk received
    StreamingChunkReceived {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        chunk_sequence: u32,
        chunk_content: StreamChunk,
        accumulated_tokens: u32,
        received_at: DateTime<Utc>,
    },
    
    /// Streaming message completed
    StreamingMessageCompleted {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        total_chunks: u32,
        final_response: ClaudeApiResponse,
        total_duration_ms: u64,
        completed_at: DateTime<Utc>,
    },
    
    /// Claude API error occurred
    ApiErrorOccurred {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        request: ClaudeApiRequest,
        error: ClaudeApiError,
        request_duration_ms: u64,
        error_occurred_at: DateTime<Utc>,
        retry_attempt: Option<u32>,
    },
    
    /// Request timeout occurred
    RequestTimeoutOccurred {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        timeout_ms: u64,
        partial_response: Option<String>,
        timed_out_at: DateTime<Utc>,
    },
    
    /// Request was cancelled
    RequestCancelled {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        cancellation_reason: String,
        was_in_flight: bool,
        cancelled_at: DateTime<Utc>,
    },
    
    /// Request retry initiated
    RequestRetryInitiated {
        command_id: ClaudeCommandId,
        original_command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        retry_attempt: u32,
        retry_delay_ms: u32,
        retry_reason: RetryReason,
        initiated_at: DateTime<Utc>,
    },
    
    /// Request retry exhausted (no more retries)
    RequestRetryExhausted {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        total_attempts: u32,
        final_error: ClaudeApiError,
        exhausted_at: DateTime<Utc>,
    },
    
    /// System prompt updated successfully
    SystemPromptUpdated {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        old_prompt: Option<ClaudeSystemPrompt>,
        new_prompt: ClaudeSystemPrompt,
        reason: String,
        updated_at: DateTime<Utc>,
    },
    
    /// Model configuration updated
    ModelConfigurationUpdated {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        old_model: ClaudeModel,
        new_model: ClaudeModel,
        old_temperature: Option<Temperature>,
        new_temperature: Option<Temperature>,
        old_max_tokens: Option<MaxTokens>,
        new_max_tokens: Option<MaxTokens>,
        reason: String,
        updated_at: DateTime<Utc>,
    },
    
    /// Tools added to conversation
    ToolsAdded {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        added_tools: Vec<ClaudeToolDefinition>,
        total_tool_count: u32,
        added_at: DateTime<Utc>,
    },
    
    /// Tools removed from conversation
    ToolsRemoved {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        removed_tool_names: Vec<String>,
        total_tool_count: u32,
        removed_at: DateTime<Utc>,
    },
    
    /// Claude requested tool use
    ToolUseRequested {
        conversation_id: ConversationId,
        tool_use_id: String,
        tool_name: String,
        tool_input: serde_json::Value,
        response_message_id: ClaudeMessageId,
        requested_at: DateTime<Utc>,
    },
    
    /// Tool execution started
    ToolExecutionStarted {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        tool_use_id: String,
        tool_name: String,
        tool_input: serde_json::Value,
        execution_timeout_ms: Option<u32>,
        started_at: DateTime<Utc>,
    },
    
    /// Tool execution completed successfully
    ToolExecutionCompleted {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        tool_use_id: String,
        tool_name: String,
        execution_result: ToolExecutionResult,
        completed_at: DateTime<Utc>,
    },
    
    /// Tool execution failed
    ToolExecutionFailed {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        tool_use_id: String,
        tool_name: String,
        error_message: String,
        error_code: Option<String>,
        execution_time_ms: u64,
        failed_at: DateTime<Utc>,
    },
    
    /// Tool result submitted back to Claude
    ToolResultSubmitted {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        tool_use_id: String,
        result: ToolExecutionResult,
        submitted_at: DateTime<Utc>,
    },
    
    /// Conversation was reset
    ConversationReset {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        preserved_system_prompt: bool,
        preserved_tools: bool,
        previous_message_count: u32,
        previous_total_tokens: u32,
        previous_cost_usd: f64,
        reason: String,
        reset_at: DateTime<Utc>,
    },
    
    /// Conversation exported
    ConversationExported {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        export_format: ExportFormat,
        export_size_bytes: u64,
        export_location: String,
        included_metadata: bool,
        included_usage_stats: bool,
        exported_at: DateTime<Utc>,
    },
    
    /// Conversation imported
    ConversationImported {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        import_source: String,
        merge_strategy: ConversationMergeStrategy,
        imported_message_count: u32,
        conflicts_resolved: u32,
        imported_at: DateTime<Utc>,
    },
    
    /// Conversation validation completed
    ConversationValidationCompleted {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        validation_results: ValidationResults,
        validated_at: DateTime<Utc>,
    },
    
    /// Rate limit encountered
    RateLimitEncountered {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        limit_type: RateLimitType,
        retry_after_seconds: Option<u32>,
        requests_remaining: Option<u32>,
        reset_time: Option<DateTime<Utc>>,
        encountered_at: DateTime<Utc>,
    },
    
    /// Usage threshold reached (warning)
    UsageThresholdReached {
        conversation_id: ConversationId,
        threshold_type: UsageThresholdType,
        current_usage: ClaudeUsage,
        threshold_limit: f64,
        threshold_percentage: f64,
        reached_at: DateTime<Utc>,
    },
    
    /// Token limit approaching
    TokenLimitApproaching {
        conversation_id: ConversationId,
        current_tokens: u32,
        max_tokens: u32,
        remaining_tokens: u32,
        percentage_used: f64,
        warned_at: DateTime<Utc>,
    },
    
    /// Cost threshold exceeded
    CostThresholdExceeded {
        conversation_id: ConversationId,
        current_cost_usd: f64,
        threshold_cost_usd: f64,
        exceeded_by_usd: f64,
        exceeded_at: DateTime<Utc>,
    },
    
    /// Health check completed
    ApiHealthCheckCompleted {
        is_healthy: bool,
        response_time_ms: u64,
        api_version: String,
        server_region: Option<String>,
        checked_at: DateTime<Utc>,
    },
    
    /// API availability changed
    ApiAvailabilityChanged {
        previous_status: ApiStatus,
        new_status: ApiStatus,
        reason: String,
        changed_at: DateTime<Utc>,
    },
}

/// Streaming chunk data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamChunk {
    pub chunk_type: StreamChunkType,
    pub content: String,
    pub token_count: Option<u32>,
    pub is_complete: bool,
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StreamChunkType {
    MessageStart,
    ContentBlockStart,
    ContentBlockDelta,
    ContentBlockStop,
    MessageDelta,
    MessageStop,
    Ping,
    Error,
}

/// Retry reasons
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetryReason {
    RateLimit,
    ServerError,
    Timeout,
    NetworkError,
    AuthenticationError,
    Overloaded,
}

/// Validation results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResults {
    pub is_valid: bool,
    pub passed_rules: Vec<String>,
    pub failed_rules: Vec<ValidationFailure>,
    pub warnings: Vec<String>,
    pub total_message_count: u32,
    pub total_tokens: u32,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationFailure {
    pub rule_name: String,
    pub failure_reason: String,
    pub current_value: serde_json::Value,
    pub expected_value: serde_json::Value,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Warning,
    Error,
    Critical,
}

/// Rate limit types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RateLimitType {
    RequestsPerMinute,
    TokensPerMinute,
    RequestsPerHour,
    TokensPerHour,
    RequestsPerDay,
    TokensPerDay,
    ConcurrentRequests,
}

/// Usage threshold types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UsageThresholdType {
    TokenCount,
    CostUsd,
    RequestCount,
    MessageCount,
    SessionDuration,
}

/// API status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApiStatus {
    Healthy,
    Degraded,
    Unavailable,
    MaintenanceMode,
    RateLimited,
    Unknown,
}

impl ClaudeApiEvent {
    pub fn conversation_id(&self) -> &ConversationId {
        match self {
            Self::MessageResponseReceived { conversation_id, .. } => conversation_id,
            Self::StreamingChunkReceived { conversation_id, .. } => conversation_id,
            Self::StreamingMessageCompleted { conversation_id, .. } => conversation_id,
            Self::ApiErrorOccurred { conversation_id, .. } => conversation_id,
            Self::RequestTimeoutOccurred { conversation_id, .. } => conversation_id,
            Self::RequestCancelled { conversation_id, .. } => conversation_id,
            Self::RequestRetryInitiated { conversation_id, .. } => conversation_id,
            Self::RequestRetryExhausted { conversation_id, .. } => conversation_id,
            Self::SystemPromptUpdated { conversation_id, .. } => conversation_id,
            Self::ModelConfigurationUpdated { conversation_id, .. } => conversation_id,
            Self::ToolsAdded { conversation_id, .. } => conversation_id,
            Self::ToolsRemoved { conversation_id, .. } => conversation_id,
            Self::ToolUseRequested { conversation_id, .. } => conversation_id,
            Self::ToolExecutionStarted { conversation_id, .. } => conversation_id,
            Self::ToolExecutionCompleted { conversation_id, .. } => conversation_id,
            Self::ToolExecutionFailed { conversation_id, .. } => conversation_id,
            Self::ToolResultSubmitted { conversation_id, .. } => conversation_id,
            Self::ConversationReset { conversation_id, .. } => conversation_id,
            Self::ConversationExported { conversation_id, .. } => conversation_id,
            Self::ConversationImported { conversation_id, .. } => conversation_id,
            Self::ConversationValidationCompleted { conversation_id, .. } => conversation_id,
            Self::RateLimitEncountered { conversation_id, .. } => conversation_id,
            Self::UsageThresholdReached { conversation_id, .. } => conversation_id,
            Self::TokenLimitApproaching { conversation_id, .. } => conversation_id,
            Self::CostThresholdExceeded { conversation_id, .. } => conversation_id,
            Self::ApiHealthCheckCompleted { .. } => {
                // Health checks don't have conversation context
                // Return a dummy conversation ID or handle differently
                panic!("ApiHealthCheckCompleted doesn't have a conversation_id")
            },
            Self::ApiAvailabilityChanged { .. } => {
                // API availability changes don't have conversation context
                panic!("ApiAvailabilityChanged doesn't have a conversation_id")
            },
        }
    }
    
    pub fn command_id(&self) -> Option<&ClaudeCommandId> {
        match self {
            Self::MessageResponseReceived { command_id, .. } => Some(command_id),
            Self::StreamingChunkReceived { command_id, .. } => Some(command_id),
            Self::StreamingMessageCompleted { command_id, .. } => Some(command_id),
            Self::ApiErrorOccurred { command_id, .. } => Some(command_id),
            Self::RequestTimeoutOccurred { command_id, .. } => Some(command_id),
            Self::RequestCancelled { command_id, .. } => Some(command_id),
            Self::RequestRetryInitiated { command_id, .. } => Some(command_id),
            Self::RequestRetryExhausted { command_id, .. } => Some(command_id),
            Self::SystemPromptUpdated { command_id, .. } => Some(command_id),
            Self::ModelConfigurationUpdated { command_id, .. } => Some(command_id),
            Self::ToolsAdded { command_id, .. } => Some(command_id),
            Self::ToolsRemoved { command_id, .. } => Some(command_id),
            Self::ToolExecutionStarted { command_id, .. } => Some(command_id),
            Self::ToolExecutionCompleted { command_id, .. } => Some(command_id),
            Self::ToolExecutionFailed { command_id, .. } => Some(command_id),
            Self::ToolResultSubmitted { command_id, .. } => Some(command_id),
            Self::ConversationReset { command_id, .. } => Some(command_id),
            Self::ConversationExported { command_id, .. } => Some(command_id),
            Self::ConversationImported { command_id, .. } => Some(command_id),
            Self::ConversationValidationCompleted { command_id, .. } => Some(command_id),
            Self::RateLimitEncountered { command_id, .. } => Some(command_id),
            _ => None, // Events without command context
        }
    }
    
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::MessageResponseReceived { received_at, .. } => *received_at,
            Self::StreamingChunkReceived { received_at, .. } => *received_at,
            Self::StreamingMessageCompleted { completed_at, .. } => *completed_at,
            Self::ApiErrorOccurred { error_occurred_at, .. } => *error_occurred_at,
            Self::RequestTimeoutOccurred { timed_out_at, .. } => *timed_out_at,
            Self::RequestCancelled { cancelled_at, .. } => *cancelled_at,
            Self::RequestRetryInitiated { initiated_at, .. } => *initiated_at,
            Self::RequestRetryExhausted { exhausted_at, .. } => *exhausted_at,
            Self::SystemPromptUpdated { updated_at, .. } => *updated_at,
            Self::ModelConfigurationUpdated { updated_at, .. } => *updated_at,
            Self::ToolsAdded { added_at, .. } => *added_at,
            Self::ToolsRemoved { removed_at, .. } => *removed_at,
            Self::ToolUseRequested { requested_at, .. } => *requested_at,
            Self::ToolExecutionStarted { started_at, .. } => *started_at,
            Self::ToolExecutionCompleted { completed_at, .. } => *completed_at,
            Self::ToolExecutionFailed { failed_at, .. } => *failed_at,
            Self::ToolResultSubmitted { submitted_at, .. } => *submitted_at,
            Self::ConversationReset { reset_at, .. } => *reset_at,
            Self::ConversationExported { exported_at, .. } => *exported_at,
            Self::ConversationImported { imported_at, .. } => *imported_at,
            Self::ConversationValidationCompleted { validated_at, .. } => *validated_at,
            Self::RateLimitEncountered { encountered_at, .. } => *encountered_at,
            Self::UsageThresholdReached { reached_at, .. } => *reached_at,
            Self::TokenLimitApproaching { warned_at, .. } => *warned_at,
            Self::CostThresholdExceeded { exceeded_at, .. } => *exceeded_at,
            Self::ApiHealthCheckCompleted { checked_at, .. } => *checked_at,
            Self::ApiAvailabilityChanged { changed_at, .. } => *changed_at,
        }
    }
    
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::ApiErrorOccurred { .. } |
            Self::RequestTimeoutOccurred { .. } |
            Self::RequestRetryExhausted { .. } |
            Self::ToolExecutionFailed { .. }
        )
    }
    
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            Self::MessageResponseReceived { .. } |
            Self::StreamingMessageCompleted { .. } |
            Self::ToolExecutionCompleted { .. } |
            Self::SystemPromptUpdated { .. } |
            Self::ModelConfigurationUpdated { .. } |
            Self::ToolsAdded { .. } |
            Self::ToolsRemoved { .. } |
            Self::ConversationReset { .. } |
            Self::ConversationExported { .. } |
            Self::ConversationImported { .. } |
            Self::ConversationValidationCompleted { .. }
        )
    }
    
    pub fn is_tool_related(&self) -> bool {
        matches!(
            self,
            Self::ToolsAdded { .. } |
            Self::ToolsRemoved { .. } |
            Self::ToolUseRequested { .. } |
            Self::ToolExecutionStarted { .. } |
            Self::ToolExecutionCompleted { .. } |
            Self::ToolExecutionFailed { .. } |
            Self::ToolResultSubmitted { .. }
        )
    }
    
    pub fn affects_cost(&self) -> bool {
        matches!(
            self,
            Self::MessageResponseReceived { .. } |
            Self::StreamingMessageCompleted { .. }
        )
    }
    
    /// Get usage information from this event if available
    pub fn usage(&self) -> Option<&ClaudeUsage> {
        match self {
            Self::MessageResponseReceived { response, .. } => Some(&response.usage),
            Self::StreamingMessageCompleted { final_response, .. } => Some(&final_response.usage),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_chunk() {
        let chunk = StreamChunk {
            chunk_type: StreamChunkType::ContentBlockDelta,
            content: "Hello".to_string(),
            token_count: Some(1),
            is_complete: false,
            metadata: None,
        };
        
        assert_eq!(chunk.chunk_type, StreamChunkType::ContentBlockDelta);
        assert_eq!(chunk.content, "Hello");
        assert!(!chunk.is_complete);
    }

    #[test]
    fn test_validation_results() {
        let results = ValidationResults {
            is_valid: false,
            passed_rules: vec!["token_limit".to_string()],
            failed_rules: vec![ValidationFailure {
                rule_name: "message_count".to_string(),
                failure_reason: "Too many messages".to_string(),
                current_value: serde_json::Value::Number(150.into()),
                expected_value: serde_json::Value::Number(100.into()),
                severity: ValidationSeverity::Error,
            }],
            warnings: vec!["Approaching cost limit".to_string()],
            total_message_count: 150,
            total_tokens: 50000,
            estimated_cost_usd: 5.75,
        };
        
        assert!(!results.is_valid);
        assert_eq!(results.failed_rules.len(), 1);
        assert_eq!(results.warnings.len(), 1);
    }

    #[test]
    fn test_event_classification() {
        let success_event = ClaudeApiEvent::MessageResponseReceived {
            command_id: ClaudeCommandId::new(),
            conversation_id: ConversationId::new(),
            request: ClaudeApiRequest::new(
                ClaudeModel::Claude35Sonnet20241022,
                vec![ClaudeMessage::user("test")],
                MaxTokens::new(100).unwrap()
            ),
            response: ClaudeApiResponse::new(
                ClaudeMessageId::new("msg_123".to_string()),
                ClaudeModel::Claude35Sonnet20241022,
                vec![ContentBlock::Text { text: "Response".to_string() }],
                StopReason::EndTurn,
                ClaudeUsage::new(10, 5),
            ),
            request_duration_ms: 1000,
            request_id: Some("req_123".to_string()),
            received_at: Utc::now(),
        };
        
        assert!(success_event.is_success());
        assert!(!success_event.is_error());
        assert!(success_event.affects_cost());
        assert!(!success_event.is_tool_related());
        
        let usage = success_event.usage().unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.output_tokens, 5);
    }
}