/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Complete Claude API Command Mapping
//! 
//! Every Claude API interaction starts as a Command in our event-sourced system.
//! This ensures 100% traceability and proper separation of concerns.

use crate::domain::claude_api::*;
use crate::domain::value_objects::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command ID for tracking Claude API commands
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClaudeCommandId(Uuid);

impl ClaudeCommandId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl std::fmt::Display for ClaudeCommandId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Complete Claude API Commands - Every API interaction maps to one of these
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClaudeApiCommand {
    /// Send message to Claude API - maps to POST /v1/messages
    SendMessage {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        session_id: SessionId,
        request: ClaudeApiRequest,
        correlation_id: CorrelationId,
        timeout_seconds: Option<u32>,
        retry_config: Option<RetryConfiguration>,
        request_metadata: RequestMetadata,
    },
    
    /// Send streaming message to Claude API - maps to POST /v1/messages with stream=true
    SendStreamingMessage {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        session_id: SessionId,
        request: ClaudeApiRequest,
        correlation_id: CorrelationId,
        stream_handler: StreamHandlerConfig,
        timeout_seconds: Option<u32>,
        request_metadata: RequestMetadata,
    },
    
    /// Update system prompt for conversation
    UpdateSystemPrompt {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        new_system_prompt: ClaudeSystemPrompt,
        reason: String,
        correlation_id: CorrelationId,
    },
    
    /// Update model configuration for conversation
    UpdateModelConfiguration {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        new_model: ClaudeModel,
        new_temperature: Option<Temperature>,
        new_max_tokens: Option<MaxTokens>,
        new_stop_sequences: Option<StopSequences>,
        reason: String,
        correlation_id: CorrelationId,
    },
    
    /// Add tools to conversation
    AddTools {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        tools: Vec<ClaudeToolDefinition>,
        correlation_id: CorrelationId,
    },
    
    /// Remove tools from conversation
    RemoveTools {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        tool_names: Vec<String>,
        correlation_id: CorrelationId,
    },
    
    /// Handle tool use response (when Claude requests tool execution)
    HandleToolUse {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        tool_use_id: String,
        tool_name: String,
        tool_input: serde_json::Value,
        correlation_id: CorrelationId,
        execution_timeout: Option<u32>,
    },
    
    /// Submit tool result back to Claude
    SubmitToolResult {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        tool_use_id: String,
        result: ToolExecutionResult,
        correlation_id: CorrelationId,
    },
    
    /// Cancel ongoing Claude API request
    CancelRequest {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        target_command_id: ClaudeCommandId,
        reason: String,
        correlation_id: CorrelationId,
    },
    
    /// Retry failed Claude API request
    RetryRequest {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        original_command_id: ClaudeCommandId,
        retry_attempt: u32,
        modified_request: Option<ClaudeApiRequest>,
        correlation_id: CorrelationId,
    },
    
    /// Reset conversation (clear history, start fresh)
    ResetConversation {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        preserve_system_prompt: bool,
        preserve_tools: bool,
        reason: String,
        correlation_id: CorrelationId,
    },
    
    /// Export conversation data
    ExportConversation {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        format: ExportFormat,
        include_metadata: bool,
        include_usage_stats: bool,
        correlation_id: CorrelationId,
    },
    
    /// Import conversation data
    ImportConversation {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        import_data: ConversationImportData,
        merge_strategy: ConversationMergeStrategy,
        correlation_id: CorrelationId,
    },
    
    /// Validate conversation state
    ValidateConversation {
        command_id: ClaudeCommandId,
        conversation_id: ConversationId,
        validation_rules: ValidationRules,
        correlation_id: CorrelationId,
    },
}

/// Retry configuration for Claude API requests
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryConfiguration {
    pub max_retries: u32,
    pub base_delay_ms: u32,
    pub max_delay_ms: u32,
    pub exponential_backoff: bool,
    pub retry_on_rate_limit: bool,
    pub retry_on_server_error: bool,
    pub retry_on_timeout: bool,
}

