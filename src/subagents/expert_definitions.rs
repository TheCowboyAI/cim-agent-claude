/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Expert Agent Definitions
//!
//! Creates concrete implementations of subagents based on the expert agent definitions
//! found in .claude/agents/. This module bridges the CIM subagent system with the
//! Claude expert agents.

use super::{Subagent, SubagentQuery, SubagentResponse, SubagentError, SubagentCapability};
use super::registry::{SubagentInfo, SubagentRegistry};
use super::sage::SageOrchestrator;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;

/// Create a concrete subagent from an agent definition
pub fn create_agent_from_definition(
    info: SubagentInfo,
    registry: Option<Arc<SubagentRegistry>>
) -> Result<Box<dyn Subagent>, Box<dyn std::error::Error>> {
    match info.id.as_str() {
        "sage" => {
            if let Some(reg) = registry {
                Ok(Box::new(SageOrchestrator::new(info, reg)))
            } else {
                Ok(Box::new(SageAgent::new(info)))
            }
        },
        "cim-expert" => Ok(Box::new(CimExpertAgent::new(info))),
        "ddd-expert" => Ok(Box::new(DddExpertAgent::new(info))),
        "event-storming-expert" => Ok(Box::new(EventStormingExpertAgent::new(info))),
        "nats-expert" => Ok(Box::new(NatsExpertAgent::new(info))),
        "network-expert" => Ok(Box::new(NetworkExpertAgent::new(info))),
        "nix-expert" => Ok(Box::new(NixExpertAgent::new(info))),
        "domain-expert" => Ok(Box::new(DomainExpertAgent::new(info))),
        "iced-ui-expert" => Ok(Box::new(IcedUiExpertAgent::new(info))),
        "elm-architecture-expert" => Ok(Box::new(ElmArchitectureExpertAgent::new(info))),
        "cim-tea-ecs-expert" => Ok(Box::new(CimTeaEcsExpertAgent::new(info))),
        "cim-domain-expert" => Ok(Box::new(CimDomainExpertAgent::new(info))),
        _ => Err(format!("Unknown agent type: {}", info.id).into()),
    }
}

/// Base structure for all expert agents
pub struct ExpertAgentBase {
    info: SubagentInfo,
    specialization: AgentSpecialization,
}

/// Specialization details for expert agents
#[derive(Debug, Clone)]
pub struct AgentSpecialization {
    pub domain_expertise: Vec<String>,
    pub tool_usage: Vec<String>,
    pub interaction_patterns: Vec<String>,
    pub mathematical_foundations: Vec<String>,
}

// SAGE - Master Orchestrator Agent
pub struct SageAgent {
    base: ExpertAgentBase,
}

impl SageAgent {
    pub fn new(info: SubagentInfo) -> Self {
        Self {
            base: ExpertAgentBase {
                info,
                specialization: AgentSpecialization {
                    domain_expertise: vec![
                        "master_orchestration".to_string(),
                        "workflow_coordination".to_string(),
                        "expert_routing".to_string(),
                        "multi_agent_synthesis".to_string(),
                    ],
                    tool_usage: vec!["Task".to_string(), "Read".to_string(), "Write".to_string(), "Edit".to_string(), "MultiEdit".to_string(), "Bash".to_string(), "WebFetch".to_string()],
                    interaction_patterns: vec!["proactive_guidance".to_string(), "adaptive_responses".to_string(), "team_coordination".to_string()],
                    mathematical_foundations: vec!["category_theory".to_string(), "workflow_algebra".to_string()],
                },
            },
        }
    }
}

#[async_trait]
impl Subagent for SageAgent {
    fn id(&self) -> &str { &self.base.info.id }
    fn name(&self) -> &str { &self.base.info.name }
    fn description(&self) -> &str { &self.base.info.description }
    fn available_tools(&self) -> Vec<String> { self.base.info.tools.clone() }
    fn capabilities(&self) -> Vec<SubagentCapability> { self.base.info.capabilities.clone() }

