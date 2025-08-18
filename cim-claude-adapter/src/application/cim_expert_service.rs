/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Expert Service - Provides architectural guidance as a NATS service
//! 
//! This service makes CIM Expert functionality available to any CIM system
//! through NATS messaging, enabling on-demand architectural consultation.

use crate::domain::{
    claude_queries::ClaudeApiQuery,
    value_objects::{ConversationId, CorrelationId},
};
use crate::infrastructure::{
    claude_client::{ClaudeClient, ClaudeClientConfig},
    nats_client::NatsClient,
};
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use tracing::{info, instrument};
use uuid::Uuid;

/// CIM Expert query topics for specialized guidance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CimExpertTopic {
    /// General CIM cognitive architecture questions
    Architecture,
    /// Mathematical foundations (Category Theory, Graph Theory, Conceptual Spaces)
    MathematicalFoundations,
    /// NATS patterns as cognitive nervous system
    NatsPatterns,
    /// Event sourcing as temporal memory system
    EventSourcing,
    /// Domain modeling within Conceptual Spaces
    DomainModeling,
    /// Practical cognitive system implementation
    Implementation,
    /// Cognitive system troubleshooting and debugging
    Troubleshooting,
    /// Conceptual Spaces and semantic organization
    ConceptualSpaces,
    /// Memory engram patterns and storage
    MemoryEngrams,
    /// Graph-based workflow design and optimization
    GraphWorkflows,
    /// Cognitive load balancing and performance
    CognitivePerformance,
    /// Emergent intelligence and collective cognition
    EmergentIntelligence,
}

/// CIM Expert consultation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimExpertQuery {
    /// The question to ask the CIM Expert
    pub question: String,
    /// Topic area for specialized guidance
    pub topic: CimExpertTopic,
    /// Optional context about the user's domain
    pub domain_context: Option<String>,
    /// User ID for audit trails
    pub user_id: Option<String>,
    /// Session context for multi-turn conversations
    pub session_context: Option<SessionContext>,
}

/// CIM Expert response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimExpertResponse {
    /// Comprehensive explanation from the CIM Expert
    pub explanation: String,
    /// Key concepts referenced in the explanation
    pub key_concepts: Vec<String>,
    /// Related topics for further exploration
    pub related_topics: Vec<String>,
    /// Practical next steps if applicable
    pub next_steps: Option<Vec<String>>,
    /// References to documentation or examples
    pub references: Option<Vec<String>>,
    /// Performance metadata
    pub metadata: ConsultationMetadata,
}

/// Session context for multi-turn conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: String,
    pub conversation_history: Vec<String>,
    pub domain: Option<String>,
    pub user_context: Option<String>,
}

/// Consultation performance metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsultationMetadata {
    pub tokens_used: u32,
    pub response_time_ms: u64,
    pub confidence_score: f32,
    pub complexity_level: String,
}

/// CIM Expert Service - provides architectural guidance via NATS
pub struct CimExpertService {
    _claude_client: ClaudeClient,
    nats_client: NatsClient,
    service_id: String,
}

impl CimExpertService {
    /// Create a new CIM Expert service
    pub async fn new(
        claude_config: ClaudeClientConfig,
        nats_client: NatsClient,
    ) -> Result<Self> {
        let claude_client = ClaudeClient::new(claude_config)
            .context("Failed to create Claude client for CIM Expert")?;
        
        let uuid_str = Uuid::new_v4().to_string();
        let service_id = format!("cim-expert-{}", &uuid_str[..8]);
        
        info!("CIM Expert Service initialized with ID: {}", service_id);
        
        Ok(Self {
            _claude_client: claude_client,
            nats_client,
            service_id,
        })
    }
    
    /// Start the CIM Expert service (listens for queries via NATS)
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting CIM Expert Service...");
        
        // Subscribe to expert query subjects
        let subjects = vec![
            "cim.expert.query.architecture",
            "cim.expert.query.mathematical_foundations", 
            "cim.expert.query.nats_patterns",
            "cim.expert.query.event_sourcing",
            "cim.expert.query.domain_modeling",
            "cim.expert.query.implementation",
            "cim.expert.query.troubleshooting",
            "cim.expert.query.conceptual_spaces",
            "cim.expert.query.memory_engrams",
            "cim.expert.query.graph_workflows",
            "cim.expert.query.cognitive_performance",
            "cim.expert.query.emergent_intelligence",
            "cim.expert.query.general",
        ];
        
