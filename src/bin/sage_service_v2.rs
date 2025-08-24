//! SAGE NATS Service V2 - Using LLM Adapter
//! 
//! An updated SAGE service that uses the cim-llm-adapter service
//! instead of direct Claude API calls. This enables:
//! - Agent personality switching
//! - Multi-provider support
//! - Better context management
//! - Event-driven architecture

use anyhow::Result;
use async_nats::{Client, jetstream};
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use tokio;
use tracing::{info, error};
use uuid::Uuid;
use chrono::Utc;

// Import the LLM client and agent loader from our crate
use cim_agent_claude::sage_llm_client::SageLlmClient;
use cim_agent_claude::agent_loader::{AgentLoader, AgentSelector, AgentPersonality};

/// SAGE Request message sent via NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageRequest {
    pub request_id: String,
    pub query: String,
    pub expert: Option<String>,
    pub context: SageContext,
}

/// SAGE Context for maintaining conversation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageContext {
    pub session_id: Option<String>,
    pub conversation_history: Vec<ConversationEntry>,
    pub project_context: Option<ProjectContext>,
}

/// Conversation entry for maintaining history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub role: String, // "user" or "sage"
    pub content: String,
    pub expert_agents: Vec<String>,
}

/// Project context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_type: String,
    pub domains: Vec<String>,
    pub current_phase: String,
}

/// SAGE Response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageResponse {
    pub request_id: String,
    pub response: String,
    pub expert_agents_used: Vec<String>,
    pub confidence: f32,
    pub context: SageContext,
}


/// SAGE Service V2 - Uses LLM Adapter
pub struct SageServiceV2 {
    nats_client: Client,
    jetstream: jetstream::Context,
    llm_client: SageLlmClient,
    agent_selector: AgentSelector,
    domain: Option<String>,
}

impl SageServiceV2 {
    /// Create new SAGE service instance
    pub async fn new(nats_url: &str) -> Result<Self> {
        let nats_client = async_nats::connect(nats_url).await?;
        let jetstream = jetstream::new(nats_client.clone());
        
        // Detect domain from environment or hostname
        let domain = Self::detect_domain();
        
        // Create LLM client for adapter communication
        let llm_client = SageLlmClient::new(nats_client.clone(), domain.clone());
        
        // Load expert agents from markdown files
        let mut agent_loader = AgentLoader::new(".claude/agents");
        let loaded_agents = agent_loader.load_all_agents().await?;
        let agent_selector = AgentSelector::new(agent_loader);
        
        info!("🧠 SAGE V2 Consciousness Initialized (using LLM Adapter)");
        info!("📚 Loaded {} expert agents from markdown files", loaded_agents.len());
        for agent in &loaded_agents {
            info!("  - {}: {}", agent.id, agent.metadata.description);
        }
        
        Ok(Self {
            nats_client,
            jetstream,
            llm_client,
            agent_selector,
            domain,
        })
    }
    
    /// Detect domain from environment or hostname
    fn detect_domain() -> Option<String> {
        if let Ok(domain) = std::env::var("CIM_DOMAIN") {
            return Some(domain);
        }
        
        hostname::get()
            .ok()
            .and_then(|h| h.to_str().map(|s| s.to_string()))
    }
    
    
    /// Start the SAGE service
    pub async fn start(&self) -> Result<()> {
        info!("🚀 SAGE V2 Service Starting");
        
        // Subscribe to SAGE requests
        let request_subject = self.build_request_subject();
        info!("📨 Subscribing to: {}", request_subject);
        
        let mut subscriber = self.nats_client.subscribe(request_subject).await?;
        
        // Process requests
        while let Some(msg) = subscriber.next().await {
            if let Err(e) = self.handle_request(msg).await {
                error!("Error handling request: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Handle a SAGE request
    async fn handle_request(&self, msg: async_nats::Message) -> Result<()> {
        let request: SageRequest = serde_json::from_slice(&msg.payload)?;
        info!("🎭 SAGE processing request: {}", request.request_id);
        
        // Select appropriate agent based on query
        let selected_agent = self.agent_selector.select_agent(&request.query, request.expert.as_deref());
        let agent_id = selected_agent.map(|a| a.id.as_str()).unwrap_or("sage");
        info!("👥 Using agent: {}", agent_id);
        
        // Get or create session ID
        let session_id = request.context.session_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Build conversation context
        let mut messages = Vec::new();
        
        // Add conversation history if available
        for entry in &request.context.conversation_history {
            messages.push(serde_json::json!({
                "role": entry.role,
                "content": entry.content
            }));
        }
        
        // Add current query
        messages.push(serde_json::json!({
            "role": "user",
            "content": request.query.clone()
        }));
        
        // Get system prompt from selected agent
        let system_prompt = selected_agent
            .map(|a| a.system_prompt.as_str())
            .unwrap_or("You are an AI assistant helping with software development.");
        
        let response_text = match self.llm_client
            .send_llm_request(messages, session_id.clone(), Some(system_prompt))
            .await {
            Ok(text) => text,
            Err(e) => {
                error!("LLM request failed: {}", e);
                format!("I apologize, but I encountered an error processing your request: {}", e)
            }
        };
        
        // Update conversation history
        let mut updated_context = request.context.clone();
        updated_context.conversation_history.push(ConversationEntry {
            timestamp: Utc::now(),
            role: "user".to_string(),
            content: request.query.clone(),
            expert_agents: vec![],
        });
        updated_context.conversation_history.push(ConversationEntry {
            timestamp: Utc::now(),
            role: "sage".to_string(),
            content: response_text.clone(),
            expert_agents: vec![agent_id.to_string()],
        });
        updated_context.session_id = Some(session_id);
        
        // Create response
        let response = SageResponse {
            request_id: request.request_id.clone(),
            response: response_text,
            expert_agents_used: vec![agent_id.to_string()],
            confidence: 0.95, // TODO: Calculate actual confidence
            context: updated_context,
        };
        
        // Send response
        let response_subject = self.build_response_subject(&request.request_id);
        let response_bytes = serde_json::to_vec(&response)?;
        self.nats_client.publish(response_subject, response_bytes.into()).await?;
        
        info!("✅ SAGE request completed: {}", request.request_id);
        Ok(())
    }
    
    
    /// Build request subject
    fn build_request_subject(&self) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.commands.sage.request", domain)
        } else {
            "commands.sage.request".to_string()
        }
    }
    
    /// Build response subject
    fn build_response_subject(&self, request_id: &str) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.events.sage.response.{}", domain, request_id)
        } else {
            format!("events.sage.response.{}", request_id)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Get NATS URL from environment or use default
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    info!("🎭 SAGE V2 Service Starting (LLM Adapter Mode)...");
    info!("NATS URL: {}", nats_url);
    info!("LLM Adapter: Using cim.llm.commands.request");
    
    // Create and start SAGE service
    let sage_service = SageServiceV2::new(&nats_url).await?;
    
    // Handle shutdown signals gracefully
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("🛑 SAGE V2 Service received shutdown signal");
    };
    
    // Run service until shutdown
    tokio::select! {
        result = sage_service.start() => {
            if let Err(e) = result {
                error!("SAGE service error: {}", e);
            }
        }
        _ = shutdown_signal => {
            info!("Shutting down SAGE V2 service...");
        }
    }
    
    info!("👋 SAGE V2 Service stopped");
    Ok(())
}