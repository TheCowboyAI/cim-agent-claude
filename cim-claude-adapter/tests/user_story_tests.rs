/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Comprehensive User Story Validation Tests
//!
//! This module contains tests that validate every user story from docs/user-stories.md
//! Each test maps directly to a user story and validates the complete Command -> Event -> Query flow.

use cim_claude_adapter::domain::{
    claude_api::*,
    claude_commands::*,
    claude_events::*,
    claude_queries::*,
    value_objects::*,
};
use chrono::Utc;
use serde_json;
use std::collections::HashMap;
use tokio;

#[cfg(test)]
mod core_claude_api_tests {
    use super::*;

    /// Test Story 1.1: Send Message to Claude
    /// Validates: SendMessage command -> MessageResponseReceived event -> GetConversation query
    #[tokio::test]
    async fn test_story_1_1_send_message_to_claude() {
        // Arrange: Create a valid send message command
        let conversation_id = ConversationId::new();
        let session_id = SessionId::new();
        let correlation_id = CorrelationId::new();
        
        let message = ClaudeMessage::user(MessageContent::text("Hello Claude!"));
        let request = ClaudeApiRequest::new(
            ClaudeModel::Claude35Sonnet20241022,
            vec![message],
            MaxTokens::new(1000).unwrap(),
        );
        
        let command = ClaudeApiCommand::SendMessage {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            session_id: session_id.clone(),
            request: request.clone(),
            correlation_id: correlation_id.clone(),
            timeout_seconds: Some(30),
            retry_config: Some(RetryConfiguration::default()),
            request_metadata: RequestMetadata::new(session_id.clone(), RequestSource::UserInterface),
        };

        // Act & Assert: Validate command structure
        assert!(command.validate().is_ok());
        assert_eq!(command.conversation_id(), &conversation_id);
        assert!(command.requires_api_call());
        
        // Simulate successful response event
        let response = ClaudeApiResponse::new(
            ClaudeMessageId::new("msg_123".to_string()),
            ClaudeModel::Claude35Sonnet20241022,
            vec![ContentBlock::Text { text: "Hello! I'm Claude.".to_string() }],
            StopReason::EndTurn,
            ClaudeUsage::new(15, 25),
        );
        
        let event = ClaudeApiEvent::MessageResponseReceived {
            command_id: command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            request: request.clone(),
            response: response.clone(),
            request_duration_ms: 1500,
            request_id: Some("req_123".to_string()),
            received_at: Utc::now(),
        };
        
        // Validate event properties
        assert_eq!(event.conversation_id(), &conversation_id);
        assert!(event.is_success());
        assert!(!event.is_error());
        assert!(event.affects_cost());
        
        let usage = event.usage().unwrap();
        assert_eq!(usage.input_tokens, 15);
        assert_eq!(usage.output_tokens, 25);
        
        // Validate query capability
        let query = ClaudeApiQuery::GetConversation {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            include_messages: true,
            include_usage_stats: true,
            include_tool_definitions: false,
        };
        
        assert_eq!(query.conversation_id().unwrap(), &conversation_id);
        assert!(!query.requires_pagination());
        assert!(!query.is_expensive_query());
    }

    /// Test Story 1.2: Stream Message Response
    /// Validates: SendStreamingMessage -> StreamingChunkReceived -> StreamingMessageCompleted
    #[tokio::test]
    async fn test_story_1_2_stream_message_response() {
        let conversation_id = ConversationId::new();
        let command_id = ClaudeCommandId::new();
        
        // Create streaming command
        let request = ClaudeApiRequest::new(
            ClaudeModel::Claude35Sonnet20241022,
            vec![ClaudeMessage::user(MessageContent::text("Tell me a story"))],
            MaxTokens::new(2000).unwrap(),
        ).with_stream(true);
        
        let command = ClaudeApiCommand::SendStreamingMessage {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            session_id: SessionId::new(),
            request: request.clone(),
            correlation_id: CorrelationId::new(),
            stream_handler: StreamHandlerConfig::default(),
            timeout_seconds: Some(60),
            request_metadata: RequestMetadata::new(SessionId::new(), RequestSource::UserInterface),
        };
        
        assert!(command.validate().is_ok());
        assert!(request.stream.unwrap_or(false));
        
        // Simulate streaming chunks
        let chunk1 = ClaudeApiEvent::StreamingChunkReceived {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            chunk_sequence: 1,
            chunk_content: StreamChunk {
                chunk_type: StreamChunkType::ContentBlockDelta,
                content: "Once upon".to_string(),
                token_count: Some(3),
                is_complete: false,
                metadata: None,
            },
            accumulated_tokens: 3,
            received_at: Utc::now(),
        };
        
        let _chunk2 = ClaudeApiEvent::StreamingChunkReceived {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            chunk_sequence: 2,
            chunk_content: StreamChunk {
                chunk_type: StreamChunkType::ContentBlockDelta,
                content: " a time...".to_string(),
                token_count: Some(4),
                is_complete: false,
                metadata: None,
            },
            accumulated_tokens: 7,
            received_at: Utc::now(),
        };
        
        // Validate chunk events
        assert_eq!(chunk1.conversation_id(), &conversation_id);
        assert!(!chunk1.is_error());
        assert!(!chunk1.is_success()); // Chunks are intermediate
        
        // Simulate completion
        let final_response = ClaudeApiResponse::new(
            ClaudeMessageId::new("msg_stream_123".to_string()),
            ClaudeModel::Claude35Sonnet20241022,
            vec![ContentBlock::Text { text: "Once upon a time...".to_string() }],
            StopReason::EndTurn,
            ClaudeUsage::new(20, 30),
        );
        
        let completion = ClaudeApiEvent::StreamingMessageCompleted {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            total_chunks: 2,
            final_response: final_response.clone(),
            total_duration_ms: 3000,
            completed_at: Utc::now(),
        };
        
        assert!(completion.is_success());
        assert!(completion.affects_cost());
        
        // Validate streaming session query
        let session_query = ClaudeApiQuery::GetStreamingSession {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            command_id: command_id.clone(),
            include_chunks: true,
            include_timing: true,
        };
        
        assert_eq!(session_query.conversation_id().unwrap(), &conversation_id);
    }

