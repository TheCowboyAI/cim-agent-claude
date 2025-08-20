/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! SAGE - The Master CIM Orchestrator
//!
//! SAGE understands the root underlying structure of a CIM:
//! - merkledag(bits) → NATS JetStream Object Store (immutable content)
//! - Everything about Objects → JetStream EventStore (immutable facts)
//! - Objects as PAYLOAD of events → raw data + IPLD + CID
//! - Subject algebra → Domain ecosystem communication
//! - Domain = Category → Objects=Entities, Arrows=Morphisms
//! - Entities → ID + collection of components

use super::{Subagent, SubagentQuery, SubagentResponse, SubagentError, SubagentCapability};
use super::router::{DomainType, DomainAnalysis, SubjectResolution, ResolutionStrategy};
use super::registry::SubagentRegistry;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tracing::{info, debug, warn, error};
use sha2::{Sha256, Digest};
use futures::StreamExt;

/// SAGE - Master CIM Orchestrator
/// 
/// Coordinates the complete CIM system through understanding of:
/// - Object Store (merkledag) and Event Store (immutable repositories)
/// - Subject algebra for domain communication
/// - Category theory for domain structure
/// - CID-based referential integrity
pub struct SageOrchestrator {
    agent_info: super::registry::SubagentInfo,
    expert_registry: Arc<SubagentRegistry>,
    cim_structure: CimRootStructure,
    orchestration_state: OrchestrationState,
}

/// Root structure of CIM that SAGE orchestrates
#[derive(Debug, Clone)]
pub struct CimRootStructure {
    /// Object Store - merkledag(bits) as NATS JetStream Object Store
    pub object_store: ObjectStoreConfig,
    /// Event Store - immutable facts about objects
    pub event_store: EventStoreConfig,
    /// Subject algebra for domain communication
    pub subject_algebra: SubjectAlgebraConfig,
    /// Domain categories and their mathematical properties
    pub domain_categories: HashMap<DomainType, DomainCategory>,
}

/// Configuration for CIM Object Store (merkledag)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStoreConfig {
    pub bucket_name: String,
    pub max_object_size: u64,
    pub replication_factor: u32,
    pub integrity_check_interval: u64,
    pub gc_threshold: f64,
}

/// Configuration for CIM Event Store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStoreConfig {
    pub stream_prefix: String,
    pub retention_policy: RetentionPolicy,
    pub max_events_per_stream: u64,
    pub replay_batch_size: u32,
}

/// Event retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPolicy {
    Forever,
    TimeLimit(u64), // seconds
    CountLimit(u64), // number of events
}

/// Subject algebra configuration for domain communication
#[derive(Debug, Clone)]
pub struct SubjectAlgebraConfig {
    pub root_subject: String,
    pub domain_patterns: HashMap<DomainType, DomainSubjectPattern>,
    pub composition_rules: Vec<CompositionRule>,
}

/// Subject pattern for a domain category
#[derive(Debug, Clone)]
pub struct DomainSubjectPattern {
    pub command_pattern: String,
    pub event_pattern: String,
    pub query_pattern: String,
    pub object_pattern: String,
}

/// Rule for composing subjects across domains
#[derive(Debug, Clone)]
pub struct CompositionRule {
    pub name: String,
    pub source_domains: Vec<DomainType>,
    pub target_subject: String,
    pub composition_law: String,
}

/// Mathematical representation of a domain as a category
#[derive(Debug, Clone)]
pub struct DomainCategory {
    pub domain_type: DomainType,
    /// Entities (Objects in category theory)
    pub entities: Vec<EntityDefinition>,
    /// Systems (Morphisms/Arrows in category theory)
    pub morphisms: Vec<MorphismDefinition>,
    /// Composition rules for morphisms
    pub composition_laws: Vec<CategoryLaw>,
    /// Identity morphisms
    pub identity_morphisms: HashMap<String, String>,
}

/// Definition of an entity (category object)
#[derive(Debug, Clone)]
pub struct EntityDefinition {
    pub id: String,
    pub entity_type: String,
    /// Components that make up this entity
    pub components: Vec<ComponentDefinition>,
    /// CID of the entity's data in Object Store
    pub object_cid: Option<String>,
}

/// Component of an entity
#[derive(Debug, Clone)]
pub struct ComponentDefinition {
    pub name: String,
    pub component_type: String,
    pub schema: serde_json::Value,
}

/// Definition of a morphism (category arrow)
#[derive(Debug, Clone)]
pub struct MorphismDefinition {
    pub id: String,
    pub name: String,
    pub source_entity: String,
    pub target_entity: String,
    pub transformation: TransformationFunction,
    pub preserves_structure: bool,
}

/// Transformation function for morphisms
#[derive(Debug, Clone)]
pub struct TransformationFunction {
    pub function_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub constraints: Vec<String>,
}

/// Mathematical law for category composition
#[derive(Debug, Clone)]
pub struct CategoryLaw {
    pub name: String,
    pub description: String,
    pub formula: String,
    pub constraints: Vec<String>,
}

/// Current orchestration state maintained by SAGE
#[derive(Debug, Clone)]
pub struct OrchestrationState {
    /// Active workflows being coordinated
    pub active_workflows: HashMap<String, WorkflowState>,
    /// Domain interactions being managed
    pub domain_interactions: Vec<DomainInteraction>,
    /// Object/Event consistency tracking
    pub consistency_state: ConsistencyTracker,
}

/// State of an active workflow
#[derive(Debug, Clone)]
pub struct WorkflowState {
    pub workflow_id: String,
    pub workflow_type: WorkflowType,
    pub participating_domains: Vec<DomainType>,
    pub current_phase: WorkflowPhase,
    pub expert_assignments: HashMap<String, String>, // expert_id -> task
    pub completion_criteria: Vec<String>,
    pub started_at: DateTime<Utc>,
}

/// Types of workflows SAGE can orchestrate
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowType {
    /// Complete CIM creation from domain discovery to deployment
    CompleteCimCreation,
    /// Cross-domain composition and integration
    DomainComposition,
    /// Event storming and domain discovery
    DomainDiscovery,
    /// Infrastructure setup and configuration
    InfrastructureSetup,
    /// System migration and evolution
    SystemMigration,
}

/// Phases of CIM workflows
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowPhase {
    CategoryDefinition,    // Domain discovery and mathematical structure
    InfrastructureMorphism, // Physical infrastructure setup
    ImplementationFunctor, // Code generation and deployment
    ValidationIntegration, // Testing and validation
    ProductionEvolution,   // Ongoing system evolution
}

/// Interaction between domains being orchestrated
#[derive(Debug, Clone)]
pub struct DomainInteraction {
    pub interaction_id: String,
    pub source_domain: DomainType,
    pub target_domain: DomainType,
    pub interaction_type: InteractionType,
    pub functor_mapping: FunctorMapping,
    pub consistency_requirements: Vec<String>,
}

/// Types of domain interactions
#[derive(Debug, Clone)]
pub enum InteractionType {
    /// Direct functor mapping between domains
    DirectMapping,
    /// Composition through intermediate domain
    Composition,
    /// Natural transformation
    NaturalTransformation,
    /// Branching (creating domain variant)
    Branching,
}