impl Default for RetryConfiguration {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            exponential_backoff: true,
            retry_on_rate_limit: true,
            retry_on_server_error: true,
            retry_on_timeout: true,
        }
    }
}

/// Request metadata for tracking and analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub initiated_by: SessionId,
    pub request_source: RequestSource,
    pub priority: RequestPriority,
    pub tags: Vec<String>,
    pub custom_metadata: std::collections::HashMap<String, String>,
    pub trace_id: Option<String>,
    pub parent_span_id: Option<String>,
}

impl RequestMetadata {
    pub fn new(initiated_by: SessionId, source: RequestSource) -> Self {
        Self {
            initiated_by,
            request_source: source,
            priority: RequestPriority::Normal,
            tags: Vec::new(),
            custom_metadata: std::collections::HashMap::new(),
            trace_id: None,
            parent_span_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestSource {
    UserInterface,
    ApiClient,
    ScheduledTask,
    WebhookTrigger,
    ToolExecution,
    SystemProcess,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Stream handling configuration for streaming requests
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamHandlerConfig {
    pub buffer_size: usize,
    pub flush_interval_ms: u32,
    pub handle_partial_responses: bool,
    pub emit_token_events: bool,
    pub emit_content_events: bool,
}

impl Default for StreamHandlerConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            flush_interval_ms: 100,
            handle_partial_responses: true,
            emit_token_events: false,
            emit_content_events: true,
        }
    }
}

/// Tool execution result for submitting back to Claude
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolExecutionResult {
    Success {
        content: String,
        metadata: std::collections::HashMap<String, serde_json::Value>,
        execution_time_ms: u64,
    },
    Error {
        error_message: String,
        error_code: Option<String>,
        is_retryable: bool,
        execution_time_ms: u64,
    },
    Timeout {
        timeout_ms: u64,
        partial_result: Option<String>,
    },
    Cancelled {
        reason: String,
    },
}

impl ToolExecutionResult {
    pub fn success(content: String, execution_time_ms: u64) -> Self {
        Self::Success {
            content,
            metadata: std::collections::HashMap::new(),
            execution_time_ms,
        }
    }
    
    pub fn error(error_message: String, execution_time_ms: u64) -> Self {
        Self::Error {
            error_message,
            error_code: None,
            is_retryable: false,
            execution_time_ms,
        }
    }
    
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }
    
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. } | Self::Timeout { .. } | Self::Cancelled { .. })
    }
}

/// Export format options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    MessagePack,
    Csv,
    Markdown,
    PlainText,
}

/// Conversation import data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationImportData {
    pub messages: Vec<ClaudeMessage>,
    pub system_prompt: Option<ClaudeSystemPrompt>,
    pub tools: Option<Vec<ClaudeToolDefinition>>,
    pub model_config: Option<ClaudeModel>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Strategy for merging imported conversation data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationMergeStrategy {
    Replace,
    Append,
    Merge,
    MergeWithTimestamps,
}