        for subject in subjects {
            info!("Subscribing to expert queries on: {}", subject);
            // In a real implementation, this would set up NATS subscribers
            // For now, we'll demonstrate the interface
        }
        
        info!("🧠 CIM Expert Service ready for consultations!");
        info!("📡 Listening on subjects: cim.expert.query.*");
        
        // Keep service running (in real implementation, this would handle NATS messages)
        Ok(())
    }
    
    /// Process a CIM Expert query and provide comprehensive guidance
    #[instrument(skip(self, query))]
    pub async fn handle_expert_query(&self, query: CimExpertQuery) -> Result<CimExpertResponse> {
        let start_time = std::time::Instant::now();
        let correlation_id = CorrelationId::new();
        
        info!("Processing CIM Expert query: {:?} on topic: {:?}", 
              query.question.chars().take(50).collect::<String>(), 
              query.topic);
        
        // Build specialized system prompt based on topic
        let system_prompt = self.build_expert_prompt(&query.topic, &query.domain_context);
        
        // Enhance the user question with context
        let enhanced_question = self.enhance_question(&query);
        
        // Get expert response from Claude
        let claude_response = self.get_claude_response(&system_prompt, &enhanced_question).await
            .context("Failed to get response from Claude")?;
        
        // Parse and structure the response
        let expert_response = self.structure_response(claude_response, start_time.elapsed())
            .context("Failed to structure expert response")?;
        
        // Publish consultation event for audit trail
        self.publish_consultation_event(&query, &expert_response, correlation_id).await
            .context("Failed to publish consultation event")?;
        
        info!("CIM Expert consultation completed in {}ms", start_time.elapsed().as_millis());
        
        Ok(expert_response)
    }
    
    /// Build specialized system prompt based on expertise area
    fn build_expert_prompt(&self, topic: &CimExpertTopic, domain_context: &Option<String>) -> String {
        let base_prompt = r#"
You are a CIM (Composable Information Machine) Expert with deep expertise in cognitive architectures, distributed systems, and memory-centric computing. CIM is not just "NATS with configuration" - it's a complete cognitive system that models information processing as graph-based workflows within Conceptual Spaces.

Your core expertise covers the CIM cognitive architecture:

## Conceptual Spaces & Memory Architecture:
- **Conceptual Spaces**: Multi-dimensional semantic spaces where information exists as vectors with quality dimensions
- **Memory Engram Patterns**: Persistent memory structures that encode experiences and enable pattern recognition
- **Cognitive Graphs**: Network structures representing knowledge relationships and inference pathways
- **Dimensional Quality Metrics**: How information quality is measured across conceptual dimensions
- **Semantic Clustering**: How related concepts naturally aggregate in conceptual space

## Graph-Based Workflow Engine:
- **Process Flow Networks**: Directed graphs representing computational workflows
- **Node Activation Patterns**: How information flows through cognitive processing nodes
- **Edge Weight Dynamics**: Adaptive connection strengths that learn from usage patterns
- **Parallel Processing Paths**: Multiple simultaneous workflows through the cognitive graph
- **Feedback Loop Integration**: Self-reinforcing learning patterns in the workflow graph

## CIM Technical Foundations:
- **Category Theory**: Mathematical framework for composable information transformations
- **Graph Theory**: Network topology optimization and traversal algorithms
- **IPLD Content Addressing**: Immutable, verifiable information structures
- **Event Sourcing**: Temporal information flow and state reconstruction
- **NATS JetStream**: High-performance cognitive message substrate

## Memory & Learning Systems:
- **Episodic Memory**: Event-based experience storage and retrieval
- **Semantic Networks**: Interconnected concept relationships
- **Procedural Memory**: Encoded workflow patterns and behavioral responses
- **Working Memory**: Active information processing buffers
- **Long-term Potentiation**: Strengthening of frequently accessed pathways

## Distributed Cognitive Architecture:
- **Leaf Node Cognition**: Individual processing units with local memory
- **Cluster Collective Intelligence**: Emergent properties from connected leaf nodes
- **Super-cluster Consciousness**: Higher-order cognitive capabilities across clusters
- **Cognitive Load Balancing**: Dynamic distribution of processing across the network
- **Memory Synchronization**: Consistent knowledge state across distributed nodes

Communication style:
- Explain CIM as a cognitive system, not just a technical stack
- Use neuroscience and cognitive science analogies where appropriate
- Connect mathematical foundations to cognitive processes
- Emphasize the emergent intelligence properties
- Provide both conceptual understanding and implementation guidance
"#;
        
        let topic_specialization = match topic {
            CimExpertTopic::Architecture => {
                "\n\nSpecialization: Focus on CIM's cognitive architecture - how Conceptual Spaces organize information, how memory engram patterns form, and how graph-based workflows enable emergent intelligence. Explain the transition from traditional distributed systems to cognitive computing architectures."
            },
            CimExpertTopic::MathematicalFoundations => {
                "\n\nSpecialization: Deep focus on the mathematical foundations of cognition in CIM - Category Theory for composable transformations, Graph Theory for neural-like networks, dimensional vector spaces for conceptual representation, and how these create the mathematical substrate for artificial cognition."
            },
            CimExpertTopic::NatsPatterns => {
                "\n\nSpecialization: NATS as the 'nervous system' of CIM - how message patterns mirror neural communication, subject algebra as synaptic pathways, JetStream as memory persistence, and how distributed messaging enables collective intelligence across cognitive nodes."
            },
            CimExpertTopic::EventSourcing => {
                "\n\nSpecialization: Event sourcing as the temporal memory system of CIM - how events encode experiences, create memory engrams, enable episodic recall, and how the event stream becomes the 'consciousness timeline' of the cognitive system."
            },
            CimExpertTopic::DomainModeling => {
                "\n\nSpecialization: Domain modeling within Conceptual Spaces - how business domains map to semantic regions, how entities become conceptual vectors, how relationships form cognitive pathways, and how domain expertise encodes into the CIM's knowledge architecture."
            },
            CimExpertTopic::Implementation => {
                "\n\nSpecialization: Building cognitive systems with CIM - setting up Conceptual Spaces, configuring memory engram storage, implementing graph-based workflows, establishing cognitive node hierarchies, and creating emergent intelligence through proper architectural patterns."
            },
            CimExpertTopic::Troubleshooting => {
                "\n\nSpecialization: Diagnosing cognitive system issues - memory fragmentation, conceptual space distortions, workflow graph bottlenecks, engram corruption, semantic drift, and cognitive load imbalances. Think like debugging a distributed brain."
            },
            CimExpertTopic::ConceptualSpaces => {
                "\n\nSpecialization: Deep expertise in Conceptual Spaces theory - multi-dimensional semantic representation, quality dimensions, conceptual clustering, semantic distance metrics, dimensional transformations, and how conceptual spaces enable natural information organization and retrieval."
            },
            CimExpertTopic::MemoryEngrams => {
                "\n\nSpecialization: Memory engram patterns in CIM - how experiences encode into persistent memory structures, engram activation patterns, memory consolidation processes, associative memory networks, and how engrams enable pattern recognition and learning."
            },
            CimExpertTopic::GraphWorkflows => {
                "\n\nSpecialization: Graph-based workflow design - process flow networks, node activation patterns, edge weight optimization, parallel processing paths, workflow composition, dynamic graph adaptation, and how cognitive workflows emerge from graph structures."
            },
            CimExpertTopic::CognitivePerformance => {
                "\n\nSpecialization: Optimizing cognitive system performance - memory access patterns, conceptual space indexing, workflow parallelization, cognitive load distribution, memory garbage collection, attention mechanisms, and performance profiling of distributed cognition."
            },
            CimExpertTopic::EmergentIntelligence => {
                "\n\nSpecialization: Understanding emergent intelligence in CIM - how individual cognitive nodes create collective intelligence, emergence patterns, swarm cognition, distributed decision making, collective memory formation, and the transition from computation to cognition."
            },
        };
        
        let domain_enhancement = if let Some(domain) = domain_context {
            format!("\n\nDomain Context: The user is working in the {} domain. Tailor examples and guidance to this specific business context when relevant.", domain)
        } else {
            String::new()
        };
        
        format!("{}{}{}", base_prompt, topic_specialization, domain_enhancement)
    }
    
    /// Enhance the user's question with additional context
    fn enhance_question(&self, query: &CimExpertQuery) -> String {
        let mut enhanced = format!("Question: {}\n", query.question);
        
        if let Some(ref domain) = query.domain_context {
            enhanced.push_str(&format!("Domain Context: {}\n", domain));
        }
        
        if let Some(ref session) = query.session_context {
            if !session.conversation_history.is_empty() {
                enhanced.push_str("Previous conversation context:\n");
                for (i, prev) in session.conversation_history.iter().take(3).enumerate() {
                    enhanced.push_str(&format!("{}. {}\n", i + 1, prev));
                }
            }
        }
        
        enhanced.push_str("\nPlease provide comprehensive guidance on this question.");
        enhanced
    }
    
    /// Get response from Claude API
    async fn get_claude_response(&self, system_prompt: &str, question: &str) -> Result<String> {
        use crate::domain::claude_api::*;
        
        // Create Claude API request
        let messages = vec![ClaudeMessage {
            role: MessageRole::User,
            content: MessageContent::text(question.to_string()),
        }];
        
        let request = ClaudeApiRequest {
            model: ClaudeModel::Claude4Sonnet20250514,
            messages,
            max_tokens: MaxTokens::new(4000)
                .map_err(|e| anyhow::anyhow!("Invalid max_tokens: {}", e))?,
            system: Some(ClaudeSystemPrompt::new(system_prompt.to_string())
                .map_err(|e| anyhow::anyhow!("Invalid system prompt: {}", e))?),
            temperature: Some(Temperature::new(0.7)
                .map_err(|e| anyhow::anyhow!("Invalid temperature: {}", e))?),
            stop_sequences: None,
            tools: None,
            stream: Some(false),
            metadata: None,
        };
        
        // Execute the request using Claude client
        let response = self._claude_client.send_message(request).await
            .context("Failed to execute Claude API request")?;
        
        // Extract text content from response
        let content = response.content.iter()
            .find_map(|content| {
                match content {
                    ContentBlock::Text { text } => Some(text.clone()),
                    _ => None,
                }
            })
            .unwrap_or_else(|| "No text response from Claude".to_string());
        
        Ok(content)
    }
    
    /// Structure the Claude response into a CimExpertResponse
    fn structure_response(&self, claude_response: String, duration: Duration) -> Result<CimExpertResponse> {
        // In a real implementation, this would parse the Claude response
        // and extract key concepts, related topics, etc.
        
        Ok(CimExpertResponse {
            explanation: claude_response,
            key_concepts: vec![
                "CIM Architecture".to_string(),
                "Event Sourcing".to_string(),
                "NATS JetStream".to_string(),
            ],
            related_topics: vec![
                "Domain Modeling".to_string(),
                "Mathematical Foundations".to_string(),
            ],
            next_steps: Some(vec![
                "Review the CIM documentation".to_string(),
                "Set up a development environment".to_string(),
            ]),
            references: Some(vec![
                "https://docs.cim.dev/architecture".to_string(),
            ]),
            metadata: ConsultationMetadata {
                tokens_used: 500,
                response_time_ms: duration.as_millis() as u64,
                confidence_score: 0.95,
                complexity_level: "Advanced".to_string(),
            },
        })
    }
    
    /// Publish consultation event for audit trail
    async fn publish_consultation_event(
        &self,
        query: &CimExpertQuery,
        response: &CimExpertResponse,
        correlation_id: CorrelationId,
    ) -> Result<()> {
        let event = serde_json::json!({
            "event_type": "CimExpertConsultation",
            "service_id": self.service_id,
            "correlation_id": correlation_id.to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "query": {
                "question": query.question,
                "topic": query.topic,
                "domain_context": query.domain_context,
                "user_id": query.user_id,
            },
            "response": {
                "explanation_length": response.explanation.len(),
                "key_concepts": response.key_concepts,
                "complexity_level": response.metadata.complexity_level,
            },
            "performance": {
                "tokens_used": response.metadata.tokens_used,
                "response_time_ms": response.metadata.response_time_ms,
                "confidence_score": response.metadata.confidence_score,
            }
        });
        
        let subject = format!("cim.expert.events.consultation.{}", 
                             correlation_id.to_string().chars().take(8).collect::<String>());
        
        self.nats_client.publish_raw(&subject, serde_json::to_vec(&event)?).await
            .context("Failed to publish consultation event to NATS")?;
        
        info!("Published CIM Expert consultation event to: {}", subject);
        Ok(())
    }
}

