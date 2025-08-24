//! LLM Adapter Service Binary
//!
//! NATS-based service that provides universal LLM access through multiple providers
//! Integrates with CIM ecosystem using event-driven architecture patterns

use anyhow::Result;
use cim_llm_adapter::{
    LlmAdapter, LlmAdapterConfig, LlmRequest, LlmResponse, LlmAdapterError,
    providers::{LlmProvider, claude::ClaudeProvider, openai::OpenAiProvider, ollama::OllamaProvider, Message, CompletionOptions},
    dialog::{DialogManager, DialogEvent},
    events::LlmEvent,
};
use async_nats::{Client, jetstream};
use futures_util::stream::StreamExt;
use serde_json;
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio;
use tracing::{info, error, warn};
use uuid::Uuid;

/// LLM Adapter Service - Main service implementation
pub struct LlmAdapterService {
    nats_client: Client,
    jetstream: jetstream::Context,
    config: LlmAdapterConfig,
    providers: HashMap<String, Box<dyn LlmProvider>>,
    dialog_manager: Arc<DialogManager>,
    domain: Option<String>,
}

impl LlmAdapterService {
    /// Create new LLM Adapter service
    pub async fn new(config: LlmAdapterConfig) -> Result<Self, LlmAdapterError> {
        // Validate configuration
        config.validate().map_err(|e| LlmAdapterError::Configuration(e))?;
        
        // Connect to NATS
        let nats_client = async_nats::connect(&config.service.nats_url).await
            .map_err(|e| LlmAdapterError::NatsConnection(e.to_string()))?;
        
        let jetstream = jetstream::new(nats_client.clone());
        
        let domain = config.service.domain.clone();
        
        // Initialize dialog manager
        let dialog_manager = Arc::new(
            DialogManager::new(nats_client.clone(), jetstream.clone(), domain.clone()).await?
        );
        
        // Initialize providers
        let mut providers: HashMap<String, Box<dyn LlmProvider>> = HashMap::new();
        
        for (name, provider_config) in &config.providers {
            match name.as_str() {
                "claude" => {
                    if provider_config.api_key.is_some() {
                        match ClaudeProvider::new(provider_config.clone()) {
                            Ok(provider) => {
                                providers.insert(name.clone(), Box::new(provider));
                                info!("✅ Claude provider initialized");
                            }
                            Err(e) => {
                                error!("❌ Failed to initialize Claude provider: {}", e);
                            }
                        }
                    } else {
                        warn!("⚠️ Claude provider config found but no API key");
                    }
                }
                "openai" => {
                    if provider_config.api_key.is_some() {
                        match OpenAiProvider::new(provider_config.clone()) {
                            Ok(provider) => {
                                providers.insert(name.clone(), Box::new(provider));
                                info!("✅ OpenAI provider initialized");
                            }
                            Err(e) => {
                                error!("❌ Failed to initialize OpenAI provider: {}", e);
                            }
                        }
                    } else {
                        warn!("⚠️ OpenAI provider config found but no API key");
                    }
                }
                "ollama" => {
                    // Ollama doesn't require an API key
                    match OllamaProvider::new(provider_config.clone()) {
                        Ok(provider) => {
                            providers.insert(name.clone(), Box::new(provider));
                            info!("✅ Ollama provider initialized");
                        }
                        Err(e) => {
                            error!("❌ Failed to initialize Ollama provider: {}", e);
                        }
                    }
                }
                _ => {
                    warn!("🚫 Unknown provider: {}", name);
                }
            }
        }
        
        if providers.is_empty() {
            return Err(LlmAdapterError::Configuration(
                "No providers successfully initialized".to_string()
            ));
        }
        
        info!("🤖 LLM Adapter Service initialized");
        if let Some(ref d) = domain {
            info!("📍 Domain: {}", d);
        }
        info!("Active providers: {:?}", providers.keys().collect::<Vec<_>>());
        
        Ok(Self {
            nats_client,
            jetstream,
            config,
            providers,
            dialog_manager,
            domain,
        })
    }
    