/// Validation rules for conversation state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationRules {
    pub max_message_count: Option<u32>,
    pub max_total_tokens: Option<u32>,
    pub max_cost_usd: Option<f64>,
    pub required_system_prompt: bool,
    pub allowed_models: Option<Vec<ClaudeModel>>,
    pub max_tool_count: Option<u32>,
    pub custom_rules: Vec<CustomValidationRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomValidationRule {
    pub name: String,
    pub description: String,
    pub rule_type: ValidationRuleType,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationRuleType {
    TokenCount,
    MessagePattern,
    ContentFilter,
    CostLimit,
    TimeLimit,
    Custom,
}

impl ClaudeApiCommand {
    pub fn command_id(&self) -> &ClaudeCommandId {
        match self {
            Self::SendMessage { command_id, .. } => command_id,
            Self::SendStreamingMessage { command_id, .. } => command_id,
            Self::UpdateSystemPrompt { command_id, .. } => command_id,
            Self::UpdateModelConfiguration { command_id, .. } => command_id,
            Self::AddTools { command_id, .. } => command_id,
            Self::RemoveTools { command_id, .. } => command_id,
            Self::HandleToolUse { command_id, .. } => command_id,
            Self::SubmitToolResult { command_id, .. } => command_id,
            Self::CancelRequest { command_id, .. } => command_id,
            Self::RetryRequest { command_id, .. } => command_id,
            Self::ResetConversation { command_id, .. } => command_id,
            Self::ExportConversation { command_id, .. } => command_id,
            Self::ImportConversation { command_id, .. } => command_id,
            Self::ValidateConversation { command_id, .. } => command_id,
        }
    }
    
    pub fn conversation_id(&self) -> &ConversationId {
        match self {
            Self::SendMessage { conversation_id, .. } => conversation_id,
            Self::SendStreamingMessage { conversation_id, .. } => conversation_id,
            Self::UpdateSystemPrompt { conversation_id, .. } => conversation_id,
            Self::UpdateModelConfiguration { conversation_id, .. } => conversation_id,
            Self::AddTools { conversation_id, .. } => conversation_id,
            Self::RemoveTools { conversation_id, .. } => conversation_id,
            Self::HandleToolUse { conversation_id, .. } => conversation_id,
            Self::SubmitToolResult { conversation_id, .. } => conversation_id,
            Self::CancelRequest { conversation_id, .. } => conversation_id,
            Self::RetryRequest { conversation_id, .. } => conversation_id,
            Self::ResetConversation { conversation_id, .. } => conversation_id,
            Self::ExportConversation { conversation_id, .. } => conversation_id,
            Self::ImportConversation { conversation_id, .. } => conversation_id,
            Self::ValidateConversation { conversation_id, .. } => conversation_id,
        }
    }
    
    pub fn correlation_id(&self) -> &CorrelationId {
        match self {
            Self::SendMessage { correlation_id, .. } => correlation_id,
            Self::SendStreamingMessage { correlation_id, .. } => correlation_id,
            Self::UpdateSystemPrompt { correlation_id, .. } => correlation_id,
            Self::UpdateModelConfiguration { correlation_id, .. } => correlation_id,
            Self::AddTools { correlation_id, .. } => correlation_id,
            Self::RemoveTools { correlation_id, .. } => correlation_id,
            Self::HandleToolUse { correlation_id, .. } => correlation_id,
            Self::SubmitToolResult { correlation_id, .. } => correlation_id,
            Self::CancelRequest { correlation_id, .. } => correlation_id,
            Self::RetryRequest { correlation_id, .. } => correlation_id,
            Self::ResetConversation { correlation_id, .. } => correlation_id,
            Self::ExportConversation { correlation_id, .. } => correlation_id,
            Self::ImportConversation { correlation_id, .. } => correlation_id,
            Self::ValidateConversation { correlation_id, .. } => correlation_id,
        }
    }
    
    pub fn requires_api_call(&self) -> bool {
        matches!(
            self,
            Self::SendMessage { .. } |
            Self::SendStreamingMessage { .. } |
            Self::SubmitToolResult { .. }
        )
    }
    
    pub fn is_configuration_change(&self) -> bool {
        matches!(
            self,
            Self::UpdateSystemPrompt { .. } |
            Self::UpdateModelConfiguration { .. } |
            Self::AddTools { .. } |
            Self::RemoveTools { .. }
        )
    }
    
    pub fn is_tool_related(&self) -> bool {
        matches!(
            self,
            Self::AddTools { .. } |
            Self::RemoveTools { .. } |
            Self::HandleToolUse { .. } |
            Self::SubmitToolResult { .. }
        )
    }
    
    /// Validate command before processing
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::SendMessage { request, .. } => {
                request.validate()?;
                if request.estimated_input_tokens() == 0 {
                    return Err("Request must have content".to_string());
                }
            }
            
            Self::SendStreamingMessage { request, .. } => {
                request.validate()?;
                if !request.stream.unwrap_or(false) {
                    return Err("Streaming request must have stream=true".to_string());
                }
            }
            
            Self::UpdateSystemPrompt { new_system_prompt, .. } => {
                if new_system_prompt.content().trim().is_empty() {
                    return Err("System prompt cannot be empty".to_string());
                }
            }
            
            Self::AddTools { tools, .. } => {
                if tools.is_empty() {
                    return Err("Must provide at least one tool".to_string());
                }
                
                for tool in tools {
                    if tool.name.trim().is_empty() {
                        return Err("Tool name cannot be empty".to_string());
                    }
                }
            }
            
            Self::RemoveTools { tool_names, .. } => {
                if tool_names.is_empty() {
                    return Err("Must specify at least one tool to remove".to_string());
                }
            }
            
            Self::HandleToolUse { tool_name, .. } => {
                if tool_name.trim().is_empty() {
                    return Err("Tool name cannot be empty".to_string());
                }
            }
            
            _ => {} // Other commands have basic validation
        }
        
        Ok(())
    }
}

