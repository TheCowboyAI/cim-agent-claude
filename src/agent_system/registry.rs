/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Agent Registry System
//! 
//! Manages the registry of available agents and provides intelligent routing
//! based on query analysis and agent capabilities.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::watch;

use crate::agent_system::{
    AgentPersonality, AgentContext, AgentLoader, CompositionType,
    AgentId, AgentError, AgentResult,
};

/// Central registry for all available agent personalities
#[derive(Debug)]
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<AgentId, AgentPersonality>>>,
    loader: AgentLoader,
    change_notifier: watch::Sender<RegistryChange>,
}

/// Registry change notifications
#[derive(Debug, Clone)]
pub enum RegistryChange {
    AgentAdded(AgentId),
    AgentRemoved(AgentId),
    AgentUpdated(AgentId),
    RegistryReloaded,
}

/// Agent routing decision with confidence score
#[derive(Debug, Clone)]
pub struct AgentRoute {
    pub agent_id: AgentId,
    pub confidence: f64,
    pub reasoning: String,
    pub composition_suggestion: Option<AgentComposition>,
}

/// Agent composition suggestion
#[derive(Debug, Clone)]
pub struct AgentComposition {
    pub orchestrator: AgentId,
    pub participants: Vec<AgentId>,
    pub composition_type: CompositionType,
    pub pattern: String,
}

impl AgentRegistry {
    /// Create a new agent registry
    pub fn new() -> (Self, watch::Receiver<RegistryChange>) {
        let (tx, rx) = watch::channel(RegistryChange::RegistryReloaded);
        
        let registry = Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            loader: AgentLoader::new(),
            change_notifier: tx,
        };
        