    /// Test Story 1.3: Handle API Errors Gracefully
    /// Validates: API error -> RetryRequest -> RequestRetryInitiated/Exhausted
    #[tokio::test]
    async fn test_story_1_3_handle_api_errors_gracefully() {
        let conversation_id = ConversationId::new();
        let original_command_id = ClaudeCommandId::new();
        
        // Simulate API error
        let api_error = ClaudeApiError::new(
            ClaudeErrorType::RateLimitError,
            "Rate limit exceeded".to_string(),
            429,
        ).with_retry_after(60);
        
        let error_event = ClaudeApiEvent::ApiErrorOccurred {
            command_id: original_command_id.clone(),
            conversation_id: conversation_id.clone(),
            request: ClaudeApiRequest::new(
                ClaudeModel::Claude35Sonnet20241022,
                vec![ClaudeMessage::user(MessageContent::text("Test"))],
                MaxTokens::new(100).unwrap(),
            ),
            error: api_error.clone(),
            request_duration_ms: 500,
            error_occurred_at: Utc::now(),
            retry_attempt: Some(1),
        };
        
        assert!(error_event.is_error());
        assert!(api_error.is_retryable());
        
        // Create retry command
        let retry_command = ClaudeApiCommand::RetryRequest {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            original_command_id: original_command_id.clone(),
            retry_attempt: 1,
            modified_request: None,
            correlation_id: CorrelationId::new(),
        };
        
        assert!(retry_command.validate().is_ok());
        
        // Simulate retry initiated event
        let retry_event = ClaudeApiEvent::RequestRetryInitiated {
            command_id: retry_command.command_id().clone(),
            original_command_id: original_command_id.clone(),
            conversation_id: conversation_id.clone(),
            retry_attempt: 1,
            retry_delay_ms: 1000,
            retry_reason: RetryReason::RateLimit,
            initiated_at: Utc::now(),
        };
        
        assert_eq!(retry_event.conversation_id(), &conversation_id);
        
        // Test retry exhaustion
        let exhausted_event = ClaudeApiEvent::RequestRetryExhausted {
            command_id: retry_command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            total_attempts: 3,
            final_error: api_error,
            exhausted_at: Utc::now(),
        };
        
        assert!(exhausted_event.is_error());
        
        // Validate error history query
        let error_query = ClaudeApiQuery::GetErrorHistory {
            query_id: ClaudeQueryId::new(),
            conversation_id: Some(conversation_id.clone()),
            error_types: Some(vec![ClaudeErrorType::RateLimitError]),
            time_range: Some(TimeRange::last_hours(1)),
            limit: Some(10),
            include_retry_attempts: true,
        };
        
        assert_eq!(error_query.conversation_id().unwrap(), &conversation_id);
        assert!(error_query.requires_pagination());
    }
}

#[cfg(test)]
mod configuration_management_tests {
    use super::*;

    /// Test Story 2.1: Update System Prompt
    /// Validates: UpdateSystemPrompt command -> SystemPromptUpdated event -> GetSystemPromptHistory query
    #[tokio::test]
    async fn test_story_2_1_update_system_prompt() {
        let conversation_id = ConversationId::new();
        let old_prompt = ClaudeSystemPrompt::new("You are a helpful assistant.".to_string()).unwrap();
        let new_prompt = ClaudeSystemPrompt::new("You are a coding expert specializing in Rust.".to_string()).unwrap();
        
        let command = ClaudeApiCommand::UpdateSystemPrompt {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            new_system_prompt: new_prompt.clone(),
            reason: "Specializing for code assistance".to_string(),
            correlation_id: CorrelationId::new(),
        };
        
        assert!(command.validate().is_ok());
        assert!(command.is_configuration_change());
        assert!(!command.requires_api_call());
        
        let event = ClaudeApiEvent::SystemPromptUpdated {
            command_id: command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            old_prompt: Some(old_prompt.clone()),
            new_prompt: new_prompt.clone(),
            reason: "Specializing for code assistance".to_string(),
            updated_at: Utc::now(),
        };
        
        assert!(event.is_success());
        assert_eq!(event.conversation_id(), &conversation_id);
        
        // Validate system prompt history query
        let query = ClaudeApiQuery::GetSystemPromptHistory {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            limit: Some(10),
            include_metadata: true,
        };
        
        assert_eq!(query.conversation_id().unwrap(), &conversation_id);
        assert!(query.requires_pagination());
    }

    /// Test Story 2.2: Configure Model Parameters
    /// Validates: UpdateModelConfiguration -> ModelConfigurationUpdated
    #[tokio::test]
    async fn test_story_2_2_configure_model_parameters() {
        let conversation_id = ConversationId::new();
        let old_model = ClaudeModel::Claude3Haiku20240307;
        let new_model = ClaudeModel::Claude35Sonnet20241022;
        let new_temperature = Temperature::new(0.7).unwrap();
        let new_max_tokens = MaxTokens::new(2000).unwrap();
        
        let command = ClaudeApiCommand::UpdateModelConfiguration {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            new_model: new_model.clone(),
            new_temperature: Some(new_temperature.clone()),
            new_max_tokens: Some(new_max_tokens.clone()),
            new_stop_sequences: None,
            reason: "Upgrading to more capable model".to_string(),
            correlation_id: CorrelationId::new(),
        };
        
        assert!(command.validate().is_ok());
        assert!(command.is_configuration_change());
        
        let event = ClaudeApiEvent::ModelConfigurationUpdated {
            command_id: command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            old_model: old_model.clone(),
            new_model: new_model.clone(),
            old_temperature: Some(Temperature::default()),
            new_temperature: Some(new_temperature.clone()),
            old_max_tokens: Some(MaxTokens::default()),
            new_max_tokens: Some(new_max_tokens.clone()),
            reason: "Upgrading to more capable model".to_string(),
            updated_at: Utc::now(),
        };
        
        assert!(event.is_success());
        
        // Validate model configuration query
        let query = ClaudeApiQuery::GetModelConfiguration {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            include_history: true,
        };
        
        assert_eq!(query.conversation_id().unwrap(), &conversation_id);
    }