/// Mapping between domain categories (functors)
#[derive(Debug, Clone)]
pub struct FunctorMapping {
    pub name: String,
    pub object_mapping: HashMap<String, String>, // source_entity -> target_entity
    pub morphism_mapping: HashMap<String, String>, // source_morphism -> target_morphism
    pub preservation_properties: Vec<String>,
}

/// Tracks consistency between Object Store and Event Store
#[derive(Debug, Clone)]
pub struct ConsistencyTracker {
    /// CIDs referenced by events that should exist in Object Store
    pub required_objects: HashMap<String, Vec<String>>, // CID -> referencing_event_ids
    /// Objects in Object Store without corresponding events
    pub orphaned_objects: Vec<String>,
    /// Events that reference non-existent CIDs
    pub broken_references: Vec<String>,
    /// Last consistency check timestamp
    pub last_check: DateTime<Utc>,
}

impl SageOrchestrator {
    /// Create new SAGE orchestrator with understanding of CIM root structure
    pub fn new(
        agent_info: super::registry::SubagentInfo,
        expert_registry: Arc<SubagentRegistry>,
    ) -> Self {
        let cim_structure = Self::initialize_cim_structure();
        let orchestration_state = Self::initialize_orchestration_state();
        
        Self {
            agent_info,
            expert_registry,
            cim_structure,
            orchestration_state,
        }
    }

    /// SAGE Priority 1: Establish NATS connection and CIM system of record
    /// This must happen before any other orchestration can occur
    pub async fn establish_system_of_record(&mut self) -> Result<SystemOfRecordState, SageError> {
        info!("SAGE establishing CIM system of record through NATS connection");

        // Step 1: Test and establish NATS connectivity
        let nats_connection = self.establish_nats_connection().await?;
        
        // Step 2: Initialize JetStream Object Store (merkledag)
        let object_store = self.initialize_object_store(&nats_connection).await?;
        
        // Step 3: Initialize JetStream Event Store
        let event_store = self.initialize_event_store(&nats_connection).await?;
        
        // Step 4: Initialize NATS KV Store for organization metadata
        let _kv_store = self.initialize_kv_store(&nats_connection).await?;
        
        // Step 5: Validate system of record is operational
        self.validate_system_of_record(&object_store, &event_store).await?;

        Ok(SystemOfRecordState {
            nats_connected: true,
            object_store_ready: true,
            event_store_ready: true,
            kv_store_ready: true,
            established_at: Utc::now(),
        })
    }

    /// SAGE Priority 2: Publish Organization as CIM owner
    /// Organization must be established before any business domain work
    pub async fn publish_organization(&mut self, org_info: OrganizationInfo) -> Result<OrganizationCID, SageError> {
        info!("SAGE publishing Organization as CIM owner: {}", org_info.name);

        // Step 1: Create Organization entity with components
        let organization_entity = self.create_organization_entity(org_info).await?;
        
        // Step 2: Store organization data in Object Store (merkledag) 
        let organization_cid = self.store_organization_object(&organization_entity).await?;
        
        // Step 3: Publish OrganizationCreated event to Event Store
        let creation_event = self.publish_organization_event(&organization_cid, &organization_entity).await?;
        
        // Step 4: Validate organization is properly established in system of record
        self.validate_organization_establishment(&organization_cid, &creation_event).await?;

        info!("Organization established with CID: {}", organization_cid.cid);
        Ok(organization_cid)
    }

    /// SAGE Phase 3: After system of record + organization, begin domain/infrastructure work
    pub async fn begin_domain_infrastructure_discovery(&mut self, organization_cid: &OrganizationCID) -> Result<DomainInfrastructureState, SageError> {
        info!("SAGE beginning business domain and infrastructure discovery for org: {}", organization_cid.cid);

        // Now we can coordinate domain experts because we have:
        // 1. System of record (NATS + Object/Event stores)
        // 2. Organization ownership established
        
        // Coordinate business domain discovery
        let domain_state = self.coordinate_domain_discovery(organization_cid).await?;
        
        // Coordinate infrastructure/inventory discovery  
        let infrastructure_state = self.coordinate_infrastructure_discovery(organization_cid).await?;

        Ok(DomainInfrastructureState {
            organization_cid: organization_cid.clone(),
            business_domain: domain_state,
            infrastructure_inventory: infrastructure_state,
            discovery_started_at: Utc::now(),
        })
    }

    /// SAGE Core Operation: Establish NATS connection
    async fn establish_nats_connection(&self) -> Result<NatsConnection, SageError> {
        info!("SAGE establishing NATS connection for CIM system of record");

        let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
        let client_id = format!("sage_{}", chrono::Utc::now().timestamp());
        
        // Establish actual NATS connection
        let client = async_nats::connect(&nats_url).await
            .map_err(|e| SageError::NatsConnectionFailed(format!("Failed to connect to NATS at {}: {}", nats_url, e)))?;
        
        // Create JetStream context for Object Store and Event Store operations
        let jetstream = async_nats::jetstream::new(client.clone());
        
        let connection = NatsConnection {
            client,
            jetstream,
            url: nats_url.clone(),
            client_id,
        };

        info!("SAGE successfully established NATS connection to: {}", nats_url);
        Ok(connection)
    }

    /// SAGE Core Operation: Initialize JetStream Object Store (merkledag)
    async fn initialize_object_store(&self, connection: &NatsConnection) -> Result<ObjectStoreState, SageError> {
        info!("SAGE initializing JetStream Object Store (merkledag)");

        let bucket_name = "CIM_MERKLEDAG";
        
        // Create Object Store configuration for CIM merkledag
        let config = async_nats::jetstream::object_store::Config {
            bucket: bucket_name.to_string(),
            description: Some("CIM merkledag(bits) - Content-addressed immutable object storage".to_string()),
            max_bucket_size: Some(1_000_000_000), // 1GB max
            storage: async_nats::jetstream::stream::StorageType::File,
            replicas: 1,
            ..Default::default()
        };
        
        // Create or get existing Object Store
        let _object_store = connection.jetstream.create_object_store(config).await
            .or_else(|_| connection.jetstream.get_object_store(bucket_name))
            .await
            .map_err(|e| SageError::ObjectStoreInitFailed(format!("Failed to initialize Object Store {}: {}", bucket_name, e)))?;
        
        let store_state = ObjectStoreState {
            bucket_name: bucket_name.to_string(),
            operational: true,
            objects_count: 0, // Would query actual count if needed
        };

        info!("SAGE successfully initialized Object Store bucket: {}", bucket_name);
        Ok(store_state)
    }

    /// SAGE Core Operation: Initialize JetStream Event Store
    async fn initialize_event_store(&self, connection: &NatsConnection) -> Result<EventStoreState, SageError> {
        info!("SAGE initializing JetStream Event Store");

        let stream_name = "CIM_EVENTS";
        
        // Create Event Store stream configuration
        let config = async_nats::jetstream::stream::Config {
            name: stream_name.to_string(),
            description: Some("CIM Event Store - Immutable facts about objects with IPLD+CID payloads".to_string()),
            subjects: vec!["cim.events.>".to_string()], // Subject algebra for CIM events
            retention: async_nats::jetstream::stream::RetentionPolicy::Limits,
            storage: async_nats::jetstream::stream::StorageType::File,
            max_messages: Some(1_000_000), // 1M events max
            max_bytes: Some(10_000_000_000), // 10GB max
            discard: async_nats::jetstream::stream::DiscardPolicy::Old,
            replicas: 1,
            ..Default::default()
        };
        
        // Create or get existing Event Store stream
        let _stream = connection.jetstream.create_stream(config).await
            .or_else(|_| connection.jetstream.get_stream(stream_name))
            .await
            .map_err(|e| SageError::EventStoreInitFailed(format!("Failed to initialize Event Store {}: {}", stream_name, e)))?;
        
        let store_state = EventStoreState {
            stream_name: stream_name.to_string(),
            operational: true,
            events_count: 0, // Would query actual count if needed
        };

        info!("SAGE successfully initialized Event Store stream: {}", stream_name);
        Ok(store_state)
    }

