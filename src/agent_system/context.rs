/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Agent Context Management
//! 
//! Provides context preservation and evolution as agents switch and collaborate.
//! Implements monadic context transformations for maintaining conversation state.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub use crate::agent_system::personality::{AgentContext, ConversationEntry, OrchestrationState};
use crate::agent_system::{AgentId, Capability, AgentError, AgentResult};

/// Context manager for preserving conversation state across agent switches
#[derive(Debug)]
pub struct ContextManager {
    active_contexts: HashMap<String, AgentContext>, // session_id -> context
    context_history: HashMap<String, Vec<ContextSnapshot>>, // session_id -> history
}

/// Snapshot of context at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent_id: AgentId,
    pub operation: ContextOperation,
    pub context: AgentContext,
}

/// Operations that can be performed on context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextOperation {
    Created,
    AgentSwitched { from: AgentId, to: AgentId },
    MessageAdded { role: String, content: String },
    CapabilityAdded { capability: Capability },
    OrchestrationStarted { orchestrator: AgentId, participants: Vec<AgentId> },
    OrchestrationCompleted { result: String },
    ContextMerged { source_session: String },
}

/// Context transformation functions (monadic operations)
impl AgentContext {
    /// Bind operation for monadic context transformation
    pub fn bind<F, T>(&self, f: F) -> AgentResult<T>
    where
        F: Fn(&Self) -> AgentResult<T>,
    {
        f(self)
    }
    
    /// Map operation for context transformation
    pub fn map<F>(&self, f: F) -> Self
    where
        F: Fn(&Self) -> Self,
    {
        f(self)
    }
    
    /// Add a new message to the conversation
    pub fn with_message(mut self, role: String, content: String, agent_id: Option<AgentId>) -> Self {
        self.add_entry(role, content, agent_id);
        self
    }
    