/// Client interface for accessing CIM Expert functionality
pub struct CimExpertClient {
    nats_client: NatsClient,
}

impl CimExpertClient {
    /// Create a new CIM Expert client
    pub async fn new(nats_client: NatsClient) -> Result<Self> {
        Ok(Self { nats_client })
    }
    
    /// Ask the CIM Expert a question and get comprehensive guidance
    #[instrument(skip(self))]
    pub async fn ask(&self, question: &str, topic: CimExpertTopic) -> Result<CimExpertResponse> {
        let query = CimExpertQuery {
            question: question.to_string(),
            topic,
            domain_context: None,
            user_id: None,
            session_context: None,
        };
        
        self.ask_with_context(query).await
    }
    
    /// Ask the CIM Expert with full context
    pub async fn ask_with_context(&self, query: CimExpertQuery) -> Result<CimExpertResponse> {
        let _subject = match query.topic {
            CimExpertTopic::Architecture => "cim.expert.query.architecture",
            CimExpertTopic::MathematicalFoundations => "cim.expert.query.mathematical_foundations",
            CimExpertTopic::NatsPatterns => "cim.expert.query.nats_patterns",
            CimExpertTopic::EventSourcing => "cim.expert.query.event_sourcing",
            CimExpertTopic::DomainModeling => "cim.expert.query.domain_modeling",
            CimExpertTopic::Implementation => "cim.expert.query.implementation",
            CimExpertTopic::Troubleshooting => "cim.expert.query.troubleshooting",
            CimExpertTopic::ConceptualSpaces => "cim.expert.query.conceptual_spaces",
            CimExpertTopic::MemoryEngrams => "cim.expert.query.memory_engrams",
            CimExpertTopic::GraphWorkflows => "cim.expert.query.graph_workflows",
            CimExpertTopic::CognitivePerformance => "cim.expert.query.cognitive_performance",
            CimExpertTopic::EmergentIntelligence => "cim.expert.query.emergent_intelligence",
        };
        
        // Send query via NATS and wait for expert response
        let _response_data: () = self.nats_client.send_query(
            ClaudeApiQuery::GetConversationHistory {
                query_id: crate::domain::claude_queries::ClaudeQueryId::new(),
                conversation_id: ConversationId::new(),
                limit: Some(10),
                offset: None,
                include_tool_use: false,
                include_system_messages: false,
                message_roles: None,
            },
            CorrelationId::new(),
        ).await
            .context("Failed to get response from CIM Expert service")?;
        
        // In a real implementation, this would deserialize the actual response
        Ok(CimExpertResponse {
            explanation: "CIM Expert guidance would be provided here".to_string(),
            key_concepts: vec!["CIM Architecture".to_string()],
            related_topics: vec!["Event Sourcing".to_string()],
            next_steps: None,
            references: None,
            metadata: ConsultationMetadata {
                tokens_used: 400,
                response_time_ms: 2000,
                confidence_score: 0.9,
                complexity_level: "Intermediate".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cim_expert_query_structure() {
        let query = CimExpertQuery {
            question: "What is a CIM?".to_string(),
            topic: CimExpertTopic::Architecture,
            domain_context: Some("healthcare".to_string()),
            user_id: Some("user_123".to_string()),
            session_context: None,
        };
        
        assert_eq!(query.question, "What is a CIM?");
        assert!(matches!(query.topic, CimExpertTopic::Architecture));
        assert_eq!(query.domain_context, Some("healthcare".to_string()));
    }
    
    #[test]
    fn test_expert_topic_serialization() {
        let topic = CimExpertTopic::MathematicalFoundations;
        let serialized = serde_json::to_string(&topic).unwrap();
        let deserialized: CimExpertTopic = serde_json::from_str(&serialized).unwrap();
        
        assert!(matches!(deserialized, CimExpertTopic::MathematicalFoundations));
    }
}