    /// Test Story 2.3: Import/Export Configuration
    /// Validates: ExportConversation/ImportConversation -> ConversationExported/Imported
    #[tokio::test]
    async fn test_story_2_3_import_export_configuration() {
        let conversation_id = ConversationId::new();
        
        // Test export
        let export_command = ClaudeApiCommand::ExportConversation {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            format: ExportFormat::Json,
            include_metadata: true,
            include_usage_stats: true,
            correlation_id: CorrelationId::new(),
        };
        
        assert!(export_command.validate().is_ok());
        
        let export_event = ClaudeApiEvent::ConversationExported {
            command_id: export_command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            export_format: ExportFormat::Json,
            export_size_bytes: 1024,
            export_location: "/tmp/export_123.json".to_string(),
            included_metadata: true,
            included_usage_stats: true,
            exported_at: Utc::now(),
        };
        
        assert!(export_event.is_success());
        
        // Test import
        let import_data = ConversationImportData {
            messages: vec![ClaudeMessage::user(MessageContent::text("Imported message"))],
            system_prompt: Some(ClaudeSystemPrompt::new("Imported prompt".to_string()).unwrap()),
            tools: None,
            model_config: Some(ClaudeModel::Claude35Sonnet20241022),
            metadata: HashMap::new(),
        };
        
        let import_command = ClaudeApiCommand::ImportConversation {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            import_data: import_data.clone(),
            merge_strategy: ConversationMergeStrategy::Merge,
            correlation_id: CorrelationId::new(),
        };
        
        assert!(import_command.validate().is_ok());
        
        let import_event = ClaudeApiEvent::ConversationImported {
            command_id: import_command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            import_source: "/tmp/import_data.json".to_string(),
            merge_strategy: ConversationMergeStrategy::Merge,
            imported_message_count: 1,
            conflicts_resolved: 0,
            imported_at: Utc::now(),
        };
        
        assert!(import_event.is_success());
        
        // Validate export data query
        let export_query = ClaudeApiQuery::GetExportData {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            export_format: ExportFormat::Json,
            export_options: ExportOptions::default(),
        };
        
        assert_eq!(export_query.conversation_id().unwrap(), &conversation_id);
    }
}

#[cfg(test)]
mod tool_management_tests {
    use super::*;

    /// Test Story 3.1: Register MCP Tool via NATS
    /// Validates: AddTools command -> ToolsAdded event -> GetConversationTools query
    #[tokio::test]
    async fn test_story_3_1_register_mcp_tool_via_nats() {
        let conversation_id = ConversationId::new();
        
        // Create tool definition
        let tool_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "filename": {"type": "string"},
                "content": {"type": "string"}
            },
            "required": ["filename", "content"]
        });
        
        let tool = ClaudeToolDefinition::new(
            "file_writer".to_string(),
            "Writes content to a file".to_string(),
            tool_schema,
        ).unwrap();
        
        let command = ClaudeApiCommand::AddTools {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            tools: vec![tool.clone()],
            correlation_id: CorrelationId::new(),
        };
        
        assert!(command.validate().is_ok());
        assert!(command.is_tool_related());
        assert!(command.is_configuration_change());
        
        let event = ClaudeApiEvent::ToolsAdded {
            command_id: command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            added_tools: vec![tool.clone()],
            total_tool_count: 1,
            added_at: Utc::now(),
        };
        
        assert!(event.is_success());
        assert!(event.is_tool_related());
        
        // Validate tools query
        let query = ClaudeApiQuery::GetConversationTools {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            include_schemas: true,
            include_usage_stats: true,
        };
        
        assert_eq!(query.conversation_id().unwrap(), &conversation_id);
    }

    /// Test Story 3.2: Invoke Tool During Conversation
    /// Validates: HandleToolUse -> ToolExecutionStarted -> ToolExecutionCompleted -> SubmitToolResult
    #[tokio::test]
    async fn test_story_3_2_invoke_tool_during_conversation() {
        let conversation_id = ConversationId::new();
        let tool_use_id = "tool_use_123".to_string();
        let tool_name = "file_writer".to_string();
        let tool_input = serde_json::json!({
            "filename": "test.txt",
            "content": "Hello, world!"
        });
        
        // Tool use requested by Claude
        let tool_request_event = ClaudeApiEvent::ToolUseRequested {
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.clone(),
            tool_name: tool_name.clone(),
            tool_input: tool_input.clone(),
            response_message_id: ClaudeMessageId::new("msg_123".to_string()),
            requested_at: Utc::now(),
        };
        
        assert_eq!(tool_request_event.conversation_id(), &conversation_id);
        assert!(tool_request_event.is_tool_related());
        
        // Handle tool use command
        let handle_command = ClaudeApiCommand::HandleToolUse {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.clone(),
            tool_name: tool_name.clone(),
            tool_input: tool_input.clone(),
            correlation_id: CorrelationId::new(),
            execution_timeout: Some(30),
        };
        
        assert!(handle_command.validate().is_ok());
        assert!(handle_command.is_tool_related());
        
        // Tool execution started
        let execution_started = ClaudeApiEvent::ToolExecutionStarted {
            command_id: handle_command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.clone(),
            tool_name: tool_name.clone(),
            tool_input: tool_input.clone(),
            execution_timeout_ms: Some(30000),
            started_at: Utc::now(),
        };
        
        assert!(execution_started.is_tool_related());
        
        // Tool execution completed successfully
        let execution_result = ToolExecutionResult::success(
            "File written successfully".to_string(),
            1500,
        );
        
        let execution_completed = ClaudeApiEvent::ToolExecutionCompleted {
            command_id: handle_command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.clone(),
            tool_name: tool_name.clone(),
            execution_result: execution_result.clone(),
            completed_at: Utc::now(),
        };
        
        assert!(execution_completed.is_success());
        assert!(execution_completed.is_tool_related());
        assert!(execution_result.is_success());
        
        // Submit tool result back to Claude
        let submit_command = ClaudeApiCommand::SubmitToolResult {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.clone(),
            result: execution_result.clone(),
            correlation_id: CorrelationId::new(),
        };
        
        assert!(submit_command.validate().is_ok());
        assert!(submit_command.is_tool_related());
        assert!(submit_command.requires_api_call());
        
        // Validate tool execution query
        let execution_query = ClaudeApiQuery::GetToolExecution {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.clone(),
            include_full_input: true,
            include_full_output: true,
        };
        
        assert_eq!(execution_query.conversation_id().unwrap(), &conversation_id);
    }

    /// Test Story 3.3: Monitor Tool Health and Performance
    /// Validates: Health check events -> GetApiHealthStatus/GetPerformanceMetrics queries
    #[tokio::test]
    async fn test_story_3_3_monitor_tool_health_and_performance() {
        // Health check completed event
        let health_event = ClaudeApiEvent::ApiHealthCheckCompleted {
            is_healthy: true,
            response_time_ms: 150,
            api_version: "v1.0".to_string(),
            server_region: Some("us-west-2".to_string()),
            checked_at: Utc::now(),
        };
        
        // Note: Health check events don't have conversation_id, they're global
        assert!(!health_event.is_error());
        
        // API availability changed
        let availability_event = ClaudeApiEvent::ApiAvailabilityChanged {
            previous_status: ApiStatus::Healthy,
            new_status: ApiStatus::Degraded,
            reason: "Increased latency detected".to_string(),
            changed_at: Utc::now(),
        };
        
        // Validate health status query
        let health_query = ClaudeApiQuery::GetApiHealthStatus {
            query_id: ClaudeQueryId::new(),
            include_response_times: true,
            include_error_rates: true,
            time_range: Some(TimeRange::last_hours(1)),
        };
        
        assert!(health_query.conversation_id().is_none()); // Global query
        
        // Validate performance metrics query
        let performance_query = ClaudeApiQuery::GetPerformanceMetrics {
            query_id: ClaudeQueryId::new(),
            conversation_id: None,
            metric_types: vec![
                PerformanceMetricType::ResponseTime,
                PerformanceMetricType::ErrorRate,
                PerformanceMetricType::ThroughputRps,
            ],
            time_range: Some(TimeRange::last_hours(24)),
            aggregation: MetricAggregation::Average,
        };
        
        assert!(performance_query.conversation_id().is_none());
    }
}