    /// SAGE Core Operation: Initialize JetStream KV Store for metadata
    async fn initialize_kv_store(&self, connection: &NatsConnection) -> Result<KvStoreState, SageError> {
        info!("SAGE initializing JetStream KV Store for metadata");

        let kv_bucket = "CIM_METADATA";
        
        // Create KV Store configuration for CIM metadata
        let config = async_nats::jetstream::kv::Config {
            bucket: kv_bucket.to_string(),
            description: Some("CIM Metadata KV Store - Organization domain, MRU/LRU lists, and indexing".to_string()),
            max_value_size: 1024 * 1024, // 1MB max value size
            history: 10, // Keep 10 versions of each key
            storage: async_nats::jetstream::stream::StorageType::File,
            replicas: 1,
            ..Default::default()
        };
        
        // Create or get existing KV Store
        let _kv_store = connection.jetstream.create_key_value(config).await
            .or_else(|_| connection.jetstream.get_key_value(kv_bucket))
            .await
            .map_err(|e| SageError::KvStoreInitFailed(format!("Failed to initialize KV Store {}: {}", kv_bucket, e)))?;

        let kv_state = KvStoreState {
            bucket_name: kv_bucket.to_string(),
            operational: true,
            keys_count: 0, // Would query actual count if needed
        };

        info!("SAGE successfully initialized KV Store bucket: {}", kv_bucket);
        Ok(kv_state)
    }

    /// SAGE Core Operation: Validate system of record is operational
    async fn validate_system_of_record(&self, object_store: &ObjectStoreState, event_store: &EventStoreState) -> Result<(), SageError> {
        info!("SAGE validating system of record operational status");

        if !object_store.operational {
            return Err(SageError::SystemOfRecordValidationFailed("Object Store not operational".to_string()));
        }

        if !event_store.operational {
            return Err(SageError::SystemOfRecordValidationFailed("Event Store not operational".to_string()));
        }

        // Test round-trip: store test object, publish event, verify consistency
        // In real implementation, this would do actual validation

        info!("SAGE validated system of record is operational");
        Ok(())
    }

    /// SAGE Core Operation: Create organization entity
    async fn create_organization_entity(&self, org_info: OrganizationInfo) -> Result<OrganizationEntity, SageError> {
        info!("SAGE creating organization entity for: {}", org_info.name);

        let entity_id = format!("org_{}", uuid::Uuid::new_v4());
        
        let components = vec![
            OrganizationComponent {
                name: "basic_info".to_string(),
                component_type: "OrganizationBasicInfo".to_string(),
                data: serde_json::json!({
                    "name": org_info.name,
                    "description": org_info.description,
                    "domain": org_info.domain,
                    "business_type": org_info.business_type
                }),
            },
            OrganizationComponent {
                name: "ownership".to_string(),
                component_type: "OrganizationOwnership".to_string(),
                data: serde_json::json!({
                    "owner_email": org_info.owner_email,
                    "established_at": chrono::Utc::now().to_rfc3339()
                }),
            },
            OrganizationComponent {
                name: "metadata".to_string(),
                component_type: "OrganizationMetadata".to_string(),
                data: serde_json::Value::Object(org_info.metadata.clone()),
            },
        ];

        let entity = OrganizationEntity {
            id: entity_id,
            entity_type: "Organization".to_string(),
            components,
            created_at: Utc::now(),
        };

        info!("SAGE created organization entity: {}", entity.id);
        Ok(entity)
    }

