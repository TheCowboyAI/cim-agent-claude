use async_nats::{Client, ConnectOptions, jetstream::{self, consumer::PullConsumer, kv, object_store}};
use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use serde_json;
use std::{sync::Arc, time::Duration, collections::HashMap};
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    domain::{commands::*, events::*, value_objects::*, errors::*, ConversationAggregate},
    ports::{ConversationPort, ConversationStatePort, PortHealth, PortMetrics},
};

/// Production-ready NATS cluster configuration
#[derive(Debug, Clone)]
pub struct NatsClusterConfig {
    pub urls: Vec<String>,
    pub credentials_path: Option<String>,
    pub tls_ca_cert: Option<String>,
    pub name: String,
    pub domain: String,
    pub account: String,
    pub reconnect_buffer_size: usize,
    pub max_reconnects: Option<usize>,
    pub reconnect_delay: Duration,
}

impl Default for NatsClusterConfig {
    fn default() -> Self {
        Self {
            urls: vec!["nats://localhost:4222".to_string()],
            credentials_path: None,
            tls_ca_cert: None,
            name: "cim-claude-adapter".to_string(),
            domain: "claude".to_string(),
            account: "CLAUDE_ADAPTER".to_string(),
            reconnect_buffer_size: 8 * 1024 * 1024, // 8MB
            max_reconnects: Some(10),
            reconnect_delay: Duration::from_millis(250),
        }
    }
}

/// NATS stream configurations for production
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub name: String,
    pub subjects: Vec<String>,
    pub retention: jetstream::stream::RetentionPolicy,
    pub storage: jetstream::stream::StorageType,
    pub max_messages: i64,
    pub max_bytes: i64,
    pub max_age: Duration,
    pub max_message_size: i32,
    pub replicas: usize,
    pub discard: jetstream::stream::DiscardPolicy,
}

/// NATS adapter implementing the conversation port with production features
pub struct NatsAdapter {
    client: Client,
    jetstream: jetstream::Context,
    config: NatsClusterConfig,
    metrics: Arc<RwLock<PortMetrics>>,
    health_status: Arc<RwLock<NatsHealthStatus>>,
    consumers: Arc<Mutex<HashMap<String, PullConsumer>>>,
    object_store: Arc<RwLock<Option<object_store::ObjectStore>>>,
    kv_stores: Arc<RwLock<HashMap<String, kv::Store>>>,
}

/// Detailed health status for NATS infrastructure
#[derive(Debug, Clone)]
pub struct NatsHealthStatus {
    pub is_connected: bool,
    pub last_heartbeat: DateTime<Utc>,
    pub cluster_size: usize,
    pub active_streams: usize,
    pub active_consumers: usize,
    pub object_store_available: bool,
    pub kv_stores_count: usize,
    pub errors: Vec<String>,
}

