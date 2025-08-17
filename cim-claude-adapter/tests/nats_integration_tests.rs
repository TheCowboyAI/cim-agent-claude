/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! NATS Integration Tests
//!
//! These tests validate the complete NATS message flow patterns described in ARCHITECTURE.md
//! Each test simulates real NATS subject patterns and message flows for Commands, Events, and Queries.

use cim_claude_adapter::domain::{
    claude_api::*,
    claude_commands::*,
    claude_events::*,
    claude_queries::*,
    value_objects::*,
};
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;
use tokio;

// Mock NATS client for testing
#[derive(Debug, Clone)]
pub struct MockNatsClient {
    pub published_messages: Vec<NatsMessage>,
    pub subscriptions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NatsMessage {
    pub subject: String,
    pub payload: Vec<u8>,
    pub headers: Option<HashMap<String, String>>,
    pub timestamp: DateTime<Utc>,
}

impl MockNatsClient {
    pub fn new() -> Self {
        Self {
            published_messages: Vec::new(),
            subscriptions: Vec::new(),
        }
    }
    
    pub async fn publish(&mut self, subject: &str, payload: &[u8]) -> Result<(), String> {
        self.published_messages.push(NatsMessage {
            subject: subject.to_string(),
            payload: payload.to_vec(),
            headers: None,
            timestamp: Utc::now(),
        });
        Ok(())
    }
    
    pub async fn publish_with_headers(
        &mut self,
        subject: &str,
        payload: &[u8],
        headers: HashMap<String, String>,
    ) -> Result<(), String> {
        self.published_messages.push(NatsMessage {
            subject: subject.to_string(),
            payload: payload.to_vec(),
            headers: Some(headers),
            timestamp: Utc::now(),
        });
        Ok(())
    }
    
    pub async fn subscribe(&mut self, subject: &str) -> Result<(), String> {
        self.subscriptions.push(subject.to_string());
        Ok(())
    }
    
    pub fn get_messages_for_subject(&self, subject_pattern: &str) -> Vec<&NatsMessage> {
        self.published_messages
            .iter()
            .filter(|msg| msg.subject.starts_with(subject_pattern))
            .collect()
    }
}

#[cfg(test)]
mod claude_api_command_flows {
    use super::*;

    /// Test NATS flow for Story 1.1: Send Message to Claude
    /// Subject Pattern: cim.claude.conv.cmd.send.{conv_id} -> cim.claude.conv.evt.response_received.{conv_id}
    #[tokio::test]
    async fn test_nats_send_message_flow() {
        let mut nats_client = MockNatsClient::new();
        let conversation_id = ConversationId::new();
        let command_id = ClaudeCommandId::new();
        let correlation_id = CorrelationId::new();
        
        // 1. Publish SendMessage command to NATS
        let command = ClaudeApiCommand::SendMessage {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            session_id: SessionId::new(),
            request: ClaudeApiRequest::new(
                ClaudeModel::Claude35Sonnet20241022,
                vec![ClaudeMessage::user(MessageContent::text("Hello Claude!"))],
                MaxTokens::new(1000).unwrap(),
            ),
            correlation_id: correlation_id.clone(),
            timeout_seconds: Some(30),
            retry_config: Some(RetryConfiguration::default()),
            request_metadata: RequestMetadata::new(SessionId::new(), RequestSource::UserInterface),
        };
        
        let command_subject = format!("cim.claude.conv.cmd.send.{}", conversation_id);
        let command_payload = serde_json::to_vec(&command).unwrap();
        
        // Add NATS headers for tracing and correlation
        let mut headers = HashMap::new();
        headers.insert("correlation-id".to_string(), correlation_id.to_string());
        headers.insert("command-id".to_string(), command_id.to_string());
        headers.insert("timestamp".to_string(), Utc::now().to_rfc3339());
        
        nats_client.publish_with_headers(&command_subject, &command_payload, headers).await.unwrap();
        
        // Verify command was published correctly
        let command_messages = nats_client.get_messages_for_subject("cim.claude.conv.cmd.send");
        assert_eq!(command_messages.len(), 1);
        assert_eq!(command_messages[0].subject, command_subject);
        
        // Verify headers
        let msg_headers = command_messages[0].headers.as_ref().unwrap();
        assert_eq!(msg_headers.get("correlation-id").unwrap(), &correlation_id.to_string());
        assert_eq!(msg_headers.get("command-id").unwrap(), &command_id.to_string());
        
        // 2. Simulate successful API response and publish event
        let response = ClaudeApiResponse::new(
            ClaudeMessageId::new("msg_123".to_string()),
            ClaudeModel::Claude35Sonnet20241022,
            vec![ContentBlock::Text { text: "Hello! How can I help you?".to_string() }],
            StopReason::EndTurn,
            ClaudeUsage::new(10, 15),
        );
        
        let event = ClaudeApiEvent::MessageResponseReceived {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            request: command.clone().into(), // Extract request from command
            response: response.clone(),
            request_duration_ms: 1200,
            request_id: Some("req_123".to_string()),
            received_at: Utc::now(),
        };
        
        let event_subject = format!("cim.claude.conv.evt.response_received.{}", conversation_id);
        let event_payload = serde_json::to_vec(&event).unwrap();
        
        let mut event_headers = HashMap::new();
        event_headers.insert("correlation-id".to_string(), correlation_id.to_string());
        event_headers.insert("command-id".to_string(), command_id.to_string());
        event_headers.insert("event-type".to_string(), "MessageResponseReceived".to_string());
        event_headers.insert("timestamp".to_string(), Utc::now().to_rfc3339());
        
        nats_client.publish_with_headers(&event_subject, &event_payload, event_headers).await.unwrap();
        
        // Verify event was published correctly
        let event_messages = nats_client.get_messages_for_subject("cim.claude.conv.evt.response_received");
        assert_eq!(event_messages.len(), 1);
        assert_eq!(event_messages[0].subject, event_subject);
        
        // Verify event correlation
        let event_msg_headers = event_messages[0].headers.as_ref().unwrap();
        assert_eq!(event_msg_headers.get("correlation-id").unwrap(), &correlation_id.to_string());
        assert_eq!(event_msg_headers.get("command-id").unwrap(), &command_id.to_string());
        
        // 3. Simulate query request
        let query = ClaudeApiQuery::GetConversation {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            include_messages: true,
            include_usage_stats: true,
            include_tool_definitions: false,
        };
        
        let query_subject = format!("cim.claude.queries.conversation.{}", conversation_id);
        let query_payload = serde_json::to_vec(&query).unwrap();
        
        nats_client.publish(&query_subject, &query_payload).await.unwrap();
        
        // Verify query was published correctly
        let query_messages = nats_client.get_messages_for_subject("cim.claude.queries.conversation");
        assert_eq!(query_messages.len(), 1);
        
        // Verify complete flow - should have command, event, and query
        assert_eq!(nats_client.published_messages.len(), 3);
    }