        (registry, rx)
    }
    
    /// Load all agents from the .claude/agents directory
    pub async fn load_all_agents(&self) -> AgentResult<usize> {
        let loaded_agents = self.loader.load_all_agents().await?;
        let count = loaded_agents.len();
        
        {
            let mut agents = self.agents.write().unwrap();
            agents.clear();
            agents.extend(loaded_agents);
        }
        
        let _ = self.change_notifier.send(RegistryChange::RegistryReloaded);
        tracing::info!("Loaded {} agent personalities", count);
        
        Ok(count)
    }
    
    /// Get all available agents
    pub fn get_all_agents(&self) -> HashMap<AgentId, AgentPersonality> {
        self.agents.read().unwrap().clone()
    }
    
    /// Get a specific agent by ID
    pub fn get_agent(&self, agent_id: &AgentId) -> Option<AgentPersonality> {
        self.agents.read().unwrap().get(agent_id).cloned()
    }
    
    /// Find the best agent(s) for a given query
    pub fn route_query(&self, query: &str, context: &AgentContext) -> Vec<AgentRoute> {
        let agents = self.agents.read().unwrap();
        let mut routes = Vec::new();
        
        for (agent_id, personality) in agents.iter() {
            let confidence = personality.can_handle(query, context);
            
            if confidence > 0.3 {  // Minimum threshold
                let mut reasoning = format!("Agent {} has {:.1}% confidence", 
                    personality.name, confidence * 100.0);
                
                // Check for composition opportunities
                let composition = self.suggest_composition(agent_id, query, context);
                if composition.is_some() {
                    reasoning.push_str(" with composition suggested");
                }
                
                routes.push(AgentRoute {
                    agent_id: agent_id.clone(),
                    confidence,
                    reasoning,
                    composition_suggestion: composition,
                });
            }
        }
        
        // Sort by confidence
        routes.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        routes
    }
    
    /// Get SAGE agent for orchestration
    pub fn get_sage_agent(&self) -> Option<AgentPersonality> {
        self.get_agent(&"sage".to_string())
    }
    
    /// Route complex queries to SAGE for orchestration
    pub fn route_to_sage(&self, query: &str, context: &AgentContext) -> AgentRoute {
        let sage = self.get_sage_agent();
        
        if let Some(_sage_personality) = sage {
            // SAGE can handle any query with orchestration
            let confidence = 0.95; // SAGE is always highly confident
            
            // Suggest multi-agent composition for complex queries
            let composition = self.suggest_multi_agent_composition(query, context);
            
            AgentRoute {
                agent_id: "sage".to_string(),
                confidence,
                reasoning: "SAGE orchestrator can coordinate multiple experts".to_string(),
                composition_suggestion: composition,
            }
        } else {
            // Fallback if SAGE not loaded
            AgentRoute {
                agent_id: "sage".to_string(),
                confidence: 0.5,
                reasoning: "SAGE agent not loaded, using default orchestration".to_string(),
                composition_suggestion: None,
            }
        }
    }
    
    /// Suggest composition for a single agent
    fn suggest_composition(
        &self, 
        agent_id: &AgentId, 
        query: &str, 
        _context: &AgentContext
    ) -> Option<AgentComposition> {
        let agents = self.agents.read().unwrap();
        let _agent = agents.get(agent_id)?;
        
        // Check if this query would benefit from multiple experts
        let query_lower = query.to_lowercase();
        let mut suggested_participants = vec![agent_id.clone()];
        
        // Add complementary agents
        if query_lower.contains("domain") && agent_id != "ddd-expert" {
            suggested_participants.push("ddd-expert".to_string());
        }
        if query_lower.contains("test") && !agent_id.contains("expert") {
            suggested_participants.push("tdd-expert".to_string());
        }
        if query_lower.contains("infrastructure") || query_lower.contains("nats") {
            if agent_id != "nats-expert" {
                suggested_participants.push("nats-expert".to_string());
            }
            if agent_id != "nix-expert" {
                suggested_participants.push("nix-expert".to_string());
            }
        }
        
        if suggested_participants.len() > 1 {
            Some(AgentComposition {
                orchestrator: "sage".to_string(),
                participants: suggested_participants,
                composition_type: CompositionType::Collaborative,
                pattern: "multi-expert-collaboration".to_string(),
            })
        } else {
            None
        }
    }
    
    /// Suggest multi-agent composition for complex queries
    fn suggest_multi_agent_composition(
        &self,
        query: &str,
        _context: &AgentContext
    ) -> Option<AgentComposition> {
        let query_lower = query.to_lowercase();
        let mut participants = Vec::new();
        
        // Analyze query complexity and suggest appropriate experts
        if query_lower.contains("domain") || query_lower.contains("ddd") {
            participants.extend_from_slice(&[
                "ddd-expert".to_string(),
                "event-storming-expert".to_string(),
                "domain-expert".to_string(),
            ]);
        }
        
        if query_lower.contains("infrastructure") || query_lower.contains("deploy") {
            participants.extend_from_slice(&[
                "nats-expert".to_string(),
                "nix-expert".to_string(),
                "network-expert".to_string(),
            ]);
        }
        
        if query_lower.contains("test") {
            participants.extend_from_slice(&[
                "tdd-expert".to_string(),
                "bdd-expert".to_string(),
                "qa-expert".to_string(),
            ]);
        }
        
        if query_lower.contains("ui") || query_lower.contains("interface") {
            participants.extend_from_slice(&[
                "iced-ui-expert".to_string(),
                "elm-architecture-expert".to_string(),
                "cim-tea-ecs-expert".to_string(),
            ]);
        }
        
        // Remove duplicates and ensure we have multiple participants
        participants.sort();
        participants.dedup();
        
        if participants.len() > 1 {
            Some(AgentComposition {
                orchestrator: "sage".to_string(),
                participants,
                composition_type: CompositionType::Hierarchical,
                pattern: "sage-orchestration".to_string(),
            })
        } else if participants.len() == 1 {
            // Single expert + CIM expert for foundation
            participants.push("cim-expert".to_string());
            Some(AgentComposition {
                orchestrator: "sage".to_string(),
                participants,
                composition_type: CompositionType::Collaborative,
                pattern: "expert-with-foundation".to_string(),
            })
        } else {
            // Default to CIM expert for unknown queries
            Some(AgentComposition {
                orchestrator: "sage".to_string(),
                participants: vec!["cim-expert".to_string()],
                composition_type: CompositionType::Sequential,
                pattern: "foundation-guidance".to_string(),
            })
        }
    }
    
    /// Check if an agent is available
    pub fn is_agent_available(&self, agent_id: &AgentId) -> bool {
        self.agents.read().unwrap().contains_key(agent_id)
    }
    
    /// Get agent statistics
    pub fn get_statistics(&self) -> RegistryStatistics {
        let agents = self.agents.read().unwrap();
        
        let mut capabilities = HashMap::new();
        let mut composition_types = HashMap::new();
        
        for personality in agents.values() {
            for capability in &personality.capabilities {
                *capabilities.entry(capability.clone()).or_insert(0) += 1;
            }
            
            for rule in &personality.composition_rules {
                let type_name = format!("{:?}", rule.composition_type);
                *composition_types.entry(type_name).or_insert(0) += 1;
            }
        }
        
        RegistryStatistics {
            total_agents: agents.len(),
            capabilities,
            composition_types,
            sage_available: agents.contains_key("sage"),
        }
    }
}