    /// Add a capability to the active set
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.active_capabilities.insert(capability);
        self
    }
    
    /// Set orchestration state
    pub fn with_orchestration(mut self, state: OrchestrationState) -> Self {
        self.orchestration_state = state;
        self
    }
    
    /// Add metadata
    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: ToString,
        V: Into<serde_json::Value>,
    {
        self.metadata.insert(key.to_string(), value.into());
        self
    }
    
    /// Extract domain knowledge from conversation history
    pub fn extract_domain_knowledge(&self) -> Vec<DomainKnowledge> {
        let mut knowledge = Vec::new();
        
        for entry in &self.conversation_history {
            // Extract domain entities mentioned
            let entities = self.extract_entities(&entry.content);
            if !entities.is_empty() {
                knowledge.push(DomainKnowledge {
                    knowledge_type: KnowledgeType::Entities,
                    content: entities.join(", "),
                    source_agent: entry.agent_id.clone(),
                    confidence: 0.7,
                });
            }
            
            // Extract events mentioned
            let events = self.extract_events(&entry.content);
            if !events.is_empty() {
                knowledge.push(DomainKnowledge {
                    knowledge_type: KnowledgeType::Events,
                    content: events.join(", "),
                    source_agent: entry.agent_id.clone(),
                    confidence: 0.8,
                });
            }
            
            // Extract commands mentioned
            let commands = self.extract_commands(&entry.content);
            if !commands.is_empty() {
                knowledge.push(DomainKnowledge {
                    knowledge_type: KnowledgeType::Commands,
                    content: commands.join(", "),
                    source_agent: entry.agent_id.clone(),
                    confidence: 0.6,
                });
            }
        }
        
        knowledge
    }
    
    /// Extract entities from text (simple pattern matching)
    fn extract_entities(&self, text: &str) -> Vec<String> {
        let mut entities = Vec::new();
        let text_lower = text.to_lowercase();
        
        // Common domain entity patterns
        let entity_patterns = [
            "order", "customer", "product", "inventory", "payment",
            "user", "account", "transaction", "item", "category",
            "aggregate", "entity", "value object",
        ];
        
        for pattern in &entity_patterns {
            if text_lower.contains(pattern) {
                entities.push(pattern.to_string());
            }
        }
        
        entities
    }
    
    /// Extract events from text
    fn extract_events(&self, text: &str) -> Vec<String> {
        let mut events = Vec::new();
        let text_lower = text.to_lowercase();
        
        // Common event patterns (past tense verbs + nouns)
        let event_patterns = [
            "order placed", "payment processed", "user registered",
            "item added", "inventory updated", "customer created",
            "created", "updated", "deleted", "processed", "completed",
        ];
        
        for pattern in &event_patterns {
            if text_lower.contains(pattern) {
                events.push(pattern.to_string());
            }
        }
        
        events
    }
    
    /// Extract commands from text
    fn extract_commands(&self, text: &str) -> Vec<String> {
        let mut commands = Vec::new();
        let text_lower = text.to_lowercase();
        
        // Common command patterns (imperative verbs)
        let command_patterns = [
            "place order", "process payment", "register user",
            "add item", "update inventory", "create customer",
            "create", "update", "delete", "process", "validate",
        ];
        
        for pattern in &command_patterns {
            if text_lower.contains(pattern) {
                commands.push(pattern.to_string());
            }
        }
        
        commands
    }
    
    /// Get conversation summary
    pub fn get_summary(&self) -> String {
        if self.conversation_history.is_empty() {
            return "No conversation history".to_string();
        }
        
        let total_messages = self.conversation_history.len();
        let agents_involved: HashSet<_> = self.conversation_history.iter()
            .filter_map(|entry| entry.agent_id.as_ref())
            .collect();
        
        let capabilities: Vec<_> = self.active_capabilities.iter().collect();
        
        format!(
            "Session: {} | Messages: {} | Agents: {} | Capabilities: {}",
            self.session_id,
            total_messages,
            agents_involved.len(),
            capabilities.len()
        )
    }
    
    /// Check if context contains discussion about a topic
    pub fn contains_discussion_about(&self, topic: &str) -> bool {
        let topic_lower = topic.to_lowercase();
        
        self.conversation_history.iter()
            .any(|entry| entry.content.to_lowercase().contains(&topic_lower))
    }
    
    /// Get the last message from a specific agent
    pub fn last_message_from(&self, agent_id: &AgentId) -> Option<&ConversationEntry> {
        self.conversation_history.iter()
            .rev()
            .find(|entry| entry.agent_id.as_ref() == Some(agent_id))
    }
    
    /// Create a context suitable for a specific agent type
    pub fn adapt_for_agent(&self, target_agent_id: &AgentId) -> Self {
        let mut adapted = self.clone();
        
        // Add agent-specific context hints
        match target_agent_id.as_str() {
            "ddd-expert" => {
                adapted.active_capabilities.insert("domain-modeling".to_string());
                if !adapted.contains_discussion_about("domain") {
                    adapted.add_entry(
                        "system".to_string(),
                        "Context: This conversation may benefit from domain-driven design expertise".to_string(),
                        Some("system".to_string())
                    );
                }
            }
            "tdd-expert" => {
                adapted.active_capabilities.insert("testing".to_string());
                if !adapted.contains_discussion_about("test") {
                    adapted.add_entry(
                        "system".to_string(),
                        "Context: This conversation may benefit from test-driven development guidance".to_string(),
                        Some("system".to_string())
                    );
                }
            }
            "nats-expert" => {
                adapted.active_capabilities.insert("messaging".to_string());
                if !adapted.contains_discussion_about("event") && !adapted.contains_discussion_about("message") {
                    adapted.add_entry(
                        "system".to_string(),
                        "Context: This conversation may involve NATS messaging and event streaming".to_string(),
                        Some("system".to_string())
                    );
                }
            }
            _ => {
                // Default adaptation
                adapted.active_capabilities.insert("general".to_string());
            }
        }
        
        adapted
    }
}

/// Domain knowledge extracted from conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainKnowledge {
    pub knowledge_type: KnowledgeType,
    pub content: String,
    pub source_agent: Option<AgentId>,
    pub confidence: f64,
}

