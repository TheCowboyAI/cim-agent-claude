/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Subject Algebra Router
//!
//! Provides query resolution through CIM subject algebra manipulation.
//! Maps user queries to appropriate domain experts by operating on NATS subject
//! hierarchies that represent the mathematical structure of CIM domains and capabilities.

use super::{SubagentQuery, SubagentCapability, TaskType, ComplexityLevel, SubagentError};
use super::registry::SubagentRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Subject algebra resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectResolution {
    pub primary_subject: String,
    pub secondary_subjects: Vec<String>,
    pub resolution_strategy: ResolutionStrategy,
    pub algebraic_confidence: f64,
    pub subject_path: String,
    pub domain_context: DomainContext,
    pub requires_orchestration: bool,
}

/// CIM domain context for subject resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainContext {
    pub domain_type: DomainType,
    pub capability_space: Vec<String>,
    pub mathematical_properties: MathematicalProperties,
    pub composition_requirements: Vec<String>,
}

/// CIM subject algebra resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResolutionStrategy {
    /// Direct subject match to single domain expert
    DirectSubject,
    /// Compositional resolution across multiple domains
    Compositional,
    /// Hierarchical resolution through parent domains
    Hierarchical,
    /// Orchestrated resolution through SAGE subject space
    Orchestrated,
    /// Collaborative resolution for multi-domain queries
    Collaborative,
}

/// Routing decision for query execution
#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub primary_agent: String,
    pub secondary_agents: Vec<String>,
    pub execution_strategy: ExecutionStrategy,
    pub confidence_score: f64,
    pub resolution_strategy: ResolutionStrategy,
}

/// Strategy for executing the routing decision
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStrategy {
    /// Execute with single agent
    Single,
    /// Execute with multiple agents sequentially
    Sequential,
    /// Execute with multiple agents in parallel
    Parallel,
    /// Execute through orchestration (SAGE)
    Orchestrated,
    /// Execute with collaborative facilitation
    Collaborative,
}

/// CIM domain types for subject algebra
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DomainType {
    Architecture,
    DomainModeling,
    Infrastructure,
    EventSourcing,
    Configuration,
    NetworkTopology,
    UserInterface,
    Orchestration,
    Collaboration,
}

/// Mathematical properties of CIM domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathematicalProperties {
    pub category_structure: CategoryStructure,
    pub morphism_type: MorphismType,
    pub composition_laws: Vec<CompositionLaw>,
    pub invariants: Vec<String>,
}

/// Category theory structure of domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStructure {
    pub objects: Vec<String>,
    pub morphisms: Vec<String>,
    pub identity_morphisms: Vec<String>,
    pub associativity_properties: Vec<String>,
}

/// Types of morphisms in CIM domain algebra
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MorphismType {
    Identity,
    Composition,
    Functor,
    NaturalTransformation,
}

/// Composition laws for domain algebra
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionLaw {
    pub name: String,
    pub description: String,
    pub formula: String,
    pub constraints: Vec<String>,
}

/// CIM Subject Algebra Router
pub struct SubagentRouter {
    registry: Arc<SubagentRegistry>,
    subject_hierarchy: SubjectHierarchy,
    domain_algebra: DomainAlgebra,
    capability_mappings: HashMap<String, DomainType>,
}

/// NATS subject hierarchy representing CIM domain structure
#[derive(Debug, Clone)]
pub struct SubjectHierarchy {
    /// Root subject for all CIM operations
    pub root: String,
    /// Domain-specific subject branches
    pub domains: HashMap<DomainType, DomainSubjects>,
    /// Cross-domain composition subjects
    pub compositions: Vec<CompositionSubject>,
    /// Orchestration subjects for multi-domain operations
    pub orchestration: OrchestrationSubjects,
}

/// Domain-specific NATS subjects
#[derive(Debug, Clone)]
pub struct DomainSubjects {
    pub command: String,
    pub query: String,
    pub event: String,
    pub state: String,
    pub composition: String,
}

/// Cross-domain composition subjects
#[derive(Debug, Clone)]
pub struct CompositionSubject {
    pub name: String,
    pub subject: String,
    pub participating_domains: Vec<DomainType>,
    pub composition_law: CompositionLaw,
}