    /// Test NATS flow for Story 1.2: Stream Message Response
    /// Multiple events: StreamingChunkReceived -> StreamingMessageCompleted
    #[tokio::test]
    async fn test_nats_streaming_message_flow() {
        let mut nats_client = MockNatsClient::new();
        let conversation_id = ConversationId::new();
        let command_id = ClaudeCommandId::new();
        
        // 1. Publish streaming command
        let command = ClaudeApiCommand::SendStreamingMessage {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            session_id: SessionId::new(),
            request: ClaudeApiRequest::new(
                ClaudeModel::Claude35Sonnet20241022,
                vec![ClaudeMessage::user(MessageContent::text("Tell me a story"))],
                MaxTokens::new(2000).unwrap(),
            ).with_stream(true),
            correlation_id: CorrelationId::new(),
            stream_handler: StreamHandlerConfig::default(),
            timeout_seconds: Some(60),
            request_metadata: RequestMetadata::new(SessionId::new(), RequestSource::UserInterface),
        };
        
        let command_subject = format!("cim.claude.conv.cmd.stream.{}", conversation_id);
        let command_payload = serde_json::to_vec(&command).unwrap();
        nats_client.publish(&command_subject, &command_payload).await.unwrap();
        
        // 2. Simulate multiple streaming chunks
        for chunk_seq in 1..=3 {
            let chunk_event = ClaudeApiEvent::StreamingChunkReceived {
                command_id: command_id.clone(),
                conversation_id: conversation_id.clone(),
                chunk_sequence: chunk_seq,
                chunk_content: StreamChunk {
                    chunk_type: StreamChunkType::ContentBlockDelta,
                    content: format!("Chunk {} content", chunk_seq),
                    token_count: Some(5),
                    is_complete: false,
                    metadata: None,
                },
                accumulated_tokens: chunk_seq * 5,
                received_at: Utc::now(),
            };
            
            let chunk_subject = format!("cim.claude.conv.evt.chunk_received.{}", conversation_id);
            let chunk_payload = serde_json::to_vec(&chunk_event).unwrap();
            
            let mut chunk_headers = HashMap::new();
            chunk_headers.insert("chunk-sequence".to_string(), chunk_seq.to_string());
            chunk_headers.insert("command-id".to_string(), command_id.to_string());
            
            nats_client.publish_with_headers(&chunk_subject, &chunk_payload, chunk_headers).await.unwrap();
        }
        
        // 3. Simulate streaming completion
        let completion_event = ClaudeApiEvent::StreamingMessageCompleted {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            total_chunks: 3,
            final_response: ClaudeApiResponse::new(
                ClaudeMessageId::new("msg_stream_123".to_string()),
                ClaudeModel::Claude35Sonnet20241022,
                vec![ContentBlock::Text { text: "Once upon a time...".to_string() }],
                StopReason::EndTurn,
                ClaudeUsage::new(20, 30),
            ),
            total_duration_ms: 3000,
            completed_at: Utc::now(),
        };
        
        let completion_subject = format!("cim.claude.conv.evt.stream_completed.{}", conversation_id);
        let completion_payload = serde_json::to_vec(&completion_event).unwrap();
        nats_client.publish(&completion_subject, &completion_payload).await.unwrap();
        
        // Verify streaming flow
        let chunk_messages = nats_client.get_messages_for_subject("cim.claude.conv.evt.chunk_received");
        assert_eq!(chunk_messages.len(), 3);
        
        let completion_messages = nats_client.get_messages_for_subject("cim.claude.conv.evt.stream_completed");
        assert_eq!(completion_messages.len(), 1);
        
        // Verify chunk ordering through headers
        for (i, msg) in chunk_messages.iter().enumerate() {
            let headers = msg.headers.as_ref().unwrap();
            assert_eq!(headers.get("chunk-sequence").unwrap(), &((i + 1) as u32).to_string());
        }
        
        // Total: 1 command + 3 chunks + 1 completion = 5 messages
        assert_eq!(nats_client.published_messages.len(), 5);
    }
}

#[cfg(test)]
mod configuration_management_flows {
    use super::*;