impl NatsAdapter {
    /// Create new production NATS adapter with clustering support
    pub async fn new(config: NatsClusterConfig) -> Result<Self, InfrastructureError> {
        let mut connect_options = ConnectOptions::new()
            .name(&config.name)
            .reconnect_buffer_size(config.reconnect_buffer_size)
            .reconnect_delay_callback(|attempts| {
                Duration::from_millis(std::cmp::min(250 * 2_u64.pow(attempts), 8000))
            });

        if let Some(max_reconnects) = config.max_reconnects {
            connect_options = connect_options.max_reconnects(max_reconnects);
        }

        if let Some(creds_path) = &config.credentials_path {
            connect_options = connect_options.credentials_file(creds_path).await
                .map_err(|e| InfrastructureError::NatsConnection(
                    format!("Failed to load credentials: {}", e)
                ))?;
        }

        let client = async_nats::connect_with_options(
            config.urls.clone(),
            connect_options,
        )
        .await
        .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;
            
        let jetstream = jetstream::new(client.clone());
        
        let health_status = NatsHealthStatus {
            is_connected: true,
            last_heartbeat: Utc::now(),
            cluster_size: 1,
            active_streams: 0,
            active_consumers: 0,
            object_store_available: false,
            kv_stores_count: 0,
            errors: Vec::new(),
        };
        
        let adapter = Self {
            client,
            jetstream,
            config: config.clone(),
            metrics: Arc::new(RwLock::new(PortMetrics::default())),
            health_status: Arc::new(RwLock::new(health_status)),
            consumers: Arc::new(Mutex::new(HashMap::new())),
            object_store: Arc::new(RwLock::new(None)),
            kv_stores: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Initialize infrastructure
        adapter.ensure_streams().await?;
        adapter.initialize_object_store().await?;
        adapter.initialize_kv_stores().await?;
        adapter.start_health_monitor();
        
        Ok(adapter)
    }

    /// Create adapter with default local configuration
    pub async fn new_local() -> Result<Self, InfrastructureError> {
        Self::new(NatsClusterConfig::default()).await
    }
    
    /// Define production stream configurations
    fn get_stream_configs(&self) -> Vec<StreamConfig> {
        let domain = &self.config.domain;
        vec![
            // Commands Stream - WorkQueue retention for command processing
            StreamConfig {
                name: format!("CIM_{}_CONV_CMD", domain.to_uppercase()),
                subjects: vec![format!("{}.conv.cmd.*", domain)],
                retention: jetstream::stream::RetentionPolicy::WorkQueue,
                storage: jetstream::stream::StorageType::File,
                max_messages: 100_000,
                max_bytes: 1024 * 1024 * 1024, // 1GB
                max_age: Duration::from_secs(24 * 60 * 60), // 24 hours
                max_message_size: 1024 * 1024, // 1MB
                replicas: 3,
                discard: jetstream::stream::DiscardPolicy::Old,
            },
            // Events Stream - Long retention for event sourcing
            StreamConfig {
                name: format!("CIM_{}_CONV_EVT", domain.to_uppercase()),
                subjects: vec![format!("{}.conv.evt.*", domain)],
                retention: jetstream::stream::RetentionPolicy::Limits,
                storage: jetstream::stream::StorageType::File,
                max_messages: 1_000_000,
                max_bytes: 10 * 1024 * 1024 * 1024, // 10GB
                max_age: Duration::from_secs(90 * 24 * 60 * 60), // 90 days
                max_message_size: 1024 * 1024, // 1MB
                replicas: 3,
                discard: jetstream::stream::DiscardPolicy::Old,
            },
            // Responses Stream - Short retention for response delivery
            StreamConfig {
                name: format!("CIM_{}_CONV_RESP", domain.to_uppercase()),
                subjects: vec![format!("{}.conv.resp.*", domain)],
                retention: jetstream::stream::RetentionPolicy::Interest,
                storage: jetstream::stream::StorageType::Memory,
                max_messages: 50_000,
                max_bytes: 512 * 1024 * 1024, // 512MB
                max_age: Duration::from_secs(60 * 60), // 1 hour
                max_message_size: 64 * 1024, // 64KB
                replicas: 2,
                discard: jetstream::stream::DiscardPolicy::Old,
            },
            // Tool Operations Stream
            StreamConfig {
                name: format!("CIM_{}_TOOL_OPS", domain.to_uppercase()),
                subjects: vec![format!("{}.tool.*", domain)],
                retention: jetstream::stream::RetentionPolicy::WorkQueue,
                storage: jetstream::stream::StorageType::File,
                max_messages: 200_000,
                max_bytes: 2 * 1024 * 1024 * 1024, // 2GB
                max_age: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
                max_message_size: 2 * 1024 * 1024, // 2MB
                replicas: 3,
                discard: jetstream::stream::DiscardPolicy::Old,
            },
            // Configuration Stream
            StreamConfig {
                name: format!("CIM_{}_CONFIG", domain.to_uppercase()),
                subjects: vec![format!("{}.config.*", domain)],
                retention: jetstream::stream::RetentionPolicy::Limits,
                storage: jetstream::stream::StorageType::File,
                max_messages: 10_000,
                max_bytes: 100 * 1024 * 1024, // 100MB
                max_age: Duration::from_secs(365 * 24 * 60 * 60), // 1 year
                max_message_size: 512 * 1024, // 512KB
                replicas: 3,
                discard: jetstream::stream::DiscardPolicy::Old,
            },
        ]
    }

    /// Ensure required JetStream streams exist with production configuration
    async fn ensure_streams(&self) -> Result<(), InfrastructureError> {
        let stream_configs = self.get_stream_configs();
        let mut stream_count = 0;

        for stream_config in stream_configs {
            let config = jetstream::stream::Config {
                name: stream_config.name.clone(),
                subjects: stream_config.subjects,
                retention: stream_config.retention,
                storage: stream_config.storage,
                max_messages: stream_config.max_messages,
                max_bytes: stream_config.max_bytes,
                max_age: stream_config.max_age,
                max_message_size: stream_config.max_message_size,
                num_replicas: stream_config.replicas,
                discard: stream_config.discard,
                ..Default::default()
            };

            match self.jetstream.get_or_create_stream(config).await {
                Ok(stream_info) => {
                    info!(
                        "Stream {} ready with {} messages, {} bytes",
                        stream_config.name,
                        stream_info.state.messages,
                        stream_info.state.bytes
                    );
                    stream_count += 1;
                }
                Err(e) => {
                    error!("Failed to create stream {}: {}", stream_config.name, e);
                    return Err(InfrastructureError::NatsConnection(
                        format!("Failed to create stream {}: {}", stream_config.name, e)
                    ));
                }
            }
        }

        // Update health status
        {
            let mut health = self.health_status.write().await;
            health.active_streams = stream_count;
            health.last_heartbeat = Utc::now();
        }

        info!("Successfully initialized {} JetStream streams", stream_count);
        Ok(())
    }
    
    /// Initialize NATS object store for attachment storage
    async fn initialize_object_store(&self) -> Result<(), InfrastructureError> {
        let store_name = format!("CIM_{}_ATTACHMENTS", self.config.domain.to_uppercase());
        
        let config = object_store::Config {
            bucket: store_name.clone(),
            description: Some("CIM Claude Adapter attachments and large objects".to_string()),
            max_bucket_size: Some(50 * 1024 * 1024 * 1024), // 50GB
            storage: jetstream::stream::StorageType::File,
            num_replicas: 3,
            ..Default::default()
        };

        match self.jetstream.create_object_store(config).await {
            Ok(store) => {
                info!("Object store {} initialized successfully", store_name);
                let mut object_store = self.object_store.write().await;
                *object_store = Some(store);
                
                let mut health = self.health_status.write().await;
                health.object_store_available = true;
                
                Ok(())
            }
            Err(e) => {
                // Try to get existing store
                match self.jetstream.get_object_store(&store_name).await {
                    Ok(store) => {
                        info!("Using existing object store: {}", store_name);
                        let mut object_store_guard = self.object_store.write().await;
                        *object_store_guard = Some(store);
                        
                        let mut health = self.health_status.write().await;
                        health.object_store_available = true;
                        
                        Ok(())
                    }
                    Err(get_err) => {
                        error!("Failed to initialize object store: create={}, get={}", e, get_err);
                        Err(InfrastructureError::NatsConnection(
                            format!("Failed to initialize object store: {}", e)
                        ))
                    }
                }
            }
        }
    }

    /// Initialize KV stores for metadata and configuration
    async fn initialize_kv_stores(&self) -> Result<(), InfrastructureError> {
        let domain_upper = self.config.domain.to_uppercase();
        let kv_configs = vec![
            // Conversation metadata
            (format!("CIM_{}_CONV_META", domain_upper), "Conversation metadata and state"),
            // Session data
            (format!("CIM_{}_SESSIONS", domain_upper), "Active session information"),
            // Configuration data
            (format!("CIM_{}_CONFIG", domain_upper), "Runtime configuration"),
            // Tool state
            (format!("CIM_{}_TOOL_STATE", domain_upper), "Tool execution state"),
            // Rate limiting data
            (format!("CIM_{}_RATE_LIMITS", domain_upper), "Rate limiting counters"),
        ];

        let mut kv_stores = self.kv_stores.write().await;
        let mut store_count = 0;

        for (bucket_name, description) in kv_configs {
            let config = kv::Config {
                bucket: bucket_name.clone(),
                description: description.to_string(),
                max_value_size: 1024 * 1024, // 1MB per value
                history: 5,
                ttl: Some(Duration::from_secs(30 * 24 * 60 * 60)), // 30 days default TTL
                max_bucket_size: Some(1024 * 1024 * 1024), // 1GB per bucket
                storage: jetstream::stream::StorageType::File,
                num_replicas: 3,
                ..Default::default()
            };

            match self.jetstream.create_key_value(config).await {
                Ok(store) => {
                    info!("KV store {} created successfully", bucket_name);
                    kv_stores.insert(bucket_name.clone(), store);
                    store_count += 1;
                }
                Err(_) => {
                    // Try to get existing store
                    match self.jetstream.get_key_value(&bucket_name).await {
                        Ok(store) => {
                            info!("Using existing KV store: {}", bucket_name);
                            kv_stores.insert(bucket_name.clone(), store);
                            store_count += 1;
                        }
                        Err(e) => {
                            warn!("Failed to initialize KV store {}: {}", bucket_name, e);
                        }
                    }
                }
            }
        }

        // Update health status
        {
            let mut health = self.health_status.write().await;
            health.kv_stores_count = store_count;
        }

        info!("Initialized {} KV stores", store_count);
        Ok(())
    }

    /// Start health monitoring background task
    fn start_health_monitor(&self) {
        let client = self.client.clone();
        let health_status = self.health_status.clone();
        let jetstream = self.jetstream.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let mut health = health_status.write().await;
                health.is_connected = matches!(
                    client.connection_state(),
                    async_nats::connection::State::Connected
                );
                health.last_heartbeat = Utc::now();
                
                // Check stream health
                if let Ok(stream_names) = jetstream.stream_names().await {
                    let mut count = 0;
                    while let Ok(Some(_)) = stream_names.try_next().await {
                        count += 1;
                    }
                    health.active_streams = count;
                }
                
                // Clear old errors
                if health.errors.len() > 10 {
                    health.errors.truncate(5);
                }
                
                if !health.is_connected {
                    health.errors.push(format!(
                        "NATS connection lost at {}",
                        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                    ));
                }
                
                drop(health);
                
                debug!("Health monitor tick completed");
            }
        });
    }

    /// Generate NATS subject for command with domain prefix
    fn command_subject(&self, session_id: &SessionId, operation: &str) -> String {
        format!("{}.conv.cmd.{}.{}", self.config.domain, session_id.as_uuid(), operation)
    }
    
    /// Generate NATS subject for event with domain prefix
    fn event_subject(&self, conversation_id: &ConversationId, event_type: &str) -> String {
        format!("{}.conv.evt.{}.{}", self.config.domain, conversation_id.as_uuid(), event_type)
    }
    
    /// Generate NATS subject for response with domain prefix
    fn response_subject(&self, conversation_id: &ConversationId) -> String {
        format!("{}.conv.resp.{}.content", self.config.domain, conversation_id.as_uuid())
    }
    
    /// Generate NATS subject for tool operations
    fn tool_subject(&self, operation: &str, tool_id: &str) -> String {
        format!("{}.tool.{}.{}", self.config.domain, operation, tool_id)
    }
    
    /// Generate NATS subject for configuration updates
    fn config_subject(&self, config_type: &str) -> String {
        format!("{}.config.{}", self.config.domain, config_type)
    }
    
    /// Extract event type from domain event
    fn event_type(event: &DomainEvent) -> &'static str {
        match event {
            DomainEvent::ConversationStarted { .. } => "conversation.started",
            DomainEvent::PromptSent { .. } => "prompt.sent",
            DomainEvent::ResponseReceived { .. } => "response.received",
            DomainEvent::ConversationEnded { .. } => "conversation.ended",
            DomainEvent::RateLimitExceeded { .. } => "rate.limit.exceeded",
            DomainEvent::ClaudeApiErrorOccurred { .. } => "claude.api.error",
        }
    }
    
    /// Update metrics
    async fn update_metrics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut PortMetrics),
    {
        let mut metrics = self.metrics.write().await;
        update_fn(&mut *metrics);
    }
}