/// Orchestration subjects for SAGE coordination
#[derive(Debug, Clone)]
pub struct OrchestrationSubjects {
    pub workflow: String,
    pub coordination: String,
    pub synthesis: String,
    pub validation: String,
}

/// Domain algebra for mathematical operations on CIM domains
#[derive(Debug, Clone)]
pub struct DomainAlgebra {
    /// Category theory operations
    pub categories: HashMap<DomainType, CategoryStructure>,
    /// Morphism definitions between domains  
    pub morphisms: HashMap<String, DomainMorphism>,
    /// Composition operations
    pub compositions: HashMap<String, CompositionOperation>,
    /// Functor mappings between domain categories
    pub functors: HashMap<String, DomainFunctor>,
}

/// Morphism between CIM domains
#[derive(Debug, Clone)]
pub struct DomainMorphism {
    pub name: String,
    pub source: DomainType,
    pub target: DomainType,
    pub transformation: MorphismTransformation,
    pub preserves_structure: bool,
}

/// Mathematical transformation function for morphisms
#[derive(Debug, Clone)]
pub struct MorphismTransformation {
    pub function_type: String,
    pub parameters: HashMap<String, String>,
    pub constraints: Vec<String>,
}

/// Composition operation in domain algebra
#[derive(Debug, Clone)]
pub struct CompositionOperation {
    pub name: String,
    pub operands: Vec<DomainType>,
    pub result: DomainType,
    pub law: CompositionLaw,
}

/// Functor mapping between domain categories
#[derive(Debug, Clone)]
pub struct DomainFunctor {
    pub name: String,
    pub source_category: DomainType,
    pub target_category: DomainType,
    pub object_mapping: HashMap<String, String>,
    pub morphism_mapping: HashMap<String, String>,
}

/// Rules for routing queries
#[derive(Debug, Clone)]
pub struct RoutingRule {
    pub name: String,
    pub condition: RoutingCondition,
    pub action: RoutingAction,
    pub priority: u32,
}

/// Conditions for routing rules
#[derive(Debug, Clone)]
pub enum RoutingCondition {
    KeywordMatch(Vec<String>),
    CapabilityRequired(SubagentCapability),
    TaskTypeMatch(TaskType),
    ComplexityThreshold(ComplexityLevel),
    MultiExpertNeeded,
    UserExplicitRequest(String), // User asks for specific agent
}

/// Actions to take when routing conditions are met
#[derive(Debug, Clone)]
pub enum RoutingAction {
    RouteToAgent(String),
    RouteToCapability(SubagentCapability),
    UseStrategy(ResolutionStrategy),
    RequireOrchestration,
}

/// Intent patterns for query analysis
#[derive(Debug, Clone)]
pub struct IntentPattern {
    pub name: String,
    pub keywords: Vec<String>,
    pub phrases: Vec<String>,
    pub capabilities: Vec<SubagentCapability>,
    pub suggested_strategy: ResolutionStrategy,
    pub complexity_hint: ComplexityLevel,
}

impl SubagentRouter {
    /// Create a new CIM subject algebra router
    pub fn new(registry: Arc<SubagentRegistry>) -> Self {
        let router = Self {
            registry,
            subject_hierarchy: Self::initialize_cim_subject_hierarchy(),
            domain_algebra: Self::initialize_domain_algebra(),
            capability_mappings: Self::initialize_capability_mappings(),
        };
        
        router
    }

