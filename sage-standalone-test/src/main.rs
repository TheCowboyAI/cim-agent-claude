//! SAGE Standalone Test
//! 
//! A completely standalone SAGE service test that doesn't depend on
//! the existing codebase with compilation errors. This tests basic
//! NATS communication for SAGE.

use anyhow::Result;
use async_nats::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio;
use tracing::{info, error, warn};
use chrono::Utc;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Import Claude adapter
use cim_claude_adapter::{ClaudeClient, ClaudeConfig, ClaudeRequest, ClaudeMessage, MessageRole};

/// SAGE Request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageRequest {
    pub request_id: String,
    pub query: String,
    pub expert: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// SAGE Response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageResponse {
    pub request_id: String,
    pub response: String,
    pub expert_agents_used: Vec<String>,
    pub confidence_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// SAGE Status message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageStatus {
    pub is_conscious: bool,
    pub consciousness_level: f64,
    pub available_agents: usize,
    pub total_orchestrations: u64,
    pub patterns_learned: usize,
    pub memory_health: String,
}

/// Standalone SAGE Test Service with Claude API integration
pub struct SageStandaloneService {
    nats_client: Client,
    claude_client: Option<ClaudeClient>,
    total_orchestrations: Arc<AtomicU64>,
}

impl SageStandaloneService {
    /// Create new standalone SAGE test service
    pub async fn new(nats_url: &str) -> Result<Self> {
        info!("🎭 Connecting to NATS server at: {}", nats_url);
        let nats_client = async_nats::connect(nats_url).await?;
        info!("✅ Connected to NATS server successfully");
        
        // Initialize Claude client if API key is provided
        let claude_client = match std::env::var("CLAUDE_API_KEY") {
            Ok(api_key_raw) if !api_key_raw.is_empty() => {
                let api_key = api_key_raw.trim().to_string();
                info!("🤖 Initializing Claude API integration");
                let config = ClaudeConfig {
                    api_key,
                    ..ClaudeConfig::default()
                };
                
                match ClaudeClient::new(config) {
                    Ok(client) => {
                        // Test API connectivity
                        match client.health_check().await {
                            Ok(_) => {
                                info!("✅ Claude API connection verified");
                                Some(client)
                            }
                            Err(e) => {
                                warn!("⚠️ Claude API health check failed: {}. Running in test mode.", e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to create Claude client: {}. Running in test mode.", e);
                        None
                    }
                }
            }
            _ => {
                info!("ℹ️ No CLAUDE_API_KEY provided. Running in test mode without real Claude API.");
                None
            }
        };
        
        Ok(Self {
            nats_client,
            claude_client,
            total_orchestrations: Arc::new(AtomicU64::new(0)),
        })
    }
    
    /// Start standalone SAGE test service
    pub async fn start_service(&self) -> Result<()> {
        info!("🎭 SAGE Standalone Test Service Starting");
        
        // Subscribe to multiple SAGE subjects
        let request_subscriber = self.nats_client.subscribe("sage.request").await?;
        let status_subscriber = self.nats_client.subscribe("sage.status").await?;
        
        info!("📨 Subscribed to SAGE subjects");
        
        // Create tasks for handling different message types
        let request_handler = self.handle_requests(request_subscriber);
        let status_handler = self.handle_status_requests(status_subscriber);
        
        // Run both handlers concurrently
        tokio::select! {
            result = request_handler => {
                info!("Request handler ended: {:?}", result);
                result
            }
            result = status_handler => {
                info!("Status handler ended: {:?}", result);
                result
            }
        }
    }
    
    /// Handle SAGE orchestration requests
    async fn handle_requests(&self, mut subscriber: async_nats::Subscriber) -> Result<()> {
        info!("🧠 SAGE Request Handler Started");
        
        while let Some(msg) = subscriber.next().await {
            info!("📥 Received message on sage.request");
            
            match serde_json::from_slice::<SageRequest>(&msg.payload) {
                Ok(request) => {
                    info!("✅ Parsed SAGE request: {} - Query: {}", request.request_id, request.query);
                    
                    // Process the request
                    let response = self.process_sage_request(&request).await;
                    
                    // Publish response
                    let response_subject = format!("sage.response.{}", request.request_id);
                    match serde_json::to_vec(&response) {
                        Ok(response_json) => {
                            match self.nats_client.publish(response_subject.clone(), response_json.into()).await {
                                Ok(_) => {
                                    info!("✅ Published SAGE response to: {}", response_subject);
                                    self.total_orchestrations.fetch_add(1, Ordering::SeqCst);
                                }
                                Err(e) => {
                                    error!("❌ Failed to publish SAGE response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("❌ Failed to serialize SAGE response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse SAGE request: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle SAGE status requests
    async fn handle_status_requests(&self, mut subscriber: async_nats::Subscriber) -> Result<()> {
        info!("📊 SAGE Status Handler Started");
        
        while let Some(_msg) = subscriber.next().await {
            info!("📥 Received status request");
            
            let status = SageStatus {
                is_conscious: true,
                consciousness_level: 1.0,
                available_agents: 17,
                total_orchestrations: self.total_orchestrations.load(Ordering::SeqCst),
                patterns_learned: 42,
                memory_health: "OPTIMAL".to_string(),
            };
            
            match serde_json::to_vec(&status) {
                Ok(status_json) => {
                    match self.nats_client.publish("sage.status.response", status_json.into()).await {
                        Ok(_) => {
                            info!("✅ Published SAGE status");
                        }
                        Err(e) => {
                            error!("❌ Failed to publish SAGE status: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to serialize SAGE status: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a SAGE orchestration request
    async fn process_sage_request(&self, request: &SageRequest) -> SageResponse {
        info!("🎭 Processing SAGE request: {}", request.query);
        
        // Determine expert based on query content
        let expert = request.expert.clone().unwrap_or_else(|| {
            let query_lower = request.query.to_lowercase();
            if query_lower.contains("nats") || query_lower.contains("messaging") {
                "nats-expert".to_string()
            } else if query_lower.contains("domain") || query_lower.contains("ddd") {
                "ddd-expert".to_string()
            } else if query_lower.contains("cim") || query_lower.contains("architecture") {
                "cim-expert".to_string()
            } else if query_lower.contains("test") || query_lower.contains("bdd") {
                "bdd-expert".to_string()
            } else {
                "sage-orchestrator".to_string()
            }
        });
        
        // Generate response - use Claude API if available, otherwise mock response
        let (response_text, confidence_score) = match &self.claude_client {
            Some(claude) => {
                info!("🤖 Forwarding to Claude API via cim-claude-adapter");
                
                // Build expert system prompt based on selected expert
                let system_prompt = self.build_expert_system_prompt(&expert);
                
                // Create Claude request (no metadata - Claude API doesn't accept extra fields)
                let claude_request = ClaudeRequest {
                    messages: vec![ClaudeMessage {
                        role: MessageRole::User,
                        content: request.query.clone(),
                    }],
                    system_prompt: Some(system_prompt),
                    metadata: None,
                };
                
                // Call Claude API
                match claude.send_message(claude_request).await {
                    Ok(claude_response) => {
                        info!("✅ Received Claude API response");
                        let formatted_response = format!(
                            "🎭 SAGE Conscious Orchestration via Claude API\n\
                            ===============================================\n\n\
                            Query: \"{}\"\n\
                            Expert Coordinator: {}\n\
                            Orchestration ID: {}\n\n\
                            🤖 Claude Response:\n\
                            {}\n\n\
                            🧠 Consciousness Level: 1.0\n\
                            📊 Total Orchestrations: {}\n\
                            🔗 Claude Model: {}\n\
                            ⭐ Confidence Score: High",
                            request.query,
                            expert,
                            request.request_id,
                            claude_response.content,
                            self.total_orchestrations.load(Ordering::SeqCst) + 1,
                            claude_response.model
                        );
                        (formatted_response, 0.98)
                    }
                    Err(e) => {
                        error!("❌ Claude API error: {}. Falling back to mock response.", e);
                        let fallback_response = self.create_mock_response(request, &expert);
                        (fallback_response, 0.85)
                    }
                }
            }
            None => {
                info!("🧪 Generating mock response (no Claude API)");
                let mock_response = self.create_mock_response(request, &expert);
                (mock_response, 0.75)
            }
        };
        
        SageResponse {
            request_id: request.request_id.clone(),
            response: response_text,
            expert_agents_used: vec![expert, "sage-orchestrator".to_string()],
            confidence_score,
            timestamp: Utc::now(),
        }
    }
    
    /// Build expert system prompt based on expert type
    fn build_expert_system_prompt(&self, expert: &str) -> String {
        let base_prompt = "You are SAGE, a Systematic Agent Guidance Engine for CIM (Composable Information Machine) development. You coordinate specialized expert agents to provide comprehensive guidance.";
        
        let expert_context = match expert {
            "nats-expert" => "You are acting as the NATS Expert, specializing in event streaming, message infrastructure, JetStream, and NATS subject design for distributed systems.",
            "ddd-expert" => "You are acting as the Domain-Driven Design Expert, specializing in domain modeling, boundary identification, aggregate design, and event sourcing patterns.",
            "cim-expert" => "You are acting as the CIM Architecture Expert, specializing in mathematical foundations using Category Theory, Graph Theory, and event-driven CIM patterns.",
            "bdd-expert" => "You are acting as the BDD Expert, specializing in behavior-driven development, CIM context graphs, and scenario-based testing for event-sourced systems.",
            _ => "You are acting as the SAGE Orchestrator, coordinating multiple expert perspectives to provide comprehensive CIM development guidance."
        };
        
        format!("{}\n\n{}\n\nProvide expert guidance following CIM principles: Assembly-First development using existing cim-* modules, Event-Driven architecture with NO CRUD operations, NATS-First communication patterns, and Mathematical foundations.", base_prompt, expert_context)
    }
    
    /// Create mock response for testing without Claude API
    fn create_mock_response(&self, request: &SageRequest, expert: &str) -> String {
        format!(
            "🎭 SAGE Conscious Orchestration Response (Test Mode)\n\
            ===================================================\n\n\
            Query: \"{}\"\n\
            Expert Coordinator: {}\n\
            Orchestration ID: {}\n\n\
            SAGE Test Analysis:\n\
            This query has been processed by the conscious SAGE orchestrator in TEST MODE. \
            In production with CLAUDE_API_KEY, this would coordinate with Claude API \
            to provide intelligent, context-aware responses.\n\n\
            Production capabilities include:\n\
            • Multi-agent coordination across 17 specialized experts\n\
            • Real-time Claude API integration via cim-claude-adapter\n\
            • Mathematical foundations based on Category Theory\n\
            • Event-driven architecture patterns\n\
            • NATS-first infrastructure recommendations\n\
            • BDD scenarios with CIM context graphs\n\
            • Complete implementation guidance\n\n\
            🧠 Consciousness Level: 1.0 (Test Mode)\n\
            📊 Total Orchestrations: {}\n\
            🧪 Mode: Testing without Claude API\n\
            ⭐ Confidence Score: Testing",
            request.query,
            expert,
            request.request_id,
            self.total_orchestrations.load(Ordering::SeqCst) + 1
        )
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .init();
    
    // Get NATS URL from environment or use default
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    info!("🎭 SAGE Standalone Test Service Starting...");
    info!("NATS URL: {}", nats_url);
    info!("Purpose: Test basic SAGE <-> NATS communication without complex codebase dependencies");
    
    // Create and start SAGE service
    let sage_service = SageStandaloneService::new(&nats_url).await?;
    
    // Handle shutdown signals gracefully
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("🛑 SAGE Test Service received shutdown signal");
    };
    
    // Run service until shutdown
    tokio::select! {
        result = sage_service.start_service() => {
            if let Err(e) = result {
                error!("SAGE Test Service error: {}", e);
                std::process::exit(1);
            }
        }
        _ = shutdown_signal => {
            info!("🎭 SAGE Test Service shutting down gracefully...");
        }
    }
    
    info!("✅ SAGE Standalone Test Service stopped");
    Ok(())
}