/// Types of domain knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeType {
    Entities,
    Events,
    Commands,
    Rules,
    Processes,
    Relationships,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Self {
        Self {
            active_contexts: HashMap::new(),
            context_history: HashMap::new(),
        }
    }
    
    /// Create a new context for a session
    pub fn create_context(&mut self, session_id: String) -> &mut AgentContext {
        let context = AgentContext::new(session_id.clone());
        
        // Record creation snapshot
        let snapshot = ContextSnapshot {
            timestamp: chrono::Utc::now(),
            agent_id: "system".to_string(),
            operation: ContextOperation::Created,
            context: context.clone(),
        };
        
        self.context_history.entry(session_id.clone()).or_default().push(snapshot);
        self.active_contexts.entry(session_id).or_insert(context)
    }
    
    /// Get context for a session
    pub fn get_context(&self, session_id: &str) -> Option<&AgentContext> {
        self.active_contexts.get(session_id)
    }
    
    /// Get mutable context for a session
    pub fn get_context_mut(&mut self, session_id: &str) -> Option<&mut AgentContext> {
        self.active_contexts.get_mut(session_id)
    }
    
    /// Switch agent in a context
    pub fn switch_agent(&mut self, session_id: &str, from_agent: AgentId, to_agent: AgentId) -> AgentResult<()> {
        if let Some(context) = self.active_contexts.get_mut(session_id) {
            *context = context.preserve_for_switch(&from_agent, &to_agent);
            
            // Record switch snapshot
            let snapshot = ContextSnapshot {
                timestamp: chrono::Utc::now(),
                agent_id: to_agent.clone(),
                operation: ContextOperation::AgentSwitched { from: from_agent, to: to_agent },
                context: context.clone(),
            };
            
            self.context_history.entry(session_id.to_string()).or_default().push(snapshot);
            
            Ok(())
        } else {
            Err(AgentError::ContextError(format!("Session not found: {}", session_id)))
        }
    }
    
    /// Add message to context
    pub fn add_message(&mut self, session_id: &str, role: String, content: String, agent_id: Option<AgentId>) -> AgentResult<()> {
        if let Some(context) = self.active_contexts.get_mut(session_id) {
            context.add_entry(role.clone(), content.clone(), agent_id.clone());
            
            // Record message snapshot
            let snapshot = ContextSnapshot {
                timestamp: chrono::Utc::now(),
                agent_id: agent_id.unwrap_or_else(|| "user".to_string()),
                operation: ContextOperation::MessageAdded { role, content },
                context: context.clone(),
            };
            
            self.context_history.entry(session_id.to_string()).or_default().push(snapshot);
            
            Ok(())
        } else {
            Err(AgentError::ContextError(format!("Session not found: {}", session_id)))
        }
    }
    
    /// Get context history for a session
    pub fn get_history(&self, session_id: &str) -> Option<&Vec<ContextSnapshot>> {
        self.context_history.get(session_id)
    }
    
    /// Clean up old contexts
    pub fn cleanup_old_contexts(&mut self, max_age: chrono::Duration) {
        let cutoff = chrono::Utc::now() - max_age;
        
        self.active_contexts.retain(|_session_id, context| {
            if let Some(last_entry) = context.conversation_history.last() {
                last_entry.timestamp > cutoff
            } else {
                false // Remove contexts with no messages
            }
        });
        
        self.context_history.retain(|_, history| {
            if let Some(last_snapshot) = history.last() {
                last_snapshot.timestamp > cutoff
            } else {
                false
            }
        });
    }
    
    /// Get statistics about managed contexts
    pub fn get_statistics(&self) -> ContextStatistics {
        let total_contexts = self.active_contexts.len();
        let total_messages: usize = self.active_contexts.values()
            .map(|ctx| ctx.conversation_history.len())
            .sum();
        
        let agents_used: HashSet<_> = self.active_contexts.values()
            .flat_map(|ctx| ctx.conversation_history.iter())
            .filter_map(|entry| entry.agent_id.as_ref())
            .collect();
        
        ContextStatistics {
            total_contexts,
            total_messages,
            unique_agents: agents_used.len(),
            total_snapshots: self.context_history.values().map(|h| h.len()).sum(),
        }
    }
}

/// Statistics about context management
#[derive(Debug)]
pub struct ContextStatistics {
    pub total_contexts: usize,
    pub total_messages: usize,
    pub unique_agents: usize,
    pub total_snapshots: usize,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_creation() {
        let mut manager = ContextManager::new();
        let context = manager.create_context("test-session".to_string());
        
        assert_eq!(context.session_id, "test-session");
        assert_eq!(context.conversation_history.len(), 0);
    }
    
    #[test]
    fn test_agent_switch() {
        let mut manager = ContextManager::new();
        let _context = manager.create_context("test-session".to_string());
        
        manager.add_message(
            "test-session",
            "user".to_string(),
            "Hello".to_string(),
            None
        ).unwrap();
        
        manager.switch_agent(
            "test-session",
            "agent1".to_string(),
            "agent2".to_string()
        ).unwrap();
        
        let context = manager.get_context("test-session").unwrap();
        assert!(context.conversation_history.len() >= 2); // Original message + switch notification
    }
    
    #[test]
    fn test_context_adaptation() {
        let context = AgentContext::new("test".to_string())
            .with_message("user".to_string(), "I need help with domain modeling".to_string(), None);
        
        let adapted = context.adapt_for_agent(&"ddd-expert".to_string());
        
        assert!(adapted.active_capabilities.contains("domain-modeling"));
        assert!(adapted.conversation_history.len() >= context.conversation_history.len());
    }
    
    #[test]
    fn test_domain_knowledge_extraction() {
        let context = AgentContext::new("test".to_string())
            .with_message("user".to_string(), "I want to create an Order aggregate with order placed events".to_string(), None);
        
        let knowledge = context.extract_domain_knowledge();
        
        assert!(!knowledge.is_empty());
        let has_entities = knowledge.iter().any(|k| matches!(k.knowledge_type, KnowledgeType::Entities));
        let has_events = knowledge.iter().any(|k| matches!(k.knowledge_type, KnowledgeType::Events));
        
        assert!(has_entities);
        assert!(has_events);
    }
}