    async fn process_query(&self, query: SubagentQuery) -> Result<SubagentResponse, SubagentError> {
        // SAGE processes queries by orchestrating other experts
        let response_text = format!(
            "SAGE orchestration for: {}\n\nBased on your request, I'll coordinate the following workflow:\n1. Analyze your requirements and current CIM development stage\n2. Route to appropriate expert agents\n3. Synthesize results into coherent guidance\n4. Provide next steps and recommendations",
            query.query_text
        );

        Ok(SubagentResponse {
            query_id: query.id,
            subagent_id: self.id().to_string(),
            response_text,
            confidence_score: 0.9,
            recommendations: vec![],
            next_actions: vec![],
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        })
    }

    fn can_handle(&self, _query: &SubagentQuery) -> bool {
        true // SAGE can coordinate any query
    }

    fn priority_score(&self, query: &SubagentQuery) -> u32 {
        // SAGE gets high priority for orchestration keywords
        let orchestration_keywords = ["complete", "full", "workflow", "orchestrate", "coordinate"];
        let text = query.query_text.to_lowercase();
        
        let score = orchestration_keywords.iter()
            .filter(|&keyword| text.contains(keyword))
            .count() as u32;
            
        if score > 0 { 100 + score } else { 50 } // Always available but prioritized for orchestration
    }
}

// CIM Expert Agent - Architecture and Mathematical Foundations
pub struct CimExpertAgent {
    base: ExpertAgentBase,
}

impl CimExpertAgent {
    pub fn new(info: SubagentInfo) -> Self {
        Self {
            base: ExpertAgentBase {
                info,
                specialization: AgentSpecialization {
                    domain_expertise: vec![
                        "category_theory".to_string(),
                        "graph_theory".to_string(),
                        "ipld_patterns".to_string(),
                        "event_sourcing".to_string(),
                        "nats_patterns".to_string(),
                    ],
                    tool_usage: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string(), "WebFetch".to_string()],
                    interaction_patterns: vec!["mathematical_explanations".to_string(), "practical_examples".to_string()],
                    mathematical_foundations: vec!["category_theory".to_string(), "graph_theory".to_string(), "content_addressing".to_string()],
                },
            },
        }
    }
}

#[async_trait]
impl Subagent for CimExpertAgent {
    fn id(&self) -> &str { &self.base.info.id }
    fn name(&self) -> &str { &self.base.info.name }
    fn description(&self) -> &str { &self.base.info.description }
    fn available_tools(&self) -> Vec<String> { self.base.info.tools.clone() }
    fn capabilities(&self) -> Vec<SubagentCapability> { self.base.info.capabilities.clone() }

    async fn process_query(&self, query: SubagentQuery) -> Result<SubagentResponse, SubagentError> {
        let response_text = format!(
            "CIM Expert Analysis for: {}\n\nLet me explain the CIM architectural principles and mathematical foundations relevant to your query. This involves Category Theory, Graph Theory, and IPLD content addressing patterns.",
            query.query_text
        );

        Ok(SubagentResponse {
            query_id: query.id,
            subagent_id: self.id().to_string(),
            response_text,
            confidence_score: 0.85,
            recommendations: vec![],
            next_actions: vec![],
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        })
    }

    fn can_handle(&self, query: &SubagentQuery) -> bool {
        let architecture_keywords = ["architecture", "design", "pattern", "mathematical", "category", "graph", "cim"];
        let text = query.query_text.to_lowercase();
        architecture_keywords.iter().any(|&keyword| text.contains(keyword))
    }

    fn priority_score(&self, query: &SubagentQuery) -> u32 {
        let keywords = ["architecture", "mathematical", "category theory", "cim", "design"];
        let text = query.query_text.to_lowercase();
        keywords.iter().filter(|&k| text.contains(k)).count() as u32 * 20
    }
}

