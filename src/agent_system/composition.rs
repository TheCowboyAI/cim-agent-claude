/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Agent Composition System
//! 
//! Implements mathematical agent composition patterns where agents can be
//! combined, orchestrated, and work together using category theory principles.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::agent_system::{
    AgentId, AgentContext, AgentPersonality, CompositionType, 
    AgentError, AgentResult,
};

/// Agent composition executor that manages multi-agent workflows
#[derive(Debug)]
pub struct AgentComposer {
    compositions: HashMap<String, CompositionPattern>,
}

/// A composition pattern defines how agents work together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionPattern {
    pub id: String,
    pub name: String,
    pub pattern_type: CompositionType,
    pub orchestrator: AgentId,
    pub participants: Vec<AgentId>,
    pub execution_order: ExecutionOrder,
    pub coordination_rules: Vec<CoordinationRule>,
}

/// How agents should be executed in the composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionOrder {
    Sequential(Vec<AgentId>),         // One after another in specified order
    Parallel(Vec<AgentId>),           // All at the same time
    ConditionalFlow(Vec<FlowStep>),   // Based on conditions
    Interactive(Vec<InteractionStep>), // Back-and-forth between agents
}

/// A step in conditional flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub condition: String,
    pub if_true: Vec<AgentId>,
    pub if_false: Vec<AgentId>,
}

/// A step in interactive execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionStep {
    pub initiator: AgentId,
    pub responder: AgentId,
    pub interaction_type: InteractionType,
}

/// Types of agent interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    Question,        // One agent asks another
    Collaboration,   // Agents work together
    Review,          // One agent reviews another's work
    Handoff,         // One agent passes work to another
}

/// Rules for how agents coordinate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationRule {
    pub rule_type: CoordinationType,
    pub participants: Vec<AgentId>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Types of coordination between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationType {
    ContextSharing,    // Share conversation context
    ResultAggregation, // Combine results from multiple agents
    ConflictResolution, // Handle disagreements between agents
    QualityControl,    // One agent validates another's output
}

/// Result of executing an agent composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionResult {
    pub pattern_id: String,
    pub orchestrator: AgentId,
    pub participants: Vec<AgentId>,
    pub execution_steps: Vec<ExecutionStep>,
    pub final_result: String,
    pub context_evolution: Vec<AgentContext>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A single step in composition execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_id: String,
    pub agent_id: AgentId,
    pub input: String,
    pub output: String,
    pub confidence: f64,
    pub execution_time_ms: u64,
    pub context_snapshot: AgentContext,
}

impl AgentComposer {
    /// Create a new agent composer
    pub fn new() -> Self {
        let mut composer = Self {
            compositions: HashMap::new(),
        };
        
        // Register standard composition patterns
        composer.register_standard_patterns();
        
        composer
    }
    
    /// Register standard CIM composition patterns
    fn register_standard_patterns(&mut self) {
        // SAGE Orchestration Pattern
        let sage_pattern = CompositionPattern {
            id: "sage-orchestration".to_string(),
            name: "SAGE Orchestration".to_string(),
            pattern_type: CompositionType::Hierarchical,
            orchestrator: "sage".to_string(),
            participants: vec![], // Dynamic based on query
            execution_order: ExecutionOrder::ConditionalFlow(vec![]),
            coordination_rules: vec![
                CoordinationRule {
                    rule_type: CoordinationType::ContextSharing,
                    participants: vec!["*".to_string()], // All participants
                    parameters: HashMap::new(),
                },
                CoordinationRule {
                    rule_type: CoordinationType::ResultAggregation,
                    participants: vec!["sage".to_string()],
                    parameters: HashMap::new(),
                },
            ],
        };
        self.compositions.insert("sage-orchestration".to_string(), sage_pattern);
        
        // Domain Expert Collaboration
        let domain_collab = CompositionPattern {
            id: "domain-collaboration".to_string(),
            name: "Domain Expert Collaboration".to_string(),
            pattern_type: CompositionType::Collaborative,
            orchestrator: "ddd-expert".to_string(),
            participants: vec![
                "event-storming-expert".to_string(),
                "domain-expert".to_string(),
                "cim-expert".to_string(),
            ],
            execution_order: ExecutionOrder::Interactive(vec![
                InteractionStep {
                    initiator: "event-storming-expert".to_string(),
                    responder: "ddd-expert".to_string(),
                    interaction_type: InteractionType::Collaboration,
                },
                InteractionStep {
                    initiator: "ddd-expert".to_string(),
                    responder: "domain-expert".to_string(),
                    interaction_type: InteractionType::Handoff,
                },
                InteractionStep {
                    initiator: "domain-expert".to_string(),
                    responder: "cim-expert".to_string(),
                    interaction_type: InteractionType::Review,
                },
            ]),
            coordination_rules: vec![
                CoordinationRule {
                    rule_type: CoordinationType::ContextSharing,
                    participants: vec!["*".to_string()],
                    parameters: HashMap::new(),
                },
            ],
        };
        self.compositions.insert("domain-collaboration".to_string(), domain_collab);
        
        // Infrastructure Pipeline
        let infra_pipeline = CompositionPattern {
            id: "infrastructure-pipeline".to_string(),
            name: "Infrastructure Setup Pipeline".to_string(),
            pattern_type: CompositionType::Sequential,
            orchestrator: "nix-expert".to_string(),
            participants: vec![
                "network-expert".to_string(),
                "nats-expert".to_string(),
                "nix-expert".to_string(),
            ],
            execution_order: ExecutionOrder::Sequential(vec![
                "network-expert".to_string(),
                "nats-expert".to_string(),
                "nix-expert".to_string(),
            ]),
            coordination_rules: vec![
                CoordinationRule {
                    rule_type: CoordinationType::ContextSharing,
                    participants: vec!["*".to_string()],
                    parameters: HashMap::new(),
                },
                CoordinationRule {
                    rule_type: CoordinationType::QualityControl,
                    participants: vec!["nix-expert".to_string()], // Final validator
                    parameters: HashMap::new(),
                },
            ],
        };
        self.compositions.insert("infrastructure-pipeline".to_string(), infra_pipeline);
    }
    
