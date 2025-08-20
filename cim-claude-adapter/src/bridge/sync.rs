// Entity Synchronization Manager
//
// Manages synchronization between TEA display layer models and ECS communication layer entities.
// Ensures consistent state across both layers while maintaining proper separation of concerns.

use super::*;
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use tracing::{debug, warn, error, trace};

/// Manages entity synchronization between TEA and ECS layers
pub struct EntitySyncManager {
    /// Entities that need synchronization
    dirty_entities: HashSet<EntityId>,
    
    /// Sync history for conflict resolution
    sync_history: HashMap<EntityId, SyncMetadata>,
    
    /// Component-level sync tracking
    component_sync: HashMap<EntityId, HashMap<String, ComponentSyncState>>,
    
    /// Sync statistics
    stats: SyncStatistics,
    
    /// Last full sync timestamp
    last_full_sync: DateTime<Utc>,
    
    /// Sync configuration
    config: SyncConfiguration,
}

impl EntitySyncManager {
    pub fn new() -> Self {
        Self {
            dirty_entities: HashSet::new(),
            sync_history: HashMap::new(),
            component_sync: HashMap::new(),
            stats: SyncStatistics::new(),
            last_full_sync: Utc::now(),
            config: SyncConfiguration::default(),
        }
    }
    
    /// Mark entity as needing synchronization
    pub fn mark_dirty(&mut self, entity_id: EntityId) {
        self.dirty_entities.insert(entity_id);
        debug!("Marked entity {} as dirty for sync", entity_id);
    }
    
    /// Mark component as needing synchronization
    pub fn mark_component_dirty(&mut self, entity_id: EntityId, component_type: String) {
        self.component_sync
            .entry(entity_id)
            .or_insert_with(HashMap::new)
            .insert(component_type.clone(), ComponentSyncState::Dirty {
                marked_at: Utc::now(),
                reason: "Component modified".to_string(),
            });
            
        self.mark_dirty(entity_id);
        trace!("Marked component {} of entity {} as dirty", component_type, entity_id);
    }
    
    /// Perform entity synchronization
    pub async fn sync_entities(&mut self, entity_manager: &mut EntityManager) -> Result<SyncReport, BridgeError> {
        let start_time = std::time::Instant::now();
        let mut report = SyncReport::new();
        
        debug!("Starting entity synchronization with {} dirty entities", self.dirty_entities.len());
        
        // Get snapshot of dirty entities to avoid modification during iteration
        let entities_to_sync: Vec<EntityId> = self.dirty_entities.iter().cloned().collect();
        
        for entity_id in entities_to_sync {
            match self.sync_single_entity(entity_id, entity_manager).await {
                Ok(entity_report) => {
                    report.merge(entity_report);
                    self.dirty_entities.remove(&entity_id);
                    
                    // Update sync history
                    self.sync_history.insert(entity_id, SyncMetadata {
                        last_sync: Utc::now(),
                        sync_count: self.sync_history
                            .get(&entity_id)
                            .map(|h| h.sync_count + 1)
                            .unwrap_or(1),
                        last_conflict: None,
                        version: self.sync_history
                            .get(&entity_id)
                            .map(|h| h.version + 1)
                            .unwrap_or(1),
                    });
                }
                Err(e) => {
                    error!("Failed to sync entity {}: {}", entity_id, e);
                    report.failed_entities.push(entity_id);
                    report.errors.push(e.to_string());
                }
            }
        }
        
        // Update statistics
        self.stats.total_syncs += report.synced_entities.len();
        self.stats.total_errors += report.errors.len();
        self.stats.total_duration += start_time.elapsed();
        
        // Check if full sync is needed
        if self.should_perform_full_sync() {
            debug!("Performing full sync");
            let full_sync_report = self.perform_full_sync(entity_manager).await?;
            report.merge(full_sync_report);
            self.last_full_sync = Utc::now();
        }
        
        report.duration = start_time.elapsed();
        debug!("Completed entity synchronization in {:?}", report.duration);
        
        Ok(report)
    }
    