    /// Test NATS flow for Story 2.1: Update System Prompt
    /// Subject Pattern: cim.claude.config.cmd.update_system_prompt.{config_id} -> cim.claude.config.evt.system_prompt_updated.{config_id}
    #[tokio::test]
    async fn test_nats_config_update_flow() {
        let mut nats_client = MockNatsClient::new();
        let config_id = "main"; // Using string ID for config
        let command_id = ClaudeCommandId::new();
        let conversation_id = ConversationId::new();
        
        // 1. Publish system prompt update command
        let old_prompt = ClaudeSystemPrompt::new("You are a helpful assistant.".to_string()).unwrap();
        let new_prompt = ClaudeSystemPrompt::new("You are a Rust programming expert.".to_string()).unwrap();
        
        let command = ClaudeApiCommand::UpdateSystemPrompt {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            new_system_prompt: new_prompt.clone(),
            reason: "Specializing for Rust development".to_string(),
            correlation_id: CorrelationId::new(),
        };
        
        let command_subject = format!("cim.claude.config.cmd.update_system_prompt.{}", config_id);
        let command_payload = serde_json::to_vec(&command).unwrap();
        
        let mut headers = HashMap::new();
        headers.insert("config-id".to_string(), config_id.to_string());
        headers.insert("command-id".to_string(), command_id.to_string());
        headers.insert("change-reason".to_string(), "Specializing for Rust development".to_string());
        
        nats_client.publish_with_headers(&command_subject, &command_payload, headers).await.unwrap();
        
        // 2. Simulate configuration processor handling command
        let config_event = ClaudeApiEvent::SystemPromptUpdated {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            old_prompt: Some(old_prompt.clone()),
            new_prompt: new_prompt.clone(),
            reason: "Specializing for Rust development".to_string(),
            updated_at: Utc::now(),
        };
        
        let event_subject = format!("cim.claude.config.evt.system_prompt_updated.{}", config_id);
        let event_payload = serde_json::to_vec(&config_event).unwrap();
        
        let mut event_headers = HashMap::new();
        event_headers.insert("config-id".to_string(), config_id.to_string());
        event_headers.insert("command-id".to_string(), command_id.to_string());
        event_headers.insert("event-type".to_string(), "SystemPromptUpdated".to_string());
        event_headers.insert("old-prompt-hash".to_string(), format!("{:x}", md5::compute(old_prompt.content())));
        event_headers.insert("new-prompt-hash".to_string(), format!("{:x}", md5::compute(new_prompt.content())));
        
        nats_client.publish_with_headers(&event_subject, &event_payload, event_headers).await.unwrap();
        
        // 3. Query system prompt history
        let query = ClaudeApiQuery::GetSystemPromptHistory {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            limit: Some(10),
            include_metadata: true,
        };
        
        let query_subject = format!("cim.claude.queries.system_prompt_history.{}", conversation_id);
        let query_payload = serde_json::to_vec(&query).unwrap();
        nats_client.publish(&query_subject, &query_payload).await.unwrap();
        
        // Verify configuration flow
        let command_messages = nats_client.get_messages_for_subject("cim.claude.config.cmd.update_system_prompt");
        assert_eq!(command_messages.len(), 1);
        
        let event_messages = nats_client.get_messages_for_subject("cim.claude.config.evt.system_prompt_updated");
        assert_eq!(event_messages.len(), 1);
        
        let query_messages = nats_client.get_messages_for_subject("cim.claude.queries.system_prompt_history");
        assert_eq!(query_messages.len(), 1);
        
        // Verify event headers contain change tracking
        let event_msg = &event_messages[0];
        let event_headers = event_msg.headers.as_ref().unwrap();
        assert!(event_headers.contains_key("old-prompt-hash"));
        assert!(event_headers.contains_key("new-prompt-hash"));
        assert_ne!(
            event_headers.get("old-prompt-hash").unwrap(),
            event_headers.get("new-prompt-hash").unwrap()
        );
        
        assert_eq!(nats_client.published_messages.len(), 3);
    }
}

#[cfg(test)]
mod tool_management_flows {
    use super::*;