    /// Execute a composition pattern
    pub async fn execute_composition(
        &self,
        pattern_id: &str,
        query: String,
        context: AgentContext,
        agent_executor: &dyn AgentExecutor,
    ) -> AgentResult<CompositionResult> {
        let pattern = self.compositions.get(pattern_id)
            .ok_or_else(|| AgentError::InvalidConfiguration(
                format!("Unknown composition pattern: {}", pattern_id)
            ))?;
        
        let mut execution_steps = Vec::new();
        let mut current_context = context;
        let mut context_evolution = vec![current_context.clone()];
        
        match &pattern.execution_order {
            ExecutionOrder::Sequential(agents) => {
                let mut accumulated_result = query.clone();
                
                for agent_id in agents {
                    let step_result = self.execute_agent_step(
                        agent_id,
                        &accumulated_result,
                        &current_context,
                        agent_executor,
                    ).await?;
                    
                    accumulated_result = step_result.output.clone();
                    current_context = step_result.context_snapshot.clone();
                    context_evolution.push(current_context.clone());
                    execution_steps.push(step_result);
                }
                
                Ok(CompositionResult {
                    pattern_id: pattern_id.to_string(),
                    orchestrator: pattern.orchestrator.clone(),
                    participants: agents.clone(),
                    execution_steps,
                    final_result: accumulated_result,
                    context_evolution,
                    metadata: HashMap::new(),
                })
            }
            
            ExecutionOrder::Parallel(agents) => {
                // Execute all agents in parallel with the same input
                let mut parallel_results = Vec::new();
                
                for agent_id in agents {
                    let step_result = self.execute_agent_step(
                        agent_id,
                        &query,
                        &current_context,
                        agent_executor,
                    ).await?;
                    
                    parallel_results.push(step_result);
                }
                
                // Aggregate results
                let final_result = self.aggregate_parallel_results(&parallel_results);
                execution_steps.extend(parallel_results);
                
                Ok(CompositionResult {
                    pattern_id: pattern_id.to_string(),
                    orchestrator: pattern.orchestrator.clone(),
                    participants: agents.clone(),
                    execution_steps,
                    final_result,
                    context_evolution,
                    metadata: HashMap::new(),
                })
            }
            
            ExecutionOrder::Interactive(interactions) => {
                let mut accumulated_result = query.clone();
                
                for interaction in interactions {
                    // Execute interactive step
                    let step_result = self.execute_interactive_step(
                        interaction,
                        &accumulated_result,
                        &current_context,
                        agent_executor,
                    ).await?;
                    
                    accumulated_result = step_result.output.clone();
                    current_context = step_result.context_snapshot.clone();
                    context_evolution.push(current_context.clone());
                    execution_steps.push(step_result);
                }
                
                Ok(CompositionResult {
                    pattern_id: pattern_id.to_string(),
                    orchestrator: pattern.orchestrator.clone(),
                    participants: pattern.participants.clone(),
                    execution_steps,
                    final_result: accumulated_result,
                    context_evolution,
                    metadata: HashMap::new(),
                })
            }
            
            ExecutionOrder::ConditionalFlow(_flow_steps) => {
                // For now, implement simple fallback to SAGE orchestration
                let step_result = self.execute_agent_step(
                    &pattern.orchestrator,
                    &query,
                    &current_context,
                    agent_executor,
                ).await?;
                
                execution_steps.push(step_result.clone());
                
                Ok(CompositionResult {
                    pattern_id: pattern_id.to_string(),
                    orchestrator: pattern.orchestrator.clone(),
                    participants: pattern.participants.clone(),
                    execution_steps,
                    final_result: step_result.output,
                    context_evolution,
                    metadata: HashMap::new(),
                })
            }
        }
    }
    