/// Conversion from ClaudeApiCommand to ClaudeApiRequest
/// For commands that contain an API request, extract it
impl From<ClaudeApiCommand> for ClaudeApiRequest {
    fn from(command: ClaudeApiCommand) -> Self {
        match command {
            ClaudeApiCommand::SendMessage { request, .. } => request,
            ClaudeApiCommand::SendStreamingMessage { request, .. } => request,
            _ => panic!("Cannot extract ClaudeApiRequest from non-message command"),
        }
    }
}

impl ClaudeApiCommand {
    /// Try to extract the ClaudeApiRequest from this command
    pub fn try_into_request(self) -> Option<ClaudeApiRequest> {
        match self {
            ClaudeApiCommand::SendMessage { request, .. } => Some(request),
            ClaudeApiCommand::SendStreamingMessage { request, .. } => Some(request),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_command_id() {
        let id1 = ClaudeCommandId::new();
        let id2 = ClaudeCommandId::new();
        assert_ne!(id1, id2);
        
        let uuid_str = id1.to_string();
        assert!(!uuid_str.is_empty());
    }

    #[test]
    fn test_retry_configuration_default() {
        let config = RetryConfiguration::default();
        assert_eq!(config.max_retries, 3);
        assert!(config.retry_on_rate_limit);
        assert!(config.retry_on_server_error);
        assert!(config.exponential_backoff);
    }

    #[test]
    fn test_tool_execution_result() {
        let success = ToolExecutionResult::success("Result".to_string(), 100);
        assert!(success.is_success());
        assert!(!success.is_error());
        
        let error = ToolExecutionResult::error("Error occurred".to_string(), 50);
        assert!(!error.is_success());
        assert!(error.is_error());
    }

    #[test]
    fn test_command_validation() {
        // Valid send message command
        let model = ClaudeModel::Claude35Sonnet20241022;
        let messages = vec![ClaudeMessage::user(MessageContent::text("Hello"))];
        let request = ClaudeApiRequest::new(model, messages, MaxTokens::new(1000).unwrap());
        
        let command = ClaudeApiCommand::SendMessage {
            command_id: ClaudeCommandId::new(),
            conversation_id: ConversationId::new(),
            session_id: SessionId::new(),
            request,
            correlation_id: CorrelationId::new(),
            timeout_seconds: None,
            retry_config: None,
            request_metadata: RequestMetadata::new(SessionId::new(), RequestSource::UserInterface),
        };
        
        assert!(command.validate().is_ok());
        assert!(command.requires_api_call());
        assert!(!command.is_configuration_change());
        assert!(!command.is_tool_related());
    }
}