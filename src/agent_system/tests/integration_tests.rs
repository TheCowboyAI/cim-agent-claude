/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Integration tests for the complete Universal Agent System
//! 
//! These tests validate end-to-end functionality of loading agents,
//! routing queries, switching contexts, and composing multi-agent workflows.

use super::*;
use async_trait::async_trait;
use crate::agent_system::{
    AgentLoader, AgentRegistry, ContextManager, AgentComposer,
    CompositionType, AgentExecutor, AgentExecutionResult,
};

/// Mock agent executor for testing
struct MockAgentExecutor {
    responses: HashMap<String, String>,
}

#[async_trait::async_trait]
impl AgentExecutor for MockAgentExecutor {
    async fn execute_agent(
        &self,
        agent_id: &str,
        query: &str,
        context: &AgentContext,
    ) -> crate::agent_system::AgentResult<AgentExecutionResult> {
        let response = self.responses.get(agent_id)
            .cloned()
            .unwrap_or_else(|| format!("{} response to: {}", agent_id, query));
        
        Ok(AgentExecutionResult {
            response,
            confidence: 0.85,
            updated_context: context.clone(),
            metadata: HashMap::new(),
        })
    }
}

#[tokio::test]
async fn test_complete_agent_system_workflow() {
    // Create test environment
    let (_temp_dir, _agent_files) = create_test_agents_dir().await;
    
    // Load agents
    let loader = AgentLoader::with_directory(_temp_dir.path().join("agents"));
    let agents = loader.load_all_agents().await.unwrap();
    
    assert_eq!(agents.len(), 3); // sage, ddd-expert, nats-expert
    assert!(agents.contains_key("sage"));
    assert!(agents.contains_key("ddd-expert"));
    assert!(agents.contains_key("nats-expert"));
    
    // Create registry and context manager
    let (mut registry, _rx) = AgentRegistry::new();
    {
        let mut registry_agents = registry.agents.write().unwrap();
        registry_agents.extend(agents);
    }
    
    let mut context_manager = ContextManager::new();
    let session_id = "test-session".to_string();
    let _context = context_manager.create_context(session_id.clone());
    
    // Test query routing
    let query = "I need help designing a domain model for order processing with NATS events";
    let initial_context = AgentContext::new(session_id.clone());
    
    let routes = registry.route_query(&query, &initial_context);
    
    assert!(!routes.is_empty());
    
    // Should route to multiple agents due to complexity
    let sage_route = registry.route_to_sage(&query, &initial_context);
    assert_eq!(sage_route.agent_id, "sage");
    assert!(sage_route.confidence > 0.9);
    assert!(sage_route.composition_suggestion.is_some());
    
    // Test agent composition
    let composer = AgentComposer::new();
    let patterns = composer.get_patterns();
    assert!(patterns.contains_key("sage-orchestration"));
    
    // Test mock execution
    let mut mock_responses = HashMap::new();
    mock_responses.insert(
        "sage".to_string(), 
        "🎭 SAGE coordinating DDD and NATS experts for your order processing domain".to_string()
    );
    mock_responses.insert(
        "ddd-expert".to_string(),
        "📐 Order aggregate should be event-sourced with OrderPlaced, OrderPaid events".to_string()
    );
    mock_responses.insert(
        "nats-expert".to_string(),
        "📨 Configure JetStream with 'orders.*' subject pattern for event streaming".to_string()
    );
    
    let executor = MockAgentExecutor { responses: mock_responses };
    
    // Execute composition
    if let Some(composition) = sage_route.composition_suggestion {
        let result = composer.execute_composition(
            "sage-orchestration",
            query,
            initial_context.clone(),
            &executor,
        ).await.unwrap();
        
        assert_eq!(result.orchestrator, "sage");
        assert!(!result.execution_steps.is_empty());
        assert!(result.final_result.contains("SAGE"));
    }
    
    // Test context switching
    context_manager.add_message(
        &session_id,
        "user".to_string(),
        query.to_string(),
        None
    ).unwrap();
    
    context_manager.switch_agent(
        &session_id,
        "ddd-expert".to_string(),
        "nats-expert".to_string()
    ).unwrap();
    
    let final_context = context_manager.get_context(&session_id).unwrap();
    assert!(final_context.conversation_history.len() >= 2); // Original message + switch notification
    
    // Verify context contains switch information
    let has_switch_message = final_context.conversation_history.iter()
        .any(|entry| entry.content.contains("Context switched"));
    assert!(has_switch_message);
}

