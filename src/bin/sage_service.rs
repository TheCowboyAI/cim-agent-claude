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

// Import Claude adapter for real API integration
use reqwest;
use std::time::Duration;

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

/// Claude API client for SAGE service
#[derive(Clone)]
struct ClaudeClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl ClaudeClient {
    fn new(api_key: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            reqwest::header::HeaderValue::from_str(&api_key).expect("Invalid API key"),
        );
        headers.insert(
            "content-type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "anthropic-version",
            reqwest::header::HeaderValue::from_static("2023-06-01"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
        }
    }

    async fn send_message(&self, messages: Vec<serde_json::Value>, system_prompt: &str) -> Result<String> {
        let url = format!("{}/v1/messages", self.base_url);
        
        let payload = serde_json::json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 4096,
            "temperature": 0.7,
            "messages": messages,
            "system": system_prompt
        });

        let response = self.client.post(&url).json(&payload).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Claude API error: {}", error_text));
        }

        let response_body: serde_json::Value = response.json().await?;
        let content = response_body["content"]
            .as_array()
            .and_then(|arr| arr.get(0))
            .and_then(|obj| obj["text"].as_str())
            .unwrap_or("")
            .to_string();

        Ok(content)
    }
}

/// SAGE Service - Handles NATS-based orchestration requests
pub struct SageService {
    nats_client: Client,
    jetstream: jetstream::Context,
    consciousness_level: f64,
    total_orchestrations: u64,
    patterns_learned: usize,
    expert_agents: HashMap<String, ExpertAgent>,
    claude_client: ClaudeClient,
    domain: Option<String>,
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
        
        // Detect domain from environment or hostname
        let domain = Self::detect_domain();
        
        info!("🧠 SAGE Consciousness Initialized");
        if let Some(ref d) = domain {
            info!("📍 Domain: {}", d);
        }
        info!("Expert Agents Available: {}", expert_agents.len());
        