#[async_trait]
impl ConversationPort for NatsAdapter {
    async fn handle_command(
        &self,
        command: Command,
        correlation_id: CorrelationId,
    ) -> Result<Vec<DomainEvent>, ApplicationError> {
        // This would typically delegate to an application service
        // For now, we'll return an empty result as this is just the adapter
        info!(
            "Received command: {:?} with correlation_id: {}", 
            command, 
            correlation_id.as_uuid()
        );
        
        self.update_metrics(|m| m.commands_processed += 1).await;
        
        // The actual command handling would be done by the application service
        // This adapter is just responsible for the transport layer
        // The correlation_id is essential for tracking causation chains
        Ok(vec![])
    }
    
    async fn publish_events(
        &self,
        events: Vec<EventEnvelope>,
    ) -> Result<(), ApplicationError> {
        for event_envelope in events {
            let conversation_id = event_envelope.event.conversation_id();
            let event_type = Self::event_type(&event_envelope.event);
            let subject = self.event_subject(conversation_id, event_type);
            
            let payload = serde_json::to_vec(&event_envelope)
                .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;
                
            // Add NATS headers for correlation tracking
            let mut headers = async_nats::HeaderMap::new();
            headers.insert(
                "correlation-id", 
                event_envelope.correlation_id.as_uuid().to_string().as_str()
            );
            headers.insert(
                "event-id",
                event_envelope.event_id.as_uuid().to_string().as_str()
            );
            headers.insert(
                "causation-id",
                event_envelope.causation_id.as_uuid().to_string().as_str()
            );
            
            match self.jetstream.publish_with_headers(
                subject.clone(),
                headers,
                Bytes::from(payload),
            ).await {
                Ok(_) => {
                    info!("Published event {} to {}", event_type, subject);
                    self.update_metrics(|m| m.events_published += 1).await;
                }
                Err(e) => {
                    error!("Failed to publish event {}: {}", event_type, e);
                    self.update_metrics(|m| m.errors_count += 1).await;
                    return Err(InfrastructureError::NatsPublish(e.to_string()).into());
                }
            }
        }
        
        Ok(())
    }
    