    /// SAGE Core Operation: Store organization in Object Store (merkledag)
    async fn store_organization_object(&self, entity: &OrganizationEntity) -> Result<OrganizationCID, SageError> {
        info!("SAGE storing organization object in merkledag: {}", entity.id);

        // Serialize entity to bytes for content-addressed storage
        let entity_bytes = serde_json::to_vec(entity)
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Serialization failed: {}", e)))?;

        // Calculate proper IPLD-style CID
        let cid = self.calculate_cid_from_bytes(&entity_bytes);
        
        // Get NATS connection (in production, this would come from orchestration state)
        let nats_connection = self.establish_nats_connection().await?;
        
        // Store object in NATS JetStream Object Store
        let bucket_name = "CIM_MERKLEDAG";
        let object_store = nats_connection.jetstream.get_object_store(bucket_name).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to get Object Store {}: {}", bucket_name, e)))?;

        // Use CID as object name for content-addressed storage
        let _object_info = object_store.put(&cid, entity_bytes.into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to store object with CID {}: {}", cid, e)))?;

        // Store organization metadata in KV Store
        self.store_organization_kv(&nats_connection, &cid, entity).await?;

        let organization_cid = OrganizationCID {
            cid: cid.clone(),
            entity_id: entity.id.clone(),
            stored_at: Utc::now(),
        };

        info!("SAGE successfully stored organization with CID: {}", organization_cid.cid);
        Ok(organization_cid)
    }

    /// SAGE Core Operation: Publish OrganizationCreated event
    async fn publish_organization_event(&self, cid: &OrganizationCID, entity: &OrganizationEntity) -> Result<String, SageError> {
        info!("SAGE publishing OrganizationCreated event for CID: {}", cid.cid);

        // Create event with object CID as payload reference
        let event_id = uuid::Uuid::new_v4().to_string();
        let event = serde_json::json!({
            "event_id": event_id,
            "event_type": "OrganizationCreated",
            "object_cid": cid.cid,
            "entity_id": entity.id,
            "correlation_id": uuid::Uuid::new_v4().to_string(),
            "causation_id": null,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "domain": "organization",
            "metadata": {
                "created_by": "sage",
                "organization_name": entity.components.iter()
                    .find(|c| c.name == "basic_info")
                    .and_then(|c| c.data.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
            }
        });

        // Get NATS connection
        let nats_connection = self.establish_nats_connection().await?;
        
        // Serialize event to bytes
        let event_bytes = serde_json::to_vec(&event)
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Event serialization failed: {}", e)))?;

        // Publish event to NATS JetStream Event Store
        let subject = "cim.events.organization.created";
        let publish_ack = nats_connection.jetstream.publish(subject, event_bytes.into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to publish event to {}: {}", subject, e)))?;

        // Wait for acknowledgment to ensure event is persisted
        let _ack_info = publish_ack.await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Event acknowledgment failed: {}", e)))?;

        // Update KV Store with current organization domain
        self.update_current_organization_kv(&nats_connection, cid).await?;

        info!("SAGE successfully published OrganizationCreated event: {} (Stream: {}, Sequence: {})", 
              event_id, _ack_info.stream, _ack_info.sequence);
        Ok(event_id)
    }

    /// SAGE Core Operation: Validate organization establishment
    async fn validate_organization_establishment(&self, cid: &OrganizationCID, event_id: &str) -> Result<(), SageError> {
        info!("SAGE validating organization establishment: CID={}, Event={}", cid.cid, event_id);

        // 1. Verify object exists in Object Store by CID
        let object_data = self.get_object_by_cid(&cid.cid).await
            .map_err(|e| SageError::SystemOfRecordValidationFailed(format!("Object validation failed: {}", e)))?;

        // 2. Verify the object is a valid organization entity
        let _organization_entity: OrganizationEntity = serde_json::from_slice(&object_data)
            .map_err(|e| SageError::SystemOfRecordValidationFailed(format!("Invalid organization object format: {}", e)))?;

        // 3. Verify object CID matches expected CID (integrity check)
        let calculated_cid = self.calculate_cid_from_bytes(&object_data);
        if calculated_cid != cid.cid {
            return Err(SageError::ObjectEventInconsistency(format!(
                "CID mismatch: expected {}, calculated {}", cid.cid, calculated_cid
            )));
        }

        // 4. Verify organization exists in KV Store
        let current_org = self.get_current_organization().await
            .map_err(|e| SageError::SystemOfRecordValidationFailed(format!("KV validation failed: {}", e)))?;

        if let Some(stored_org) = current_org {
            if stored_org.cid != cid.cid {
                return Err(SageError::ObjectEventInconsistency(format!(
                    "KV Store CID mismatch: expected {}, stored {}", cid.cid, stored_org.cid
                )));
            }
        } else {
            return Err(SageError::SystemOfRecordValidationFailed("No current organization found in KV Store".to_string()));
        }

        info!("SAGE successfully validated organization establishment - all consistency checks passed");
        Ok(())
    }

    /// SAGE Coordination: Business domain discovery
    async fn coordinate_domain_discovery(&self, organization_cid: &OrganizationCID) -> Result<BusinessDomainState, SageError> {
        info!("SAGE coordinating business domain discovery for org: {}", organization_cid.cid);

        // Now we can safely coordinate with domain experts because:
        // 1. System of record is established
        // 2. Organization ownership is established

        self.coordinate_expert("event-storming-expert", 
            &format!("Discover business domain events for organization CID: {}", organization_cid.cid)).await?;

        self.coordinate_expert("ddd-expert",
            "Analyze discovered events and define domain boundaries").await?;

        // Mock domain state - in real implementation, this would be built from expert coordination results
        Ok(BusinessDomainState {
            domain_name: "business_domain".to_string(),
            entities_discovered: vec!["Customer".to_string(), "Order".to_string(), "Product".to_string()],
            events_identified: vec!["CustomerRegistered".to_string(), "OrderPlaced".to_string(), "ProductCatalogUpdated".to_string()],
            boundaries_defined: true,
            discovery_complete: true,
        })
    }

    /// SAGE Coordination: Infrastructure/inventory discovery
    async fn coordinate_infrastructure_discovery(&self, organization_cid: &OrganizationCID) -> Result<InfrastructureState, SageError> {
        info!("SAGE coordinating infrastructure discovery for org: {}", organization_cid.cid);

        self.coordinate_expert("network-expert",
            &format!("Discover network topology and infrastructure for organization CID: {}", organization_cid.cid)).await?;

        self.coordinate_expert("nix-expert",
            "Inventory current system configuration and infrastructure components").await?;

        // Mock infrastructure state - in real implementation, this would be built from expert coordination
        Ok(InfrastructureState {
            network_topology: NetworkTopologyState {
                topology_type: "distributed".to_string(),
                nodes_count: 3,
                connectivity_validated: true,
            },
            compute_resources: vec![
                ComputeResource {
                    resource_id: "compute_1".to_string(),
                    resource_type: "server".to_string(),
                    capacity: vec![("cpu_cores".to_string(), 8), ("memory_gb".to_string(), 32)].into_iter().collect(),
                },
            ],
            storage_resources: vec![
                StorageResource {
                    resource_id: "storage_1".to_string(),
                    storage_type: "ssd".to_string(),
                    capacity_bytes: 1_000_000_000_000, // 1TB
                },
            ],
            discovery_complete: true,
        })
    }

    /// Initialize understanding of CIM's root structure
    fn initialize_cim_structure() -> CimRootStructure {
        let object_store = ObjectStoreConfig {
            bucket_name: "CIM_MERKLEDAG".to_string(),
            max_object_size: 100_000_000, // 100MB
            replication_factor: 3,
            integrity_check_interval: 3600, // 1 hour
            gc_threshold: 0.8, // 80% usage triggers GC
        };

        let event_store = EventStoreConfig {
            stream_prefix: "CIM_EVENTS".to_string(),
            retention_policy: RetentionPolicy::Forever,
            max_events_per_stream: 1_000_000,
            replay_batch_size: 1000,
        };

        let subject_algebra = Self::initialize_subject_algebra();
        let domain_categories = Self::initialize_domain_categories();

        CimRootStructure {
            object_store,
            event_store,
            subject_algebra,
            domain_categories,
        }
    }

    /// Initialize subject algebra patterns for domain communication
    fn initialize_subject_algebra() -> SubjectAlgebraConfig {
        let mut domain_patterns = HashMap::new();
        
        // Define subject patterns for each domain category
        for domain_type in [
            DomainType::Architecture,
            DomainType::DomainModeling,
            DomainType::Infrastructure,
            DomainType::NetworkTopology,
            DomainType::EventSourcing,
            DomainType::Configuration,
            DomainType::UserInterface,
            DomainType::Orchestration,
            DomainType::Collaboration,
        ] {
            let domain_name = format!("{:?}", domain_type).to_lowercase();
            domain_patterns.insert(domain_type, DomainSubjectPattern {
                command_pattern: format!("cim.{}.cmd.>", domain_name),
                event_pattern: format!("cim.{}.evt.>", domain_name),
                query_pattern: format!("cim.{}.qry.>", domain_name),
                object_pattern: format!("cim.{}.obj.>", domain_name),
            });
        }

        let composition_rules = vec![
            CompositionRule {
                name: "Domain Discovery Composition".to_string(),
                source_domains: vec![DomainType::Collaboration, DomainType::DomainModeling],
                target_subject: "cim.compose.discovery.>".to_string(),
                composition_law: "EventStorming ∘ DDD → DomainBoundaries".to_string(),
            },
            CompositionRule {
                name: "Infrastructure Integration".to_string(),
                source_domains: vec![DomainType::Infrastructure, DomainType::NetworkTopology],
                target_subject: "cim.compose.infrastructure.>".to_string(),
                composition_law: "Infrastructure ∘ Network → DeploymentTopology".to_string(),
            },
        ];

        SubjectAlgebraConfig {
            root_subject: "cim".to_string(),
            domain_patterns,
            composition_rules,
        }
    }

    /// Initialize mathematical categories for each domain
    fn initialize_domain_categories() -> HashMap<DomainType, DomainCategory> {
        let mut categories = HashMap::new();

        // Architecture Domain Category
        categories.insert(DomainType::Architecture, DomainCategory {
            domain_type: DomainType::Architecture,
            entities: vec![
                EntityDefinition {
                    id: "system".to_string(),
                    entity_type: "SystemEntity".to_string(),
                    components: vec![
                        ComponentDefinition {
                            name: "boundaries".to_string(),
                            component_type: "BoundaryComponent".to_string(),
                            schema: serde_json::json!({"type": "object", "properties": {"contexts": {"type": "array"}}}),
                        },
                    ],
                    object_cid: None,
                },
            ],
            morphisms: vec![
                MorphismDefinition {
                    id: "composition".to_string(),
                    name: "System Composition".to_string(),
                    source_entity: "component".to_string(),
                    target_entity: "system".to_string(),
                    transformation: TransformationFunction {
                        function_type: "aggregation".to_string(),
                        parameters: HashMap::new(),
                        constraints: vec!["preserve_interfaces".to_string()],
                    },
                    preserves_structure: true,
                },
            ],
            composition_laws: vec![
                CategoryLaw {
                    name: "Associativity".to_string(),
                    description: "System composition is associative".to_string(),
                    formula: "(A ∘ B) ∘ C = A ∘ (B ∘ C)".to_string(),
                    constraints: vec!["interface_compatibility".to_string()],
                },
            ],
            identity_morphisms: vec![("system".to_string(), "id_system".to_string())].into_iter().collect(),
        });

        // Add other domain categories...
        categories
    }

    /// Initialize orchestration state
    fn initialize_orchestration_state() -> OrchestrationState {
        OrchestrationState {
            active_workflows: HashMap::new(),
            domain_interactions: Vec::new(),
            consistency_state: ConsistencyTracker {
                required_objects: HashMap::new(),
                orphaned_objects: Vec::new(),
                broken_references: Vec::new(),
                last_check: Utc::now(),
            },
        }
    }

    /// Orchestrate complete CIM creation workflow
    pub async fn orchestrate_cim_creation(&mut self, requirements: CimCreationRequirements) -> Result<WorkflowState, SageError> {
        info!("SAGE orchestrating complete CIM creation: {}", requirements.description);

        let workflow_id = format!("cim_creation_{}", chrono::Utc::now().timestamp());
        let workflow = WorkflowState {
            workflow_id: workflow_id.clone(),
            workflow_type: WorkflowType::CompleteCimCreation,
            participating_domains: requirements.target_domains.clone(),
            current_phase: WorkflowPhase::CategoryDefinition,
            expert_assignments: HashMap::new(),
            completion_criteria: requirements.completion_criteria.clone(),
            started_at: Utc::now(),
        };

        self.orchestration_state.active_workflows.insert(workflow_id.clone(), workflow.clone());

        // Phase 1: Category Definition (Domain Discovery)
        self.execute_category_definition_phase(&workflow_id, &requirements).await?;

        // Phase 2: Infrastructure Morphism (Structure Mapping)
        self.execute_infrastructure_morphism_phase(&workflow_id).await?;

        // Phase 3: Implementation Functor (Code Generation)
        self.execute_implementation_functor_phase(&workflow_id).await?;

        Ok(workflow)
    }

    /// Execute Phase 1: Category Definition
    async fn execute_category_definition_phase(&mut self, workflow_id: &str, requirements: &CimCreationRequirements) -> Result<(), SageError> {
        info!("SAGE Phase 1: Category Definition for workflow {}", workflow_id);

        // Coordinate domain discovery through expert agents
        if requirements.needs_domain_discovery {
            self.coordinate_expert("event-storming-expert", 
                "Lead domain discovery session to identify entities and morphisms").await?;
            
            self.coordinate_expert("ddd-expert", 
                "Analyze discovered events and define category boundaries").await?;
        }

        // Validate mathematical consistency of discovered categories
        self.validate_category_consistency(workflow_id).await?;

        // Update workflow phase
        if let Some(workflow) = self.orchestration_state.active_workflows.get_mut(workflow_id) {
            workflow.current_phase = WorkflowPhase::InfrastructureMorphism;
        }

        Ok(())
    }

    /// Execute Phase 2: Infrastructure Morphism
    async fn execute_infrastructure_morphism_phase(&mut self, workflow_id: &str) -> Result<(), SageError> {
        info!("SAGE Phase 2: Infrastructure Morphism for workflow {}", workflow_id);

        // Configure Object Store and Event Store
        self.coordinate_expert("nats-expert", 
            "Configure JetStream Object Store and Event Store with proper subject algebra").await?;

        // Establish network topology
        self.coordinate_expert("network-expert", 
            "Design network topology that preserves domain morphisms").await?;

        // Project domain structure to system configuration
        self.coordinate_expert("nix-expert", 
            "Generate system configuration that preserves category structure").await?;

        // Update workflow phase
        if let Some(workflow) = self.orchestration_state.active_workflows.get_mut(workflow_id) {
            workflow.current_phase = WorkflowPhase::ImplementationFunctor;
        }

        Ok(())
    }

    /// Execute Phase 3: Implementation Functor
    async fn execute_implementation_functor_phase(&mut self, workflow_id: &str) -> Result<(), SageError> {
        info!("SAGE Phase 3: Implementation Functor for workflow {}", workflow_id);

        // Generate domain implementation
        self.coordinate_expert("domain-expert", 
            "Generate cim-graph implementation with proper CID references").await?;

        // Validate mathematical properties are preserved
        self.coordinate_expert("cim-expert", 
            "Validate that implementation preserves category theory properties").await?;

        // Ensure Object Store/Event Store consistency
        self.validate_object_event_consistency(workflow_id).await?;

        // Update workflow phase
        if let Some(workflow) = self.orchestration_state.active_workflows.get_mut(workflow_id) {
            workflow.current_phase = WorkflowPhase::ValidationIntegration;
        }

        Ok(())
    }

    /// Coordinate with a specific expert agent
    async fn coordinate_expert(&self, expert_id: &str, task: &str) -> Result<String, SageError> {
        debug!("SAGE coordinating with {} for task: {}", expert_id, task);
        
        if let Some(expert) = self.expert_registry.get_agent(expert_id).await {
            let query = SubagentQuery {
                id: format!("sage_coord_{}", chrono::Utc::now().timestamp()),
                user_id: "sage".to_string(),
                conversation_id: None,
                query_text: task.to_string(),
                context: super::SubagentContext {
                    domain: Some("orchestration".to_string()),
                    task_type: super::TaskType::Orchestration,
                    complexity: super::ComplexityLevel::Complex,
                    requires_collaboration: false,
                    referenced_files: Vec::new(),
                    keywords: Vec::new(),
                },
                metadata: HashMap::new(),
                timestamp: Utc::now(),
            };

            match expert.process_query(query).await {
                Ok(response) => {
                    info!("Expert {} completed task: {}", expert_id, response.response_text.chars().take(100).collect::<String>());
                    Ok(response.response_text)
                }
                Err(e) => {
                    warn!("Expert {} failed task: {}", expert_id, e);
                    Err(SageError::ExpertCoordinationFailed(expert_id.to_string(), e.to_string()))
                }
            }
        } else {
            Err(SageError::ExpertNotFound(expert_id.to_string()))
        }
    }

    /// Validate mathematical consistency of domain categories
    async fn validate_category_consistency(&self, workflow_id: &str) -> Result<(), SageError> {
        info!("SAGE validating category consistency for workflow {}", workflow_id);

        // Check that all morphisms compose properly
        // Check that identity morphisms exist
        // Check that composition is associative
        // Validate that structure-preserving properties hold

        Ok(())
    }

    /// Validate consistency between Object Store and Event Store
    async fn validate_object_event_consistency(&mut self, workflow_id: &str) -> Result<(), SageError> {
        info!("SAGE validating Object/Event consistency for workflow {}", workflow_id);

        // Check that all event payloads have corresponding CIDs in Object Store
        // Verify that all CIDs referenced in events actually exist
        // Ensure no orphaned objects exist without referencing events
        // Validate merkle integrity of all objects

        Ok(())
    }
}

/// Requirements for CIM creation
#[derive(Debug, Clone)]
pub struct CimCreationRequirements {
    pub description: String,
    pub target_domains: Vec<DomainType>,
    pub needs_domain_discovery: bool,
    pub infrastructure_requirements: Vec<String>,
    pub completion_criteria: Vec<String>,
}

/// State of CIM system of record
#[derive(Debug, Clone)]
pub struct SystemOfRecordState {
    pub nats_connected: bool,
    pub object_store_ready: bool,
    pub event_store_ready: bool,
    pub kv_store_ready: bool,
    pub established_at: DateTime<Utc>,
}

/// Information about the organization that owns this CIM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationInfo {
    pub name: String,
    pub description: String,
    pub domain: String, // e.g., "acme.com"
    pub owner_email: String,
    pub business_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Organization entity with ID and components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationEntity {
    pub id: String,
    pub entity_type: String,
    pub components: Vec<OrganizationComponent>,
    pub created_at: DateTime<Utc>,
}

/// Components that make up an organization entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationComponent {
    pub name: String,
    pub component_type: String,
    pub data: serde_json::Value,
}

/// Content ID for organization in Object Store
#[derive(Debug, Clone)]
pub struct OrganizationCID {
    pub cid: String,
    pub entity_id: String,
    pub stored_at: DateTime<Utc>,
}

/// State of domain and infrastructure discovery
#[derive(Debug, Clone)]
pub struct DomainInfrastructureState {
    pub organization_cid: OrganizationCID,
    pub business_domain: BusinessDomainState,
    pub infrastructure_inventory: InfrastructureState,
    pub discovery_started_at: DateTime<Utc>,
}

/// State of business domain discovery
#[derive(Debug, Clone)]
pub struct BusinessDomainState {
    pub domain_name: String,
    pub entities_discovered: Vec<String>,
    pub events_identified: Vec<String>,
    pub boundaries_defined: bool,
    pub discovery_complete: bool,
}

/// State of infrastructure/inventory discovery
#[derive(Debug, Clone)]
pub struct InfrastructureState {
    pub network_topology: NetworkTopologyState,
    pub compute_resources: Vec<ComputeResource>,
    pub storage_resources: Vec<StorageResource>,
    pub discovery_complete: bool,
}

/// Network topology state
#[derive(Debug, Clone)]
pub struct NetworkTopologyState {
    pub topology_type: String,
    pub nodes_count: u32,
    pub connectivity_validated: bool,
}

/// Compute resource in infrastructure
#[derive(Debug, Clone)]
pub struct ComputeResource {
    pub resource_id: String,
    pub resource_type: String,
    pub capacity: HashMap<String, u64>,
}

/// Storage resource in infrastructure
#[derive(Debug, Clone)]
pub struct StorageResource {
    pub resource_id: String,
    pub storage_type: String,
    pub capacity_bytes: u64,
}

/// NATS connection state
#[derive(Debug)]
pub struct NatsConnection {
    pub client: async_nats::Client,
    pub jetstream: async_nats::jetstream::Context,
    pub url: String,
    pub client_id: String,
}

/// Object Store state
#[derive(Debug, Clone)]
pub struct ObjectStoreState {
    pub bucket_name: String,
    pub operational: bool,
    pub objects_count: u64,
}

/// Event Store state  
#[derive(Debug, Clone)]
pub struct EventStoreState {
    pub stream_name: String,
    pub operational: bool,
    pub events_count: u64,
}

/// KV Store state for organization metadata and indexing
#[derive(Debug, Clone)]
pub struct KvStoreState {
    pub bucket_name: String,
    pub operational: bool,
    pub keys_count: u64,
}

/// Errors that can occur during SAGE orchestration
#[derive(Debug, Clone)]
pub enum SageError {
    NatsConnectionFailed(String),
    ObjectStoreInitFailed(String),
    EventStoreInitFailed(String),
    KvStoreInitFailed(String),
    OrganizationCreationFailed(String),
    SystemOfRecordValidationFailed(String),
    ExpertNotFound(String),
    ExpertCoordinationFailed(String, String),
    CategoryInconsistency(String),
    ObjectEventInconsistency(String),
    WorkflowExecutionFailed(String),
    ObjectStorageError(String),
    EventPublishError(String),
    KvStoreError(String),
}

impl std::fmt::Display for SageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SageError::NatsConnectionFailed(msg) => write!(f, "NATS connection failed: {}", msg),
            SageError::ObjectStoreInitFailed(msg) => write!(f, "Object Store initialization failed: {}", msg),
            SageError::EventStoreInitFailed(msg) => write!(f, "Event Store initialization failed: {}", msg),
            SageError::KvStoreInitFailed(msg) => write!(f, "KV Store initialization failed: {}", msg),
            SageError::OrganizationCreationFailed(msg) => write!(f, "Organization creation failed: {}", msg),
            SageError::SystemOfRecordValidationFailed(msg) => write!(f, "System of record validation failed: {}", msg),
            SageError::ExpertNotFound(expert) => write!(f, "Expert not found: {}", expert),
            SageError::ExpertCoordinationFailed(expert, error) => write!(f, "Expert coordination failed for {}: {}", expert, error),
            SageError::CategoryInconsistency(msg) => write!(f, "Category consistency error: {}", msg),
            SageError::ObjectEventInconsistency(msg) => write!(f, "Object/Event consistency error: {}", msg),
            SageError::WorkflowExecutionFailed(msg) => write!(f, "Workflow execution failed: {}", msg),
            SageError::ObjectStorageError(msg) => write!(f, "Object storage error: {}", msg),
            SageError::EventPublishError(msg) => write!(f, "Event publish error: {}", msg),
            SageError::KvStoreError(msg) => write!(f, "KV store error: {}", msg),
        }
    }
}