    /// Start the LLM adapter service
    pub async fn start_service(&mut self) -> Result<(), LlmAdapterError> {
        info!("🚀 LLM Adapter Service Starting");
        
        // Initialize NATS streams
        self.initialize_streams().await?;
        
        // Start health check task
        self.start_health_check_task().await;
        
        // Start processing requests
        self.process_requests().await
    }
    
    /// Initialize NATS streams for the service
    async fn initialize_streams(&self) -> Result<(), LlmAdapterError> {
        info!("🌊 Initializing LLM Adapter NATS Streams");
        
        let stream_name = if let Some(ref domain) = self.domain {
            format!("LLM_{}_EVENTS", domain.to_uppercase().replace("-", "_"))
        } else {
            "LLM_EVENTS".to_string()
        };
        
        let events_pattern = if let Some(ref domain) = self.domain {
            format!("{}.llm.events.>", domain)
        } else {
            "cim.llm.events.>".to_string()
        };
        
        info!("📊 Creating or getting stream: {} with subjects: {}", stream_name, events_pattern);
        
        // Try to get the stream first, create if it doesn't exist
        let _events_stream = match self.jetstream.get_stream(&stream_name).await {
            Ok(stream) => {
                info!("✅ Using existing stream: {}", stream_name);
                stream
            }
            Err(_) => {
                // Stream doesn't exist, create it
                self.jetstream.create_stream(jetstream::stream::Config {
                    name: stream_name.clone(),
                    subjects: vec![events_pattern],
                    retention: jetstream::stream::RetentionPolicy::WorkQueue,
                    storage: jetstream::stream::StorageType::File,
                    ..Default::default()
                }).await.map_err(|e| LlmAdapterError::StreamCreation(e.to_string()))?
            }
        };
        
        info!("✅ LLM streams initialized");
        Ok(())
    }
    
