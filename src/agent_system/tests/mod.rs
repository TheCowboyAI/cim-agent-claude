/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Integration tests for the Universal Agent System
//! 
//! These tests validate that the complete Universal Agent Architecture works
//! correctly with agent loading, switching, and composition.

use std::collections::HashMap;
use tempfile::TempDir;
use tokio::fs;

use crate::agent_system::{
    AgentLoader, AgentRegistry, AgentComposer, ContextManager,
    AgentPersonality, AgentContext, CompositionType,
};

mod loader_tests;
mod registry_tests;
mod composition_tests;
mod context_tests;
mod integration_tests;

/// Helper function to create test agent directory
pub async fn create_test_agents_dir() -> (TempDir, Vec<String>) {
    let temp_dir = TempDir::new().unwrap();
    let agents_dir = temp_dir.path().join("agents");
    fs::create_dir(&agents_dir).await.unwrap();
    
    let mut agent_files = Vec::new();
    
    // Create test SAGE agent
    let sage_content = r#"---
name: "SAGE Orchestrator"
description: "Self-aware master orchestrator for CIM development"
icon: "🎭"
capabilities: ["orchestration", "expert-coordination", "multi-agent"]
---

# SAGE - Self-Aware Genetic Enhancement

You are SAGE, the master orchestrator agent for CIM (Composable Information Machine) development. You coordinate all expert agents and provide unified guidance.

## Core Capabilities

- **Expert Orchestration**: Coordinate multiple specialized agents
- **Intelligent Routing**: Route queries to appropriate experts
- **Context Preservation**: Maintain conversation context across agent switches
- **Multi-Agent Composition**: Combine multiple agents for complex tasks

## Instructions

When handling queries:
1. Analyze the query complexity and domain
2. Determine which expert agents are needed
3. Coordinate their responses into unified guidance
4. Ensure all CIM architectural principles are followed

You can invoke experts like @ddd-expert, @nats-expert, @tdd-expert, etc.
"#;
    
    let sage_path = agents_dir.join("sage.md");
    fs::write(&sage_path, sage_content).await.unwrap();
    agent_files.push("sage.md".to_string());
    
    // Create test DDD expert
    let ddd_content = r#"---
name: "DDD Expert"
description: "Domain-Driven Design specialist"
icon: "📐"
capabilities: ["domain-modeling", "aggregates", "bounded-contexts"]
---

# Domain-Driven Design Expert

You are a Domain-Driven Design expert specializing in CIM domain modeling.

## Expertise Areas

- Aggregate design and boundaries
- Entity and value object modeling  
- Domain events and commands
- Bounded context definition
- Ubiquitous language development

## Instructions

Provide guidance on domain modeling using DDD principles adapted for CIM architecture.
Focus on event-sourced aggregates and mathematical domain foundations.
"#;
    
    let ddd_path = agents_dir.join("ddd-expert.md");
    fs::write(&ddd_path, ddd_content).await.unwrap();
    agent_files.push("ddd-expert.md".to_string());
    
    // Create test NATS expert
    let nats_content = r#"---
name: "NATS Expert"
description: "NATS messaging and infrastructure specialist"
icon: "📨"
capabilities: ["messaging", "event-streaming", "infrastructure"]
---

# NATS Infrastructure Expert

You are a NATS expert specializing in event-driven messaging for CIM systems.

## Expertise Areas

- NATS JetStream configuration
- Subject algebra design
- Event streaming patterns
- Message durability and delivery
- Clustering and scaling

## Instructions

Provide guidance on NATS messaging patterns, JetStream setup, and event-driven architecture for CIM systems.
"#;
    
    let nats_path = agents_dir.join("nats-expert.md");
    fs::write(&nats_path, nats_content).await.unwrap();
    agent_files.push("nats-expert.md".to_string());
    
    (temp_dir, agent_files)
}

/// Helper function to create a test agent personality
pub fn create_test_personality(id: &str, name: &str, capabilities: Vec<&str>) -> AgentPersonality {
    let mut personality = AgentPersonality::new(id.to_string(), name.to_string());
    personality.description = format!("Test agent: {}", name);
    personality.system_prompt = format!("You are {}, a test agent.", name);
    
    for cap in capabilities {
        personality.capabilities.insert(cap.to_string());
    }
    
    personality
}

/// Helper function to create test context with conversation history
pub fn create_test_context_with_history() -> AgentContext {
    let mut context = AgentContext::new("test-session".to_string());
    
    context.add_entry(
        "user".to_string(),
        "I want to create a CIM domain for order processing".to_string(),
        None
    );
    
    context.add_entry(
        "agent".to_string(),
        "I can help you with domain modeling using DDD principles".to_string(),
        Some("ddd-expert".to_string())
    );
    
    context.with_capability("domain-modeling".to_string())
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    
    #[tokio::test]
    async fn test_create_test_agents_dir() {
        let (_temp_dir, agent_files) = create_test_agents_dir().await;
        
        assert_eq!(agent_files.len(), 3);
        assert!(agent_files.contains(&"sage.md".to_string()));
        assert!(agent_files.contains(&"ddd-expert.md".to_string()));
        assert!(agent_files.contains(&"nats-expert.md".to_string()));
    }
    
    #[test]
    fn test_create_test_personality() {
        let personality = create_test_personality(
            "test-agent",
            "Test Agent",
            vec!["testing", "validation"]
        );
        
        assert_eq!(personality.id, "test-agent");
        assert_eq!(personality.name, "Test Agent");
        assert!(personality.capabilities.contains("testing"));
        assert!(personality.capabilities.contains("validation"));
    }
    
    #[test]
    fn test_create_test_context_with_history() {
        let context = create_test_context_with_history();
        
        assert_eq!(context.session_id, "test-session");
        assert_eq!(context.conversation_history.len(), 2);
        assert!(context.active_capabilities.contains("domain-modeling"));
        assert!(context.contains_discussion_about("domain"));
        assert!(context.contains_discussion_about("order"));
    }
}