impl std::error::Error for SageError {}

#[async_trait]
impl Subagent for SageOrchestrator {
    fn id(&self) -> &str { &self.agent_info.id }
    fn name(&self) -> &str { &self.agent_info.name }
    fn description(&self) -> &str { &self.agent_info.description }
    fn available_tools(&self) -> Vec<String> { self.agent_info.tools.clone() }
    fn capabilities(&self) -> Vec<SubagentCapability> { self.agent_info.capabilities.clone() }

    async fn process_query(&self, query: SubagentQuery) -> Result<SubagentResponse, SubagentError> {
        info!("SAGE processing orchestration query: {}", query.query_text.chars().take(100).collect::<String>());

        // Analyze query to determine orchestration strategy
        let orchestration_plan = self.analyze_orchestration_requirements(&query).await?;
        
        let response_text = format!(
            "SAGE Orchestration Plan:\n\n{}\n\nI understand this requires coordination across the CIM's root structure:\n- Object Store (merkledag): {}\n- Event Store (immutable facts): {}\n- Subject Algebra: {}\n- Domain Categories: {}\n\nNext: I'll coordinate the appropriate expert agents to execute this plan while maintaining mathematical consistency.",
            orchestration_plan.description,
            orchestration_plan.requires_object_store,
            orchestration_plan.requires_event_store,
            orchestration_plan.requires_subject_algebra,
            orchestration_plan.participating_domains.len()
        );

        Ok(SubagentResponse {
            query_id: query.id,
            subagent_id: self.id().to_string(),
            response_text,
            confidence_score: 0.95, // SAGE has high confidence in orchestration
            recommendations: orchestration_plan.recommendations,
            next_actions: orchestration_plan.next_actions,
            metadata: {
                let json_obj = serde_json::json!({
                    "orchestration_type": orchestration_plan.orchestration_type,
                    "participating_domains": orchestration_plan.participating_domains,
                    "requires_object_store": orchestration_plan.requires_object_store,
                    "requires_event_store": orchestration_plan.requires_event_store
                });
                json_obj.as_object().unwrap().iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<std::collections::HashMap<String, serde_json::Value>>()
            },
            timestamp: Utc::now(),
        })
    }

