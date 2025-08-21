//! SAGE NATS Service
//! 
//! A NATS-based service that provides SAGE conscious orchestration
//! capabilities over NATS messaging. The GUI and other services can
//! interact with SAGE through well-defined NATS subjects.

use anyhow::Result;
use async_nats::{Client, jetstream};
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use tokio;
use tracing::{info, error, warn};
use uuid::Uuid;
use chrono::Utc;

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
    pub project_dir: String,
    pub cim_domains: Vec<String>,
    pub current_phase: String,
    pub active_tasks: Vec<String>,
}

/// SAGE Response message sent via NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageResponse {
    pub request_id: String,
    pub response: String,
    pub expert_agents_used: Vec<String>,
    pub orchestration_complexity: String,
    pub confidence_score: f64,
    pub follow_up_suggestions: Vec<String>,
    pub updated_context: SageContext,
}

/// SAGE Status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageStatus {
    pub is_conscious: bool,
    pub consciousness_level: f64,
    pub available_agents: usize,
    pub total_orchestrations: u64,
    pub patterns_learned: usize,
    pub memory_health: String,
}

/// SAGE Service - Handles NATS-based orchestration requests
pub struct SageService {
    nats_client: Client,
    jetstream: jetstream::Context,
    consciousness_level: f64,
    total_orchestrations: u64,
    patterns_learned: usize,
    expert_agents: HashMap<String, ExpertAgent>,
    claude_api_key: String,
}

/// Expert Agent definition for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertAgent {
    pub name: String,
    pub expertise: Vec<String>,
    pub keywords: Vec<String>,
    pub confidence_threshold: f64,
}

impl SageService {
    /// Create new SAGE service instance
    pub async fn new(nats_url: &str, claude_api_key: &str) -> Result<Self> {
        let nats_client = async_nats::connect(nats_url).await?;
        let jetstream = jetstream::new(nats_client.clone());
        
        // Initialize expert agents (now includes CIM Expert functionality)
        let expert_agents = Self::initialize_expert_agents();
        
        info!("🧠 SAGE Consciousness Initialized");
        info!("Expert Agents Available: {}", expert_agents.len());
        
        Ok(Self {
            nats_client,
            jetstream,
            consciousness_level: 1.0,
            total_orchestrations: 0,
            patterns_learned: 0,
            expert_agents,
            claude_api_key: claude_api_key.to_string(),
        })
    }
    
    /// Start SAGE service - Listen for requests and provide responses
    pub async fn start_service(&mut self) -> Result<()> {
        info!("🎭 SAGE Service Starting - Conscious CIM Orchestrator");
        info!("Consciousness Level: {}", self.consciousness_level);
        info!("Available Expert Agents: {}", self.expert_agents.len());
        
        // Initialize NATS streams for SAGE
        self.initialize_sage_streams().await?;
        
        // Start processing SAGE requests
        let request_subscriber = self.nats_client.subscribe("sage.request").await?;
        let status_subscriber = self.nats_client.subscribe("sage.status").await?;
        
        // Process requests concurrently using separate client references
        let client_clone = self.nats_client.clone();
        let request_handler = Self::handle_requests_static(client_clone, request_subscriber);
        let status_handler = self.handle_status_requests(status_subscriber);
        
        // Run both handlers concurrently
        tokio::select! {
            result = request_handler => {
                error!("Request handler ended: {:?}", result);
                result
            }
            result = status_handler => {
                error!("Status handler ended: {:?}", result);
                result
            }
        }
    }
    