/// Registry statistics
#[derive(Debug)]
pub struct RegistryStatistics {
    pub total_agents: usize,
    pub capabilities: HashMap<String, usize>,
    pub composition_types: HashMap<String, usize>,
    pub sage_available: bool,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        let (registry, _) = Self::new();
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_system::{AgentPersonality, AgentContext, InvocationPattern};
    
    fn create_test_agent(id: &str, keywords: Vec<&str>) -> AgentPersonality {
        let mut agent = AgentPersonality::new(id.to_string(), id.to_string());
        
        agent.invocation_patterns.push(InvocationPattern {
            keywords: keywords.into_iter().map(|s| s.to_string()).collect(),
            context_requirements: Vec::new(),
            confidence_threshold: 0.8,
        });
        
        agent
    }
    
    #[tokio::test]
    async fn test_agent_routing() {
        let (registry, _rx) = AgentRegistry::new();
        
        // Add test agents manually
        {
            let mut agents = registry.agents.write().unwrap();
            agents.insert("ddd-expert".to_string(), create_test_agent("ddd-expert", vec!["domain", "ddd"]));
            agents.insert("tdd-expert".to_string(), create_test_agent("tdd-expert", vec!["test", "tdd"]));
        }
        
        let context = AgentContext::default();
        let routes = registry.route_query("How do I design domain aggregates?", &context);
        
        assert!(!routes.is_empty());
        assert_eq!(routes[0].agent_id, "ddd-expert");
        assert!(routes[0].confidence > 0.7);
    }
    
    #[tokio::test]
    async fn test_sage_orchestration() {
        let (registry, _rx) = AgentRegistry::new();
        
        let context = AgentContext::default();
        let route = registry.route_to_sage("Complex CIM development with domain modeling and testing", &context);
        
        assert_eq!(route.agent_id, "sage");
        assert!(route.confidence > 0.9);
        assert!(route.composition_suggestion.is_some());
        
        if let Some(composition) = route.composition_suggestion {
            assert_eq!(composition.orchestrator, "sage");
            assert!(composition.participants.len() > 1);
        }
    }
    
    #[test]
    fn test_registry_statistics() {
        let (registry, _rx) = AgentRegistry::new();
        
        {
            let mut agents = registry.agents.write().unwrap();
            agents.insert("test1".to_string(), create_test_agent("test1", vec!["test"]));
            agents.insert("test2".to_string(), create_test_agent("test2", vec!["domain"]));
        }
        
        let stats = registry.get_statistics();
        assert_eq!(stats.total_agents, 2);
        assert!(!stats.sage_available); // SAGE not added in this test
    }
}