    /// Test NATS flow for Story 3.1: Register MCP Tool via NATS
    /// Subject Pattern: cim.core.event.cmd.register_tool.{tool_id} -> cim.core.event.evt.tool_registered.{tool_id}
    #[tokio::test]
    async fn test_nats_tool_registration_flow() {
        let mut nats_client = MockNatsClient::new();
        let tool_id = "file_operations";
        let conversation_id = ConversationId::new();
        
        // 1. Tool registers itself via NATS
        let tool_definition = ClaudeToolDefinition::new(
            "file_reader".to_string(),
            "Reads file contents".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "encoding": {"type": "string", "default": "utf-8"}
                },
                "required": ["path"]
            }),
        ).unwrap();
        
        let register_command = ClaudeApiCommand::AddTools {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            tools: vec![tool_definition.clone()],
            correlation_id: CorrelationId::new(),
        };
        
        let register_subject = format!("cim.core.event.cmd.register_tool.{}", tool_id);
        let register_payload = serde_json::to_vec(&register_command).unwrap();
        
        let mut register_headers = HashMap::new();
        register_headers.insert("tool-id".to_string(), tool_id.to_string());
        register_headers.insert("tool-name".to_string(), tool_definition.name.clone());
        register_headers.insert("tool-version".to_string(), "1.0.0".to_string());
        register_headers.insert("nats-subject".to_string(), format!("tools.{}.request", tool_id));
        
        nats_client.publish_with_headers(&register_subject, &register_payload, register_headers).await.unwrap();
        
        // 2. Tool registry publishes registration event
        let registration_event = ClaudeApiEvent::ToolsAdded {
            command_id: register_command.command_id().clone(),
            conversation_id: conversation_id.clone(),
            added_tools: vec![tool_definition.clone()],
            total_tool_count: 1,
            added_at: Utc::now(),
        };
        
        let event_subject = format!("cim.core.event.evt.tool_registered.{}", tool_id);
        let event_payload = serde_json::to_vec(&registration_event).unwrap();
        
        let mut event_headers = HashMap::new();
        event_headers.insert("tool-id".to_string(), tool_id.to_string());
        event_headers.insert("event-type".to_string(), "ToolsAdded".to_string());
        event_headers.insert("registry-status".to_string(), "active".to_string());
        
        nats_client.publish_with_headers(&event_subject, &event_payload, event_headers).await.unwrap();
        
        // 3. Subscribe to tool invocation subject (simulating tool readiness)
        let tool_invoke_subject = format!("tools.{}.request", tool_id);
        nats_client.subscribe(&tool_invoke_subject).await.unwrap();
        
        // 4. Query available tools
        let tools_query = ClaudeApiQuery::GetConversationTools {
            query_id: ClaudeQueryId::new(),
            conversation_id: conversation_id.clone(),
            include_schemas: true,
            include_usage_stats: false,
        };
        
        let query_subject = format!("cim.claude.queries.conversation_tools.{}", conversation_id);
        let query_payload = serde_json::to_vec(&tools_query).unwrap();
        nats_client.publish(&query_subject, &query_payload).await.unwrap();
        
        // Verify tool registration flow
        let register_messages = nats_client.get_messages_for_subject("cim.core.event.cmd.register_tool");
        assert_eq!(register_messages.len(), 1);
        
        let event_messages = nats_client.get_messages_for_subject("cim.core.event.evt.tool_registered");
        assert_eq!(event_messages.len(), 1);
        
        // Verify tool is subscribed to invocation subject
        assert!(nats_client.subscriptions.contains(&tool_invoke_subject));
        
        // Verify tool metadata in headers
        let register_msg = &register_messages[0];
        let register_msg_headers = register_msg.headers.as_ref().unwrap();
        assert_eq!(register_msg_headers.get("tool-name").unwrap(), &tool_definition.name);
        assert!(register_msg_headers.contains_key("nats-subject"));
        
        assert_eq!(nats_client.published_messages.len(), 3);
        assert_eq!(nats_client.subscriptions.len(), 1);
    }

    /// Test NATS flow for Story 3.2: Tool Invocation Flow
    /// Complete request-reply pattern for tool execution
    #[tokio::test]
    async fn test_nats_tool_invocation_flow() {
        let mut nats_client = MockNatsClient::new();
        let tool_id = "file_reader";
        let conversation_id = ConversationId::new();
        let tool_use_id = "tool_use_123";
        let command_id = ClaudeCommandId::new();
        
        // 1. Claude requests tool use (from API response parsing)
        let tool_request_event = ClaudeApiEvent::ToolUseRequested {
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.to_string(),
            tool_name: tool_id.to_string(),
            tool_input: serde_json::json!({
                "path": "/tmp/test.txt",
                "encoding": "utf-8"
            }),
            response_message_id: ClaudeMessageId::new("msg_123".to_string()),
            requested_at: Utc::now(),
        };
        
        let request_subject = format!("cim.claude.conv.evt.tool_use_requested.{}", conversation_id);
        let request_payload = serde_json::to_vec(&tool_request_event).unwrap();
        
        let mut request_headers = HashMap::new();
        request_headers.insert("tool-use-id".to_string(), tool_use_id.to_string());
        request_headers.insert("tool-name".to_string(), tool_id.to_string());
        request_headers.insert("correlation-id".to_string(), command_id.to_string());
        
        nats_client.publish_with_headers(&request_subject, &request_payload, request_headers).await.unwrap();
        
        // 2. Tool handler issues execution command
        let handle_command = ClaudeApiCommand::HandleToolUse {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.to_string(),
            tool_name: tool_id.to_string(),
            tool_input: serde_json::json!({
                "path": "/tmp/test.txt",
                "encoding": "utf-8"
            }),
            correlation_id: CorrelationId::new(),
            execution_timeout: Some(30),
        };
        
        let invoke_subject = format!("cim.core.event.cmd.invoke_tool.{}", tool_id);
        let invoke_payload = serde_json::to_vec(&handle_command).unwrap();
        nats_client.publish(&invoke_subject, &invoke_payload).await.unwrap();
        
        // 3. Tool execution started event
        let execution_started = ClaudeApiEvent::ToolExecutionStarted {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.to_string(),
            tool_name: tool_id.to_string(),
            tool_input: serde_json::json!({
                "path": "/tmp/test.txt",
                "encoding": "utf-8"
            }),
            execution_timeout_ms: Some(30000),
            started_at: Utc::now(),
        };
        
        let started_subject = format!("cim.core.event.evt.tool_execution_started.{}", tool_id);
        let started_payload = serde_json::to_vec(&execution_started).unwrap();
        nats_client.publish(&started_subject, &started_payload).await.unwrap();
        
        // 4. NATS request-reply to actual tool
        let tool_request_subject = format!("tools.{}.request", tool_id);
        let tool_request_payload = serde_json::json!({
            "tool_use_id": tool_use_id,
            "input": {
                "path": "/tmp/test.txt",
                "encoding": "utf-8"
            },
            "timeout_ms": 30000
        });
        
        nats_client.publish(&tool_request_subject, &serde_json::to_vec(&tool_request_payload).unwrap()).await.unwrap();
        
        // 5. Simulate tool response (would come via NATS reply)
        let tool_response_subject = format!("tools.{}.response.{}", tool_id, tool_use_id);
        let tool_response_payload = serde_json::json!({
            "tool_use_id": tool_use_id,
            "status": "success",
            "result": "File contents here...",
            "execution_time_ms": 150
        });
        
        nats_client.publish(&tool_response_subject, &serde_json::to_vec(&tool_response_payload).unwrap()).await.unwrap();
        
        // 6. Tool execution completed event
        let execution_completed = ClaudeApiEvent::ToolExecutionCompleted {
            command_id: command_id.clone(),
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.to_string(),
            tool_name: tool_id.to_string(),
            execution_result: ToolExecutionResult::success(
                "File contents here...".to_string(),
                150,
            ),
            completed_at: Utc::now(),
        };
        
        let completed_subject = format!("cim.core.event.evt.tool_execution_completed.{}", tool_id);
        let completed_payload = serde_json::to_vec(&execution_completed).unwrap();
        
        let mut completed_headers = HashMap::new();
        completed_headers.insert("tool-use-id".to_string(), tool_use_id.to_string());
        completed_headers.insert("execution-status".to_string(), "success".to_string());
        completed_headers.insert("execution-time-ms".to_string(), "150".to_string());
        
        nats_client.publish_with_headers(&completed_subject, &completed_payload, completed_headers).await.unwrap();
        
        // 7. Submit result back to Claude API
        let submit_command = ClaudeApiCommand::SubmitToolResult {
            command_id: ClaudeCommandId::new(),
            conversation_id: conversation_id.clone(),
            tool_use_id: tool_use_id.to_string(),
            result: ToolExecutionResult::success(
                "File contents here...".to_string(),
                150,
            ),
            correlation_id: CorrelationId::new(),
        };
        
        let submit_subject = format!("cim.claude.conv.cmd.submit_tool_result.{}", conversation_id);
        let submit_payload = serde_json::to_vec(&submit_command).unwrap();
        nats_client.publish(&submit_subject, &submit_payload).await.unwrap();
        
        // Verify complete tool invocation flow
        assert_eq!(nats_client.published_messages.len(), 7);
        
        // Verify message sequence
        let subjects: Vec<&str> = nats_client.published_messages
            .iter()
            .map(|msg| msg.subject.split('.').last().unwrap_or(&msg.subject))
            .collect();
        
        // Expected sequence pattern (last part of subjects)
        assert!(subjects.iter().any(|s| s.contains("tool_use_requested")));
        assert!(subjects.iter().any(|s| s.contains("invoke_tool")));
        assert!(subjects.iter().any(|s| s.contains("execution_started")));
        assert!(subjects.iter().any(|s| s.contains("request")));
        assert!(subjects.iter().any(|s| s.contains("response")));
        assert!(subjects.iter().any(|s| s.contains("execution_completed")));
        assert!(subjects.iter().any(|s| s.contains("submit_tool_result")));
        
        // Verify tool execution completed has proper metadata
        let completed_messages = nats_client.get_messages_for_subject("cim.core.event.evt.tool_execution_completed");
        let completed_msg = completed_messages.first().unwrap();
        let completed_headers = completed_msg.headers.as_ref().unwrap();
        assert_eq!(completed_headers.get("execution-status").unwrap(), "success");
        assert_eq!(completed_headers.get("execution-time-ms").unwrap(), "150");
    }
}