    /// Initialize CIM subject hierarchy following established patterns
    fn initialize_cim_subject_hierarchy() -> SubjectHierarchy {
        let mut domains = HashMap::new();
        
        // Architecture domain subjects
        domains.insert(DomainType::Architecture, DomainSubjects {
            command: "cim.architecture.cmd.>".to_string(),
            query: "cim.architecture.qry.>".to_string(),
            event: "cim.architecture.evt.>".to_string(),
            state: "cim.architecture.state.>".to_string(),
            composition: "cim.architecture.comp.>".to_string(),
        });
        
        // Domain modeling subjects
        domains.insert(DomainType::DomainModeling, DomainSubjects {
            command: "cim.domain.cmd.>".to_string(),
            query: "cim.domain.qry.>".to_string(),
            event: "cim.domain.evt.>".to_string(),
            state: "cim.domain.state.>".to_string(),
            composition: "cim.domain.comp.>".to_string(),
        });
        
        // Infrastructure domain subjects
        domains.insert(DomainType::Infrastructure, DomainSubjects {
            command: "cim.infra.cmd.>".to_string(),
            query: "cim.infra.qry.>".to_string(),
            event: "cim.infra.evt.>".to_string(),
            state: "cim.infra.state.>".to_string(),
            composition: "cim.infra.comp.>".to_string(),
        });
        
        // Network topology subjects
        domains.insert(DomainType::NetworkTopology, DomainSubjects {
            command: "cim.network.cmd.>".to_string(),
            query: "cim.network.qry.>".to_string(),
            event: "cim.network.evt.>".to_string(),
            state: "cim.network.state.>".to_string(),
            composition: "cim.network.comp.>".to_string(),
        });
        
        // Event sourcing subjects
        domains.insert(DomainType::EventSourcing, DomainSubjects {
            command: "cim.events.cmd.>".to_string(),
            query: "cim.events.qry.>".to_string(),
            event: "cim.events.evt.>".to_string(),
            state: "cim.events.state.>".to_string(),
            composition: "cim.events.comp.>".to_string(),
        });
        
        // Configuration domain subjects
        domains.insert(DomainType::Configuration, DomainSubjects {
            command: "cim.config.cmd.>".to_string(),
            query: "cim.config.qry.>".to_string(),
            event: "cim.config.evt.>".to_string(),
            state: "cim.config.state.>".to_string(),
            composition: "cim.config.comp.>".to_string(),
        });
        
        // User interface subjects
        domains.insert(DomainType::UserInterface, DomainSubjects {
            command: "cim.ui.cmd.>".to_string(),
            query: "cim.ui.qry.>".to_string(),
            event: "cim.ui.evt.>".to_string(),
            state: "cim.ui.state.>".to_string(),
            composition: "cim.ui.comp.>".to_string(),
        });
        
        // Orchestration subjects
        domains.insert(DomainType::Orchestration, DomainSubjects {
            command: "cim.sage.cmd.>".to_string(),
            query: "cim.sage.qry.>".to_string(),
            event: "cim.sage.evt.>".to_string(),
            state: "cim.sage.state.>".to_string(),
            composition: "cim.sage.comp.>".to_string(),
        });
        
        // Collaboration subjects  
        domains.insert(DomainType::Collaboration, DomainSubjects {
            command: "cim.collab.cmd.>".to_string(),
            query: "cim.collab.qry.>".to_string(),
            event: "cim.collab.evt.>".to_string(),
            state: "cim.collab.state.>".to_string(),
            composition: "cim.collab.comp.>".to_string(),
        });

        // Cross-domain compositions using established CIM patterns
        let compositions = vec![
            CompositionSubject {
                name: "Architecture + Domain Modeling".to_string(),
                subject: "cim.comp.arch_domain.>".to_string(),
                participating_domains: vec![DomainType::Architecture, DomainType::DomainModeling],
                composition_law: CompositionLaw {
                    name: "Structure Preserving Architecture".to_string(),
                    description: "Domain models must preserve architectural invariants".to_string(),
                    formula: "F(A ∘ D) = F(A) ∘ F(D)".to_string(),
                    constraints: vec!["bounded_context_integrity".to_string(), "event_flow_preservation".to_string()],
                },
            },
            CompositionSubject {
                name: "Infrastructure + Network".to_string(),
                subject: "cim.comp.infra_network.>".to_string(),
                participating_domains: vec![DomainType::Infrastructure, DomainType::NetworkTopology],
                composition_law: CompositionLaw {
                    name: "Network Infrastructure Composition".to_string(),
                    description: "Network topology must align with infrastructure capabilities".to_string(),
                    formula: "N ∘ I → Service_Topology".to_string(),
                    constraints: vec!["bandwidth_constraints".to_string(), "latency_requirements".to_string()],
                },
            },
        ];
        
        SubjectHierarchy {
            root: "cim".to_string(),
            domains,
            compositions,
            orchestration: OrchestrationSubjects {
                workflow: "cim.sage.workflow.>".to_string(),
                coordination: "cim.sage.coord.>".to_string(),
                synthesis: "cim.sage.synthesis.>".to_string(),
                validation: "cim.sage.validate.>".to_string(),
            },
        }
    }