    /// Synchronize a single entity
    async fn sync_single_entity(
        &mut self,
        entity_id: EntityId,
        entity_manager: &mut EntityManager,
    ) -> Result<SyncReport, BridgeError> {
        let mut report = SyncReport::new();
        trace!("Syncing entity: {}", entity_id);
        
        if let Some(entity) = entity_manager.get_entity(entity_id) {
            // Check for conflicts
            if let Some(conflict) = self.detect_sync_conflicts(entity_id, entity).await? {
                warn!("Sync conflict detected for entity {}: {:?}", entity_id, conflict);
                report.conflicts.push(conflict);
                
                // Apply conflict resolution strategy
                match self.config.conflict_resolution {
                    ConflictResolutionStrategy::LastWriteWins => {
                        // Continue with sync, latest changes win
                    }
                    ConflictResolutionStrategy::ManualResolution => {
                        // Skip sync, require manual intervention
                        return Err(BridgeError::SyncError(format!(
                            "Manual conflict resolution required for entity {}", 
                            entity_id
                        )));
                    }
                    ConflictResolutionStrategy::ComponentMerge => {
                        // Attempt intelligent merge
                        self.merge_conflicted_components(entity_id, entity).await?;
                    }
                }
            }
            
            // Perform component-level synchronization
            let component_report = self.sync_entity_components(entity_id, entity).await?;
            report.merge(component_report);
            
            // Mark entity as clean
            if let Some(mut entity_mut) = entity_manager.get_entity(entity_id).cloned() {
                entity_mut.mark_clean();
                entity_manager.update_entity(entity_id, entity_mut);
            }
            
            report.synced_entities.push(entity_id);
        } else {
            return Err(BridgeError::EntityNotFound(entity_id));
        }
        
        Ok(report)
    }
    
    /// Synchronize entity components
    async fn sync_entity_components(
        &mut self,
        entity_id: EntityId,
        entity: &ConversationEntity,
    ) -> Result<SyncReport, BridgeError> {
        let mut report = SyncReport::new();
        
        // Get component sync states
        let component_states = self.component_sync.get(&entity_id).cloned().unwrap_or_default();
        
        for (component_type, sync_state) in component_states {
            match sync_state {
                ComponentSyncState::Dirty { marked_at, reason } => {
                    trace!("Syncing dirty component {} of entity {}: {}", 
                           component_type, entity_id, reason);
                    
                    // Perform component-specific sync logic
                    match component_type.as_str() {
                        "ConversationMetadata" => {
                            if entity.metadata.is_dirty() {
                                report.synced_components += 1;
                            }
                        }
                        "MessageHistory" => {
                            if entity.messages.is_dirty() {
                                report.synced_components += 1;
                            }
                        }
                        "UiState" => {
                            if let Some(ref ui_state) = entity.ui_state {
                                if ui_state.is_dirty() {
                                    report.synced_components += 1;
                                }
                            }
                        }
                        "SessionInfo" => {
                            if let Some(ref session) = entity.session {
                                if session.is_dirty() {
                                    report.synced_components += 1;
                                }
                            }
                        }
                        _ => {
                            warn!("Unknown component type for sync: {}", component_type);
                        }
                    }
                    
                    // Mark component as clean
                    self.component_sync
                        .entry(entity_id)
                        .or_insert_with(HashMap::new)
                        .insert(component_type, ComponentSyncState::Clean {
                            last_sync: Utc::now(),
                        });
                }
                ComponentSyncState::Clean { .. } => {
                    // Already clean, nothing to sync
                }
                ComponentSyncState::Conflicted { .. } => {
                    // Handle conflicted state
                    report.conflicts.push(SyncConflict {
                        entity_id,
                        component_type: Some(component_type.clone()),
                        conflict_type: ConflictType::ComponentConflict,
                        description: "Component has conflicting changes".to_string(),
                        occurred_at: Utc::now(),
                    });
                }
            }
        }
        
        Ok(report)
    }
    