#[cfg(test)]
mod query_pattern_flows {
    use super::*;

    /// Test NATS flow for query patterns
    /// Subject Pattern: cim.claude.queries.* -> Response pattern
    #[tokio::test]
    async fn test_nats_query_patterns_flow() {
        let mut nats_client = MockNatsClient::new();
        let conversation_id = ConversationId::new();
        let query_id = ClaudeQueryId::new();
        
        // 1. Conversation details query
        let conversation_query = ClaudeApiQuery::GetConversation {
            query_id: query_id.clone(),
            conversation_id: conversation_id.clone(),
            include_messages: true,
            include_usage_stats: true,
            include_tool_definitions: true,
        };
        
        let conv_query_subject = format!("cim.claude.queries.conversation.{}", conversation_id);
        let conv_query_payload = serde_json::to_vec(&conversation_query).unwrap();
        
        let mut conv_headers = HashMap::new();
        conv_headers.insert("query-id".to_string(), query_id.to_string());
        conv_headers.insert("query-type".to_string(), "GetConversation".to_string());
        conv_headers.insert("include-messages".to_string(), "true".to_string());
        
        nats_client.publish_with_headers(&conv_query_subject, &conv_query_payload, conv_headers).await.unwrap();
        
        // 2. Usage statistics query
        let usage_query = ClaudeApiQuery::GetUsageStatistics {
            query_id: ClaudeQueryId::new(),
            conversation_id: Some(conversation_id.clone()),
            time_range: Some(TimeRange::last_days(7)),
            group_by: UsageGroupBy::Day,
            include_cost_breakdown: true,
        };
        
        let usage_query_subject = format!("cim.claude.queries.usage_statistics.{}", conversation_id);
        let usage_query_payload = serde_json::to_vec(&usage_query).unwrap();
        nats_client.publish(&usage_query_subject, &usage_query_payload).await.unwrap();
        
        // 3. Message search query (global)
        let search_query = ClaudeApiQuery::SearchMessages {
            query_id: ClaudeQueryId::new(),
            conversation_id: None, // Global search
            search_query: "rust programming".to_string(),
            search_options: MessageSearchOptions::default(),
            limit: Some(20),
            offset: Some(0),
        };
        
        let search_subject = "cim.claude.queries.search_messages.global";
        let search_payload = serde_json::to_vec(&search_query).unwrap();
        
        let mut search_headers = HashMap::new();
        search_headers.insert("search-query".to_string(), "rust programming".to_string());
        search_headers.insert("search-scope".to_string(), "global".to_string());
        search_headers.insert("limit".to_string(), "20".to_string());
        
        nats_client.publish_with_headers(search_subject, &search_payload, search_headers).await.unwrap();
        
        // 4. Performance metrics query (system-wide)
        let perf_query = ClaudeApiQuery::GetPerformanceMetrics {
            query_id: ClaudeQueryId::new(),
            conversation_id: None,
            metric_types: vec![
                PerformanceMetricType::ResponseTime,
                PerformanceMetricType::ErrorRate,
            ],
            time_range: Some(TimeRange::last_hours(24)),
            aggregation: MetricAggregation::P95,
        };
        
        let perf_subject = "cim.claude.queries.performance_metrics.system";
        let perf_payload = serde_json::to_vec(&perf_query).unwrap();
        nats_client.publish(&perf_subject, &perf_payload).await.unwrap();
        
        // 5. Simulate query responses (would be handled by query handlers)
        // Response for conversation query
        let conv_response_subject = format!("cim.claude.queries.conversation.{}.response", conversation_id);
        let conv_response = ConversationDetails {
            conversation_id: conversation_id.clone(),
            session_id: SessionId::new(),
            current_model: ClaudeModel::Claude35Sonnet20241022,
            system_prompt: Some(ClaudeSystemPrompt::new("Test prompt".to_string()).unwrap()),
            tool_definitions: vec![],
            message_count: 5,
            total_usage: ClaudeUsage::new(100, 150),
            estimated_cost_usd: 0.25,
            created_at: Utc::now() - chrono::Duration::hours(2),
            last_activity: Utc::now(),
            status: ConversationStatus::Active,
            messages: Some(vec![]),
            error_count: 0,
            metadata: HashMap::new(),
        };
        let conv_response_payload = serde_json::to_vec(&conv_response).unwrap();
        nats_client.publish(&conv_response_subject, &conv_response_payload).await.unwrap();
        
        // Verify query flow
        let conv_queries = nats_client.get_messages_for_subject("cim.claude.queries.conversation");
        assert_eq!(conv_queries.len(), 1);
        
        let usage_queries = nats_client.get_messages_for_subject("cim.claude.queries.usage_statistics");
        assert_eq!(usage_queries.len(), 1);
        
        let search_queries = nats_client.get_messages_for_subject("cim.claude.queries.search_messages");
        assert_eq!(search_queries.len(), 1);
        
        let perf_queries = nats_client.get_messages_for_subject("cim.claude.queries.performance_metrics");
        assert_eq!(perf_queries.len(), 1);
        
        let responses = nats_client.get_messages_for_subject("cim.claude.queries.conversation");
        assert_eq!(responses.len(), 1); // Includes response
        
        // Verify search query headers
        let search_msg = &search_queries[0];
        let search_msg_headers = search_msg.headers.as_ref().unwrap();
        assert_eq!(search_msg_headers.get("search-query").unwrap(), "rust programming");
        assert_eq!(search_msg_headers.get("search-scope").unwrap(), "global");
        
        // Total: 4 queries + 1 response = 5 messages
        assert_eq!(nats_client.published_messages.len(), 5);
    }
}