    /// Initialize domain algebra with Category Theory structures
    fn initialize_domain_algebra() -> DomainAlgebra {
        let mut categories = HashMap::new();
        let mut morphisms = HashMap::new();
        let mut compositions = HashMap::new();
        let mut functors = HashMap::new();

        // Define category structures for each domain type
        categories.insert(DomainType::Architecture, CategoryStructure {
            objects: vec!["System".to_string(), "Component".to_string(), "Interface".to_string(), "Pattern".to_string()],
            morphisms: vec!["dependency".to_string(), "composition".to_string(), "abstraction".to_string()],
            identity_morphisms: vec!["id_system".to_string(), "id_component".to_string()],
            associativity_properties: vec!["(A ∘ B) ∘ C = A ∘ (B ∘ C)".to_string()],
        });

        categories.insert(DomainType::DomainModeling, CategoryStructure {
            objects: vec!["Entity".to_string(), "ValueObject".to_string(), "Aggregate".to_string(), "Service".to_string()],
            morphisms: vec!["aggregation".to_string(), "association".to_string(), "composition".to_string()],
            identity_morphisms: vec!["id_entity".to_string(), "id_aggregate".to_string()],
            associativity_properties: vec!["aggregate_composition_associative".to_string()],
        });

        // Define morphisms between domains
        morphisms.insert("arch_to_domain".to_string(), DomainMorphism {
            name: "Architecture to Domain Mapping".to_string(),
            source: DomainType::Architecture,
            target: DomainType::DomainModeling,
            transformation: MorphismTransformation {
                function_type: "structure_preserving".to_string(),
                parameters: vec![("preserve_boundaries".to_string(), "true".to_string())].into_iter().collect(),
                constraints: vec!["maintain_invariants".to_string()],
            },
            preserves_structure: true,
        });

        // Define composition operations
        compositions.insert("full_stack_composition".to_string(), CompositionOperation {
            name: "Full Stack CIM Composition".to_string(),
            operands: vec![DomainType::Architecture, DomainType::DomainModeling, DomainType::Infrastructure],
            result: DomainType::Orchestration,
            law: CompositionLaw {
                name: "CIM Composition Law".to_string(),
                description: "Complete CIM assembly preserving all mathematical properties".to_string(),
                formula: "CIM = Architecture ∘ Domain ∘ Infrastructure".to_string(),
                constraints: vec!["event_driven".to_string(), "immutable_events".to_string()],
            },
        });

        // Define functors between categories
        functors.insert("domain_to_events".to_string(), DomainFunctor {
            name: "Domain to Event Functor".to_string(),
            source_category: DomainType::DomainModeling,
            target_category: DomainType::EventSourcing,
            object_mapping: vec![
                ("Entity".to_string(), "EntityEvent".to_string()),
                ("Aggregate".to_string(), "AggregateEvent".to_string()),
            ].into_iter().collect(),
            morphism_mapping: vec![
                ("aggregation".to_string(), "event_composition".to_string()),
            ].into_iter().collect(),
        });

        DomainAlgebra {
            categories,
            morphisms,
            compositions,
            functors,
        }
    }

    /// Initialize capability mappings from agent names to domain types
    fn initialize_capability_mappings() -> HashMap<String, DomainType> {
        vec![
            ("sage".to_string(), DomainType::Orchestration),
            ("cim-expert".to_string(), DomainType::Architecture),
            ("ddd-expert".to_string(), DomainType::DomainModeling),
            ("event-storming-expert".to_string(), DomainType::Collaboration),
            ("nats-expert".to_string(), DomainType::Infrastructure),
            ("network-expert".to_string(), DomainType::NetworkTopology),
            ("nix-expert".to_string(), DomainType::Configuration),
            ("domain-expert".to_string(), DomainType::DomainModeling),
            ("iced-ui-expert".to_string(), DomainType::UserInterface),
            ("elm-architecture-expert".to_string(), DomainType::Architecture),
            ("cim-tea-ecs-expert".to_string(), DomainType::Architecture),
            ("cim-domain-expert".to_string(), DomainType::DomainModeling),
        ].into_iter().collect()
    }