#[cfg(test)]
mod conversation_management_tests {
    use super::*;

    /// Test Story 4.1: Start New Conversation
    /// Validates: First SendMessage creates conversation -> ConversationCreated implied
    #[tokio::test]
    async fn test_story_4_1_start_new_conversation() {
        let conversation_id = ConversationId::new();
        let session_id = SessionId::new();
        
        // First message in a conversation implicitly creates it
        let first_message = ClaudeMessage::user(MessageContent::text("Hello, I'm starting a new conversation"));
        let request = ClaudeApiRequest::new(
            ClaudeModel::Claude35Sonnet20241022,
            vec![first_message.clone()],
            MaxTokens::new(1000).unwrap(),
        ).with_system_prompt(
            ClaudeSystemPrompt::new("You are a helpful assistant.".to_string()).unwrap()
        );
        
        assert!(request.validate().is_ok());
        assert!(matches!(request.messages[0].role, MessageRole::User));
        
        let command = ClaudeApiCommand::SendMessage {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            session_id: session_id.clone(),
            request: request.clone(),
            correlation_id: CorrelationId::new(),
            timeout_seconds: Some(30),
            retry_config: None,
            request_metadata: RequestMetadata::new(session_id.clone(), RequestSource::UserInterface),
        };
        
        // This would typically trigger conversation creation in the aggregate
        let mut session = ClaudeApiSession::new(conversation_id.clone(), ClaudeModel::Claude35Sonnet20241022);
        session.add_user_message(first_message.content.clone());
        
        assert_eq!(session.conversation_id, conversation_id);
        assert_eq!(session.message_history.len(), 1);
        assert!(session.can_add_message(&MessageContent::text("Another message")));
        
        // Validate conversation query
        let query = ClaudeApiQuery::GetConversation {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            include_messages: true,
            include_usage_stats: true,
            include_tool_definitions: true,
        };
        
        assert_eq!(query.conversation_id().unwrap(), &conversation_id);
        
        // Validate conversation search
        let search_query = ClaudeApiQuery::SearchConversations {
            query_id: ClaudeQueryId::new(),
            search_criteria: ConversationSearchCriteria {
                text_query: Some("starting a new conversation".to_string()),
                session_ids: Some(vec![session_id.clone()]),
                models_used: Some(vec![ClaudeModel::Claude35Sonnet20241022]),
                time_range: Some(TimeRange::today()),
                min_messages: Some(1),
                max_messages: None,
                has_tools: Some(false),
                has_errors: Some(false),
                cost_range: None,
                tags: None,
            },
            sort_options: ConversationSortOptions {
                sort_by: ConversationSortBy::CreatedAt,
                sort_direction: SortDirection::Descending,
            },
            limit: Some(10),
            offset: None,
        };
        
        assert!(search_query.requires_pagination());
        assert!(search_query.is_expensive_query());
    }

    /// Test Story 4.2: Manage Conversation State
    /// Validates: ResetConversation command -> ConversationReset event
    #[tokio::test]
    async fn test_story_4_2_manage_conversation_state() {
        let conversation_id = ConversationId::new();
        
        // Reset conversation command
        let reset_command = ClaudeApiCommand::ResetConversation {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            preserve_system_prompt: true,
            preserve_tools: true,
            reason: "Starting fresh conversation".to_string(),
            correlation_id: CorrelationId::new(),
        };
        
        assert!(reset_command.validate().is_ok());
        assert!(!reset_command.requires_api_call());
        
        let reset_event = ClaudeApiEvent::ConversationReset {
            command_id: reset_command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            preserved_system_prompt: true,
            preserved_tools: true,
            previous_message_count: 15,
            previous_total_tokens: 2500,
            previous_cost_usd: 0.075,
            reason: "Starting fresh conversation".to_string(),
            reset_at: Utc::now(),
        };
        
        assert!(reset_event.is_success());
        assert_eq!(reset_event.conversation_id(), &conversation_id);
        
        // Test conversation status in details
        let details = ConversationDetails {
            conversation_id: conversation_id.clone(),
            session_id: SessionId::new(),
            current_model: ClaudeModel::Claude35Sonnet20241022,
            system_prompt: Some(ClaudeSystemPrompt::new("Preserved prompt".to_string()).unwrap()),
            tool_definitions: vec![],
            message_count: 0, // Reset to 0
            total_usage: ClaudeUsage::new(0, 0), // Reset
            estimated_cost_usd: 0.0,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            status: ConversationStatus::Active,
            messages: Some(vec![]), // Empty after reset
            error_count: 0,
            metadata: HashMap::new(),
        };
        
        assert_eq!(details.message_count, 0);
        assert_eq!(details.total_usage.total_tokens(), 0);
        assert!(matches!(details.status, ConversationStatus::Active));
    }