// DDD Expert Agent - Domain-Driven Design
pub struct DddExpertAgent {
    base: ExpertAgentBase,
}

impl DddExpertAgent {
    pub fn new(info: SubagentInfo) -> Self {
        Self {
            base: ExpertAgentBase {
                info,
                specialization: AgentSpecialization {
                    domain_expertise: vec![
                        "domain_boundaries".to_string(),
                        "aggregate_design".to_string(),
                        "entity_modeling".to_string(),
                        "value_objects".to_string(),
                        "state_machines".to_string(),
                    ],
                    tool_usage: vec!["Task".to_string(), "Read".to_string(), "Write".to_string(), "Edit".to_string(), "MultiEdit".to_string(), "Bash".to_string(), "WebFetch".to_string()],
                    interaction_patterns: vec!["domain_analysis".to_string(), "boundary_identification".to_string()],
                    mathematical_foundations: vec!["domain_algebra".to_string(), "invariant_preservation".to_string()],
                },
            },
        }
    }
}

#[async_trait]
impl Subagent for DddExpertAgent {
    fn id(&self) -> &str { &self.base.info.id }
    fn name(&self) -> &str { &self.base.info.name }
    fn description(&self) -> &str { &self.base.info.description }
    fn available_tools(&self) -> Vec<String> { self.base.info.tools.clone() }
    fn capabilities(&self) -> Vec<SubagentCapability> { self.base.info.capabilities.clone() }

    async fn process_query(&self, query: SubagentQuery) -> Result<SubagentResponse, SubagentError> {
        let response_text = format!(
            "DDD Expert Analysis for: {}\n\nI'll help you with domain-driven design analysis, including boundary identification, aggregate design, and state machine modeling using CIM framework principles.",
            query.query_text
        );

        Ok(SubagentResponse {
            query_id: query.id,
            subagent_id: self.id().to_string(),
            response_text,
            confidence_score: 0.8,
            recommendations: vec![],
            next_actions: vec![],
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        })
    }

    fn can_handle(&self, query: &SubagentQuery) -> bool {
        let ddd_keywords = ["domain", "ddd", "aggregate", "entity", "boundary", "modeling"];
        let text = query.query_text.to_lowercase();
        ddd_keywords.iter().any(|&keyword| text.contains(keyword))
    }

    fn priority_score(&self, query: &SubagentQuery) -> u32 {
        let keywords = ["domain", "ddd", "aggregate", "boundary", "modeling"];
        let text = query.query_text.to_lowercase();
        keywords.iter().filter(|&k| text.contains(k)).count() as u32 * 25
    }
}

// Macro to generate similar expert agents with different specializations
macro_rules! create_expert_agent {
    ($name:ident, $id:literal, $keywords:expr, $expertise:expr) => {
        pub struct $name {
            base: ExpertAgentBase,
        }

        impl $name {
            pub fn new(info: SubagentInfo) -> Self {
                Self {
                    base: ExpertAgentBase {
                        info,
                        specialization: AgentSpecialization {
                            domain_expertise: $expertise.iter().map(|s| s.to_string()).collect(),
                            tool_usage: vec!["Task".to_string(), "Read".to_string(), "Write".to_string()],
                            interaction_patterns: vec!["expert_guidance".to_string()],
                            mathematical_foundations: vec!["domain_specific".to_string()],
                        },
                    },
                }
            }
        }

        #[async_trait]
        impl Subagent for $name {
            fn id(&self) -> &str { &self.base.info.id }
            fn name(&self) -> &str { &self.base.info.name }
            fn description(&self) -> &str { &self.base.info.description }
            fn available_tools(&self) -> Vec<String> { self.base.info.tools.clone() }
            fn capabilities(&self) -> Vec<SubagentCapability> { self.base.info.capabilities.clone() }

            async fn process_query(&self, query: SubagentQuery) -> Result<SubagentResponse, SubagentError> {
                let response_text = format!(
                    "{} Expert Analysis for: {}\n\nProviding specialized guidance in my domain of expertise.",
                    self.name(),
                    query.query_text
                );

                Ok(SubagentResponse {
                    query_id: query.id,
                    subagent_id: self.id().to_string(),
                    response_text,
                    confidence_score: 0.8,
                    recommendations: vec![],
                    next_actions: vec![],
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                })
            }

            fn can_handle(&self, query: &SubagentQuery) -> bool {
                let text = query.query_text.to_lowercase();
                $keywords.iter().any(|keyword| text.contains(keyword))
            }

            fn priority_score(&self, query: &SubagentQuery) -> u32 {
                let text = query.query_text.to_lowercase();
                ($keywords.iter().filter(|k| text.contains(k)).count() as u32) * 20
            }
        }
    };
}

