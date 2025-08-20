// Entity-Component Model Implementation
//
// Models represent Entity[Components] structures shared between TEA and ECS layers.
// Components are pure data containers with no inherent behavior.
// Models can exist at multiple scopes: global, feature, component, session, cache.

use super::*;
use crate::domain::{
    conversation_aggregate::ConversationId,
    value_objects::{MessageId, UserId},
};

/// Core trait for all components in the system
pub trait Component: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + std::fmt::Debug {
    /// Component type identifier for runtime dispatch
    fn component_type(&self) -> &'static str;
    
    /// Whether this component has been modified since last sync
    fn is_dirty(&self) -> bool;
    
    /// Mark component as clean after sync
    fn mark_clean(&mut self);
    
    /// Mark component as dirty for next sync
    fn mark_dirty(&mut self);
}

/// Metadata component for conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub participant_count: u32,
    pub status: ConversationStatus,
    pub tags: Vec<String>,
    
    // Component state tracking
    #[serde(skip)]
    dirty: bool,
}

impl Component for ConversationMetadata {
    fn component_type(&self) -> &'static str {
        "ConversationMetadata"
    }
    
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    fn mark_clean(&mut self) {
        self.dirty = false;
    }
    
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl ConversationMetadata {
    pub fn new(title: String) -> Self {
        Self {
            title,
            created_at: Utc::now(),
            last_active: Utc::now(),
            participant_count: 1,
            status: ConversationStatus::Active,
            tags: Vec::new(),
            dirty: true,
        }
    }
    
    pub fn update_activity(&mut self) {
        self.last_active = Utc::now();
        self.mark_dirty();
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.mark_dirty();
        }
    }
}

/// Message history component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistory {
    pub messages: Vec<ConversationMessage>,
    pub total_tokens: u32,
    pub cost_estimate: f64,
    pub message_count: u32,
    
    #[serde(skip)]
    dirty: bool,
}

impl Component for MessageHistory {
    fn component_type(&self) -> &'static str {
        "MessageHistory"
    }
    
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    fn mark_clean(&mut self) {
        self.dirty = false;
    }
    
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl MessageHistory {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            total_tokens: 0,
            cost_estimate: 0.0,
            message_count: 0,
            dirty: false,
        }
    }
    
    pub fn add_message(&mut self, message: ConversationMessage) {
        self.messages.push(message);
        self.message_count += 1;
        if let Some(tokens) = message.token_count {
            self.total_tokens += tokens;
            // Rough cost estimation (adjust based on actual Claude pricing)
            self.cost_estimate += (tokens as f64) * 0.008 / 1000.0;
        }
        self.mark_dirty();
    }
    
    pub fn get_recent_messages(&self, limit: usize) -> &[ConversationMessage] {
        let start = self.messages.len().saturating_sub(limit);
        &self.messages[start..]
    }
}

/// Individual conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: MessageId,
    pub content: String,
    pub role: MessageRole,
    pub timestamp: DateTime<Utc>,
    pub token_count: Option<u32>,
    pub tool_calls: Vec<ToolCall>,
    pub attachments: Vec<AttachmentReference>,
}

/// Message role enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Tool call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub parameters: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Reference to attachment in object store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentReference {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: u64,
    pub object_store_key: String,
}

/// Conversation status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversationStatus {
    Active,
    Paused,
    Archived,
    Error,
}

/// UI state component for display layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiState {
    pub selected_conversation: Option<EntityId>,
    pub input_buffer: String,
    pub view_mode: ViewMode,
    pub sidebar_collapsed: bool,
    pub theme: UiTheme,
    pub error_message: Option<String>,
    
    #[serde(skip)]
    dirty: bool,
}

impl Component for UiState {
    fn component_type(&self) -> &'static str {
        "UiState"
    }
    
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    fn mark_clean(&mut self) {
        self.dirty = false;
    }
    
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewMode {
    ConversationList,
    ChatView,
    Settings,
    Help,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiTheme {
    Light,
    Dark,
    Auto,
}

/// Session information component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user_id: Option<UserId>,
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub preferences: UserPreferences,
    
    #[serde(skip)]
    dirty: bool,
}

impl Component for SessionInfo {
    fn component_type(&self) -> &'static str {
        "SessionInfo"
    }
    
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    fn mark_clean(&mut self) {
        self.dirty = false;
    }
    
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: UiTheme,
    pub auto_save: bool,
    pub notification_enabled: bool,
    pub default_model: String,
}

/// Entity representing a complete conversation
#[derive(Debug, Clone)]
pub struct ConversationEntity {
    pub id: EntityId,
    pub metadata: ConversationMetadata,
    pub messages: MessageHistory,
    pub ui_state: Option<UiState>,
    pub session: Option<SessionInfo>,
}