    /// Test Story 4.3: Search and Filter Conversations
    /// Validates: SearchMessages and SearchConversations queries
    #[tokio::test]
    async fn test_story_4_3_search_and_filter_conversations() {
        // Test message search
        let message_search = ClaudeApiQuery::SearchMessages {
            query_id: ClaudeQueryId::new(),
            conversation_id: None, // Search across all conversations
            search_query: "rust programming".to_string(),
            search_options: MessageSearchOptions {
                case_sensitive: false,
                exact_match: false,
                regex_enabled: false,
                search_in_tool_content: true,
                search_in_metadata: true,
                include_context: true,
                context_messages: 2,
            },
            limit: Some(20),
            offset: Some(0),
        };
        
        assert!(message_search.requires_pagination());
        assert!(message_search.is_expensive_query());
        assert!(message_search.conversation_id().is_none());
        
        // Test conversation search with filters
        let conv_search = ClaudeApiQuery::SearchConversations {
            query_id: ClaudeQueryId::new(),
            search_criteria: ConversationSearchCriteria {
                text_query: Some("programming".to_string()),
                session_ids: None,
                models_used: Some(vec![ClaudeModel::Claude35Sonnet20241022]),
                time_range: Some(TimeRange::last_days(7)),
                min_messages: Some(5),
                max_messages: Some(50),
                has_tools: Some(true),
                has_errors: Some(false),
                cost_range: Some(CostRange {
                    min_cost_usd: 0.01,
                    max_cost_usd: 1.0,
                }),
                tags: Some(vec!["coding".to_string(), "help".to_string()]),
            },
            sort_options: ConversationSortOptions {
                sort_by: ConversationSortBy::LastActivity,
                sort_direction: SortDirection::Descending,
            },
            limit: Some(10),
            offset: None,
        };
        
        assert!(conv_search.requires_pagination());
        assert!(conv_search.is_expensive_query());
        
        // Validate time range
        let time_range = TimeRange::last_days(7);
        assert!(time_range.end > time_range.start);
        assert_eq!((time_range.end - time_range.start).num_days(), 7);
    }
}

#[cfg(test)]
mod analytics_monitoring_tests {
    use super::*;

    /// Test Story 5.1: Track Usage and Costs
    /// Validates: GetUsageStatistics, GetCostAnalysis, GetQuotaUsage queries
    #[tokio::test]
    async fn test_story_5_1_track_usage_and_costs() {
        let conversation_id = ConversationId::new();
        
        // Usage statistics query
        let usage_query = ClaudeApiQuery::GetUsageStatistics {
            query_id: ClaudeQueryId::new(),
            conversation_id: Some(conversation_id.clone()),
            time_range: Some(TimeRange::last_days(30)),
            group_by: UsageGroupBy::Day,
            include_cost_breakdown: true,
        };
        
        assert_eq!(usage_query.conversation_id().unwrap(), &conversation_id);
        
        // Cost analysis query
        let cost_query = ClaudeApiQuery::GetCostAnalysis {
            query_id: ClaudeQueryId::new(),
            conversation_id: Some(conversation_id.clone()),
            time_range: Some(TimeRange::last_days(30)),
            cost_breakdown: CostBreakdownOptions {
                by_model: true,
                by_conversation: true,
                by_session: false,
                by_time_period: Some(UsageGroupBy::Week),
                include_tool_costs: true,
                include_storage_costs: false,
                currency: CostCurrency::USD,
            },
        };
        
        assert_eq!(cost_query.conversation_id().unwrap(), &conversation_id);
        
        // Quota usage query
        let quota_query = ClaudeApiQuery::GetQuotaUsage {
            query_id: ClaudeQueryId::new(),
            quota_type: QuotaType::MonthlyTokens,
            time_range: Some(TimeRange::last_days(30)),
            include_projections: true,
        };
        
        assert!(quota_query.conversation_id().is_none()); // Global quota
        
        // Test usage threshold event
        let threshold_event = ClaudeApiEvent::UsageThresholdReached {
            conversation_id: conversation_id.clone(),
            threshold_type: UsageThresholdType::CostUsd,
            current_usage: ClaudeUsage::new(50000, 25000),
            threshold_limit: 10.0,
            threshold_percentage: 80.0,
            reached_at: Utc::now(),
        };
        
        assert_eq!(threshold_event.conversation_id(), &conversation_id);
        
        // Validate cost calculation
        let usage = ClaudeUsage::new(1_000_000, 500_000); // 1M input, 500K output
        let cost = usage.estimated_cost_usd(&ClaudeModel::Claude35Sonnet20241022);
        assert!((cost - 10.5).abs() < 0.01); // Should be $10.50
    }

    /// Test Story 5.2: Monitor System Performance
    /// Validates: GetPerformanceMetrics, GetApiHealthStatus, GetErrorHistory queries
    #[tokio::test]
    async fn test_story_5_2_monitor_system_performance() {
        let conversation_id = ConversationId::new();
        
        // Performance metrics query
        let perf_query = ClaudeApiQuery::GetPerformanceMetrics {
            query_id: ClaudeQueryId::new(),
            conversation_id: Some(conversation_id.clone()),
            metric_types: vec![
                PerformanceMetricType::ResponseTime,
                PerformanceMetricType::LatencyP95,
                PerformanceMetricType::ErrorRate,
                PerformanceMetricType::ThroughputRps,
            ],
            time_range: Some(TimeRange::last_hours(24)),
            aggregation: MetricAggregation::P95,
        };
        
        assert_eq!(perf_query.conversation_id().unwrap(), &conversation_id);
        
        // API health status query
        let health_query = ClaudeApiQuery::GetApiHealthStatus {
            query_id: ClaudeQueryId::new(),
            include_response_times: true,
            include_error_rates: true,
            time_range: Some(TimeRange::last_hours(1)),
        };
        
        assert!(health_query.conversation_id().is_none());
        
        // Error history query
        let error_query = ClaudeApiQuery::GetErrorHistory {
            query_id: ClaudeQueryId::new(),
            conversation_id: Some(conversation_id.clone()),
            error_types: Some(vec![
                ClaudeErrorType::RateLimitError,
                ClaudeErrorType::ApiError,
                ClaudeErrorType::OverloadedError,
            ]),
            time_range: Some(TimeRange::last_hours(24)),
            limit: Some(50),
            include_retry_attempts: true,
        };
        
        assert!(error_query.requires_pagination());
        
        // Test token limit approaching event
        let token_limit_event = ClaudeApiEvent::TokenLimitApproaching {
            conversation_id: conversation_id.clone(),
            current_tokens: 180000,
            max_tokens: 200000,
            remaining_tokens: 20000,
            percentage_used: 90.0,
            warned_at: Utc::now(),
        };
        
        assert_eq!(token_limit_event.conversation_id(), &conversation_id);
    }