#[cfg(test)]
mod error_handling_flows {
    use super::*;

    /// Test NATS flow for error handling and retries
    /// Complete retry flow with exponential backoff simulation
    #[tokio::test]
    async fn test_nats_error_handling_retry_flow() {
        let mut nats_client = MockNatsClient::new();
        let conversation_id = ConversationId::new();
        let original_command_id = ClaudeCommandId::new();
        
        // 1. Original command that will fail
        let original_command = ClaudeApiCommand::SendMessage {
            command_id: original_command_id.clone(),
            conversation_id: conversation_id.clone(),
            session_id: SessionId::new(),
            request: ClaudeApiRequest::new(
                ClaudeModel::Claude35Sonnet20241022,
                vec![ClaudeMessage::user(MessageContent::text("Test message"))],
                MaxTokens::new(1000).unwrap(),
            ),
            correlation_id: CorrelationId::new(),
            timeout_seconds: Some(30),
            retry_config: Some(RetryConfiguration::default()),
            request_metadata: RequestMetadata::new(SessionId::new(), RequestSource::UserInterface),
        };
        
        let original_subject = format!("cim.claude.conv.cmd.send.{}", conversation_id);
        let original_payload = serde_json::to_vec(&original_command).unwrap();
        nats_client.publish(&original_subject, &original_payload).await.unwrap();
        
        // 2. API error occurs
        let api_error = ClaudeApiError::new(
            ClaudeErrorType::RateLimitError,
            "Rate limit exceeded".to_string(),
            429,
        ).with_retry_after(60);
        
        let error_event = ClaudeApiEvent::ApiErrorOccurred {
            command_id: original_command_id.clone(),
            conversation_id: conversation_id.clone(),
            request: original_command.clone().into(),
            error: api_error.clone(),
            request_duration_ms: 500,
            error_occurred_at: Utc::now(),
            retry_attempt: None,
        };
        
        let error_subject = format!("cim.claude.conv.evt.api_error.{}", conversation_id);
        let error_payload = serde_json::to_vec(&error_event).unwrap();
        
        let mut error_headers = HashMap::new();
        error_headers.insert("error-type".to_string(), "RateLimitError".to_string());
        error_headers.insert("http-status".to_string(), "429".to_string());
        error_headers.insert("retry-after".to_string(), "60".to_string());
        error_headers.insert("is-retryable".to_string(), "true".to_string());
        
        nats_client.publish_with_headers(&error_subject, &error_payload, error_headers).await.unwrap();
        
        // 3. Retry attempts (simulate exponential backoff)
        for attempt in 1..=3 {
            let retry_command_id = ClaudeCommandId::new();
            
            // Retry command
            let retry_command = ClaudeApiCommand::RetryRequest {
                command_id: retry_command_id.clone(),
                conversation_id: conversation_id.clone(),
                original_command_id: original_command_id.clone(),
                retry_attempt: attempt,
                modified_request: None,
                correlation_id: CorrelationId::new(),
            };
            
            let retry_subject = format!("cim.claude.conv.cmd.retry.{}", conversation_id);
            let retry_payload = serde_json::to_vec(&retry_command).unwrap();
            
            let mut retry_headers = HashMap::new();
            retry_headers.insert("retry-attempt".to_string(), attempt.to_string());
            retry_headers.insert("original-command-id".to_string(), original_command_id.to_string());
            retry_headers.insert("retry-delay-ms".to_string(), (1000 * 2_u32.pow(attempt - 1)).to_string());
            
            nats_client.publish_with_headers(&retry_subject, &retry_payload, retry_headers).await.unwrap();
            
            // Retry initiated event
            let retry_event = ClaudeApiEvent::RequestRetryInitiated {
                command_id: retry_command_id.clone(),
                original_command_id: original_command_id.clone(),
                conversation_id: conversation_id.clone(),
                retry_attempt: attempt,
                retry_delay_ms: 1000 * 2_u32.pow(attempt - 1),
                retry_reason: RetryReason::RateLimit,
                initiated_at: Utc::now(),
            };
            
            let retry_event_subject = format!("cim.claude.conv.evt.retry_initiated.{}", conversation_id);
            let retry_event_payload = serde_json::to_vec(&retry_event).unwrap();
            nats_client.publish(&retry_event_subject, &retry_event_payload).await.unwrap();
            
            // Simulate continued failure for first 2 attempts
            if attempt < 3 {
                let retry_error_event = ClaudeApiEvent::ApiErrorOccurred {
                    command_id: retry_command_id.clone(),
                    conversation_id: conversation_id.clone(),
                    request: original_command.clone().into(),
                    error: api_error.clone(),
                    request_duration_ms: 300,
                    error_occurred_at: Utc::now(),
                    retry_attempt: Some(attempt),
                };
                
                let retry_error_subject = format!("cim.claude.conv.evt.api_error.{}", conversation_id);
                let retry_error_payload = serde_json::to_vec(&retry_error_event).unwrap();
                nats_client.publish(&retry_error_subject, &retry_error_payload).await.unwrap();
            } else {
                // Third attempt succeeds
                let success_response = ClaudeApiResponse::new(
                    ClaudeMessageId::new("msg_retry_success".to_string()),
                    ClaudeModel::Claude35Sonnet20241022,
                    vec![ContentBlock::Text { text: "Success after retries".to_string() }],
                    StopReason::EndTurn,
                    ClaudeUsage::new(10, 20),
                );
                
                let success_event = ClaudeApiEvent::MessageResponseReceived {
                    command_id: retry_command_id.clone(),
                    conversation_id: conversation_id.clone(),
                    request: original_command.clone().into(),
                    response: success_response,
                    request_duration_ms: 1200,
                    request_id: Some("req_retry_success".to_string()),
                    received_at: Utc::now(),
                };
                
                let success_subject = format!("cim.claude.conv.evt.response_received.{}", conversation_id);
                let success_payload = serde_json::to_vec(&success_event).unwrap();
                
                let mut success_headers = HashMap::new();
                success_headers.insert("retry-success".to_string(), "true".to_string());
                success_headers.insert("total-attempts".to_string(), "3".to_string());
                success_headers.insert("original-command-id".to_string(), original_command_id.to_string());
                
                nats_client.publish_with_headers(&success_subject, &success_payload, success_headers).await.unwrap();
            }
        }
        
        // Verify retry flow
        let original_commands = nats_client.get_messages_for_subject("cim.claude.conv.cmd.send");
        assert_eq!(original_commands.len(), 1);
        
        let error_events = nats_client.get_messages_for_subject("cim.claude.conv.evt.api_error");
        assert_eq!(error_events.len(), 3); // Original + 2 retry failures
        
        let retry_commands = nats_client.get_messages_for_subject("cim.claude.conv.cmd.retry");
        assert_eq!(retry_commands.len(), 3);
        
        let retry_events = nats_client.get_messages_for_subject("cim.claude.conv.evt.retry_initiated");
        assert_eq!(retry_events.len(), 3);
        
        let success_events = nats_client.get_messages_for_subject("cim.claude.conv.evt.response_received");
        assert_eq!(success_events.len(), 1);
        
        // Verify exponential backoff in retry headers
        for (i, msg) in retry_commands.iter().enumerate() {
            let headers = msg.headers.as_ref().unwrap();
            let expected_delay = 1000 * 2_u32.pow(i as u32);
            assert_eq!(headers.get("retry-delay-ms").unwrap(), &expected_delay.to_string());
        }
        
        // Verify final success includes retry metadata
        let success_msg = &success_events[0];
        let success_headers = success_msg.headers.as_ref().unwrap();
        assert_eq!(success_headers.get("retry-success").unwrap(), "true");
        assert_eq!(success_headers.get("total-attempts").unwrap(), "3");
        
        // Total: 1 original + 1 error + 3 retries + 3 retry events + 2 retry errors + 1 success = 11 messages
        assert_eq!(nats_client.published_messages.len(), 11);
    }
}

