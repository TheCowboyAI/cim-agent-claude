/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! SAGE Client for GUI
//! 
//! Provides GUI interface to interact with SAGE conscious orchestrator
//! through NATS messaging. This module handles sending requests to SAGE
//! and receiving orchestrated responses.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// SAGE Request message that GUI sends via NATS
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
    pub timestamp: DateTime<Utc>,
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

/// SAGE Response message received via NATS
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

/// SAGE Client for GUI interactions
#[derive(Debug, Clone)]
pub struct SageClient {
    session_id: String,
    conversation_history: Vec<ConversationEntry>,
    project_context: Option<ProjectContext>,
}

impl Default for SageClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SageClient {
    /// Create new SAGE client instance
    pub fn new() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            conversation_history: Vec::new(),
            project_context: None,
        }
    }
    
    /// Create SAGE request for general orchestration
    pub fn create_request(&self, query: String) -> SageRequest {
        SageRequest {
            request_id: Uuid::new_v4().to_string(),
            query,
            expert: None,
            context: SageContext {
                session_id: Some(self.session_id.clone()),
                conversation_history: self.conversation_history.clone(),
                project_context: self.project_context.clone(),
            },
        }
    }
    
    /// Create SAGE request for specific expert consultation
    pub fn create_expert_request(&self, query: String, expert: String) -> SageRequest {
        SageRequest {
            request_id: Uuid::new_v4().to_string(),
            query,
            expert: Some(expert),
            context: SageContext {
                session_id: Some(self.session_id.clone()),
                conversation_history: self.conversation_history.clone(),
                project_context: self.project_context.clone(),
            },
        }
    }
    
    /// Update context with SAGE response
    pub fn update_with_response(&mut self, response: &SageResponse) {
        self.conversation_history = response.updated_context.conversation_history.clone();
        if let Some(project_ctx) = &response.updated_context.project_context {
            self.project_context = Some(project_ctx.clone());
        }
    }
    
    /// Set project context
    pub fn set_project_context(&mut self, project_dir: String, cim_domains: Vec<String>, current_phase: String) {
        self.project_context = Some(ProjectContext {
            project_dir,
            cim_domains,
            current_phase,
            active_tasks: Vec::new(),
        });
    }
    
    /// Add task to project context
    pub fn add_active_task(&mut self, task: String) {
        if let Some(ref mut project_ctx) = self.project_context {
            project_ctx.active_tasks.push(task);
        }
    }
    
    /// Remove task from project context
    pub fn remove_active_task(&mut self, task: &str) {
        if let Some(ref mut project_ctx) = self.project_context {
            project_ctx.active_tasks.retain(|t| t != task);
        }
    }
    
    /// Get conversation summary
    pub fn get_conversation_summary(&self) -> String {
        if self.conversation_history.is_empty() {
            return "No conversation history".to_string();
        }
        
        let total_entries = self.conversation_history.len();
        let user_messages = self.conversation_history.iter()
            .filter(|entry| entry.role == "user")
            .count();
        let sage_responses = self.conversation_history.iter()
            .filter(|entry| entry.role == "sage")
            .count();
            
        format!(
            "Conversation Summary:\n• Total exchanges: {}\n• User messages: {}\n• SAGE responses: {}",
            total_entries, user_messages, sage_responses
        )
    }
    
    /// Get list of expert agents used in conversation
    pub fn get_expert_agents_used(&self) -> Vec<String> {
        let mut agents = std::collections::HashSet::new();
        
        for entry in &self.conversation_history {
            for agent in &entry.expert_agents {
                agents.insert(agent.clone());
            }
        }
        
        agents.into_iter().collect()
    }
    
    /// Clear conversation history but keep session
    pub fn clear_conversation(&mut self) {
        self.conversation_history.clear();
    }
    
    /// Start new session (clears everything)
    pub fn new_session(&mut self) {
        self.session_id = Uuid::new_v4().to_string();
        self.conversation_history.clear();
        self.project_context = None;
    }
    
    /// Get session ID (for mock data access)
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    
    /// Get project context (for mock data access)
    pub fn project_context(&self) -> &Option<ProjectContext> {
        &self.project_context
    }
}