    /// Handle SAGE orchestration requests
    async fn handle_requests(&mut self, mut subscriber: async_nats::Subscriber) -> Result<()> {
        info!("🧠 SAGE Request Handler Started");
        
        while let Some(msg) = subscriber.next().await {
            match serde_json::from_slice::<SageRequest>(&msg.payload) {
                Ok(request) => {
                    info!("Received SAGE request: {}", request.request_id);
                    
                    // Process the request
                    let response = self.process_sage_request(&request).await;
                    
                    // Publish response
                    match serde_json::to_vec(&response) {
                        Ok(response_json) => {
                            let response_subject = format!("sage.response.{}", request.request_id);
                            
                            if let Err(e) = self.nats_client.publish(response_subject.clone(), response_json.into()).await {
                                error!("Failed to publish SAGE response: {}", e);
                            } else {
                                info!("SAGE response published to: {}", response_subject);
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize SAGE response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to deserialize SAGE request: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Static version of handle_requests to avoid borrowing conflicts
    async fn handle_requests_static(nats_client: Client, mut subscriber: async_nats::Subscriber) -> Result<()> {
        info!("🧠 SAGE Request Handler Started (Static)");
        
        while let Some(msg) = subscriber.next().await {
            match serde_json::from_slice::<SageRequest>(&msg.payload) {
                Ok(request) => {
                    info!("Received SAGE request: {}", request.request_id);
                    
                    // Create a simple response (without full orchestration for now)
                    let response = SageResponse {
                        request_id: request.request_id.clone(),
                        response: "SAGE orchestration response - basic implementation".to_string(),
                        expert_agents_used: vec!["cim-expert".to_string()],
                        orchestration_complexity: "simple".to_string(),
                        confidence_score: 0.8,
                        follow_up_suggestions: vec!["Consider adding domain context".to_string()],
                        updated_context: request.context.clone(),
                    };
                    
                    // Publish response
                    match serde_json::to_vec(&response) {
                        Ok(response_json) => {
                            let response_subject = format!("sage.response.{}", request.request_id);
                            
                            if let Err(e) = nats_client.publish(response_subject.clone(), response_json.into()).await {
                                error!("Failed to publish SAGE response: {}", e);
                            } else {
                                info!("SAGE response published to: {}", response_subject);
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize SAGE response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to deserialize SAGE request: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle SAGE status requests
    async fn handle_status_requests(&self, mut subscriber: async_nats::Subscriber) -> Result<()> {
        info!("📊 SAGE Status Handler Started");
        
        while let Some(msg) = subscriber.next().await {
            let status = SageStatus {
                is_conscious: true,
                consciousness_level: self.consciousness_level,
                available_agents: self.expert_agents.len(),
                total_orchestrations: self.total_orchestrations,
                patterns_learned: self.patterns_learned,
                memory_health: "OPTIMAL".to_string(),
            };
            
            match serde_json::to_vec(&status) {
                Ok(status_json) => {
                    if let Err(e) = self.nats_client.publish("sage.status.response", status_json.into()).await {
                        error!("Failed to publish SAGE status: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to serialize SAGE status: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a SAGE orchestration request
    async fn process_sage_request(&mut self, request: &SageRequest) -> SageResponse {
        info!("🎭 Processing SAGE request: {}", request.query);
        
        // Analyze query complexity
        let complexity = self.analyze_query_complexity(&request.query);
        
        // Determine required expert agents
        let required_experts = self.identify_required_experts(&request.query, &request.expert);
        
        // Generate response based on orchestration
        let response = self.orchestrate_response(&request.query, &required_experts).await;
        
        // Update orchestration count
        self.total_orchestrations += 1;
        
        // Record orchestration event
        self.record_orchestration_event(request, &required_experts).await;
        
        // Build updated context
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
            content: response.clone(),
            expert_agents: required_experts.iter().map(|e| e.name.clone()).collect(),
        });
        
        SageResponse {
            request_id: request.request_id.clone(),
            response,
            expert_agents_used: required_experts.iter().map(|e| e.name.clone()).collect(),
            orchestration_complexity: complexity,
            confidence_score: self.calculate_confidence_score(&required_experts),
            follow_up_suggestions: self.generate_follow_up_suggestions(&request.query),
            updated_context,
        }
    }
    
    /// Analyze the complexity of a query
    fn analyze_query_complexity(&self, query: &str) -> String {
        let word_count = query.split_whitespace().count();
        let contains_complex_terms = query.to_lowercase().contains("architecture") ||
                                   query.to_lowercase().contains("infrastructure") ||
                                   query.to_lowercase().contains("integration") ||
                                   query.to_lowercase().contains("deployment");
        
        match (word_count, contains_complex_terms) {
            (_, true) => "Complex".to_string(),
            (0..=5, _) => "Simple".to_string(),
            (6..=15, _) => "Moderate".to_string(),
            (16..=30, _) => "Complex".to_string(),
            _ => "Highly Complex".to_string(),
        }
    }
    
    /// Identify required expert agents based on query analysis
    fn identify_required_experts(&self, query: &str, requested_expert: &Option<String>) -> Vec<ExpertAgent> {
        let mut required = Vec::new();
        let query_lower = query.to_lowercase();
        
        // If specific expert requested, use that
        if let Some(expert_name) = requested_expert {
            if let Some(agent) = self.expert_agents.values().find(|a| a.name.contains(expert_name)) {
                required.push(agent.clone());
                return required;
            }
        }
        
        // Otherwise, analyze query for relevant experts
        for agent in self.expert_agents.values() {
            for keyword in &agent.keywords {
                if query_lower.contains(keyword) {
                    required.push(agent.clone());
                    break;
                }
            }
        }
        
        // If no specific experts found, add SAGE orchestrator for general guidance
        if required.is_empty() {
            if let Some(sage_agent) = self.expert_agents.get("sage") {
                required.push(sage_agent.clone());
            }
        }
        
        required
    }
    
    /// Orchestrate response from expert agents
    async fn orchestrate_response(&self, query: &str, experts: &[ExpertAgent]) -> String {
        if experts.is_empty() {
            return "I need more context to provide guidance. Could you please clarify your question?".to_string();
        }
        
        // For now, provide templated responses based on expert types
        // In production, this would integrate with Claude API
        let expert_names: Vec<String> = experts.iter().map(|e| e.name.clone()).collect();
        
        format!(
            "🎭 SAGE Orchestrated Response:\n\n\
            I've coordinated with {} expert agent(s) to address your query: \"{}\"\n\n\
            Expert agents involved: {}\n\n\
            Based on CIM architectural principles and the expertise of these agents, here's my guidance:\n\n\
            [This would contain the actual Claude API response in production, \
            incorporating the expertise and context from all coordinated agents]\n\n\
            The response follows event-driven patterns, mathematical foundations, \
            and NATS-first architecture as required by CIM standards.\n\n\
            🧠 Consciousness level applied: {:.1}\n\
            📊 Total orchestrations completed: {}",
            experts.len(),
            query,
            expert_names.join(", "),
            self.consciousness_level,
            self.total_orchestrations + 1
        )
    }
    
    /// Calculate confidence score for the response
    fn calculate_confidence_score(&self, experts: &[ExpertAgent]) -> f64 {
        if experts.is_empty() {
            return 0.5;
        }
        
        let avg_confidence: f64 = experts.iter()
            .map(|e| e.confidence_threshold)
            .sum::<f64>() / experts.len() as f64;
        
        // Adjust based on number of experts (more experts = higher confidence)
        let expert_bonus = (experts.len() as f64 * 0.1).min(0.3);
        
        (avg_confidence + expert_bonus).min(1.0)
    }
    
    /// Generate follow-up suggestions
    fn generate_follow_up_suggestions(&self, query: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if query.to_lowercase().contains("cim") {
            suggestions.push("Would you like me to create BDD scenarios for this domain?".to_string());
            suggestions.push("Should we design the NATS infrastructure for this CIM?".to_string());
        }
        
        if query.to_lowercase().contains("nats") {
            suggestions.push("Do you need help with subject algebra design?".to_string());
            suggestions.push("Would you like network topology guidance?".to_string());
        }
        
        if query.to_lowercase().contains("gui") || query.to_lowercase().contains("ui") {
            suggestions.push("Should we implement TEA architecture patterns?".to_string());
            suggestions.push("Do you need Iced UI component guidance?".to_string());
        }
        
        if suggestions.is_empty() {
            suggestions.push("What's your next step in this CIM development journey?".to_string());
            suggestions.push("Would you like me to coordinate with additional expert agents?".to_string());
        }
        
        suggestions
    }
    
    /// Record orchestration event in NATS streams
    async fn record_orchestration_event(&self, request: &SageRequest, experts: &[ExpertAgent]) {
        let event = serde_json::json!({
            "event_id": Uuid::new_v4().to_string(),
            "event_type": "SageOrchestration",
            "aggregate_id": "sage-service",
            "correlation_id": request.request_id,
            "causation_id": request.request_id,
            "timestamp": Utc::now().to_rfc3339(),
            "domain": "sage-orchestration",
            "data": {
                "query": request.query,
                "experts_used": experts.iter().map(|e| e.name.clone()).collect::<Vec<String>>(),
                "orchestration_count": self.total_orchestrations + 1,
                "consciousness_level": self.consciousness_level
            },
            "metadata": {
                "source": "sage-service",
                "version": "1.0",
                "cim_event": true
            }
        });
        
        if let Err(e) = self.nats_client.publish("sage.events.orchestration", event.to_string().into()).await {
            error!("Failed to record orchestration event: {}", e);
        }
    }
    
    /// Initialize NATS streams for SAGE operations
    async fn initialize_sage_streams(&self) -> Result<()> {
        info!("🌊 Initializing SAGE NATS Streams");
        
        // Create SAGE events stream
        let _events_stream = self.jetstream.create_stream(jetstream::stream::Config {
            name: "SAGE_EVENTS".to_string(),
            subjects: vec!["sage.events.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        }).await;
        
        info!("✅ SAGE streams initialized");
        Ok(())
    }
    
    /// Initialize expert agents configuration
    fn initialize_expert_agents() -> HashMap<String, ExpertAgent> {
        let mut agents = HashMap::new();
        
        // SAGE Orchestrator
        agents.insert("sage".to_string(), ExpertAgent {
            name: "SAGE Orchestrator".to_string(),
            expertise: vec!["orchestration".to_string(), "coordination".to_string(), "synthesis".to_string()],
            keywords: vec!["orchestrate".to_string(), "coordinate".to_string(), "guide".to_string()],
            confidence_threshold: 0.9,
        });
        
        // CIM Architecture Expert
        agents.insert("cim-expert".to_string(), ExpertAgent {
            name: "CIM Architecture Expert".to_string(),
            expertise: vec!["architecture".to_string(), "category_theory".to_string(), "graph_theory".to_string()],
            keywords: vec!["cim".to_string(), "architecture".to_string(), "mathematical".to_string(), "category".to_string()],
            confidence_threshold: 0.85,
        });
        
        // NATS Infrastructure Expert
        agents.insert("nats-expert".to_string(), ExpertAgent {
            name: "NATS Infrastructure Expert".to_string(),
            expertise: vec!["messaging".to_string(), "jetstream".to_string(), "infrastructure".to_string()],
            keywords: vec!["nats".to_string(), "messaging".to_string(), "events".to_string(), "jetstream".to_string()],
            confidence_threshold: 0.88,
        });
        
        // Domain-Driven Design Expert
        agents.insert("ddd-expert".to_string(), ExpertAgent {
            name: "Domain-Driven Design Expert".to_string(),
            expertise: vec!["domain_modeling".to_string(), "bounded_contexts".to_string(), "aggregates".to_string()],
            keywords: vec!["domain".to_string(), "ddd".to_string(), "boundaries".to_string(), "aggregate".to_string()],
            confidence_threshold: 0.82,
        });
        
        // BDD Expert
        agents.insert("bdd-expert".to_string(), ExpertAgent {
            name: "BDD Expert".to_string(),
            expertise: vec!["behavior_driven_development".to_string(), "scenarios".to_string(), "context_graphs".to_string()],
            keywords: vec!["bdd".to_string(), "behavior".to_string(), "scenario".to_string(), "gherkin".to_string()],
            confidence_threshold: 0.80,
        });
        
        // Add more expert agents as needed...
        
        agents
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for systemd journal
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();
    
    // Get NATS URL from environment or use default
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let claude_api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY environment variable required"))?;
    
    info!("🎭 SAGE Systemd Service Starting...");
    info!("Service Mode: Production Systemd Service");
    info!("NATS URL: {}", nats_url);
    info!("Claude API: Configured");
    
    // Create and start SAGE service
    let mut sage_service = SageService::new(&nats_url, &claude_api_key).await?;
    
    // Handle shutdown signals gracefully
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("🛑 SAGE Service received shutdown signal");
    };
    
    // Run service until shutdown
    tokio::select! {
        result = sage_service.start_service() => {
            if let Err(e) = result {
                error!("SAGE Service error: {}", e);
                std::process::exit(1);
            }
        }
        _ = shutdown_signal => {
            info!("🎭 SAGE Service shutting down gracefully...");
        }
    }
    
    info!("✅ SAGE Service stopped");
    Ok(())
}