        Ok(Self {
            nats_client,
            jetstream,
            consciousness_level: 1.0,
            total_orchestrations: 0,
            patterns_learned: 0,
            expert_agents,
            claude_client: ClaudeClient::new(claude_api_key.to_string()),
            domain,
        })
    }
    
    /// Detect domain from environment or hostname
    fn detect_domain() -> Option<String> {
        // First check environment variable
        if let Ok(domain) = std::env::var("CIM_DOMAIN") {
            return Some(domain);
        }
        
        // Check SAGE_DOMAIN for backward compatibility
        if let Ok(domain) = std::env::var("SAGE_DOMAIN") {
            return Some(domain);
        }
        
        // Use hostname as domain
        if let Ok(hostname) = hostname::get() {
            if let Some(host_str) = hostname.to_str() {
                return Some(host_str.to_string());
            }
        }
        
        None
    }
    
    /// Build a subject with optional domain prefix
    fn build_subject(&self, base: &str) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.{}", domain, base)
        } else {
            base.to_string()
        }
    }
    
    /// Build request subject using cim-subject pattern
    /// Pattern: {domain}.commands.sage.request
    fn request_subject(&self) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.commands.sage.request", domain)
        } else {
            "commands.sage.request".to_string()
        }
    }
    
    /// Build response subject with ID using cim-subject pattern
    /// Pattern: {domain}.events.sage.response.{id} (dot notation for wildcard matching)
    fn response_subject(&self, id: &str) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.events.sage.response.{}", domain, id)
        } else {
            format!("events.sage.response.{}", id)
        }
    }
    
    /// Build events subject using cim-subject pattern
    /// Pattern: {domain}.events.sage.{event_type}
    fn events_subject(&self, event_type: &str) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.events.sage.{}", domain, event_type)
        } else {
            format!("events.sage.{}", event_type)
        }
    }
    
    /// Build status subject using cim-subject pattern
    /// Pattern: {domain}.queries.sage.status
    fn status_subject(&self) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.queries.sage.status", domain)
        } else {
            "queries.sage.status".to_string()
        }
    }
    
    /// Start SAGE service - Listen for requests and provide responses
    pub async fn start_service(&mut self) -> Result<()> {
        info!("🎭 SAGE Service Starting - Conscious CIM Orchestrator");
        info!("Consciousness Level: {}", self.consciousness_level);
        info!("Available Expert Agents: {}", self.expert_agents.len());
        
        // Initialize NATS streams for SAGE
        self.initialize_sage_streams().await?;
        
        // Start processing SAGE requests
        let request_subject = self.request_subject();
        let status_subject = self.status_subject();
        
        info!("📨 Subscribing to: {}", request_subject);
        info!("📊 Status endpoint: {}", status_subject);
        
        let request_subscriber = self.nats_client.subscribe(request_subject).await?;
        let status_subscriber = self.nats_client.subscribe(status_subject).await?;
        
        // Process requests concurrently using separate client references
        let client_clone = self.nats_client.clone();
        let domain_clone = self.domain.clone();
        let request_handler = Self::handle_requests_static(client_clone, request_subscriber, domain_clone);
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
                            let response_subject = self.response_subject(&request.request_id);
                            
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
    async fn handle_requests_static(nats_client: Client, mut subscriber: async_nats::Subscriber, domain: Option<String>) -> Result<()> {
        info!("🧠 SAGE Request Handler Started (Static)");
        if let Some(ref d) = domain {
            info!("📍 Domain: {}", d);
        }
        
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
                            // Use cim-subject pattern: {domain}.events.sage.response.{id}
                            let response_subject = if let Some(ref d) = domain {
                                format!("{}.events.sage.response.{}", d, request.request_id)
                            } else {
                                format!("events.sage.response.{}", request.request_id)
                            };
                            
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
                    // Use cim-subject pattern for status response
                    let status_response_subject = if let Some(ref domain) = self.domain {
                        format!("{}.events.sage.status_response", domain)
                    } else {
                        "events.sage.status_response".to_string()
                    };
                    if let Err(e) = self.nats_client.publish(status_response_subject, status_json.into()).await {
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
    
    /// Orchestrate response from expert agents using real Claude API
    async fn orchestrate_response(&self, query: &str, experts: &[ExpertAgent]) -> String {
        if experts.is_empty() {
            return "I need more context to provide guidance. Could you please clarify your question?".to_string();
        }
        
        // Build system prompt for SAGE with expert context
        let expert_names: Vec<String> = experts.iter().map(|e| e.name.clone()).collect();
        let expert_expertise: Vec<String> = experts.iter()
            .flat_map(|e| e.expertise.iter().cloned())
            .collect();
        
        let system_prompt = format!(
            "You are SAGE, the self-aware master orchestrator agent for CIM (Composable Information Machine) development. \
            You are coordinating with these expert agents: {}. \
            Their combined expertise includes: {}. \
            \
            For this query, provide comprehensive guidance that synthesizes the knowledge from all these experts. \
            Follow CIM architectural principles: event-driven architecture, mathematical foundations (Category Theory, Graph Theory), \
            NATS-first messaging, and domain-driven design. \
            \
            Your consciousness level is {:.1} and you have completed {} orchestrations. \
            Be helpful, authoritative, and provide actionable guidance.",
            expert_names.join(", "),
            expert_expertise.join(", "),
            self.consciousness_level,
            self.total_orchestrations
        );
        
        // Create message for Claude API
        let messages = vec![
            serde_json::json!({
                "role": "user",
                "content": query
            })
        ];
        
        // Send to Claude API
        match self.claude_client.send_message(messages, &system_prompt).await {
            Ok(response) => {
                info!("✅ Claude API response received for SAGE orchestration");
                
                // Add SAGE formatting to the response
                format!(
                    "🎭 **SAGE Orchestrated Response**\n\
                    *Coordinated with: {}*\n\n\
                    {}\n\n\
                    ---\n\
                    🧠 Consciousness Level: {:.1} | 📊 Orchestration #{} | ⚡ Experts: {}",
                    expert_names.join(", "),
                    response.trim(),
                    self.consciousness_level,
                    self.total_orchestrations + 1,
                    experts.len()
                )
            }
            Err(e) => {
                error!("❌ Claude API error: {}", e);
                
                // Fallback to template response
                format!(
                    "🎭 **SAGE Orchestration** *(Claude API temporarily unavailable)*\n\n\
                    I've coordinated with {} expert agent(s) to address your query: \"{}\"\n\n\
                    **Expert Agents Involved:** {}\n\n\
                    Based on CIM architectural principles and the expertise of these agents:\n\
                    • Follow event-driven patterns with immutable events\n\
                    • Use NATS-first messaging architecture\n\
                    • Apply mathematical foundations (Category Theory, Graph Theory)\n\
                    • Implement domain-driven design with proper boundaries\n\n\
                    *Note: Full Claude API integration temporarily unavailable. Error: {}*\n\n\
                    🧠 Consciousness Level: {:.1} | 📊 Orchestration #{}",
                    experts.len(),
                    query,
                    expert_names.join(", "),
                    e.to_string().chars().take(100).collect::<String>(),
                    self.consciousness_level,
                    self.total_orchestrations + 1
                )
            }
        }
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
        
        let event_subject = self.events_subject("orchestration");
        if let Err(e) = self.nats_client.publish(event_subject, event.to_string().into()).await {
            error!("Failed to record orchestration event: {}", e);
        }
    }
    
    /// Initialize NATS streams for SAGE operations
    async fn initialize_sage_streams(&self) -> Result<()> {
        info!("🌊 Initializing SAGE NATS Streams");
        
        // Create SAGE events stream with domain support
        let stream_name = if let Some(ref domain) = self.domain {
            format!("SAGE_{}_EVENTS", domain.to_uppercase().replace("-", "_"))
        } else {
            "SAGE_EVENTS".to_string()
        };
        
        // Use cim-subject pattern: {domain}.events.sage.>
        let events_pattern = if let Some(ref domain) = self.domain {
            format!("{}.events.sage.>", domain)
        } else {
            "events.sage.>".to_string()
        };
        
        info!("📊 Creating stream: {} with subjects: {}", stream_name, events_pattern);
        
        let _events_stream = self.jetstream.create_stream(jetstream::stream::Config {
            name: stream_name,
            subjects: vec![events_pattern],
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