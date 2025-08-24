/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Agent Personality System
//! 
//! Parses agent personality configurations from `.claude/agents/*.md` files
//! and provides mathematical foundations for agent composition.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::agent_system::{AgentId, Capability, AgentError, AgentResult};

/// Agent personality configuration loaded from markdown files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPersonality {
    pub id: AgentId,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub system_prompt: String,
    pub capabilities: HashSet<Capability>,
    pub invocation_patterns: Vec<InvocationPattern>,
    pub composition_rules: Vec<CompositionRule>,
    pub metadata: HashMap<String, String>,
}

/// Patterns for when an agent should be invoked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationPattern {
    pub keywords: Vec<String>,
    pub context_requirements: Vec<String>,
    pub confidence_threshold: f64,
}

/// Rules for how agents can be composed together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionRule {
    pub compatible_agents: Vec<AgentId>,
    pub composition_type: CompositionType,
    pub orchestration_pattern: String,
}

/// Types of agent composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositionType {
    Sequential,     // One agent after another
    Parallel,       // Multiple agents simultaneously  
    Hierarchical,   // One agent orchestrating others
    Collaborative,  // Agents working together on shared task
}

impl AgentPersonality {
    /// Create a new agent personality
    pub fn new(id: AgentId, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            icon: "🤖".to_string(),
            system_prompt: String::new(),
            capabilities: HashSet::new(),
            invocation_patterns: Vec::new(),
            composition_rules: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Get the agent's display icon
    pub fn icon(&self) -> &str {
        &self.icon
    }
    
    /// Check if this agent can handle a given query
    pub fn can_handle(&self, query: &str, context: &AgentContext) -> f64 {
        let mut confidence: f64 = 0.0;
        
        for pattern in &self.invocation_patterns {
            let keyword_match = pattern.keywords.iter()
                .any(|keyword| query.to_lowercase().contains(&keyword.to_lowercase()));
            
            if keyword_match {
                confidence = confidence.max(pattern.confidence_threshold);
            }
        }
        
        // Boost confidence if capabilities align with context
        for capability in &self.capabilities {
            if context.requires_capability(capability) {
                confidence += 0.1;
            }
        }
        
        confidence.min(1.0)
    }
    
    /// Check if this agent can be composed with another
    pub fn can_compose_with(&self, other_agent_id: &AgentId) -> Option<CompositionType> {
        for rule in &self.composition_rules {
            if rule.compatible_agents.contains(other_agent_id) {
                return Some(rule.composition_type.clone());
            }
        }
        None
    }
    
    /// Generate system prompt with dynamic context injection
    pub fn render_system_prompt(&self, context: &AgentContext) -> String {
        let mut prompt = self.system_prompt.clone();
        
        // Inject conversation history
        if !context.conversation_history.is_empty() {
            prompt.push_str("\n\n## Conversation Context\n");
            for entry in &context.conversation_history {
                prompt.push_str(&format!("- {}: {}\n", entry.role, entry.content));
            }
        }
        
        // Inject active capabilities
        if !context.active_capabilities.is_empty() {
            prompt.push_str("\n\n## Active Capabilities\n");
            for capability in &context.active_capabilities {
                prompt.push_str(&format!("- {}\n", capability));
            }
        }
        
        prompt
    }
}

/// Agent execution context that preserves state across switches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub session_id: String,
    pub conversation_history: Vec<ConversationEntry>,
    pub active_capabilities: HashSet<Capability>,
    pub orchestration_state: OrchestrationState,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub role: String,
    pub content: String,
    pub agent_id: Option<AgentId>,
    pub expert_agents: Vec<AgentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationState {
    SingleAgent,
    MultiAgent { orchestrator: AgentId, participants: Vec<AgentId> },
    Composition { pattern: String, state: HashMap<String, serde_json::Value> },
}

impl AgentContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            conversation_history: Vec::new(),
            active_capabilities: HashSet::new(),
            orchestration_state: OrchestrationState::SingleAgent,
            metadata: HashMap::new(),
        }
    }
    
    /// Check if context requires a specific capability
    pub fn requires_capability(&self, capability: &str) -> bool {
        self.active_capabilities.contains(capability) ||
        self.conversation_history.iter()
            .any(|entry| entry.content.to_lowercase().contains(&capability.to_lowercase()))
    }
    
    /// Add a new conversation entry
    pub fn add_entry(&mut self, role: String, content: String, agent_id: Option<AgentId>) {
        self.conversation_history.push(ConversationEntry {
            timestamp: chrono::Utc::now(),
            role,
            content,
            agent_id,
            expert_agents: Vec::new(),
        });
    }
    
    /// Preserve context when switching agents
    pub fn preserve_for_switch(&self, from_agent: &AgentId, to_agent: &AgentId) -> Self {
        let mut preserved = self.clone();
        
        // Add context switch entry
        preserved.add_entry(
            "system".to_string(),
            format!("Context switched from @{} to @{}", from_agent, to_agent),
            None
        );
        
        preserved
    }
}

impl Default for AgentContext {
    fn default() -> Self {
        Self::new(uuid::Uuid::new_v4().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_personality_creation() {
        let personality = AgentPersonality::new(
            "test-agent".to_string(),
            "Test Agent".to_string()
        );
        
        assert_eq!(personality.id, "test-agent");
        assert_eq!(personality.name, "Test Agent");
        assert_eq!(personality.icon(), "🤖");
    }
    
    #[test]
    fn test_agent_can_handle_query() {
        let mut personality = AgentPersonality::new(
            "ddd-expert".to_string(),
            "DDD Expert".to_string()
        );
        
        personality.invocation_patterns.push(InvocationPattern {
            keywords: vec!["domain".to_string(), "aggregate".to_string()],
            context_requirements: vec![],
            confidence_threshold: 0.8,
        });
        
        let context = AgentContext::default();
        let confidence = personality.can_handle("How do I design domain aggregates?", &context);
        
        assert!(confidence >= 0.8);
    }
    
    #[test]
    fn test_context_preservation() {
        let mut context = AgentContext::new("test-session".to_string());
        context.add_entry("user".to_string(), "Test message".to_string(), None);
        
        let preserved = context.preserve_for_switch(&"agent1".to_string(), &"agent2".to_string());
        
        assert_eq!(preserved.conversation_history.len(), 2); // Original + switch entry
        assert!(preserved.conversation_history.last().unwrap().content.contains("Context switched"));
    }
}