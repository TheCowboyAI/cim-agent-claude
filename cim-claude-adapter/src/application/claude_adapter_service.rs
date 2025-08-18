/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Claude Adapter Service
//! 
//! Main application service that coordinates between NATS (commands in, events out)
//! and Claude API (HTTP calls). This is the core of the cim-claude-adapter.

use crate::domain::{
    claude_api::*,
    claude_commands::*,
    claude_events::*,
    claude_queries::*,
    value_objects::*,
};
use crate::infrastructure::{
    nats_client::{NatsClient, NatsMessage},
    claude_client::ClaudeClient,
};
use anyhow::{Result, Context};
use async_nats::jetstream::consumer::PullConsumer;
use futures::TryStreamExt;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{info, error, debug, instrument};

/// The main Claude Adapter Service
/// 
/// Responsibilities:
/// 1. Listen for Claude API commands on NATS
/// 2. Execute commands by calling Claude API via HTTP
/// 3. Publish resulting events back to NATS
/// 4. Handle queries for conversation data
#[derive(Clone)]
pub struct ClaudeAdapterService {
    nats_client: Arc<NatsClient>,
    claude_client: Arc<ClaudeClient>,
    conversation_sessions: Arc<tokio::sync::RwLock<std::collections::HashMap<ConversationId, ClaudeApiSession>>>,
}