// Generate expert agent implementations
create_expert_agent!(EventStormingExpertAgent, "event-storming-expert", 
    &["event", "storming", "workshop", "collaboration", "discovery"],
    &["event_discovery", "facilitation", "collaborative_sessions"]
);

create_expert_agent!(NatsExpertAgent, "nats-expert",
    &["nats", "messaging", "jetstream", "streams", "broker"],
    &["message_infrastructure", "jetstream_config", "security"]
);

create_expert_agent!(NetworkExpertAgent, "network-expert",
    &["network", "topology", "routing", "connectivity", "infrastructure"],
    &["network_design", "topology_planning", "secure_pathways"]
);

create_expert_agent!(NixExpertAgent, "nix-expert",
    &["nix", "configuration", "declarative", "system", "flake"],
    &["system_configuration", "declarative_infrastructure", "nix_patterns"]
);

create_expert_agent!(DomainExpertAgent, "domain-expert",
    &["domain", "creation", "graph", "validation", "implementation"],
    &["domain_creation", "graph_generation", "mathematical_validation"]
);

create_expert_agent!(IcedUiExpertAgent, "iced-ui-expert",
    &["ui", "iced", "interface", "gui", "reactive"],
    &["ui_development", "reactive_patterns", "cross_platform"]
);

create_expert_agent!(ElmArchitectureExpertAgent, "elm-architecture-expert",
    &["elm", "architecture", "functional", "reactive", "tea"],
    &["functional_architecture", "model_view_update", "immutable_state"]
);

create_expert_agent!(CimTeaEcsExpertAgent, "cim-tea-ecs-expert",
    &["tea", "ecs", "entity", "component", "performance"],
    &["tea_ecs_integration", "performance_optimization", "entity_systems"]
);

create_expert_agent!(CimDomainExpertAgent, "cim-domain-expert",
    &["domain", "cim", "modeling", "boundaries", "composition"],
    &["cim_domain_patterns", "boundary_analysis", "composition_design"]
);

create_expert_agent!(QaExpertAgent, "qa-expert",
    &["quality", "assurance", "compliance", "audit", "validation", "rules"],
    &["compliance_analysis", "rule_enforcement", "quality_validation"]
);

create_expert_agent!(BddExpertAgent, "bdd-expert", 
    &["bdd", "behavior", "gherkin", "scenarios", "acceptance", "user", "story"],
    &["behavior_driven_development", "user_story_creation", "acceptance_criteria"]
);

create_expert_agent!(TddExpertAgent, "tdd-expert",
    &["tdd", "test", "driven", "unit", "testing", "red", "green", "refactor"],
    &["test_driven_development", "unit_testing", "test_creation"]
);

create_expert_agent!(GitExpertAgent, "git-expert",
    &["git", "github", "repository", "version", "control", "workflow", "branch"],
    &["git_operations", "github_integration", "repository_management"]
);

create_expert_agent!(SubjectExpertAgent, "subject-expert",
    &["subject", "algebra", "routing", "patterns", "nats", "hierarchies"],
    &["subject_design", "routing_patterns", "subject_optimization"]
);