    /// Resolve query through CIM subject algebra manipulation
    pub async fn resolve_query(&self, query: &SubagentQuery) -> Result<SubjectResolution, SubagentError> {
        info!("Resolving query through subject algebra: {}", query.query_text.chars().take(100).collect::<String>());

        // Step 1: Analyze query intent and map to domain space
        let domain_analysis = self.analyze_query_domain(query).await?;
        debug!("Domain analysis: {:?}", domain_analysis);

        // Step 2: Apply subject algebra operations to find optimal path
        let subject_path = self.compute_subject_path(&domain_analysis)?;
        debug!("Subject path computed: {}", subject_path);

        // Step 3: Resolve to concrete agents through domain algebra
        let resolution = self.apply_domain_algebra(&subject_path, &domain_analysis).await?;

        Ok(resolution)
    }

    /// Analyze query and map to CIM domain space
    async fn analyze_query_domain(&self, query: &SubagentQuery) -> Result<DomainAnalysis, SubagentError> {
        let text = query.query_text.to_lowercase();
        let mut domain_scores: HashMap<DomainType, f64> = HashMap::new();

        // Use mathematical domain signatures for analysis
        let domain_signatures = self.get_domain_signatures();
        
        for (domain_type, signatures) in domain_signatures {
            let mut score = 0.0;
            for signature in &signatures {
                if text.contains(signature.as_str()) {
                    score += 1.0;
                }
            }
            
            // Normalize by signature count
            if score > 0.0 {
                domain_scores.insert(domain_type, score / signatures.len() as f64);
            }
        }

        // Find primary domain with highest score
        let primary_domain = domain_scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(domain, _)| domain.clone())
            .unwrap_or(DomainType::Architecture); // Default fallback

