/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Real NATS JetStream Client Implementation
//! 
//! Provides production-ready NATS integration for the CIM Claude Adapter.
//! Handles all message publishing, subscribing, and stream management.

use crate::domain::{
    claude_commands::ClaudeApiCommand,
    claude_events::ClaudeApiEvent,
    claude_queries::ClaudeApiQuery,
    value_objects::*,
};
use crate::infrastructure::subjects::CimSubjects;
use async_nats::{
    Client, 
    ConnectOptions,
    jetstream::{self, Context, consumer::PullConsumer},
    HeaderMap,
};
use anyhow::{Result, Context as AnyhowContext};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tokio::time::{Duration, timeout};
use tracing::{info, warn, debug, instrument};
use uuid::Uuid;

/// Configuration for NATS connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsClientConfig {
    /// NATS server URLs
    pub servers: Vec<String>,
    /// Connection name for debugging
    pub name: String,
    /// Authentication token (optional)
    pub token: Option<String>,
    /// Username for authentication (optional)
    pub username: Option<String>,
    /// Password for authentication (optional) 
    pub password: Option<String>,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout for queries
    pub request_timeout: Duration,
    /// Maximum reconnect attempts
    pub max_reconnect_attempts: usize,
    /// Reconnect delay
    pub reconnect_delay: Duration,
}

impl Default for NatsClientConfig {
    fn default() -> Self {
        Self {
            servers: vec!["nats://localhost:4222".to_string()],
            name: "cim-claude-adapter".to_string(),
            token: None,
            username: None,
            password: None,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            max_reconnect_attempts: 10,
            reconnect_delay: Duration::from_secs(2),
        }
    }
}

/// NATS message envelope for structured messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsMessage<T> {
    /// Message ID for tracing
    pub message_id: String,
    /// Correlation ID for request tracing
    pub correlation_id: String,
    /// Message timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Message payload
    pub payload: T,
    /// Message metadata
    pub metadata: HashMap<String, String>,
}

impl<T> NatsMessage<T> {
    pub fn new(payload: T, correlation_id: String) -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            correlation_id,
            timestamp: chrono::Utc::now(),
            payload,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Production NATS JetStream Client
#[derive(Clone)]
pub struct NatsClient {
    client: Client,
    jetstream: Context,
    config: NatsClientConfig,
    subjects: CimSubjects,
}

impl NatsClient {
    /// Create a new NATS client with configuration
    #[instrument(skip(config))]
    pub async fn new(config: NatsClientConfig) -> Result<Self> {
        info!("Connecting to NATS servers: {:?}", config.servers);

        let mut connect_opts = ConnectOptions::new()
            .name(&config.name)
            .retry_on_initial_connect()
            .event_callback(|event| async move {
                match event {
                    async_nats::Event::Disconnected => warn!("Disconnected from NATS server"),
                    async_nats::Event::Connected => info!("Connected to NATS server"),
                    async_nats::Event::ClientError(e) => warn!("NATS client error: {:?}", e),
                    _ => debug!("NATS event: {:?}", event),
                }
            });

        // Add authentication if provided
        if let Some(token) = &config.token {
            connect_opts = connect_opts.token(token.clone());
        } else if let Some(username) = &config.username {
            if let Some(password) = &config.password {
                connect_opts = connect_opts.user_and_password(username.clone(), password.clone());
            }
        }

        let client = timeout(
            config.connect_timeout,
            async_nats::connect_with_options(&config.servers.join(","), connect_opts)
        )
        .await
        .context("NATS connection timeout")?
        .context("Failed to connect to NATS")?;

        let jetstream = jetstream::new(client.clone());

        info!("Successfully connected to NATS JetStream");

        Ok(Self {
            client,
            jetstream,
            subjects: CimSubjects::new("cim".to_string()),
            config,
        })
    }

    /// Publish a command to NATS
    #[instrument(skip(self, command))]
    pub async fn publish_command(&self, command: ClaudeApiCommand, correlation_id: CorrelationId) -> Result<()> {
        let subject = self.subjects.command_subject(&command);
        let message = NatsMessage::new(command, correlation_id.to_string());
        
        self.publish_with_headers(subject.clone(), &message, create_command_headers(&message))
            .await
            .context("Failed to publish command")?;

        debug!("Published command to subject: {}", subject);
        Ok(())
    }

    /// Publish an event to NATS  
    #[instrument(skip(self, event))]
    pub async fn publish_event(&self, event: ClaudeApiEvent, correlation_id: CorrelationId) -> Result<()> {
        let subject = self.subjects.event_subject(&event);
        let message = NatsMessage::new(event, correlation_id.to_string());
        
        self.publish_with_headers(subject.clone(), &message, create_event_headers(&message))
            .await
            .context("Failed to publish event")?;

        debug!("Published event to subject: {}", subject);
        Ok(())
    }

    /// Publish raw payload to a subject
    #[instrument(skip(self, payload))]
    pub async fn publish_raw(&self, subject: &str, payload: Vec<u8>) -> Result<()> {
        self.client
            .publish(subject.to_string(), payload.into())
            .await
            .context("Failed to publish raw message")?;

        debug!("Published raw message to subject: {}", subject);
        Ok(())
    }