    /// Execute a single agent step
    async fn execute_agent_step(
        &self,
        agent_id: &AgentId,
        query: &str,
        context: &AgentContext,
        agent_executor: &dyn AgentExecutor,
    ) -> AgentResult<ExecutionStep> {
        let start_time = std::time::Instant::now();
        
        let result = agent_executor.execute_agent(agent_id, query, context).await?;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(ExecutionStep {
            step_id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.clone(),
            input: query.to_string(),
            output: result.response,
            confidence: result.confidence,
            execution_time_ms: execution_time,
            context_snapshot: result.updated_context,
        })
    }
    
    /// Execute an interactive step between two agents
    async fn execute_interactive_step(
        &self,
        interaction: &InteractionStep,
        query: &str,
        context: &AgentContext,
        agent_executor: &dyn AgentExecutor,
    ) -> AgentResult<ExecutionStep> {
        // For now, execute the initiator agent
        // In a full implementation, this would handle the interaction protocol
        self.execute_agent_step(&interaction.initiator, query, context, agent_executor).await
    }
    
    /// Aggregate results from parallel execution
    fn aggregate_parallel_results(&self, results: &[ExecutionStep]) -> String {
        if results.is_empty() {
            return "No results to aggregate".to_string();
        }
        
        let mut aggregated = String::new();
        aggregated.push_str("## Multi-Agent Collaboration Results\n\n");
        
        for (i, step) in results.iter().enumerate() {
            aggregated.push_str(&format!("### Agent: {}\n", step.agent_id));
            aggregated.push_str(&format!("**Confidence:** {:.1}%\n", step.confidence * 100.0));
            aggregated.push_str(&format!("**Response:**\n{}\n\n", step.output));
            
            if i < results.len() - 1 {
                aggregated.push_str("---\n\n");
            }
        }
        
        aggregated
    }
    
    /// Get available composition patterns
    pub fn get_patterns(&self) -> HashMap<String, CompositionPattern> {
        self.compositions.clone()
    }
}

/// Trait for executing individual agents
#[async_trait::async_trait]
pub trait AgentExecutor {
    async fn execute_agent(
        &self,
        agent_id: &AgentId,
        query: &str,
        context: &AgentContext,
    ) -> AgentResult<AgentExecutionResult>;
}

/// Result of executing a single agent
#[derive(Debug, Clone)]
pub struct AgentExecutionResult {
    pub response: String,
    pub confidence: f64,
    pub updated_context: AgentContext,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for AgentComposer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    struct MockAgentExecutor {
        responses: HashMap<AgentId, String>,
    }
    
    #[async_trait::async_trait]
    impl AgentExecutor for MockAgentExecutor {
        async fn execute_agent(
            &self,
            agent_id: &AgentId,
            query: &str,
            context: &AgentContext,
        ) -> AgentResult<AgentExecutionResult> {
            let response = self.responses.get(agent_id)
                .cloned()
                .unwrap_or_else(|| format!("{} response to: {}", agent_id, query));
            
            Ok(AgentExecutionResult {
                response,
                confidence: 0.8,
                updated_context: context.clone(),
                metadata: HashMap::new(),
            })
        }
    }
    
    #[tokio::test]
    async fn test_sequential_composition() {
        let composer = AgentComposer::new();
        let context = AgentContext::default();
        
        let mut responses = HashMap::new();
        responses.insert("network-expert".to_string(), "Network configured".to_string());
        responses.insert("nats-expert".to_string(), "NATS configured".to_string());
        responses.insert("nix-expert".to_string(), "System configured".to_string());
        
        let executor = MockAgentExecutor { responses };
        
        let result = composer.execute_composition(
            "infrastructure-pipeline",
            "Set up CIM infrastructure".to_string(),
            context,
            &executor,
        ).await.unwrap();
        
        assert_eq!(result.pattern_id, "infrastructure-pipeline");
        assert_eq!(result.execution_steps.len(), 3);
        assert!(result.final_result.contains("System configured"));
    }
    
    #[test]
    fn test_composition_pattern_registration() {
        let composer = AgentComposer::new();
        let patterns = composer.get_patterns();
        
        assert!(patterns.contains_key("sage-orchestration"));
        assert!(patterns.contains_key("domain-collaboration"));
        assert!(patterns.contains_key("infrastructure-pipeline"));
        
        let sage_pattern = &patterns["sage-orchestration"];
        assert_eq!(sage_pattern.orchestrator, "sage");
        assert_eq!(sage_pattern.pattern_type, CompositionType::Hierarchical);
    }
}