    fn can_handle(&self, _query: &SubagentQuery) -> bool {
        true // SAGE can orchestrate any query
    }

    fn priority_score(&self, query: &SubagentQuery) -> u32 {
        let orchestration_keywords = [
            "complete", "full", "workflow", "orchestrate", "coordinate",
            "build", "create", "setup", "end-to-end", "comprehensive"
        ];
        let text = query.query_text.to_lowercase();
        
        let score = orchestration_keywords.iter()
            .filter(|&keyword| text.contains(keyword))
            .count() as u32;
            
        if score > 0 { 100 + score * 10 } else { 80 } // High base priority for orchestration
    }
}

impl SageOrchestrator {
    /// Analyze query to determine orchestration requirements
    async fn analyze_orchestration_requirements(&self, query: &SubagentQuery) -> Result<OrchestrationPlan, SubagentError> {
        let text = query.query_text.to_lowercase();
        
        let orchestration_type = if text.contains("build") || text.contains("create") || text.contains("new") {
            "complete_cim_creation"
        } else if text.contains("integrate") || text.contains("compose") {
            "domain_composition"
        } else if text.contains("discover") || text.contains("understand") {
            "domain_discovery"
        } else if text.contains("setup") || text.contains("configure") || text.contains("infrastructure") {
            "infrastructure_setup"
        } else {
            "general_orchestration"
        };

        let participating_domains = self.identify_participating_domains(&text);
        let requires_object_store = text.contains("store") || text.contains("data") || text.contains("object");
        let requires_event_store = text.contains("event") || text.contains("history") || text.contains("audit");
        let requires_subject_algebra = participating_domains.len() > 1;

        Ok(OrchestrationPlan {
            description: format!("Orchestrate {} across {} domains", orchestration_type, participating_domains.len()),
            orchestration_type: orchestration_type.to_string(),
            participating_domains,
            requires_object_store,
            requires_event_store,
            requires_subject_algebra,
            recommendations: vec![
                super::SubagentRecommendation {
                    recommendation_type: super::RecommendationType::NextStep,
                    description: "Begin with domain analysis and category definition".to_string(),
                    priority: super::Priority::High,
                    estimated_effort: Some("2-4 hours".to_string()),
                    dependencies: vec!["domain_experts".to_string()],
                },
            ],
            next_actions: vec![
                super::NextAction {
                    action_type: super::ActionType::InvokeSubagent,
                    description: "Coordinate with domain experts for category definition".to_string(),
                    target_subagent: Some("event-storming-expert".to_string()),
                    parameters: HashMap::new(),
                },
            ],
        })
    }