#[tokio::test]
async fn test_agent_discovery_and_capabilities() {
    let (_temp_dir, _agent_files) = create_test_agents_dir().await;
    
    let loader = AgentLoader::with_directory(_temp_dir.path().join("agents"));
    let agents = loader.load_all_agents().await.unwrap();
    
    // Test SAGE capabilities
    let sage = agents.get("sage").unwrap();
    assert_eq!(sage.name, "SAGE Orchestrator");
    assert!(sage.capabilities.contains("orchestration"));
    assert!(sage.capabilities.contains("expert-coordination"));
    assert!(sage.system_prompt.contains("master orchestrator"));
    
    // Test DDD expert capabilities
    let ddd_expert = agents.get("ddd-expert").unwrap();
    assert_eq!(ddd_expert.name, "DDD Expert");
    assert!(ddd_expert.capabilities.contains("domain-modeling"));
    assert!(ddd_expert.capabilities.contains("aggregates"));
    
    // Test NATS expert capabilities
    let nats_expert = agents.get("nats-expert").unwrap();
    assert_eq!(nats_expert.name, "NATS Expert");
    assert!(nats_expert.capabilities.contains("messaging"));
    assert!(nats_expert.capabilities.contains("event-streaming"));
}

#[tokio::test]
async fn test_intelligent_query_routing() {
    let (_temp_dir, _agent_files) = create_test_agents_dir().await;
    
    let loader = AgentLoader::with_directory(_temp_dir.path().join("agents"));
    let agents = loader.load_all_agents().await.unwrap();
    
    let (mut registry, _rx) = AgentRegistry::new();
    {
        let mut registry_agents = registry.agents.write().unwrap();
        registry_agents.extend(agents);
    }
    
    let context = AgentContext::default();
    
    // Test domain modeling query
    let domain_query = "How do I design aggregates for my order processing domain?";
    let domain_routes = registry.route_query(domain_query, &context);
    
    assert!(!domain_routes.is_empty());
    let best_domain_route = &domain_routes[0];
    assert_eq!(best_domain_route.agent_id, "ddd-expert");
    assert!(best_domain_route.confidence > 0.5);
    
    // Test infrastructure query
    let infra_query = "How do I set up NATS JetStream for event streaming?";
    let infra_routes = registry.route_query(infra_query, &context);
    
    assert!(!infra_routes.is_empty());
    let best_infra_route = &infra_routes[0];
    assert_eq!(best_infra_route.agent_id, "nats-expert");
    assert!(best_infra_route.confidence > 0.5);
    
    // Test complex query that should go to SAGE
    let complex_query = "I need to build a complete CIM system with domain modeling, NATS infrastructure, and testing";
    let sage_route = registry.route_to_sage(complex_query, &context);
    
    assert_eq!(sage_route.agent_id, "sage");
    assert!(sage_route.confidence > 0.9);
    assert!(sage_route.composition_suggestion.is_some());
    
    if let Some(composition) = sage_route.composition_suggestion {
        assert!(composition.participants.len() > 1);
        assert!(composition.participants.contains(&"ddd-expert".to_string()) || 
                composition.participants.contains(&"nats-expert".to_string()));
    }
}

#[tokio::test]
async fn test_context_preservation_across_agents() {
    let mut context_manager = ContextManager::new();
    let session_id = "preservation-test".to_string();
    
    // Create initial context
    let _context = context_manager.create_context(session_id.clone());
    
    // Add some conversation history
    context_manager.add_message(
        &session_id,
        "user".to_string(),
        "I want to create an order processing system".to_string(),
        None
    ).unwrap();
    
    context_manager.add_message(
        &session_id,
        "agent".to_string(),
        "I'll help you design the Order aggregate".to_string(),
        Some("ddd-expert".to_string())
    ).unwrap();
    
    // Switch to another agent
    context_manager.switch_agent(
        &session_id,
        "ddd-expert".to_string(),
        "nats-expert".to_string()
    ).unwrap();
    
    // Verify context preservation
    let preserved_context = context_manager.get_context(&session_id).unwrap();
    
    // Should have original messages plus switch notification
    assert!(preserved_context.conversation_history.len() >= 3);
    
    // Should contain original domain discussion
    assert!(preserved_context.contains_discussion_about("order"));
    assert!(preserved_context.contains_discussion_about("processing"));
    
    // Should have switch notification
    let has_switch_notification = preserved_context.conversation_history.iter()
        .any(|entry| entry.content.contains("Context switched") && 
                     entry.content.contains("ddd-expert") &&
                     entry.content.contains("nats-expert"));
    
    assert!(has_switch_notification);
    
    // Test context adaptation for new agent
    let adapted_context = preserved_context.adapt_for_agent(&"nats-expert".to_string());
    assert!(adapted_context.active_capabilities.contains("messaging"));
}