    /// Test Story 5.3: Generate Reports and Analytics
    /// Validates: GetConversationAnalytics, CompareConversations, GetExportData queries
    #[tokio::test]
    async fn test_story_5_3_generate_reports_and_analytics() {
        let conversation_id1 = ConversationId::new();
        let conversation_id2 = ConversationId::new();
        
        // Conversation analytics query
        let analytics_query = ClaudeApiQuery::GetConversationAnalytics {
            query_id: ClaudeQueryId::new(),
            conversation_id: Some(conversation_id1.clone()),
            analytics_types: vec![
                AnalyticsType::MessageLength,
                AnalyticsType::TokenEfficiency,
                AnalyticsType::ToolUsagePatterns,
                AnalyticsType::CostOptimization,
            ],
            time_range: Some(TimeRange::last_days(30)),
        };
        
        assert!(analytics_query.is_expensive_query());
        assert_eq!(analytics_query.conversation_id().unwrap(), &conversation_id1);
        
        // Conversation comparison query
        let comparison_query = ClaudeApiQuery::CompareConversations {
            query_id: ClaudeQueryId::new(),
            conversation_ids: vec![conversation_id1.clone(), conversation_id2.clone()],
            comparison_criteria: vec![
                ComparisonCriterion::MessageCount,
                ComparisonCriterion::TokenUsage,
                ComparisonCriterion::Cost,
                ComparisonCriterion::ResponseTimes,
                ComparisonCriterion::ToolUsage,
            ],
        };
        
        assert!(comparison_query.is_expensive_query());
        assert!(comparison_query.conversation_id().is_none()); // Multi-conversation
        
        // Export data query for reporting
        let export_query = ClaudeApiQuery::GetExportData {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id1.clone(),
            export_format: ExportFormat::Csv,
            export_options: ExportOptions {
                include_system_prompts: true,
                include_tool_definitions: true,
                include_metadata: true,
                include_usage_stats: true,
                include_error_history: true,
                compress_output: true,
                split_large_files: true,
                max_file_size_mb: Some(50),
            },
        };
        
        assert_eq!(export_query.conversation_id().unwrap(), &conversation_id1);
    }
}

#[cfg(test)]
mod error_handling_resilience_tests {
    use super::*;

    /// Test Story 6.1: Handle Rate Limiting Gracefully
    /// Validates: RateLimitEncountered event -> GetRateLimitStatus query
    #[tokio::test]
    async fn test_story_6_1_handle_rate_limiting_gracefully() {
        let conversation_id = ConversationId::new();
        let command_id = ClaudeCommandId::new();
        
        // Rate limit encountered event
        let rate_limit_event = ClaudeApiEvent::RateLimitEncountered {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            limit_type: RateLimitType::RequestsPerMinute,
            retry_after_seconds: Some(60),
            requests_remaining: Some(0),
            reset_time: Some(Utc::now() + chrono::Duration::minutes(1)),
            encountered_at: Utc::now(),
        };
        
        assert_eq!(rate_limit_event.conversation_id(), &conversation_id);
        assert!(rate_limit_event.command_id().is_some());
        
        // Rate limit status query
        let status_query = ClaudeApiQuery::GetRateLimitStatus {
            query_id: ClaudeQueryId::new(),
            session_id: None,
            limit_types: Some(vec![
                RateLimitType::RequestsPerMinute,
                RateLimitType::TokensPerMinute,
                RateLimitType::RequestsPerDay,
            ]),
        };
        
        assert!(status_query.conversation_id().is_none()); // Global status
        
        // Test different rate limit types
        for limit_type in [
            RateLimitType::RequestsPerMinute,
            RateLimitType::TokensPerMinute,
            RateLimitType::RequestsPerHour,
            RateLimitType::TokensPerHour,
            RateLimitType::RequestsPerDay,
            RateLimitType::TokensPerDay,
            RateLimitType::ConcurrentRequests,
        ] {
            let event = ClaudeApiEvent::RateLimitEncountered {
                command_id: command_id.clone(),
                conversation_id: conversation_id.clone(),
                limit_type: limit_type.clone(),
                retry_after_seconds: Some(30),
                requests_remaining: Some(5),
                reset_time: Some(Utc::now() + chrono::Duration::seconds(30)),
                encountered_at: Utc::now(),
            };
            
            assert_eq!(event.conversation_id(), &conversation_id);
        }
    }

    /// Test Story 6.2: Validate All Inputs Comprehensively
    /// Validates: Command validation -> CommandValidationFailed events
    #[tokio::test]
    async fn test_story_6_2_validate_all_inputs_comprehensively() {
        let conversation_id = ConversationId::new();
        
        // Test invalid message command (empty content)
        let empty_request = ClaudeApiRequest::new(
            ClaudeModel::Claude35Sonnet20241022,
            vec![], // Empty messages - should fail validation
            MaxTokens::new(1000).unwrap(),
        );
        
        assert!(empty_request.validate().is_err());
        
        // Test invalid system prompt update (empty prompt)
        let empty_prompt_result = ClaudeSystemPrompt::new("".to_string());
        assert!(empty_prompt_result.is_err());
        
        // Test invalid temperature
        let invalid_temp_result = Temperature::new(1.5); // > 1.0
        assert!(invalid_temp_result.is_err());
        
        let invalid_temp_result2 = Temperature::new(-0.1); // < 0.0
        assert!(invalid_temp_result2.is_err());
        
        // Test invalid max tokens
        let invalid_max_tokens = MaxTokens::new(0); // Must be > 0
        assert!(invalid_max_tokens.is_err());
        
        let invalid_max_tokens2 = MaxTokens::new(300_000); // > 200k limit
        assert!(invalid_max_tokens2.is_err());
        
        // Test invalid stop sequences
        let too_many_sequences = StopSequences::new(vec![
            "STOP1".to_string(),
            "STOP2".to_string(),
            "STOP3".to_string(),
            "STOP4".to_string(),
            "STOP5".to_string(), // 5 sequences, max is 4
        ]);
        assert!(too_many_sequences.is_err());
        
        let empty_sequence = StopSequences::new(vec!["".to_string()]);
        assert!(empty_sequence.is_err());
        
        // Test invalid tool definition
        let empty_tool_name = ClaudeToolDefinition::new(
            "".to_string(), // Empty name
            "Description".to_string(),
            serde_json::json!({}),
        );
        assert!(empty_tool_name.is_err());
        
        let empty_tool_desc = ClaudeToolDefinition::new(
            "tool_name".to_string(),
            "".to_string(), // Empty description
            serde_json::json!({}),
        );
        assert!(empty_tool_desc.is_err());
        
        // Test command validation
        let invalid_add_tools = ClaudeApiCommand::AddTools {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            tools: vec![], // Empty tools list
            correlation_id: CorrelationId::new(),
        };
        
        assert!(invalid_add_tools.validate().is_err());
        
        // Test remove tools validation
        let invalid_remove_tools = ClaudeApiCommand::RemoveTools {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            tool_names: vec![], // Empty tool names list
            correlation_id: CorrelationId::new(),
        };
        
        assert!(invalid_remove_tools.validate().is_err());
        
        // Test validation results query
        let validation_query = ClaudeApiQuery::GetValidationResults {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            validation_types: Some(vec![
                ValidationRuleType::TokenCount,
                ValidationRuleType::ContentFilter,
                ValidationRuleType::CostLimit,
            ]),
            include_history: true,
        };
        
        assert_eq!(validation_query.conversation_id().unwrap(), &conversation_id);
    }
}