    async fn subscribe_to_commands<F>(&self, handler: F) -> Result<(), ApplicationError>
    where
        F: Fn(CommandEnvelope) -> Result<(), ApplicationError> + Send + Sync + 'static,
    {
        let consumer: PullConsumer = self.jetstream
            .create_consumer_on_stream(
                jetstream::consumer::pull::Config {
                    durable_name: Some("claude-command-processor".to_string()),
                    filter_subject: "claude.cmd.*".to_string(),
                    ..Default::default()
                },
                "CLAUDE_COMMANDS",
            )
            .await
            .map_err(|e| InfrastructureError::NatsSubscribe(e.to_string()))?;
        
        let handler = Arc::new(handler);
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            let mut messages = consumer.messages().await.unwrap();
            
            while let Some(message) = messages.next().await {
                match message {
                    Ok(msg) => {
                        let start_time = std::time::Instant::now();
                        
                        match serde_json::from_slice::<CommandEnvelope>(&msg.payload) {
                            Ok(command_envelope) => {
                                info!(
                                    "Received command: {:?} with correlation ID: {}",
                                    command_envelope.command,
                                    command_envelope.correlation_id.as_uuid()
                                );
                                
                                match handler(command_envelope) {
                                    Ok(_) => {
                                        let _ = msg.ack().await;
                                        let mut m = metrics.write().await;
                                        m.commands_processed += 1;
                                        m.average_processing_time_ms = 
                                            (m.average_processing_time_ms * (m.commands_processed - 1) as f64 + 
                                             start_time.elapsed().as_millis() as f64) / m.commands_processed as f64;
                                    }
                                    Err(e) => {
                                        warn!("Failed to handle command: {}", e);
                                        // In async-nats 0.40+, message handling is different
                                        // For now, we'll just log the error
                                        let mut m = metrics.write().await;
                                        m.errors_count += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize command: {}", e);
                                // In async-nats 0.40+, message handling is different
                                // For now, we'll just log the error
                                let mut m = metrics.write().await;
                                m.errors_count += 1;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                        let mut m = metrics.write().await;
                        m.errors_count += 1;
                    }
                }
            }
        });
        
        info!("Started command subscription");
        Ok(())
    }
    
    async fn health_check(&self) -> Result<PortHealth, ApplicationError> {
        // Check NATS connection
        match self.client.connection_state() {
            async_nats::connection::State::Connected => {
                let metrics = self.metrics.read().await.clone();
                Ok(PortHealth::healthy(
                    "NATS connection healthy".to_string()
                ).with_metrics(metrics))
            }
            state => Ok(PortHealth::unhealthy(
                format!("NATS connection state: {:?}", state)
            )),
        }
    }
}

#[async_trait]
impl ConversationStatePort for NatsAdapter {
    async fn load_conversation(
        &self,
        id: &ConversationId,
    ) -> Result<Option<ConversationAggregate>, ApplicationError> {
        // Use NATS KV store to retrieve conversation state
        let kv = self.jetstream
            .get_key_value("CONVERSATION_STATE")
            .await
            .map_err(|e| InfrastructureError::NatsKvStore(e.to_string()))?;
        
        let key = format!("conversation:{}", id.as_uuid());
        
        match kv.get(&key).await {
            Ok(Some(entry)) => {
                let aggregate = serde_json::from_slice(&entry)
                    .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;
                Ok(Some(aggregate))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(InfrastructureError::NatsKvStore(e.to_string()).into()),
        }
    }
    
    async fn save_conversation(
        &self,
        aggregate: &ConversationAggregate,
        expected_version: u64,
    ) -> Result<(), ApplicationError> {
        // Use NATS KV store with optimistic locking
        let kv = match self.jetstream.get_key_value("CONVERSATION_STATE").await {
            Ok(kv) => kv,
            Err(_) => {
                // Create the KV store if it doesn't exist
                self.jetstream
                    .create_key_value(kv::Config {
                        bucket: "CONVERSATION_STATE".to_string(),
                        description: "Conversation aggregate state storage".to_string(),
                        max_value_size: 1024 * 1024, // 1MB per conversation
                        history: 10,
                        ..Default::default()
                    })
                    .await
                    .map_err(|e| InfrastructureError::NatsKvStore(e.to_string()))?
            }
        };
        
        let key = format!("conversation:{}", aggregate.id().as_uuid());
        let value = serde_json::to_vec(aggregate)
            .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;
        
        // For now, just do a simple put - optimistic locking can be enhanced later
        // In production, you'd use the expected_version for proper optimistic concurrency control
        info!(
            "Saving conversation {} at version {} (expected: {})", 
            aggregate.id().as_uuid(), 
            aggregate.version(),
            expected_version
        );
        
        match kv.put(&key, value.into()).await {
            Ok(_) => {
                info!("Saved conversation {} successfully", aggregate.id().as_uuid());
                Ok(())
            }
            Err(e) => Err(InfrastructureError::NatsKvStore(e.to_string()).into()),
        }
    }
    
    async fn find_active_conversations(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<ConversationId>, ApplicationError> {
        // Use NATS KV store to find conversations by session
        let kv = self.jetstream
            .get_key_value("CONVERSATION_STATE")
            .await
            .map_err(|e| InfrastructureError::NatsKvStore(e.to_string()))?;
        
        let mut active_conversations = Vec::new();
        
        // For now, we'll use a simple approach - in production you'd want to use watch/scan patterns
        // This is a simplified implementation for getting the service running
        info!("Finding active conversations for session: {}", session_id.as_uuid());
        
        // In a real implementation, you might maintain an index or use NATS streams for queries
        // For now, we'll return an empty list and rely on direct lookups
        
        Ok(active_conversations)
    }
    
    async fn cleanup_expired_conversations(&self) -> Result<u32, ApplicationError> {
        // Clean up expired conversations from KV store
        let kv = self.jetstream
            .get_key_value("CONVERSATION_STATE")
            .await
            .map_err(|e| InfrastructureError::NatsKvStore(e.to_string()))?;
        
        let mut cleaned_count = 0u32;
        
        // Simplified cleanup for initial implementation
        info!("Background cleanup check performed - would scan for expired conversations");
        
        // In production, you'd implement proper key scanning/watching here
        
        if cleaned_count > 0 {
            info!("Cleaned up {} expired conversations", cleaned_count);
        }
        
        Ok(cleaned_count)
    }
}

/// Helper for testing
impl NatsAdapter {
    pub async fn publish_command(
        &self,
        command: CommandEnvelope,
        session_id: &SessionId,
    ) -> Result<(), InfrastructureError> {
        let operation = match &command.command {
            Command::StartConversation { .. } => "start",
            Command::SendPrompt { .. } => "prompt",
            Command::EndConversation { .. } => "end",
        };
        
        let subject = self.command_subject(session_id, operation);
        let payload = serde_json::to_vec(&command)
            .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;
            
        let mut headers = async_nats::HeaderMap::new();
        headers.insert(
            "correlation-id",
            command.correlation_id.as_uuid().to_string().as_str()
        );
        
        self.jetstream
            .publish_with_headers(subject, headers, Bytes::from(payload))
            .await
            .map_err(|e| InfrastructureError::NatsPublish(e.to_string()))?;
            
        Ok(())
    }
    
    pub async fn get_metrics(&self) -> PortMetrics {
        self.metrics.read().await.clone()
    }

    /// Get comprehensive health status
    pub async fn get_health_status(&self) -> NatsHealthStatus {
        self.health_status.read().await.clone()
    }

    /// Store large object in NATS object store
    pub async fn store_object(&self, name: &str, data: Vec<u8>) -> Result<String, InfrastructureError> {
        let object_store = self.object_store.read().await;
        
        if let Some(store) = object_store.as_ref() {
            let info = store.put(name, data.into()).await
                .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;
            
            info!("Stored object: {} ({} bytes)", name, info.size);
            Ok(info.nuid)
        } else {
            Err(InfrastructureError::NatsConnection("Object store not available".to_string()))
        }
    }

    /// Retrieve large object from NATS object store
    pub async fn get_object(&self, name: &str) -> Result<Vec<u8>, InfrastructureError> {
        let object_store = self.object_store.read().await;
        
        if let Some(store) = object_store.as_ref() {
            let mut object = store.get(name).await
                .map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;
                
            let mut data = Vec::new();
            while let Some(chunk) = object.next().await {
                let chunk = chunk.map_err(|e| InfrastructureError::NatsConnection(e.to_string()))?;
                data.extend_from_slice(&chunk);
            }
            
            debug!("Retrieved object: {} ({} bytes)", name, data.len());
            Ok(data)
        } else {
            Err(InfrastructureError::NatsConnection("Object store not available".to_string()))
        }
    }

    /// Store key-value data in specific KV bucket
    pub async fn kv_put(&self, bucket: &str, key: &str, value: Vec<u8>) -> Result<u64, InfrastructureError> {
        let kv_stores = self.kv_stores.read().await;
        
        if let Some(store) = kv_stores.get(bucket) {
            let revision = store.put(key, value.into()).await
                .map_err(|e| InfrastructureError::NatsKvStore(e.to_string()))?;
            
            debug!("KV put: {}:{} -> revision {}", bucket, key, revision);
            Ok(revision)
        } else {
            Err(InfrastructureError::NatsKvStore(format!("Bucket not found: {}", bucket)))
        }
    }

    /// Get key-value data from specific KV bucket
    pub async fn kv_get(&self, bucket: &str, key: &str) -> Result<Option<Vec<u8>>, InfrastructureError> {
        let kv_stores = self.kv_stores.read().await;
        
        if let Some(store) = kv_stores.get(bucket) {
            match store.get(key).await {
                Ok(Some(entry)) => {
                    debug!("KV get: {}:{} -> {} bytes", bucket, key, entry.len());
                    Ok(Some(entry.to_vec()))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(InfrastructureError::NatsKvStore(e.to_string())),
            }
        } else {
            Err(InfrastructureError::NatsKvStore(format!("Bucket not found: {}", bucket)))
        }
    }

    /// Delete key from specific KV bucket
    pub async fn kv_delete(&self, bucket: &str, key: &str) -> Result<(), InfrastructureError> {
        let kv_stores = self.kv_stores.read().await;
        
        if let Some(store) = kv_stores.get(bucket) {
            store.delete(key).await
                .map_err(|e| InfrastructureError::NatsKvStore(e.to_string()))?;
            
            debug!("KV delete: {}:{}", bucket, key);
            Ok(())
        } else {
            Err(InfrastructureError::NatsKvStore(format!("Bucket not found: {}", bucket)))
        }
    }

    /// Create a durable consumer for reliable message processing
    pub async fn create_durable_consumer(
        &self,
        stream_name: &str,
        consumer_name: &str,
        filter_subject: Option<String>,
        max_deliver: Option<i64>,
    ) -> Result<(), InfrastructureError> {
        let config = jetstream::consumer::pull::Config {
            durable_name: Some(consumer_name.to_string()),
            description: Some(format!("Durable consumer for {}", stream_name)),
            filter_subject: filter_subject.unwrap_or_else(|| ">".to_string()),
            max_deliver: max_deliver.or(Some(3)),
            ack_wait: Duration::from_secs(30),
            max_waiting: 512,
            max_ack_pending: 1024,
            replay_policy: jetstream::consumer::ReplayPolicy::Instant,
            ..Default::default()
        };

        let consumer = self.jetstream
            .create_consumer_on_stream(config, stream_name)
            .await
            .map_err(|e| InfrastructureError::NatsSubscribe(e.to_string()))?;

        // Store consumer reference
        {
            let mut consumers = self.consumers.lock().await;
            consumers.insert(consumer_name.to_string(), consumer);
        }

        // Update health status
        {
            let mut health = self.health_status.write().await;
            health.active_consumers += 1;
        }

        info!("Created durable consumer: {} on stream: {}", consumer_name, stream_name);
        Ok(())
    }

    /// Get consumer by name
    pub async fn get_consumer(&self, consumer_name: &str) -> Option<PullConsumer> {
        let consumers = self.consumers.lock().await;
        consumers.get(consumer_name).cloned()
    }

    /// Publish tool operation result
    pub async fn publish_tool_result(
        &self,
        tool_id: &str,
        operation: &str,
        result: serde_json::Value,
        correlation_id: Uuid,
    ) -> Result<(), InfrastructureError> {
        let subject = self.tool_subject(operation, tool_id);
        
        let payload = serde_json::json!({
            "tool_id": tool_id,
            "operation": operation,
            "result": result,
            "correlation_id": correlation_id,
            "timestamp": Utc::now(),
        });
        
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("tool-id", tool_id);
        headers.insert("operation", operation);
        headers.insert("correlation-id", &correlation_id.to_string());
        
        self.jetstream
            .publish_with_headers(
                subject.clone(),
                headers,
                serde_json::to_vec(&payload)
                    .map_err(|e| InfrastructureError::Serialization(e.to_string()))?
                    .into(),
            )
            .await
            .map_err(|e| InfrastructureError::NatsPublish(e.to_string()))?;
            
        debug!("Published tool result: {} -> {}", tool_id, subject);
        Ok(())
    }

    /// Subscribe to configuration changes
    pub async fn subscribe_to_config_changes<F>(
        &self,
        config_type: &str,
        handler: F,
    ) -> Result<(), InfrastructureError>
    where
        F: Fn(serde_json::Value) + Send + Sync + 'static,
    {
        let subject = self.config_subject(config_type);
        let handler = Arc::new(handler);
        let client = self.client.clone();
        
        tokio::spawn(async move {
            if let Ok(mut subscription) = client.subscribe(subject.clone()).await {
                info!("Subscribed to config changes: {}", subject);
                
                while let Some(message) = subscription.next().await {
                    if let Ok(config_value) = serde_json::from_slice::<serde_json::Value>(&message.payload) {
                        handler(config_value);
                    } else {
                        warn!("Failed to parse config message from: {}", subject);
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Publish configuration update
    pub async fn publish_config_update(
        &self,
        config_type: &str,
        config_data: serde_json::Value,
    ) -> Result<(), InfrastructureError> {
        let subject = self.config_subject(config_type);
        
        let payload = serde_json::json!({
            "config_type": config_type,
            "data": config_data,
            "timestamp": Utc::now(),
            "version": Uuid::new_v4(),
        });
        
        self.client
            .publish(subject.clone(), serde_json::to_vec(&payload)
                .map_err(|e| InfrastructureError::Serialization(e.to_string()))?
                .into())
            .await
            .map_err(|e| InfrastructureError::NatsPublish(e.to_string()))?;
            
        info!("Published config update: {} -> {}", config_type, subject);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_subject_generation() {
        let session_id = SessionId::new();
        let conversation_id = ConversationId::new();
        
        let config = NatsClusterConfig::default();
        let adapter = NatsAdapter::new_local().await.unwrap();
        let cmd_subject = adapter.command_subject(&session_id, "start");
        assert!(cmd_subject.starts_with("claude.cmd."));
        assert!(cmd_subject.ends_with(".start"));
        
        let event_subject = adapter.event_subject(&conversation_id, "conversation_started");
        assert!(event_subject.starts_with("claude.conv.evt."));
        assert!(event_subject.ends_with(".conversation_started"));
        
        let resp_subject = adapter.response_subject(&conversation_id);
        assert!(resp_subject.starts_with("claude.conv.resp."));
        assert!(resp_subject.ends_with(".content"));
    }
    
    #[test]
    fn test_event_type_mapping() {
        let event = DomainEvent::ConversationStarted {
            conversation_id: ConversationId::new(),
            session_id: SessionId::new(),
            initial_prompt: crate::domain::Prompt::new("test".to_string()).unwrap(),
            context: ConversationContext::default(),
        };
        
        assert_eq!(NatsAdapter::event_type(&event), "conversation.started");
    }
}