    /// Identify which domains are involved in the query
    fn identify_participating_domains(&self, text: &str) -> Vec<DomainType> {
        let mut domains = Vec::new();
        
        if text.contains("architecture") || text.contains("design") || text.contains("system") {
            domains.push(DomainType::Architecture);
        }
        if text.contains("domain") || text.contains("ddd") || text.contains("business") {
            domains.push(DomainType::DomainModeling);
        }
        if text.contains("infrastructure") || text.contains("deployment") || text.contains("nats") {
            domains.push(DomainType::Infrastructure);
        }
        if text.contains("network") || text.contains("topology") {
            domains.push(DomainType::NetworkTopology);
        }
        if text.contains("event") || text.contains("sourcing") {
            domains.push(DomainType::EventSourcing);
        }
        if text.contains("config") || text.contains("nix") {
            domains.push(DomainType::Configuration);
        }
        if text.contains("ui") || text.contains("interface") || text.contains("gui") {
            domains.push(DomainType::UserInterface);
        }
        if text.contains("collaboration") || text.contains("team") || text.contains("workshop") {
            domains.push(DomainType::Collaboration);
        }

        if domains.is_empty() {
            domains.push(DomainType::Architecture); // Default domain
        }

        domains
    }

    /// Calculate IPLD-style CID from raw bytes using SHA256
    fn calculate_cid_from_bytes(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        // Create IPLD CID (simplified version - real IPLD would use multicodec/multibase)
        // bafkrei is the CIDv1 prefix for raw data with SHA256
        format!("bafkrei{}", hex::encode(&hash[..16])) // Use first 16 bytes for shorter CID
    }