impl ConversationEntity {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata: ConversationMetadata::new(title),
            messages: MessageHistory::new(),
            ui_state: None,
            session: None,
        }
    }
    
    pub fn with_ui_state(mut self, ui_state: UiState) -> Self {
        self.ui_state = Some(ui_state);
        self
    }
    
    pub fn with_session(mut self, session: SessionInfo) -> Self {
        self.session = Some(session);
        self
    }
    
    /// Check if any component is dirty (needs sync)
    pub fn is_dirty(&self) -> bool {
        self.metadata.is_dirty() ||
        self.messages.is_dirty() ||
        self.ui_state.as_ref().map_or(false, |ui| ui.is_dirty()) ||
        self.session.as_ref().map_or(false, |s| s.is_dirty())
    }
    
    /// Mark all components as clean
    pub fn mark_clean(&mut self) {
        self.metadata.mark_clean();
        self.messages.mark_clean();
        if let Some(ref mut ui) = self.ui_state {
            ui.mark_clean();
        }
        if let Some(ref mut session) = self.session {
            session.mark_clean();
        }
    }
    
    /// Get all dirty components for partial sync
    pub fn get_dirty_components(&self) -> Vec<&dyn Component> {
        let mut dirty = Vec::new();
        
        if self.metadata.is_dirty() {
            dirty.push(&self.metadata as &dyn Component);
        }
        if self.messages.is_dirty() {
            dirty.push(&self.messages as &dyn Component);
        }
        if let Some(ref ui) = self.ui_state {
            if ui.is_dirty() {
                dirty.push(ui as &dyn Component);
            }
        }
        if let Some(ref session) = self.session {
            if session.is_dirty() {
                dirty.push(session as &dyn Component);
            }
        }
        
        dirty
    }
}

/// Entity manager for handling multiple entities at different scopes
#[derive(Debug)]
pub struct EntityManager {
    // Global entities (application-wide state)
    global_entities: HashMap<EntityId, ConversationEntity>,
    
    // Feature entities (scoped to specific features)
    feature_entities: HashMap<String, HashMap<EntityId, ConversationEntity>>,
    
    // Component entities (local to UI components)
    component_entities: HashMap<String, HashMap<EntityId, ConversationEntity>>,
    
    // Session entities (temporary state)
    session_entities: HashMap<String, HashMap<EntityId, ConversationEntity>>,
    
    // Cache entities (performance optimization)
    cache_entities: HashMap<EntityId, ConversationEntity>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            global_entities: HashMap::new(),
            feature_entities: HashMap::new(),
            component_entities: HashMap::new(),
            session_entities: HashMap::new(),
            cache_entities: HashMap::new(),
        }
    }
    
    /// Add entity to global scope
    pub fn add_global_entity(&mut self, entity: ConversationEntity) {
        self.global_entities.insert(entity.id, entity);
    }
    
    /// Add entity to feature scope
    pub fn add_feature_entity(&mut self, feature: String, entity: ConversationEntity) {
        self.feature_entities
            .entry(feature)
            .or_insert_with(HashMap::new)
            .insert(entity.id, entity);
    }
    
    /// Get entity from any scope
    pub fn get_entity(&self, id: EntityId) -> Option<&ConversationEntity> {
        // Try cache first for performance
        if let Some(entity) = self.cache_entities.get(&id) {
            return Some(entity);
        }
        
        // Check global
        if let Some(entity) = self.global_entities.get(&id) {
            return Some(entity);
        }
        
        // Check all feature scopes
        for feature_map in self.feature_entities.values() {
            if let Some(entity) = feature_map.get(&id) {
                return Some(entity);
            }
        }
        
        // Check component scopes
        for component_map in self.component_entities.values() {
            if let Some(entity) = component_map.get(&id) {
                return Some(entity);
            }
        }
        
        // Check session scopes
        for session_map in self.session_entities.values() {
            if let Some(entity) = session_map.get(&id) {
                return Some(entity);
            }
        }
        
        None
    }
    
    /// Update entity in appropriate scope
    pub fn update_entity(&mut self, id: EntityId, updated_entity: ConversationEntity) {
        // Update in all scopes where it exists
        if self.global_entities.contains_key(&id) {
            self.global_entities.insert(id, updated_entity.clone());
        }
        
        for feature_map in self.feature_entities.values_mut() {
            if feature_map.contains_key(&id) {
                feature_map.insert(id, updated_entity.clone());
            }
        }
        
        for component_map in self.component_entities.values_mut() {
            if component_map.contains_key(&id) {
                component_map.insert(id, updated_entity.clone());
            }
        }
        
        for session_map in self.session_entities.values_mut() {
            if session_map.contains_key(&id) {
                session_map.insert(id, updated_entity.clone());
            }
        }
        
        // Always update cache
        self.cache_entities.insert(id, updated_entity);
    }
    
    /// Get all entities that need synchronization
    pub fn get_dirty_entities(&self) -> Vec<(EntityId, &ConversationEntity)> {
        let mut dirty = Vec::new();
        
        for (id, entity) in &self.global_entities {
            if entity.is_dirty() {
                dirty.push(*id, entity);
            }
        }
        
        for feature_map in self.feature_entities.values() {
            for (id, entity) in feature_map {
                if entity.is_dirty() {
                    dirty.push(*id, entity);
                }
            }
        }
        
        dirty
    }
    
    /// Get all global entities
    pub fn get_global_entities(&self) -> &HashMap<EntityId, ConversationEntity> {
        &self.global_entities
    }
    
    /// Get entities for specific feature
    pub fn get_feature_entities(&self, feature: &str) -> Option<&HashMap<EntityId, ConversationEntity>> {
        self.feature_entities.get(feature)
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}