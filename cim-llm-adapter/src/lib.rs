//! CIM LLM Adapter
//!
//! Universal LLM abstraction layer with NATS integration for CIM ecosystems.
//! Provides consistent interfaces for multiple LLM providers (Claude, OpenAI, Ollama)
//! with dialog management, context preservation, and event-driven architecture.

pub mod providers;
pub mod dialog;
pub mod config;
pub mod events;
pub mod error;
pub mod sage_integration;

// Re-export main types
pub use providers::{LlmProvider, ProviderConfig, ProviderResponse, ProviderError};
pub use dialog::{DialogManager, DialogContext, ConversationEntry};
pub use config::LlmAdapterConfig;
pub use events::{LlmEvent, DialogEvent};
pub use error::LlmAdapterError;
pub use sage_integration::SageLlmClient;

use async_nats::{Client, jetstream};
use std::sync::Arc;
use tracing::{info, error};
use uuid::Uuid;

/// Main LLM Adapter service for coordinating LLM providers
pub struct LlmAdapter {
    nats_client: Client,
    jetstream: jetstream::Context,
    dialog_manager: Arc<DialogManager>,
    config: LlmAdapterConfig,
    domain: Option<String>,
}

impl LlmAdapter {
    /// Create new LLM Adapter instance
    pub async fn new(
        nats_url: &str,
        config: LlmAdapterConfig,
    ) -> Result<Self, LlmAdapterError> {
        let nats_client = async_nats::connect(nats_url).await
            .map_err(|e| LlmAdapterError::NatsConnection(e.to_string()))?;
        
        let jetstream = jetstream::new(nats_client.clone());
        
        // Detect domain from environment
        let domain = Self::detect_domain();
        
        let dialog_manager = Arc::new(
            DialogManager::new(nats_client.clone(), jetstream.clone(), domain.clone()).await?
        );
        
        info!("🤖 LLM Adapter initialized");
        if let Some(ref d) = domain {
            info!("📍 Domain: {}", d);
        }
        info!("Available providers: {:?}", config.available_providers());
        
        Ok(Self {
            nats_client,
            jetstream,
            dialog_manager,
            config,
            domain,
        })
    }
    
    /// Detect domain from environment
    fn detect_domain() -> Option<String> {
        std::env::var("CIM_DOMAIN").ok()
            .or_else(|| std::env::var("LLM_ADAPTER_DOMAIN").ok())
            .or_else(|| hostname::get().ok()?.to_str().map(|s| s.to_string()))
    }
    
    /// Start the LLM adapter service
    pub async fn start_service(&mut self) -> Result<(), LlmAdapterError> {
        info!("🚀 LLM Adapter Service Starting");
        
        // Initialize NATS streams
        self.initialize_streams().await?;
        
        // Start processing requests
        self.process_requests().await
    }
    
    /// Process LLM requests from NATS
    async fn process_requests(&self) -> Result<(), LlmAdapterError> {
        let request_subject = self.build_request_subject();
        info!("📨 Subscribing to: {}", request_subject);
        
        let subscriber = self.nats_client.subscribe(request_subject).await
            .map_err(|e| LlmAdapterError::NatsConnection(e.to_string()))?;
        
        // Process requests in loop
        self.handle_request_stream(subscriber).await
    }
    
    /// Handle incoming request stream
    async fn handle_request_stream(
        &self,
        mut subscriber: async_nats::Subscriber
    ) -> Result<(), LlmAdapterError> {
        use futures_util::stream::StreamExt;
        
        while let Some(msg) = subscriber.next().await {
            if let Err(e) = self.handle_single_request(msg).await {
                error!("Request handling error: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Handle a single LLM request
    async fn handle_single_request(
        &self,
        _msg: async_nats::Message
    ) -> Result<(), LlmAdapterError> {
        // Implementation will be added in next phase
        todo!("Implement request handling")
    }
    
    /// Build request subject using cim-subject patterns
    fn build_request_subject(&self) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.commands.llm.request", domain)
        } else {
            "commands.llm.request".to_string()
        }
    }
    
    /// Initialize NATS streams for LLM operations
    async fn initialize_streams(&self) -> Result<(), LlmAdapterError> {
        info!("🌊 Initializing LLM Adapter NATS Streams");
        
        let stream_name = if let Some(ref domain) = self.domain {
            format!("LLM_{}_EVENTS", domain.to_uppercase().replace("-", "_"))
        } else {
            "LLM_EVENTS".to_string()
        };
        
        let events_pattern = if let Some(ref domain) = self.domain {
            format!("{}.events.llm.>", domain)
        } else {
            "events.llm.>".to_string()
        };
        
        info!("📊 Creating stream: {} with subjects: {}", stream_name, events_pattern);
        
        let _events_stream = self.jetstream.create_stream(jetstream::stream::Config {
            name: stream_name,
            subjects: vec![events_pattern],
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        }).await.map_err(|e| LlmAdapterError::StreamCreation(e.to_string()))?;
        
        info!("✅ LLM streams initialized");
        Ok(())
    }
}

/// Request/Response types for NATS messaging
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmRequest {
    pub request_id: String,
    pub provider: String,
    pub messages: Vec<serde_json::Value>,
    pub context: DialogContext,
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmResponse {
    pub request_id: String,
    pub response: String,
    pub provider_used: String,
    pub token_count: Option<u32>,
    pub updated_context: DialogContext,
}