/// SAGE NATS Commands for GUI integration
pub mod nats_commands {
    use super::*;
    use crate::nats_client::get_nats_client;
    use crate::messages::Message;
    use tracing::{info, error};
    
    /// Send SAGE request via NATS
    pub async fn send_sage_request(request: SageRequest) -> Message {
        match get_nats_client() {
            Some(client) => {
                match serde_json::to_string(&request) {
                    Ok(json) => {
                        info!("Sending SAGE request: {}", request.request_id);
                        // Using cim-subject pattern: commands.sage.request
                        match client.publish("commands.sage.request", json.into()).await {
                            Ok(_) => {
                                info!("SAGE request sent successfully");
                                Message::SageRequestSent(request.request_id)
                            }
                            Err(e) => {
                                error!("Failed to send SAGE request: {}", e);
                                Message::Error(format!("SAGE request failed: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize SAGE request: {}", e);
                        Message::Error(format!("Serialization failed: {}", e))
                    }
                }
            }
            None => {
                error!("NATS client not initialized");
                Message::Error("NATS client not initialized".to_string())
            }
        }
    }
    
    /// Request SAGE status via NATS
    pub async fn request_sage_status() -> Message {
        match get_nats_client() {
            Some(client) => {
                info!("Requesting SAGE status");
                // Using cim-subject pattern: queries.sage.status
                match client.publish("queries.sage.status", "{}".into()).await {
                    Ok(_) => {
                        info!("SAGE status request sent");
                        Message::SageStatusRequested
                    }
                    Err(e) => {
                        error!("Failed to request SAGE status: {}", e);
                        Message::Error(format!("Status request failed: {}", e))
                    }
                }
            }
            None => {
                error!("NATS client not initialized");
                Message::Error("NATS client not initialized".to_string())
            }
        }
    }
    
    // get_nats_client is now imported from crate::nats_client
}

/// Expert agent definitions for GUI selection
pub fn get_available_experts() -> HashMap<String, String> {
    let mut experts = HashMap::new();
    
    experts.insert("sage".to_string(), 
                  "SAGE Orchestrator - Master coordinator for complete CIM development".to_string());
    experts.insert("cim-expert".to_string(), 
                  "CIM Architecture Expert - Mathematical foundations and architecture".to_string());
    experts.insert("nats-expert".to_string(), 
                  "NATS Infrastructure Expert - Messaging and event infrastructure".to_string());
    experts.insert("ddd-expert".to_string(), 
                  "Domain-Driven Design Expert - Domain modeling and boundaries".to_string());
    experts.insert("bdd-expert".to_string(), 
                  "BDD Expert - Behavior-driven development and scenarios".to_string());
    experts.insert("tdd-expert".to_string(), 
                  "TDD Expert - Test-driven development patterns".to_string());
    experts.insert("qa-expert".to_string(), 
                  "QA Expert - Quality assurance and compliance validation".to_string());
    experts.insert("git-expert".to_string(), 
                  "Git Expert - Version control and repository management".to_string());
    experts.insert("nix-expert".to_string(), 
                  "Nix Expert - System configuration and infrastructure as code".to_string());
    experts.insert("network-expert".to_string(), 
                  "Network Expert - Network topology and security".to_string());
    experts.insert("subject-expert".to_string(), 
                  "Subject Expert - NATS subject algebra and routing patterns".to_string());
    experts.insert("domain-expert".to_string(), 
                  "Domain Expert - Domain creation and validation".to_string());
    experts.insert("event-storming-expert".to_string(), 
                  "Event Storming Expert - Collaborative domain discovery".to_string());
    experts.insert("iced-ui-expert".to_string(), 
                  "Iced UI Expert - Modern Rust GUI development".to_string());
    experts.insert("elm-architecture-expert".to_string(), 
                  "Elm Architecture Expert - Functional reactive patterns".to_string());
    experts.insert("cim-tea-ecs-expert".to_string(), 
                  "CIM TEA ECS Expert - TEA + ECS integration patterns".to_string());
    experts.insert("cim-domain-expert".to_string(), 
                  "CIM Domain Expert - Advanced domain implementation patterns".to_string());
    
    experts
}