#[cfg(test)]
mod nats_subject_validation_tests {
    use super::*;

    /// Validate all NATS subject patterns follow the documented conventions
    #[tokio::test]
    async fn test_nats_subject_pattern_validation() {
        let conversation_id = ConversationId::new();
        let config_id = "main";
        let tool_id = "test_tool";
        
        // Claude API Commands
        let claude_command_subjects = vec![
            format!("cim.claude.conv.cmd.start.{}", conversation_id),
            format!("cim.claude.conv.cmd.send.{}", conversation_id),
            format!("cim.claude.conv.cmd.stream.{}", conversation_id),
            format!("cim.claude.conv.cmd.end.{}", conversation_id),
        ];
        
        // Claude API Events
        let claude_event_subjects = vec![
            format!("cim.claude.conv.evt.prompt_sent.{}", conversation_id),
            format!("cim.claude.conv.evt.response_received.{}", conversation_id),
            format!("cim.claude.conv.evt.rate_limited.{}", conversation_id),
            format!("cim.claude.conv.evt.api_error.{}", conversation_id),
        ];
        
        // Configuration Commands
        let config_command_subjects = vec![
            format!("cim.claude.config.cmd.update_system_prompt.{}", config_id),
            format!("cim.claude.config.cmd.update_model_params.{}", config_id),
            format!("cim.claude.config.cmd.update_conversation_settings.{}", config_id),
        ];
        
        // Configuration Events
        let config_event_subjects = vec![
            format!("cim.claude.config.evt.system_prompt_updated.{}", config_id),
            format!("cim.claude.config.evt.model_params_updated.{}", config_id),
            format!("cim.claude.config.evt.config_reset.{}", config_id),
        ];
        
        // Tool Commands
        let tool_command_subjects = vec![
            format!("cim.core.event.cmd.register_tool.{}", tool_id),
            format!("cim.core.event.cmd.invoke_tool.{}", tool_id),
            format!("cim.core.event.cmd.health_check_tool.{}", tool_id),
        ];
        
        // Tool Events
        let tool_event_subjects = vec![
            format!("cim.core.event.evt.tool_registered.{}", tool_id),
            format!("cim.core.event.evt.tool_invocation_started.{}", tool_id),
            format!("cim.core.event.evt.tool_invocation_completed.{}", tool_id),
        ];
        
        // User Control Commands
        let user_command_subjects = vec![
            format!("cim.user.conv.cmd.pause.{}", conversation_id),
            format!("cim.user.conv.cmd.archive.{}", conversation_id),
            format!("cim.user.conv.cmd.fork.{}", conversation_id),
        ];
        
        // User Control Events
        let user_event_subjects = vec![
            format!("cim.user.conv.evt.paused.{}", conversation_id),
            format!("cim.user.conv.evt.archived.{}", conversation_id),
        ];
        
        // Query Subjects
        let query_subjects = vec![
            format!("cim.claude.queries.conversation.{}", conversation_id),
            format!("cim.claude.queries.usage_statistics.{}", conversation_id),
            format!("cim.claude.queries.conversation_history.{}", conversation_id),
            "cim.claude.queries.search_messages.global".to_string(),
            "cim.claude.queries.performance_metrics.system".to_string(),
        ];
        
        // Validate subject patterns
        let all_subjects = [
            claude_command_subjects,
            claude_event_subjects,
            config_command_subjects,
            config_event_subjects,
            tool_command_subjects,
            tool_event_subjects,
            user_command_subjects,
            user_event_subjects,
            query_subjects,
        ].concat();
        
        for subject in &all_subjects {
            // All subjects should start with "cim."
            assert!(subject.starts_with("cim."), "Subject should start with 'cim.': {}", subject);
            
            // Should have proper hierarchical structure
            let parts: Vec<&str> = subject.split('.').collect();
            assert!(parts.len() >= 3, "Subject should have at least 3 parts: {}", subject);
            assert_eq!(parts[0], "cim", "First part should be 'cim': {}", subject);
            
            // Validate domain separation
            match parts[1] {
                "claude" => {
                    assert!(parts[2] == "conv" || parts[2] == "config" || parts[2] == "queries", 
                           "Claude domain should have 'conv', 'config', or 'queries': {}", subject);
                }
                "core" => {
                    assert_eq!(parts[2], "event", "Core domain should have 'event': {}", subject);
                }
                "user" => {
                    assert_eq!(parts[2], "conv", "User domain should have 'conv': {}", subject);
                }
                _ => panic!("Unknown domain: {}", parts[1]),
            }
            
            // Validate command/event/query patterns
            if parts.len() >= 4 {
                match parts[3] {
                    "cmd" => {
                        assert!(parts.len() >= 5, "Commands should have action: {}", subject);
                        assert!(parts.len() >= 6, "Commands should have entity ID: {}", subject);
                    }
                    "evt" => {
                        assert!(parts.len() >= 5, "Events should have event type: {}", subject);
                        assert!(parts.len() >= 6, "Events should have entity ID: {}", subject);
                    }
                    "queries" => {
                        assert!(parts.len() >= 5, "Queries should have query type: {}", subject);
                    }
                    _ => {} // Other patterns like tool subjects
                }
            }
        }
        
        // Test subject uniqueness (no duplicates)
        let mut unique_subjects = std::collections::HashSet::new();
        for subject in &all_subjects {
            assert!(unique_subjects.insert(subject), "Duplicate subject found: {}", subject);
        }
        
        // Should have comprehensive coverage
        assert!(all_subjects.len() >= 25, "Should have comprehensive subject coverage");
        
        println!("Validated {} unique NATS subjects", all_subjects.len());
    }
}