#[tokio::test]
async fn test_multi_agent_composition_patterns() {
    let composer = AgentComposer::new();
    let patterns = composer.get_patterns();
    
    // Verify standard patterns are registered
    assert!(patterns.contains_key("sage-orchestration"));
    assert!(patterns.contains_key("domain-collaboration"));
    assert!(patterns.contains_key("infrastructure-pipeline"));
    
    // Test SAGE orchestration pattern
    let sage_pattern = &patterns["sage-orchestration"];
    assert_eq!(sage_pattern.orchestrator, "sage");
    assert_eq!(sage_pattern.pattern_type, CompositionType::Hierarchical);
    assert!(sage_pattern.coordination_rules.iter()
        .any(|rule| matches!(rule.rule_type, crate::agent_system::CoordinationType::ContextSharing)));
    
    // Test domain collaboration pattern
    let domain_pattern = &patterns["domain-collaboration"];
    assert_eq!(domain_pattern.orchestrator, "ddd-expert");
    assert_eq!(domain_pattern.pattern_type, CompositionType::Collaborative);
    assert!(domain_pattern.participants.contains(&"event-storming-expert".to_string()));
    
    // Test infrastructure pipeline pattern
    let infra_pattern = &patterns["infrastructure-pipeline"];
    assert_eq!(infra_pattern.orchestrator, "nix-expert");
    assert_eq!(infra_pattern.pattern_type, CompositionType::Sequential);
    assert!(infra_pattern.participants.contains(&"nats-expert".to_string()));
}

#[tokio::test]
async fn test_registry_statistics_and_health() {
    let (_temp_dir, _agent_files) = create_test_agents_dir().await;
    
    let loader = AgentLoader::with_directory(_temp_dir.path().join("agents"));
    let agents = loader.load_all_agents().await.unwrap();
    
    let (mut registry, _rx) = AgentRegistry::new();
    {
        let mut registry_agents = registry.agents.write().unwrap();
        registry_agents.extend(agents);
    }
    
    let stats = registry.get_statistics();
    
    assert_eq!(stats.total_agents, 3);
    assert!(stats.sage_available);
    assert!(!stats.capabilities.is_empty());
    assert!(!stats.composition_types.is_empty());
    
    // Verify specific capabilities are tracked
    assert!(stats.capabilities.contains_key("orchestration"));
    assert!(stats.capabilities.contains_key("domain-modeling"));
    assert!(stats.capabilities.contains_key("messaging"));
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    // Test loading from non-existent directory
    let loader = AgentLoader::with_directory("/non/existent/path");
    let result = loader.load_all_agents().await;
    assert!(result.is_err());
    
    // Test invalid agent file
    let temp_dir = tempfile::TempDir::new().unwrap();
    let agents_dir = temp_dir.path().join("agents");
    fs::create_dir(&agents_dir).await.unwrap();
    
    let invalid_content = "This is not valid YAML frontmatter\n---\n# But has some markdown";
    fs::write(agents_dir.join("invalid.md"), invalid_content).await.unwrap();
    
    let loader = AgentLoader::with_directory(&agents_dir);
    let result = loader.load_all_agents().await;
    // Should succeed but with no agents loaded due to invalid content
    assert!(result.is_ok());
    let agents = result.unwrap();
    assert!(agents.is_empty() || agents.values().all(|a| a.system_prompt.contains("invalid")));
    
    // Test context manager with invalid session
    let mut context_manager = ContextManager::new();
    let result = context_manager.switch_agent(
        "non-existent-session",
        "agent1".to_string(),
        "agent2".to_string()
    );
    assert!(result.is_err());
}