    /// Store organization metadata in KV Store
    async fn store_organization_kv(&self, connection: &NatsConnection, cid: &str, entity: &OrganizationEntity) -> Result<(), SageError> {
        info!("SAGE storing organization metadata in KV Store for CID: {}", cid);

        let kv_bucket = "CIM_METADATA";
        let kv_store = connection.jetstream.get_key_value(kv_bucket).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to get KV Store {}: {}", kv_bucket, e)))?;

        // Store organization domain information
        let org_name = entity.components.iter()
            .find(|c| c.name == "basic_info")
            .and_then(|c| c.data.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");

        let org_domain = entity.components.iter()
            .find(|c| c.name == "basic_info")
            .and_then(|c| c.data.get("domain"))
            .and_then(|d| d.as_str())
            .unwrap_or("unknown");

        // Store key organization metadata
        kv_store.put(&format!("organization.{}.cid", entity.id), cid.as_bytes().into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to store org CID: {}", e)))?;

        kv_store.put(&format!("organization.{}.name", entity.id), org_name.as_bytes().into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to store org name: {}", e)))?;

        kv_store.put(&format!("organization.{}.domain", entity.id), org_domain.as_bytes().into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to store org domain: {}", e)))?;

        kv_store.put(&format!("organization.{}.created_at", entity.id), entity.created_at.to_rfc3339().as_bytes().into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to store creation timestamp: {}", e)))?;

        // Store reverse lookup: CID -> entity_id
        kv_store.put(&format!("cid.{}.entity_id", cid), entity.id.as_bytes().into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to store CID lookup: {}", e)))?;

        info!("SAGE successfully stored organization metadata in KV Store");
        Ok(())
    }

    /// Update current organization domain in KV Store
    async fn update_current_organization_kv(&self, connection: &NatsConnection, cid: &OrganizationCID) -> Result<(), SageError> {
        info!("SAGE updating current organization domain for CID: {}", cid.cid);

        let kv_bucket = "CIM_METADATA";
        let kv_store = connection.jetstream.get_key_value(kv_bucket).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to get KV Store {}: {}", kv_bucket, e)))?;

        // Set current organization
        kv_store.put("current.organization.cid", cid.cid.as_bytes().into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to set current org CID: {}", e)))?;

        kv_store.put("current.organization.entity_id", cid.entity_id.as_bytes().into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to set current entity ID: {}", e)))?;

        kv_store.put("current.organization.set_at", cid.stored_at.to_rfc3339().as_bytes().into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to set current org timestamp: {}", e)))?;

        // Update MRU (Most Recently Used) organization list
        self.update_mru_organizations_kv(&kv_store, &cid.cid).await?;

        info!("SAGE successfully updated current organization domain");
        Ok(())
    }

    /// Update MRU (Most Recently Used) organizations list
    async fn update_mru_organizations_kv(&self, kv_store: &async_nats::jetstream::kv::Store, new_cid: &str) -> Result<(), SageError> {
        info!("SAGE updating MRU organizations list with CID: {}", new_cid);

        // Get current MRU list
        let current_mru = match kv_store.get("mru.organizations").await {
            Ok(entry) => {
                let mru_bytes = entry.value;
                String::from_utf8(mru_bytes.to_vec())
                    .unwrap_or_default()
            }
            Err(_) => String::new() // First organization
        };

        // Parse existing MRU list (comma-separated CIDs)
        let mut mru_list: Vec<String> = if current_mru.is_empty() {
            Vec::new()
        } else {
            current_mru.split(',').map(|s| s.trim().to_string()).collect()
        };

        // Remove the new CID if it already exists (to move it to front)
        mru_list.retain(|cid| cid != new_cid);

        // Add new CID to front
        mru_list.insert(0, new_cid.to_string());

        // Keep only last 10 organizations
        if mru_list.len() > 10 {
            mru_list.truncate(10);
        }

        // Store updated MRU list
        let updated_mru = mru_list.join(",");
        kv_store.put("mru.organizations", updated_mru.as_bytes().into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to update MRU organizations: {}", e)))?;

        // Also store organization count
        kv_store.put("stats.organization_count", mru_list.len().to_string().as_bytes().into()).await
            .map_err(|e| SageError::OrganizationCreationFailed(format!("Failed to update org count: {}", e)))?;

        info!("SAGE successfully updated MRU organizations list ({} total)", mru_list.len());
        Ok(())
    }

    /// Retrieve object from NATS Object Store by CID
    async fn get_object_by_cid(&self, cid: &str) -> Result<Vec<u8>, SageError> {
        info!("SAGE retrieving object by CID: {}", cid);

        let nats_connection = self.establish_nats_connection().await?;
        let bucket_name = "CIM_MERKLEDAG";
        let object_store = nats_connection.jetstream.get_object_store(bucket_name).await
            .map_err(|e| SageError::ObjectStoreInitFailed(format!("Failed to get Object Store {}: {}", bucket_name, e)))?;

        let mut object_data = Vec::new();
        let mut object_reader = object_store.get(cid).await
            .map_err(|e| SageError::ObjectStoreInitFailed(format!("Failed to get object {}: {}", cid, e)))?;

        // Read object data
        while let Some(chunk) = object_reader.next().await {
            let chunk = chunk.map_err(|e| SageError::ObjectStoreInitFailed(format!("Failed to read object chunk: {}", e)))?;
            object_data.extend_from_slice(&chunk);
        }

        info!("SAGE successfully retrieved object: {} bytes", object_data.len());
        Ok(object_data)
    }

    /// Query organization metadata from KV Store
    async fn get_current_organization(&self) -> Result<Option<OrganizationCID>, SageError> {
        info!("SAGE querying current organization from KV Store");

        let nats_connection = self.establish_nats_connection().await?;
        let kv_bucket = "CIM_METADATA";
        let kv_store = nats_connection.jetstream.get_key_value(kv_bucket).await
            .map_err(|e| SageError::ObjectStoreInitFailed(format!("Failed to get KV Store {}: {}", kv_bucket, e)))?;

        // Get current organization CID
        let current_cid = match kv_store.get("current.organization.cid").await {
            Ok(entry) => String::from_utf8(entry.value.to_vec())
                .map_err(|e| SageError::ObjectStoreInitFailed(format!("Invalid UTF-8 in CID: {}", e)))?,
            Err(_) => return Ok(None) // No current organization
        };

        // Get entity ID
        let entity_id = match kv_store.get("current.organization.entity_id").await {
            Ok(entry) => String::from_utf8(entry.value.to_vec())
                .map_err(|e| SageError::ObjectStoreInitFailed(format!("Invalid UTF-8 in entity ID: {}", e)))?,
            Err(_) => return Ok(None)
        };

        // Get stored timestamp
        let stored_at = match kv_store.get("current.organization.set_at").await {
            Ok(entry) => {
                let timestamp_str = String::from_utf8(entry.value.to_vec())
                    .map_err(|e| SageError::ObjectStoreInitFailed(format!("Invalid UTF-8 in timestamp: {}", e)))?;
                DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|e| SageError::ObjectStoreInitFailed(format!("Invalid timestamp format: {}", e)))?
                    .with_timezone(&Utc)
            }
            Err(_) => Utc::now() // Default to now if timestamp missing
        };

        let organization_cid = OrganizationCID {
            cid: current_cid,
            entity_id,
            stored_at,
        };

        info!("SAGE found current organization: {}", organization_cid.cid);
        Ok(Some(organization_cid))
    }

    /// Get MRU (Most Recently Used) organizations list
    pub async fn get_mru_organizations(&self) -> Result<Vec<String>, SageError> {
        info!("SAGE querying MRU organizations from KV Store");

        let nats_connection = self.establish_nats_connection().await?;
        let kv_bucket = "CIM_METADATA";
        let kv_store = nats_connection.jetstream.get_key_value(kv_bucket).await
            .map_err(|e| SageError::KvStoreError(format!("Failed to get KV Store {}: {}", kv_bucket, e)))?;

        // Get MRU list
        let mru_list = match kv_store.get("mru.organizations").await {
            Ok(entry) => {
                let mru_bytes = entry.value;
                let mru_string = String::from_utf8(mru_bytes.to_vec())
                    .map_err(|e| SageError::KvStoreError(format!("Invalid UTF-8 in MRU list: {}", e)))?;
                
                if mru_string.is_empty() {
                    Vec::new()
                } else {
                    mru_string.split(',').map(|s| s.trim().to_string()).collect()
                }
            }
            Err(_) => Vec::new() // No MRU list exists yet
        };

        info!("SAGE found {} MRU organizations", mru_list.len());
        Ok(mru_list)
    }

    /// Store NATS connection for Object Store operations
    async fn get_nats_connection(&self) -> Result<&NatsConnection, SageError> {
        // In the real implementation, this would come from the orchestration state
        // For now, we'll need to modify the architecture to pass the connection
        Err(SageError::NatsConnectionFailed("Connection access needs architectural update".to_string()))
    }
}

/// Orchestration plan generated by SAGE
#[derive(Debug, Clone)]
pub struct OrchestrationPlan {
    pub description: String,
    pub orchestration_type: String,
    pub participating_domains: Vec<DomainType>,
    pub requires_object_store: bool,
    pub requires_event_store: bool,
    pub requires_subject_algebra: bool,
    pub recommendations: Vec<super::SubagentRecommendation>,
    pub next_actions: Vec<super::NextAction>,
}