/// Helper functions for integration testing
#[cfg(test)]
mod test_helpers {
    use super::*;
    
    /// Helper to simulate complete user story flow through NATS
    pub async fn simulate_complete_user_story_flow(
        nats_client: &mut MockNatsClient,
        story_name: &str,
    ) -> Result<usize, String> {
        let conversation_id = ConversationId::new();
        let mut message_count = 0;
        
        match story_name {
            "send_message" => {
                // Command -> Event -> Query flow
                let command = ClaudeApiCommand::SendMessage {
                    command_id: ClaudeCommandId::new(),
                    conversation_id: conversation_id.clone(),
                    session_id: SessionId::new(),
                    request: ClaudeApiRequest::new(
                        ClaudeModel::Claude35Sonnet20241022,
                        vec![ClaudeMessage::user(MessageContent::text("Test"))],
                        MaxTokens::new(100).unwrap(),
                    ),
                    correlation_id: CorrelationId::new(),
                    timeout_seconds: Some(30),
                    retry_config: None,
                    request_metadata: RequestMetadata::new(SessionId::new(), RequestSource::UserInterface),
                };
                
                let command_subject = format!("cim.claude.conv.cmd.send.{}", conversation_id);
                nats_client.publish(&command_subject, &serde_json::to_vec(&command).unwrap()).await?;
                message_count += 1;
                
                // Simulate event
                let event = ClaudeApiEvent::MessageResponseReceived {
                    command_id: command.command_id().clone(),
                    conversation_id: conversation_id.clone(),
                    request: command.clone().into(),
                    response: ClaudeApiResponse::new(
                        ClaudeMessageId::new("msg_test".to_string()),
                        ClaudeModel::Claude35Sonnet20241022,
                        vec![ContentBlock::Text { text: "Response".to_string() }],
                        StopReason::EndTurn,
                        ClaudeUsage::new(5, 10),
                    ),
                    request_duration_ms: 1000,
                    request_id: Some("req_test".to_string()),
                    received_at: Utc::now(),
                };
                
                let event_subject = format!("cim.claude.conv.evt.response_received.{}", conversation_id);
                nats_client.publish(&event_subject, &serde_json::to_vec(&event).unwrap()).await?;
                message_count += 1;
                
                // Simulate query
                let query = ClaudeApiQuery::GetConversation {
                    query_id: ClaudeQueryId::new(),
                    conversation_id: conversation_id.clone(),
                    include_messages: true,
                    include_usage_stats: true,
                    include_tool_definitions: false,
                };
                
                let query_subject = format!("cim.claude.queries.conversation.{}", conversation_id);
                nats_client.publish(&query_subject, &serde_json::to_vec(&query).unwrap()).await?;
                message_count += 1;
            }
            
            _ => return Err(format!("Unknown user story: {}", story_name)),
        }
        
        Ok(message_count)
    }
    
    /// Validate that a NATS message follows expected patterns
    pub fn validate_nats_message(message: &NatsMessage, expected_pattern: &str) -> bool {
        // Check subject pattern
        if !message.subject.contains(expected_pattern) {
            return false;
        }
        
        // Check payload is valid JSON
        if serde_json::from_slice::<serde_json::Value>(&message.payload).is_err() {
            return false;
        }
        
        // Check timestamp is recent
        let now = Utc::now();
        let age = now - message.timestamp;
        if age.num_seconds() > 60 {
            return false;
        }
        
        true
    }
}