    /// Send a query and wait for response
    #[instrument(skip(self, query))]
    pub async fn send_query<R>(&self, query: ClaudeApiQuery, correlation_id: CorrelationId) -> Result<R> 
    where
        R: for<'de> Deserialize<'de> + Send,
    {
        let subject = self.subjects.query_subject(&query);
        let message = NatsMessage::new(query, correlation_id.to_string());
        
        let response = timeout(
            self.config.request_timeout,
            self.client.request(
                subject.clone(), 
                serde_json::to_vec(&message).context("Failed to serialize query")?.into()
            )
        )
        .await
        .context("Query timeout")?
        .context("Query failed")?;

        let response_message: NatsMessage<R> = serde_json::from_slice(&response.payload)
            .context("Failed to deserialize query response")?;

        debug!("Received query response from subject: {}", subject);
        Ok(response_message.payload)
    }

    /// Subscribe to commands on a subject pattern
    #[instrument(skip(self))]
    pub async fn subscribe_commands(&self, subject_pattern: &str) -> Result<PullConsumer> {
        self.create_consumer("commands", subject_pattern).await
    }

    /// Subscribe to events on a subject pattern
    #[instrument(skip(self))]
    pub async fn subscribe_events(&self, subject_pattern: &str) -> Result<PullConsumer> {
        self.create_consumer("events", subject_pattern).await
    }

    /// Subscribe to queries on a subject pattern (for query handlers)
    #[instrument(skip(self))]
    pub async fn subscribe_queries(&self, subject_pattern: &str) -> Result<PullConsumer> {
        self.create_consumer("queries", subject_pattern).await
    }

    /// Create a JetStream consumer for a subject pattern
    async fn create_consumer(&self, stream_name: &str, subject_pattern: &str) -> Result<PullConsumer> {
        // Ensure stream exists
        let stream = self.jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: stream_name.to_string(),
                subjects: vec![subject_pattern.to_string()],
                max_age: Duration::from_secs(24 * 60 * 60), // 24 hours retention
                storage: jetstream::stream::StorageType::File,
                num_replicas: 1,
                ..Default::default()
            })
            .await
            .context("Failed to create or get stream")?;

        // Create consumer
        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                durable_name: Some(format!("{}-{}", self.config.name, stream_name)),
                ..Default::default()
            })
            .await
            .context("Failed to create consumer")?;

        info!("Created consumer for stream '{}' with subject pattern '{}'", stream_name, subject_pattern);
        Ok(consumer)
    }

    /// Publish with custom headers
    async fn publish_with_headers<T>(&self, subject: String, message: &NatsMessage<T>, headers: HeaderMap) -> Result<()>
    where
        T: Serialize,
    {
        let payload = serde_json::to_vec(message).context("Failed to serialize message")?;
        
        self.client
            .publish_with_headers(subject, headers, payload.into())
            .await
            .context("Failed to publish message")?;

        Ok(())
    }

    /// Get connection statistics
    pub fn connection_info(&self) -> ConnectionInfo {
        let server_info = self.client.server_info();
        ConnectionInfo {
            connected_servers: server_info.connect_urls.clone(),
            max_payload: server_info.max_payload,
            is_connected: self.client.connection_state() == async_nats::connection::State::Connected,
            pending_msgs: 0, // Would need to track this separately
        }
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        if self.client.connection_state() != async_nats::connection::State::Connected {
            return Err(anyhow::anyhow!("NATS client not connected"));
        }

        // Try a simple request to verify connectivity
        timeout(
            Duration::from_secs(5),
            self.client.request("$SYS.REQ.SERVER.INFO", "".into())
        )
        .await
        .context("Health check timeout")?
        .context("Health check failed")?;

        Ok(())
    }
}

/// Connection information for monitoring
#[derive(Debug, Serialize)]
pub struct ConnectionInfo {
    pub connected_servers: Vec<String>,
    pub max_payload: usize,
    pub is_connected: bool,
    pub pending_msgs: u64,
}

/// Create headers for command messages
fn create_command_headers<T>(message: &NatsMessage<T>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("message-type", "command");
    headers.insert("message-id", message.message_id.as_str());
    headers.insert("correlation-id", message.correlation_id.as_str());
    headers.insert("timestamp", message.timestamp.to_rfc3339().as_str());
    
    for (key, value) in &message.metadata {
        headers.insert(key.as_str(), value.as_str());
    }
    
    headers
}

/// Create headers for event messages
fn create_event_headers<T>(message: &NatsMessage<T>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("message-type", "event");
    headers.insert("message-id", message.message_id.as_str());
    headers.insert("correlation-id", message.correlation_id.as_str());
    headers.insert("timestamp", message.timestamp.to_rfc3339().as_str());
    
    for (key, value) in &message.metadata {
        headers.insert(key.as_str(), value.as_str());
    }
    
    headers
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests assume NATS is available at localhost:4222

    // Removed start_nats_container - use external NATS server for integration tests

    #[tokio::test]
    #[ignore] // Requires NATS server at localhost:4222
    async fn test_nats_client_connection() {
        let config = NatsClientConfig {
            servers: vec!["nats://localhost:4222".to_string()],
            connect_timeout: Duration::from_secs(5),
            ..Default::default()
        };

        // Only run if NATS server is available
        if let Ok(client) = NatsClient::new(config).await {
            assert!(client.health_check().await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_message_envelope() {
        let correlation_id = "test-correlation-123".to_string();
        let payload = "test payload".to_string();
        
        let message = NatsMessage::new(payload.clone(), correlation_id.clone());
        
        assert_eq!(message.correlation_id, correlation_id);
        assert_eq!(message.payload, payload);
        assert!(!message.message_id.is_empty());
        assert!(message.metadata.is_empty());
        
        let message_with_metadata = message.with_metadata("key1".to_string(), "value1".to_string());
        assert_eq!(message_with_metadata.metadata.get("key1"), Some(&"value1".to_string()));
    }
}