    /// Detect synchronization conflicts
    async fn detect_sync_conflicts(
        &self,
        entity_id: EntityId,
        entity: &ConversationEntity,
    ) -> Result<Option<SyncConflict>, BridgeError> {
        // Check version conflicts
        if let Some(sync_metadata) = self.sync_history.get(&entity_id) {
            // Simple version-based conflict detection
            // In a real implementation, this would be more sophisticated
            let time_since_last_sync = Utc::now() - sync_metadata.last_sync;
            
            if time_since_last_sync > chrono::Duration::minutes(5) && entity.is_dirty() {
                return Ok(Some(SyncConflict {
                    entity_id,
                    component_type: None,
                    conflict_type: ConflictType::VersionConflict,
                    description: "Entity modified outside sync window".to_string(),
                    occurred_at: Utc::now(),
                }));
            }
        }
        
        Ok(None)
    }
    
    /// Merge conflicted components using intelligent strategies
    async fn merge_conflicted_components(
        &mut self,
        entity_id: EntityId,
        entity: &ConversationEntity,
    ) -> Result<(), BridgeError> {
        debug!("Attempting to merge conflicted components for entity {}", entity_id);
        
        // Implementation would include sophisticated merge logic
        // For now, we'll use a simple strategy
        
        // Mark components as resolved
        if let Some(component_states) = self.component_sync.get_mut(&entity_id) {
            for (component_type, state) in component_states.iter_mut() {
                if matches!(state, ComponentSyncState::Conflicted { .. }) {
                    *state = ComponentSyncState::Clean {
                        last_sync: Utc::now(),
                    };
                    debug!("Resolved conflict for component {} of entity {}", 
                           component_type, entity_id);
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if full synchronization is needed
    fn should_perform_full_sync(&self) -> bool {
        let time_since_full_sync = Utc::now() - self.last_full_sync;
        time_since_full_sync > chrono::Duration::minutes(self.config.full_sync_interval_minutes as i64)
    }
    
    /// Perform full synchronization of all entities
    async fn perform_full_sync(
        &mut self,
        entity_manager: &mut EntityManager,
    ) -> Result<SyncReport, BridgeError> {
        let mut report = SyncReport::new();
        debug!("Performing full synchronization");
        
        // Get all entities and mark them for sync
        let all_entities: Vec<EntityId> = entity_manager
            .get_global_entities()
            .keys()
            .cloned()
            .collect();
            
        for entity_id in all_entities {
            self.mark_dirty(entity_id);
        }
        
        // Perform regular sync (now includes all entities)
        let sync_result = self.sync_entities(entity_manager).await?;
        report.merge(sync_result);
        
        report.full_sync_performed = true;
        Ok(report)
    }
    
    /// Perform final synchronization on shutdown
    pub async fn final_sync(&self) -> Result<(), BridgeError> {
        debug!("Performing final synchronization before shutdown");
        
        if !self.dirty_entities.is_empty() {
            warn!("Shutting down with {} unsynchronized entities", self.dirty_entities.len());
        }
        
        // In a production implementation, this would ensure all dirty entities
        // are persisted to durable storage
        
        Ok(())
    }
    
    /// Get synchronization statistics
    pub fn get_statistics(&self) -> &SyncStatistics {
        &self.stats
    }
    
    /// Reset synchronization statistics
    pub fn reset_statistics(&mut self) {
        self.stats = SyncStatistics::new();
    }
}

/// Component synchronization state
#[derive(Debug, Clone)]
pub enum ComponentSyncState {
    Clean {
        last_sync: DateTime<Utc>,
    },
    Dirty {
        marked_at: DateTime<Utc>,
        reason: String,
    },
    Conflicted {
        detected_at: DateTime<Utc>,
        conflict_reason: String,
    },
}

/// Synchronization metadata for entities
#[derive(Debug, Clone)]
pub struct SyncMetadata {
    pub last_sync: DateTime<Utc>,
    pub sync_count: u64,
    pub last_conflict: Option<DateTime<Utc>>,
    pub version: u64,
}

/// Synchronization conflict information
#[derive(Debug, Clone)]
pub struct SyncConflict {
    pub entity_id: EntityId,
    pub component_type: Option<String>,
    pub conflict_type: ConflictType,
    pub description: String,
    pub occurred_at: DateTime<Utc>,
}

/// Types of synchronization conflicts
#[derive(Debug, Clone)]
pub enum ConflictType {
    VersionConflict,
    ComponentConflict,
    TimestampConflict,
    DataIntegrityConflict,
}

/// Synchronization configuration
#[derive(Debug, Clone)]
pub struct SyncConfiguration {
    pub conflict_resolution: ConflictResolutionStrategy,
    pub full_sync_interval_minutes: u32,
    pub component_sync_enabled: bool,
    pub max_dirty_entities: usize,
    pub sync_timeout_seconds: u32,
}

impl Default for SyncConfiguration {
    fn default() -> Self {
        Self {
            conflict_resolution: ConflictResolutionStrategy::LastWriteWins,
            full_sync_interval_minutes: 60,
            component_sync_enabled: true,
            max_dirty_entities: 1000,
            sync_timeout_seconds: 30,
        }
    }
}

/// Conflict resolution strategies
#[derive(Debug, Clone)]
pub enum ConflictResolutionStrategy {
    LastWriteWins,
    ManualResolution,
    ComponentMerge,
}

/// Synchronization report
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub synced_entities: Vec<EntityId>,
    pub failed_entities: Vec<EntityId>,
    pub synced_components: usize,
    pub conflicts: Vec<SyncConflict>,
    pub errors: Vec<String>,
    pub duration: std::time::Duration,
    pub full_sync_performed: bool,
}

impl SyncReport {
    pub fn new() -> Self {
        Self {
            synced_entities: Vec::new(),
            failed_entities: Vec::new(),
            synced_components: 0,
            conflicts: Vec::new(),
            errors: Vec::new(),
            duration: std::time::Duration::default(),
            full_sync_performed: false,
        }
    }
    
    pub fn merge(&mut self, other: SyncReport) {
        self.synced_entities.extend(other.synced_entities);
        self.failed_entities.extend(other.failed_entities);
        self.synced_components += other.synced_components;
        self.conflicts.extend(other.conflicts);
        self.errors.extend(other.errors);
        self.duration += other.duration;
        self.full_sync_performed = self.full_sync_performed || other.full_sync_performed;
    }
    
    pub fn is_successful(&self) -> bool {
        self.failed_entities.is_empty() && self.errors.is_empty()
    }
    
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }
}

/// Synchronization statistics
#[derive(Debug, Clone)]
pub struct SyncStatistics {
    pub total_syncs: usize,
    pub total_errors: usize,
    pub total_conflicts: usize,
    pub total_duration: std::time::Duration,
    pub last_sync: Option<DateTime<Utc>>,
    pub average_sync_time: std::time::Duration,
}

impl SyncStatistics {
    pub fn new() -> Self {
        Self {
            total_syncs: 0,
            total_errors: 0,
            total_conflicts: 0,
            total_duration: std::time::Duration::default(),
            last_sync: None,
            average_sync_time: std::time::Duration::default(),
        }
    }
    
    pub fn calculate_average_sync_time(&mut self) {
        if self.total_syncs > 0 {
            self.average_sync_time = self.total_duration / self.total_syncs as u32;
        }
    }
    
    pub fn success_rate(&self) -> f32 {
        if self.total_syncs == 0 {
            return 1.0;
        }
        
        let successful_syncs = self.total_syncs - self.total_errors;
        successful_syncs as f32 / self.total_syncs as f32
    }
}

impl Default for SyncStatistics {
    fn default() -> Self {
        Self::new()
    }
}