    /// Start health check background task
    async fn start_health_check_task(&self) {
        let providers = self.providers.keys().cloned().collect::<Vec<_>>();
        let nats_client = self.nats_client.clone();
        let domain = self.domain.clone();
        let interval = self.config.service.health_check_interval_seconds;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(
                std::time::Duration::from_secs(interval)
            );
            
            loop {
                interval_timer.tick().await;
                
                // This is a placeholder for health checks
                // In the full implementation, we'd check each provider
                info!("💓 Health check - {} providers active", providers.len());
                
                // TODO: Implement actual health checks and publish health events
            }
        });
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
        msg: async_nats::Message
    ) -> Result<(), LlmAdapterError> {
        let start_time = Instant::now();
        
        // Deserialize request
        let request: LlmRequest = serde_json::from_slice(&msg.payload)
            .map_err(|e| LlmAdapterError::Deserialization(e.to_string()))?;
        
        // Clone request_id for later use
        let request_id = request.request_id.clone();
        
        info!("📥 Processing LLM request: {} (provider: {})", request_id, request.provider);
        
        // Record request event
        let request_event = LlmEvent::completion_requested(
            request.context.session_id.clone(),
            request.provider.clone(),
            "unknown".to_string(), // We'll get the actual model from provider
            request.messages.len(),
        );
        self.publish_event(request_event).await?;
        
        // Get or create dialog context
        let mut context = self.dialog_manager
            .get_or_create_context(&request.context.session_id)
            .await?;
        
        // Convert request messages to provider format
        let provider_messages: Vec<Message> = request.messages
            .iter()
            .map(|msg| Message {
                role: msg.get("role").and_then(|r| r.as_str()).unwrap_or("user").to_string(),
                content: msg.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string(),
                metadata: None,
            })
            .collect();
        
        // Add user message to context
        if let Some(last_message) = provider_messages.last() {
            if last_message.role == "user" {
                context.add_message(
                    last_message.role.clone(),
                    last_message.content.clone(),
                    Some(request.provider.clone()),
                );
            }
        }
        
        // Get provider
        let provider = self.providers.get(&request.provider)
            .ok_or_else(|| LlmAdapterError::Provider(
                format!("Provider '{}' not found", request.provider)
            ))?;
        
        // Parse completion options
        let options = if let Some(opts) = request.options {
            serde_json::from_value(opts).ok()
        } else {
            None
        };
        
        // Call provider
        let result = provider.complete(provider_messages, options).await;
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        let response = match result {
            Ok(provider_response) => {
                // Add assistant response to context
                context.add_message(
                    "assistant".to_string(),
                    provider_response.content.clone(),
                    Some(request.provider.clone()),
                );
                
                // Record success event
                let success_event = LlmEvent::completion_completed(
                    request.context.session_id.clone(),
                    request.provider.clone(),
                    provider_response.model_used.clone(),
                    provider_response.token_count.as_ref().map(|tc| tc.total_tokens),
                    duration_ms,
                );
                self.publish_event(success_event).await?;
                
                LlmResponse {
                    request_id: request.request_id,
                    response: provider_response.content,
                    provider_used: request.provider,
                    token_count: provider_response.token_count.as_ref().map(|tc| tc.total_tokens),
                    updated_context: context.clone(),
                }
            }
            Err(e) => {
                // Record failure event
                let failure_event = LlmEvent::completion_failed(
                    request.context.session_id.clone(),
                    request.provider.clone(),
                    e.to_string(),
                    duration_ms,
                );
                self.publish_event(failure_event).await?;
                
                LlmResponse {
                    request_id: request.request_id,
                    response: format!("Error: {}", e),
                    provider_used: request.provider,
                    token_count: None,
                    updated_context: context.clone(),
                }
            }
        };
        
        // Save updated context
        self.dialog_manager.save_context(&context).await?;
        
        // Publish response
        let response_subject = self.build_response_subject(&request_id);
        let response_json = serde_json::to_vec(&response)
            .map_err(|e| LlmAdapterError::Serialization(e.to_string()))?;
        
        self.nats_client.publish(response_subject, response_json.into()).await
            .map_err(|e| LlmAdapterError::NatsConnection(e.to_string()))?;
        
        info!("✅ LLM request completed: {} ({}ms)", request_id, duration_ms);
        
        Ok(())
    }
    
    /// Publish event to NATS
    async fn publish_event(&self, event: LlmEvent) -> Result<(), LlmAdapterError> {
        let event_subject = if let Some(ref domain) = self.domain {
            format!("{}.events.llm.{}", domain, event.event_type.to_lowercase())
        } else {
            format!("events.llm.{}", event.event_type.to_lowercase())
        };
        
        let event_json = serde_json::to_vec(&event)
            .map_err(|e| LlmAdapterError::Serialization(e.to_string()))?;
        
        self.nats_client.publish(event_subject, event_json.into()).await
            .map_err(|e| LlmAdapterError::NatsConnection(e.to_string()))?;
        
        Ok(())
    }
    
    /// Build request subject using cim-subject patterns
    fn build_request_subject(&self) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.llm.commands.request", domain)
        } else {
            "cim.llm.commands.request".to_string()
        }
    }
    
    /// Build response subject for specific request
    fn build_response_subject(&self, request_id: &str) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.llm.events.response.{}", domain, request_id)
        } else {
            format!("cim.llm.events.response.{}", request_id)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();
    
    info!("🤖 LLM Adapter Service Starting...");
    
    // Create configuration from environment
    let config = LlmAdapterConfig::from_env();
    
    info!("Service Configuration:");
    info!("  NATS URL: {}", config.service.nats_url);
    info!("  Domain: {:?}", config.service.domain);
    info!("  Available providers: {:?}", config.available_providers());
    info!("  Default provider: {}", config.default_provider);
    
    // Create and start service
    let mut service = LlmAdapterService::new(config).await
        .map_err(|e| anyhow::anyhow!("Failed to create service: {}", e))?;
    
    // Handle shutdown signals gracefully
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("🛑 LLM Adapter Service received shutdown signal");
    };
    
    // Run service until shutdown
    tokio::select! {
        result = service.start_service() => {
            if let Err(e) = result {
                error!("LLM Adapter Service error: {}", e);
                std::process::exit(1);
            }
        }
        _ = shutdown_signal => {
            info!("🤖 LLM Adapter Service shutting down gracefully...");
        }
    }
    
    info!("✅ LLM Adapter Service stopped");
    Ok(())
}