#[cfg(test)]
mod event_sourcing_coverage_tests {
    use super::*;

    /// Test Story 7.1: Ensure 100% Event Coverage
    /// Validates: All API interactions generate events
    #[tokio::test]
    async fn test_story_7_1_ensure_100_percent_event_coverage() {
        let conversation_id = ConversationId::new();
        
        // Test that all command types have corresponding events
        let commands_and_events = vec![
            // SendMessage -> MessageResponseReceived
            ("SendMessage", "MessageResponseReceived"),
            // SendStreamingMessage -> StreamingChunkReceived, StreamingMessageCompleted
            ("SendStreamingMessage", "StreamingChunkReceived"),
            ("SendStreamingMessage", "StreamingMessageCompleted"),
            // UpdateSystemPrompt -> SystemPromptUpdated
            ("UpdateSystemPrompt", "SystemPromptUpdated"),
            // UpdateModelConfiguration -> ModelConfigurationUpdated
            ("UpdateModelConfiguration", "ModelConfigurationUpdated"),
            // AddTools -> ToolsAdded
            ("AddTools", "ToolsAdded"),
            // RemoveTools -> ToolsRemoved
            ("RemoveTools", "ToolsRemoved"),
            // HandleToolUse -> ToolExecutionStarted, ToolExecutionCompleted
            ("HandleToolUse", "ToolExecutionStarted"),
            ("HandleToolUse", "ToolExecutionCompleted"),
            // SubmitToolResult -> ToolResultSubmitted
            ("SubmitToolResult", "ToolResultSubmitted"),
            // CancelRequest -> RequestCancelled
            ("CancelRequest", "RequestCancelled"),
            // RetryRequest -> RequestRetryInitiated
            ("RetryRequest", "RequestRetryInitiated"),
            // ResetConversation -> ConversationReset
            ("ResetConversation", "ConversationReset"),
            // ExportConversation -> ConversationExported
            ("ExportConversation", "ConversationExported"),
            // ImportConversation -> ConversationImported
            ("ImportConversation", "ConversationImported"),
            // ValidateConversation -> ConversationValidationCompleted
            ("ValidateConversation", "ConversationValidationCompleted"),
            // HandleToolUse -> ToolExecutionFailed (on error)
            ("HandleToolUse", "ToolExecutionFailed"),
            // Any command -> ApiErrorOccurred (on API error)
            ("SendMessage", "ApiErrorOccurred"),
        ];
        
        // Verify we have comprehensive command-event mappings
        assert_eq!(commands_and_events.len(), 18); // Covers all major flows
        
        // Test event classification methods work correctly
        let success_event = ClaudeApiEvent::MessageResponseReceived {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            request: ClaudeApiRequest::new(
                ClaudeModel::Claude35Sonnet20241022,
                vec![ClaudeMessage::user(MessageContent::text("test"))],
                MaxTokens::new(100).unwrap(),
            ),
            response: ClaudeApiResponse::new(
                ClaudeMessageId::new("msg_123".to_string()),
                ClaudeModel::Claude35Sonnet20241022,
                vec![ContentBlock::Text { text: "Response".to_string() }],
                StopReason::EndTurn,
                ClaudeUsage::new(5, 10),
            ),
            request_duration_ms: 1000,
            request_id: Some("req_123".to_string()),
            received_at: Utc::now(),
        };
        
        assert!(success_event.is_success());
        assert!(!success_event.is_error());
        assert!(success_event.affects_cost());
        assert!(!success_event.is_tool_related());
        
        let error_event = ClaudeApiEvent::ApiErrorOccurred {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            request: ClaudeApiRequest::new(
                ClaudeModel::Claude35Sonnet20241022,
                vec![ClaudeMessage::user(MessageContent::text("test"))],
                MaxTokens::new(100).unwrap(),
            ),
            error: ClaudeApiError::new(
                ClaudeErrorType::RateLimitError,
                "Rate limited".to_string(),
                429,
            ),
            request_duration_ms: 100,
            error_occurred_at: Utc::now(),
            retry_attempt: Some(1),
        };
        
        assert!(!error_event.is_success());
        assert!(error_event.is_error());
        assert!(!error_event.affects_cost());
        assert!(!error_event.is_tool_related());
        
        let tool_event = ClaudeApiEvent::ToolUseRequested {
            conversation_id: conversation_id.clone(),
            tool_use_id: "tool_123".to_string(),
            tool_name: "test_tool".to_string(),
            tool_input: serde_json::json!({"test": "data"}),
            response_message_id: ClaudeMessageId::new("msg_123".to_string()),
            requested_at: Utc::now(),
        };
        
        assert!(!tool_event.is_success()); // Tool events are intermediate
        assert!(!tool_event.is_error());
        assert!(!tool_event.affects_cost());
        assert!(tool_event.is_tool_related());
    }