        // Identify secondary domains for potential composition
        let mut secondary_domains: Vec<DomainType> = domain_scores.iter()
            .filter(|(domain, score)| **domain != primary_domain && **score > 0.3)
            .map(|(domain, _)| domain.clone())
            .collect();
        secondary_domains.sort_by(|a, b| 
            domain_scores.get(b).unwrap_or(&0.0)
                .partial_cmp(domain_scores.get(a).unwrap_or(&0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        );

        // Determine if composition is needed
        let requires_composition = secondary_domains.len() > 0 || 
            text.contains("complete") || text.contains("full") || text.contains("end-to-end");

        Ok(DomainAnalysis {
            primary_domain,
            secondary_domains,
            domain_scores,
            requires_composition,
            complexity_indicators: self.extract_complexity_indicators(&text),
        })
    }

    /// Get domain signatures for mathematical analysis
    fn get_domain_signatures(&self) -> HashMap<DomainType, Vec<String>> {
        vec![
            (DomainType::Architecture, vec![
                "architecture".to_string(), "design".to_string(), "pattern".to_string(), 
                "system".to_string(), "component".to_string(), "interface".to_string(),
                "category theory".to_string(), "mathematical".to_string(),
            ]),
            (DomainType::DomainModeling, vec![
                "domain".to_string(), "ddd".to_string(), "aggregate".to_string(),
                "entity".to_string(), "value object".to_string(), "bounded context".to_string(),
                "business".to_string(), "invariant".to_string(),
            ]),
            (DomainType::Infrastructure, vec![
                "infrastructure".to_string(), "deployment".to_string(), "nats".to_string(),
                "jetstream".to_string(), "server".to_string(), "service".to_string(),
                "messaging".to_string(), "broker".to_string(),
            ]),
            (DomainType::NetworkTopology, vec![
                "network".to_string(), "topology".to_string(), "routing".to_string(),
                "connectivity".to_string(), "pathway".to_string(), "cluster".to_string(),
            ]),
            (DomainType::EventSourcing, vec![
                "event".to_string(), "sourcing".to_string(), "stream".to_string(),
                "immutable".to_string(), "append".to_string(), "projection".to_string(),
            ]),
            (DomainType::Configuration, vec![
                "config".to_string(), "nix".to_string(), "declarative".to_string(),
                "system".to_string(), "setup".to_string(), "environment".to_string(),
            ]),
            (DomainType::UserInterface, vec![
                "ui".to_string(), "interface".to_string(), "gui".to_string(),
                "iced".to_string(), "reactive".to_string(), "component".to_string(),
            ]),
            (DomainType::Orchestration, vec![
                "orchestration".to_string(), "workflow".to_string(), "coordination".to_string(),
                "sage".to_string(), "complete".to_string(), "full".to_string(),
            ]),
            (DomainType::Collaboration, vec![
                "collaboration".to_string(), "team".to_string(), "workshop".to_string(),
                "storming".to_string(), "facilitation".to_string(), "session".to_string(),
            ]),
        ].into_iter().collect()
    }

    /// Extract complexity indicators from query text
    fn extract_complexity_indicators(&self, text: &str) -> Vec<String> {
        let indicators = vec![
            "complete", "full", "entire", "comprehensive", "end-to-end",
            "multiple", "several", "many", "complex", "advanced",
            "team", "organization", "enterprise", "production",
        ];
        
        indicators.iter()
            .filter(|&indicator| text.contains(indicator))
            .map(|s| s.to_string())
            .collect()
    }

    /// Compute NATS subject path using CIM subject algebra
    fn compute_subject_path(&self, analysis: &DomainAnalysis) -> Result<String, SubagentError> {
        let primary_subjects = self.subject_hierarchy.domains
            .get(&analysis.primary_domain)
            .ok_or_else(|| SubagentError::NotFound(format!("No subjects for domain {:?}", analysis.primary_domain)))?;

        // For simple queries, use direct query subject
        if !analysis.requires_composition && analysis.secondary_domains.is_empty() {
            return Ok(primary_subjects.query.clone());
        }

        // For composition queries, check if we have defined composition subjects
        for composition in &self.subject_hierarchy.compositions {
            let domains_match = composition.participating_domains.contains(&analysis.primary_domain);
            if domains_match {
                for secondary_domain in &analysis.secondary_domains {
                    if composition.participating_domains.contains(secondary_domain) {
                        return Ok(composition.subject.clone());
                    }
                }
            }
        }

        // If no specific composition found, use orchestration
        if analysis.requires_composition || analysis.secondary_domains.len() > 1 {
            return Ok(self.subject_hierarchy.orchestration.workflow.clone());
        }

        // Default to primary domain query subject
        Ok(primary_subjects.query.clone())
    }

    /// Apply domain algebra to resolve subject path to concrete agents
    async fn apply_domain_algebra(&self, subject_path: &str, analysis: &DomainAnalysis) -> Result<SubjectResolution, SubagentError> {
        // Find primary agent based on domain type
        let primary_agent = self.find_agent_for_domain(&analysis.primary_domain).await?;
        
        // Find secondary agents for composition scenarios
        let mut secondary_agents = Vec::new();
        for secondary_domain in &analysis.secondary_domains {
            if let Ok(agent) = self.find_agent_for_domain(secondary_domain).await {
                secondary_agents.push(agent);
            }
        }

        // Determine resolution strategy based on mathematical properties
        let strategy = if analysis.requires_composition {
            if analysis.secondary_domains.len() > 1 {
                ResolutionStrategy::Orchestrated
            } else {
                ResolutionStrategy::Compositional
            }
        } else {
            ResolutionStrategy::DirectSubject
        };

        // Calculate algebraic confidence based on domain scores
        let algebraic_confidence = analysis.domain_scores.values().sum::<f64>() / analysis.domain_scores.len() as f64;

        // Build domain context
        let domain_context = DomainContext {
            domain_type: analysis.primary_domain.clone(),
            capability_space: self.get_capability_space(&analysis.primary_domain),
            mathematical_properties: self.get_mathematical_properties(&analysis.primary_domain),
            composition_requirements: self.get_composition_requirements(analysis),
        };

        Ok(SubjectResolution {
            primary_subject: primary_agent,
            secondary_subjects: secondary_agents,
            resolution_strategy: strategy.clone(),
            algebraic_confidence,
            subject_path: subject_path.to_string(),
            domain_context,
            requires_orchestration: matches!(strategy, ResolutionStrategy::Orchestrated),
        })
    }

    /// Find agent for a specific domain type
    async fn find_agent_for_domain(&self, domain_type: &DomainType) -> Result<String, SubagentError> {
        // Find agents that match this domain type
        let matching_agents: Vec<String> = self.capability_mappings.iter()
            .filter_map(|(agent_id, agent_domain)| {
                if agent_domain == domain_type {
                    Some(agent_id.clone())
                } else {
                    None
                }
            })
            .collect();

        if matching_agents.is_empty() {
            return Err(SubagentError::NotFound(format!("No agents available for domain {:?}", domain_type)));
        }

        // For now, return the first matching agent
        // In a more sophisticated implementation, we could score agents based on specific capabilities
        Ok(matching_agents[0].clone())
    }

    /// Get capability space for a domain
    fn get_capability_space(&self, domain_type: &DomainType) -> Vec<String> {
        match domain_type {
            DomainType::Architecture => vec!["design".to_string(), "patterns".to_string(), "mathematics".to_string()],
            DomainType::DomainModeling => vec!["boundaries".to_string(), "aggregates".to_string(), "entities".to_string()],
            DomainType::Infrastructure => vec!["messaging".to_string(), "deployment".to_string(), "scaling".to_string()],
            DomainType::NetworkTopology => vec!["routing".to_string(), "topology".to_string(), "security".to_string()],
            DomainType::EventSourcing => vec!["events".to_string(), "streams".to_string(), "projections".to_string()],
            DomainType::Configuration => vec!["declarative".to_string(), "immutable".to_string(), "reproducible".to_string()],
            DomainType::UserInterface => vec!["reactive".to_string(), "components".to_string(), "state".to_string()],
            DomainType::Orchestration => vec!["coordination".to_string(), "workflow".to_string(), "synthesis".to_string()],
            DomainType::Collaboration => vec!["facilitation".to_string(), "discovery".to_string(), "alignment".to_string()],
        }
    }

    /// Get mathematical properties for a domain
    fn get_mathematical_properties(&self, domain_type: &DomainType) -> MathematicalProperties {
        if let Some(category_structure) = self.domain_algebra.categories.get(domain_type) {
            MathematicalProperties {
                category_structure: category_structure.clone(),
                morphism_type: MorphismType::Identity, // Default for single domain
                composition_laws: vec![],
                invariants: vec!["structure_preservation".to_string()],
            }
        } else {
            // Default mathematical properties
            MathematicalProperties {
                category_structure: CategoryStructure {
                    objects: vec!["default".to_string()],
                    morphisms: vec!["identity".to_string()],
                    identity_morphisms: vec!["id".to_string()],
                    associativity_properties: vec![],
                },
                morphism_type: MorphismType::Identity,
                composition_laws: vec![],
                invariants: vec![],
            }
        }
    }

    /// Get composition requirements for domain analysis
    fn get_composition_requirements(&self, analysis: &DomainAnalysis) -> Vec<String> {
        let mut requirements = Vec::new();
        
        if analysis.requires_composition {
            requirements.push("preserve_domain_boundaries".to_string());
            requirements.push("maintain_event_flow".to_string());
        }
        
        if analysis.secondary_domains.len() > 1 {
            requirements.push("coordinate_multiple_domains".to_string());
        }
        
        if analysis.complexity_indicators.contains(&"enterprise".to_string()) {
            requirements.push("scalability_requirements".to_string());
            requirements.push("security_compliance".to_string());
        }
        
        requirements
    }
}

/// Domain analysis results for CIM subject algebra
#[derive(Debug, Clone)]
pub struct DomainAnalysis {
    pub primary_domain: DomainType,
    pub secondary_domains: Vec<DomainType>,
    pub domain_scores: HashMap<DomainType, f64>,
    pub requires_composition: bool,
    pub complexity_indicators: Vec<String>,
}