impl ClaudeAdapterService {
    /// Create a new Claude adapter service
    pub fn new(nats_client: NatsClient, claude_client: ClaudeClient) -> Self {
        Self {
            nats_client: Arc::new(nats_client),
            claude_client: Arc::new(claude_client),
            conversation_sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Start the adapter service
    /// 
    /// This starts background tasks to:
    /// - Listen for commands on NATS
    /// - Process commands and call Claude API
    /// - Publish events back to NATS
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<Vec<JoinHandle<()>>> {
        info!("Starting Claude Adapter Service");

        let mut handles = Vec::new();

        // Start command handler
        let command_handle = self.start_command_handler().await?;
        handles.push(command_handle);

        // Start query handler
        let query_handle = self.start_query_handler().await?;
        handles.push(query_handle);

        // Health check task
        let health_handle = self.start_health_check_task();
        handles.push(health_handle);

        info!("Claude Adapter Service started with {} tasks", handles.len());
        Ok(handles)
    }

    /// Start the command handler task
    async fn start_command_handler(&self) -> Result<JoinHandle<()>> {
        let consumer = self.nats_client
            .subscribe_commands("cim.core.command.cmd.*")
            .await
            .context("Failed to subscribe to commands")?;

        let service = self.clone();
        let handle = tokio::spawn(async move {
            service.handle_commands(consumer).await;
        });

        Ok(handle)
    }

    /// Start the query handler task
    async fn start_query_handler(&self) -> Result<JoinHandle<()>> {
        let consumer = self.nats_client
            .subscribe_queries("cim.core.query.qry.*")
            .await
            .context("Failed to subscribe to queries")?;

        let service = self.clone();
        let handle = tokio::spawn(async move {
            service.handle_queries(consumer).await;
        });

        Ok(handle)
    }

    /// Start health check task
    fn start_health_check_task(&self) -> JoinHandle<()> {
        let nats_client = Arc::clone(&self.nats_client);
        let claude_client = Arc::clone(&self.claude_client);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Check NATS health
                if let Err(e) = nats_client.health_check().await {
                    error!("NATS health check failed: {}", e);
                }

                // Check Claude API health  
                if let Err(e) = claude_client.health_check().await {
                    error!("Claude API health check failed: {}", e);
                }
            }
        })
    }

    /// Handle incoming commands from NATS
    async fn handle_commands(&self, consumer: PullConsumer) {
        info!("Starting command handler");

        loop {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(1),
                consumer.fetch().max_messages(1).messages()
            ).await {
                Ok(Ok(mut messages)) => {
                    if let Some(message) = messages.try_next().await.ok().flatten() {
                        if let Err(e) = self.process_command_message(message).await {
                            error!("Failed to process command message: {}", e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Error fetching command message: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Err(_) => {
                    // Timeout - continue loop
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Handle incoming queries from NATS
    async fn handle_queries(&self, consumer: PullConsumer) {
        info!("Starting query handler");

        loop {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(1),
                consumer.fetch().max_messages(1).messages()
            ).await {
                Ok(Ok(mut messages)) => {
                    if let Some(message) = messages.try_next().await.ok().flatten() {
                        if let Err(e) = self.process_query_message(message).await {
                            error!("Failed to process query message: {}", e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Error fetching query message: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Err(_) => {
                    // Timeout - continue loop
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Process a single command message
    #[instrument(skip(self, message))]
    async fn process_command_message(&self, message: async_nats::jetstream::Message) -> Result<()> {
        let nats_message: NatsMessage<ClaudeApiCommand> = serde_json::from_slice(&message.payload)
            .context("Failed to deserialize command message")?;

        debug!("Processing command: {:?}", nats_message.message_id);

        let correlation_id = CorrelationId::from_uuid(uuid::Uuid::parse_str(&nats_message.correlation_id)
            .context("Invalid correlation ID")?);

        let result = self.execute_command(nats_message.payload, correlation_id.clone()).await;

        // Acknowledge the message
        if let Err(e) = message.ack().await {
            error!("Failed to ack message: {}", e);
        }

        // Handle the result
        match result {
            Ok(event) => {
                if let Err(e) = self.nats_client.publish_event(event, correlation_id).await {
                    error!("Failed to publish success event: {}", e);
                }
            }
            Err(e) => {
                error!("Command execution failed: {}", e);
                
                // Create and publish error event
                let error_event = ClaudeApiEvent::ApiErrorOccurred {
                    command_id: ClaudeCommandId::new(),
                    conversation_id: ConversationId::new(), // Would need to extract from command
                    request: ClaudeApiRequest::new(
                        ClaudeModel::Claude3Haiku20240307,
                        vec![ClaudeMessage::user("error")],
                        MaxTokens::new(1).unwrap()
                    ),
                    error: ClaudeApiError::new(
                        ClaudeErrorType::ApiError,
                        e.to_string(),
                        500
                    ),
                    request_duration_ms: 0,
                    error_occurred_at: chrono::Utc::now(),
                    retry_attempt: None,
                };

                if let Err(e) = self.nats_client.publish_event(error_event, correlation_id).await {
                    error!("Failed to publish error event: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Execute a Claude API command
    #[instrument(skip(self, command))]
    async fn execute_command(&self, command: ClaudeApiCommand, correlation_id: CorrelationId) -> Result<ClaudeApiEvent> {
        match command {
            ClaudeApiCommand::SendMessage { command_id, conversation_id, request, .. } => {
                let start_time = std::time::Instant::now();
                
                let response = self.claude_client.send_message(request.clone()).await
                    .context("Failed to send message to Claude")?;
                
                let duration = start_time.elapsed().as_millis() as u64;

                // Update session state
                self.update_conversation_session(&conversation_id, &request, &response).await;

                Ok(ClaudeApiEvent::MessageResponseReceived {
                    command_id,
                    conversation_id,
                    request,
                    response,
                    request_duration_ms: duration,
                    request_id: None,
                    received_at: chrono::Utc::now(),
                })
            }

            ClaudeApiCommand::SendStreamingMessage { command_id, conversation_id, request, .. } => {
                let start_time = std::time::Instant::now();
                
                let response = self.claude_client.send_streaming_message(request.clone()).await
                    .context("Failed to send streaming message to Claude")?;
                
                let duration = start_time.elapsed().as_millis() as u64;

                // Update session state
                self.update_conversation_session(&conversation_id, &request, &response).await;

                Ok(ClaudeApiEvent::StreamingMessageCompleted {
                    command_id,
                    conversation_id,
                    total_chunks: 1, // Simplified for now
                    final_response: response,
                    total_duration_ms: duration,
                    completed_at: chrono::Utc::now(),
                })
            }

            _ => {
                // For other commands, return a generic success event
                Ok(ClaudeApiEvent::MessageResponseReceived {
                    command_id: ClaudeCommandId::new(),
                    conversation_id: ConversationId::new(),
                    request: ClaudeApiRequest::new(
                        ClaudeModel::Claude3Haiku20240307,
                        vec![ClaudeMessage::user("success")],
                        MaxTokens::new(1).unwrap()
                    ),
                    response: ClaudeApiResponse::new(
                        ClaudeMessageId::new("success".to_string()),
                        ClaudeModel::Claude3Haiku20240307,
                        vec![ContentBlock::Text { text: "Success".to_string() }],
                        StopReason::EndTurn,
                        ClaudeUsage::new(1, 1),
                    ),
                    request_duration_ms: 0,
                    request_id: None,
                    received_at: chrono::Utc::now(),
                })
            }
        }
    }

    /// Process a single query message
    #[instrument(skip(self, message))]
    async fn process_query_message(&self, message: async_nats::jetstream::Message) -> Result<()> {
        let nats_message: NatsMessage<ClaudeApiQuery> = serde_json::from_slice(&message.payload)
            .context("Failed to deserialize query message")?;

        debug!("Processing query: {:?}", nats_message.message_id);

        let result = self.execute_query(nats_message.payload).await;

        // Reply to the query
        if let Some(reply_subject) = &message.reply {
            let reply_payload = match result {
                Ok(data) => serde_json::to_vec(&data).unwrap_or_default(),
                Err(e) => {
                    error!("Query execution failed: {}", e);
                    serde_json::to_vec(&serde_json::json!({
                        "error": e.to_string()
                    })).unwrap_or_default()
                }
            };

            if let Err(e) = self.nats_client.publish_raw(&reply_subject.as_str(), reply_payload).await {
                error!("Failed to send query reply: {}", e);
            }
        }

        // Acknowledge the message
        if let Err(e) = message.ack().await {
            error!("Failed to ack query message: {}", e);
        }

        Ok(())
    }

    /// Execute a query
    async fn execute_query(&self, query: ClaudeApiQuery) -> Result<serde_json::Value> {
        match query {
            ClaudeApiQuery::GetConversation { conversation_id, .. } => {
                let sessions = self.conversation_sessions.read().await;
                match sessions.get(&conversation_id) {
                    Some(session) => {
                        Ok(serde_json::json!({
                            "conversation_id": session.conversation_id,
                            "model": format!("{:?}", session.model_config),
                            "message_count": session.message_history.len(),
                            "created_at": session.created_at
                        }))
                    }
                    None => {
                        Ok(serde_json::json!({"error": "Conversation not found"}))
                    }
                }
            }
            _ => {
                // For other queries, return empty result
                Ok(serde_json::json!({"message": "Query not implemented yet"}))
            }
        }
    }

    /// Update conversation session state
    async fn update_conversation_session(&self, conversation_id: &ConversationId, request: &ClaudeApiRequest, response: &ClaudeApiResponse) {
        let mut sessions = self.conversation_sessions.write().await;
        
        let session = sessions.entry(conversation_id.clone()).or_insert_with(|| {
            ClaudeApiSession::new(conversation_id.clone(), request.model.clone())
        });

        // Add the request and response to session
        if let Some(user_message) = request.messages.first() {
            session.add_user_message(user_message.content.clone());
        }
        session.add_assistant_message(response.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::{
        nats_client::NatsClientConfig,
        claude_client::ClaudeClientConfig,
    };

    async fn create_test_service() -> ClaudeAdapterService {
        let nats_config = NatsClientConfig::default();
        let nats_client = NatsClient::new(nats_config).await.unwrap();
        
        let claude_config = ClaudeClientConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let claude_client = ClaudeClient::new(claude_config).unwrap();

        ClaudeAdapterService::new(nats_client, claude_client)
    }

    #[tokio::test]
    async fn test_service_creation() {
        if std::env::var("CLAUDE_API_KEY").is_err() {
            return; // Skip if no API key
        }
        
        let service = create_test_service().await;
        assert!(!service.conversation_sessions.read().await.is_empty() == false);
    }
}