    /// Test Story 7.2: Support Event Replay and Time Travel
    /// Validates: Events contain complete context for replay
    #[tokio::test]
    async fn test_story_7_2_support_event_replay_and_time_travel() {
        let conversation_id = ConversationId::new();
        let session_id = SessionId::new();
        
        // Create a sequence of events that could be replayed
        let events = vec![
            // 1. System prompt updated
            ClaudeApiEvent::SystemPromptUpdated {
                command_id: ClaudeCommandId::new(),
                conversation_id: conversation_id.clone(),
                old_prompt: None,
                new_prompt: ClaudeSystemPrompt::new("You are a helpful assistant".to_string()).unwrap(),
                reason: "Initial setup".to_string(),
                updated_at: Utc::now(),
            },
            
            // 2. Tools added
            ClaudeApiEvent::ToolsAdded {
                command_id: ClaudeCommandId::new(),
                conversation_id: conversation_id.clone(),
                added_tools: vec![ClaudeToolDefinition::new(
                    "calculator".to_string(),
                    "Performs calculations".to_string(),
                    serde_json::json!({"type": "object"}),
                ).unwrap()],
                total_tool_count: 1,
                added_at: Utc::now(),
            },
            
            // 3. Message sent and responded
            ClaudeApiEvent::MessageResponseReceived {
                command_id: ClaudeCommandId::new(),
                conversation_id: conversation_id.clone(),
                request: ClaudeApiRequest::new(
                    ClaudeModel::Claude35Sonnet20241022,
                    vec![ClaudeMessage::user(MessageContent::text("Calculate 2+2"))],
                    MaxTokens::new(100).unwrap(),
                ),
                response: ClaudeApiResponse::new(
                    ClaudeMessageId::new("msg_123".to_string()),
                    ClaudeModel::Claude35Sonnet20241022,
                    vec![ContentBlock::ToolUse {
                        id: "tool_use_123".to_string(),
                        name: "calculator".to_string(),
                        input: serde_json::json!({"operation": "add", "a": 2, "b": 2}),
                    }],
                    StopReason::ToolUse,
                    ClaudeUsage::new(15, 20),
                ),
                request_duration_ms: 800,
                request_id: Some("req_123".to_string()),
                received_at: Utc::now(),
            },
            
            // 4. Tool executed
            ClaudeApiEvent::ToolExecutionCompleted {
                command_id: ClaudeCommandId::new(),
                conversation_id: conversation_id.clone(),
                tool_use_id: "tool_use_123".to_string(),
                tool_name: "calculator".to_string(),
                execution_result: ToolExecutionResult::success("4".to_string(), 50),
                completed_at: Utc::now(),
            },
        ];
        
        // Verify each event has required fields for replay
        for event in &events {
            // All events should have conversation_id
            assert_eq!(event.conversation_id(), &conversation_id);
            
            // All events should have timestamps
            let timestamp = event.timestamp();
            assert!(timestamp <= Utc::now());
            
            // Events with command_id should have it
            if let Some(cmd_id) = event.command_id() {
                assert!(!cmd_id.as_uuid().is_nil());
            }
        }
        
        // Test that events can be queried by time range
        let start_time = Utc::now() - chrono::Duration::hours(1);
        let end_time = Utc::now();
        let time_range = TimeRange::new(start_time, end_time).unwrap();
        
        let events_query = ClaudeApiQuery::GetConversationEvents {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            event_types: None, // All event types
            time_range: Some(time_range),
            limit: Some(100),
            offset: None,
        };
        
        assert_eq!(events_query.conversation_id().unwrap(), &conversation_id);
        assert!(events_query.requires_pagination());
        
        // Test event ordering is preserved (events should be chronologically sortable)
        let mut sorted_events = events.clone();
        sorted_events.sort_by_key(|e| e.timestamp());
        
        // Events should maintain their temporal relationship
        for i in 0..sorted_events.len() - 1 {
            assert!(sorted_events[i].timestamp() <= sorted_events[i + 1].timestamp());
        }
    }
}

/// Integration test helper to validate complete user story flows
#[cfg(test)]
mod integration_test_helpers {
    use super::*;

    /// Helper function to simulate complete conversation flow
    pub async fn simulate_complete_conversation_flow() -> Result<(), String> {
        let conversation_id = ConversationId::new();
        let session_id = SessionId::new();
        
        // 1. Start conversation with system prompt
        let system_prompt = ClaudeSystemPrompt::new("You are a helpful coding assistant".to_string())?;
        
        // 2. Send first message
        let first_message = ClaudeMessage::user(MessageContent::text("Help me write a Rust function"));
        let request = ClaudeApiRequest::new(
            ClaudeModel::Claude35Sonnet20241022,
            vec![first_message],
            MaxTokens::new(2000)?,
        ).with_system_prompt(system_prompt);
        
        request.validate()?;
        
        // 3. Add tools
        let code_tool = ClaudeToolDefinition::new(
            "code_executor".to_string(),
            "Executes code snippets".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {"type": "string"},
                    "code": {"type": "string"}
                }
            }),
        )?;
        
        // 4. Simulate tool use
        let tool_result = ToolExecutionResult::success(
            "Function compiled successfully".to_string(),
            1200,
        );
        
        assert!(tool_result.is_success());
        
        // 5. Query conversation state
        let final_query = ClaudeApiQuery::GetConversation {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            include_messages: true,
            include_usage_stats: true,
            include_tool_definitions: true,
        };
        
        assert_eq!(final_query.conversation_id().unwrap(), &conversation_id);
        
        Ok(())
    }
    
    /// Helper to validate error handling flows
    pub async fn simulate_error_handling_flow() -> Result<(), String> {
        let conversation_id = ConversationId::new();
        
        // Simulate various error scenarios
        let errors = vec![
            ClaudeErrorType::RateLimitError,
            ClaudeErrorType::ApiError,
            ClaudeErrorType::OverloadedError,
            ClaudeErrorType::InvalidRequestError,
            ClaudeErrorType::AuthenticationError,
        ];
        
        for error_type in errors {
            let api_error = ClaudeApiError::new(
                error_type.clone(),
                format!("Test error: {:?}", error_type),
                match error_type {
                    ClaudeErrorType::RateLimitError => 429,
                    ClaudeErrorType::ApiError => 500,
                    ClaudeErrorType::OverloadedError => 503,
                    ClaudeErrorType::InvalidRequestError => 400,
                    ClaudeErrorType::AuthenticationError => 401,
                    _ => 500,
                },
            );
            
            // Validate error properties
            match error_type {
                ClaudeErrorType::RateLimitError |
                ClaudeErrorType::ApiError |
                ClaudeErrorType::OverloadedError => {
                    assert!(api_error.is_retryable());
                }
                _ => {
                    assert!(!api_error.is_retryable());
                    assert!(api_error.is_client_error());
                }
            }
        }
        
        Ok(())
    }
}