// TEA Events - Updates sent from ECS layer to TEA display layer
//
// Events represent state changes that occurred in the ECS communication layer
// that need to be reflected in the TEA display layer. These events trigger
// synchronous model updates in the UI.

use super::*;

/// Events sent from ECS communication layer to TEA display layer
#[derive(Debug, Clone)]
pub enum TeaEvent {
    /// New conversation was created
    ConversationCreated {
        conversation_id: EntityId,
        timestamp: DateTime<Utc>,
    },
    
    /// Conversation was updated
    ConversationUpdated {
        conversation_id: EntityId,
        entity: ConversationEntity,
        timestamp: DateTime<Utc>,
    },
    
    /// Message was added to conversation
    MessageAdded {
        conversation_id: EntityId,
        message_content: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Claude response received
    ClaudeResponseReceived {
        conversation_id: EntityId,
        response_content: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Tool invocation completed
    ToolInvocationCompleted {
        conversation_id: EntityId,
        tool_id: String,
        result: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    
    /// Tool invocation failed
    ToolInvocationFailed {
        conversation_id: EntityId,
        tool_id: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Configuration was updated
    ConfigurationUpdated {
        config_key: String,
        new_value: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    
    /// Conversation status changed
    ConversationStatusChanged {
        conversation_id: EntityId,
        old_status: ConversationStatus,
        new_status: ConversationStatus,
        timestamp: DateTime<Utc>,
    },
    
    /// Search results available
    SearchResultsReady {
        query: String,
        results: Vec<ConversationSearchResult>,
        total_count: usize,
        timestamp: DateTime<Utc>,
    },
    
    /// User preferences updated
    UserPreferencesUpdated {
        preferences: UserPreferences,
        timestamp: DateTime<Utc>,
    },
    
    /// Export completed
    ExportCompleted {
        conversation_id: EntityId,
        format: ExportFormat,
        file_path: Option<String>,
        timestamp: DateTime<Utc>,
    },
    
    /// Import completed
    ImportCompleted {
        conversation_id: EntityId,
        format: ImportFormat,
        message_count: usize,
        timestamp: DateTime<Utc>,
    },
    
    /// Error occurred in ECS layer
    ErrorOccurred {
        error: String,
        timestamp: DateTime<Utc>,
    },
    
    /// System health status
    HealthStatusUpdate {
        status: SystemHealthStatus,
        timestamp: DateTime<Utc>,
    },
    
    /// Connection status changed
    ConnectionStatusChanged {
        service: String,
        connected: bool,
        timestamp: DateTime<Utc>,
    },
    
    /// Rate limit warning
    RateLimitWarning {
        service: String,
        requests_remaining: u32,
        reset_time: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },
    
    /// Cleanup completed
    CleanupCompleted {
        items_removed: u32,
        bytes_freed: u64,
        timestamp: DateTime<Utc>,
    },
    
    /// Entity synchronized
    EntitySynchronized {
        entity_id: EntityId,
        sync_type: SyncType,
        timestamp: DateTime<Utc>,
    },
}

impl TeaEvent {
    /// Get event type as string for logging and metrics
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::ConversationCreated { .. } => "conversation_created",
            Self::ConversationUpdated { .. } => "conversation_updated",
            Self::MessageAdded { .. } => "message_added",
            Self::ClaudeResponseReceived { .. } => "claude_response_received",
            Self::ToolInvocationCompleted { .. } => "tool_invocation_completed",
            Self::ToolInvocationFailed { .. } => "tool_invocation_failed",
            Self::ConfigurationUpdated { .. } => "configuration_updated",
            Self::ConversationStatusChanged { .. } => "conversation_status_changed",
            Self::SearchResultsReady { .. } => "search_results_ready",
            Self::UserPreferencesUpdated { .. } => "user_preferences_updated",
            Self::ExportCompleted { .. } => "export_completed",
            Self::ImportCompleted { .. } => "import_completed",
            Self::ErrorOccurred { .. } => "error_occurred",
            Self::HealthStatusUpdate { .. } => "health_status_update",
            Self::ConnectionStatusChanged { .. } => "connection_status_changed",
            Self::RateLimitWarning { .. } => "rate_limit_warning",
            Self::CleanupCompleted { .. } => "cleanup_completed",
            Self::EntitySynchronized { .. } => "entity_synchronized",
        }
    }
    
    /// Get timestamp of the event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::ConversationCreated { timestamp, .. } |
            Self::ConversationUpdated { timestamp, .. } |
            Self::MessageAdded { timestamp, .. } |
            Self::ClaudeResponseReceived { timestamp, .. } |
            Self::ToolInvocationCompleted { timestamp, .. } |
            Self::ToolInvocationFailed { timestamp, .. } |
            Self::ConfigurationUpdated { timestamp, .. } |
            Self::ConversationStatusChanged { timestamp, .. } |
            Self::SearchResultsReady { timestamp, .. } |
            Self::UserPreferencesUpdated { timestamp, .. } |
            Self::ExportCompleted { timestamp, .. } |
            Self::ImportCompleted { timestamp, .. } |
            Self::ErrorOccurred { timestamp, .. } |
            Self::HealthStatusUpdate { timestamp, .. } |
            Self::ConnectionStatusChanged { timestamp, .. } |
            Self::RateLimitWarning { timestamp, .. } |
            Self::CleanupCompleted { timestamp, .. } |
            Self::EntitySynchronized { timestamp, .. } => *timestamp,
        }
    }
    
    /// Get conversation ID if applicable
    pub fn conversation_id(&self) -> Option<EntityId> {
        match self {
            Self::ConversationCreated { conversation_id, .. } |
            Self::ConversationUpdated { conversation_id, .. } |
            Self::MessageAdded { conversation_id, .. } |
            Self::ClaudeResponseReceived { conversation_id, .. } |
            Self::ToolInvocationCompleted { conversation_id, .. } |
            Self::ToolInvocationFailed { conversation_id, .. } |
            Self::ConversationStatusChanged { conversation_id, .. } |
            Self::ExportCompleted { conversation_id, .. } |
            Self::ImportCompleted { conversation_id, .. } => Some(*conversation_id),
            _ => None,
        }
    }
    
    /// Check if event is an error
    pub fn is_error(&self) -> bool {
        matches!(self, 
            Self::ErrorOccurred { .. } |
            Self::ToolInvocationFailed { .. }
        )
    }
    
    /// Check if event should trigger UI update
    pub fn should_update_ui(&self) -> bool {
        match self {
            Self::ConversationCreated { .. } |
            Self::ConversationUpdated { .. } |
            Self::MessageAdded { .. } |
            Self::ClaudeResponseReceived { .. } |
            Self::ConversationStatusChanged { .. } |
            Self::SearchResultsReady { .. } |
            Self::UserPreferencesUpdated { .. } |
            Self::ErrorOccurred { .. } |
            Self::ConnectionStatusChanged { .. } => true,
            
            Self::ToolInvocationCompleted { .. } |
            Self::ToolInvocationFailed { .. } |
            Self::ExportCompleted { .. } |
            Self::ImportCompleted { .. } |
            Self::CleanupCompleted { .. } => true,
            
            // Background events that don't require immediate UI update
            Self::ConfigurationUpdated { .. } |
            Self::HealthStatusUpdate { .. } |
            Self::RateLimitWarning { .. } |
            Self::EntitySynchronized { .. } => false,
        }
    }
    
    /// Get priority for event processing (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            Self::ErrorOccurred { .. } => 255,
            Self::ConnectionStatusChanged { .. } => 200,
            Self::ClaudeResponseReceived { .. } => 180,
            Self::MessageAdded { .. } => 170,
            Self::ConversationCreated { .. } => 150,
            Self::ConversationUpdated { .. } => 140,
            Self::ToolInvocationCompleted { .. } |
            Self::ToolInvocationFailed { .. } => 130,
            Self::ConversationStatusChanged { .. } => 120,
            Self::SearchResultsReady { .. } => 100,
            Self::UserPreferencesUpdated { .. } => 80,
            Self::RateLimitWarning { .. } => 70,
            Self::ExportCompleted { .. } |
            Self::ImportCompleted { .. } => 50,
            Self::ConfigurationUpdated { .. } => 30,
            Self::HealthStatusUpdate { .. } => 20,
            Self::CleanupCompleted { .. } => 10,
            Self::EntitySynchronized { .. } => 1,
        }
    }
}

/// Search result for conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSearchResult {
    pub conversation_id: EntityId,
    pub title: String,
    pub snippet: String,
    pub last_active: DateTime<Utc>,
    pub message_count: u32,
    pub relevance_score: f32,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStatus {
    pub overall_status: HealthStatus,
    pub components: HashMap<String, ComponentHealthStatus>,
    pub last_check: DateTime<Utc>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthStatus {
    pub status: HealthStatus,
    pub response_time: Option<std::time::Duration>,
    pub error_rate: Option<f32>,
    pub last_error: Option<String>,
}

/// Synchronization type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncType {
    FullSync,
    PartialSync,
    ComponentSync(String),
    ErrorRecovery,
}

/// Event builder for creating complex events
pub struct TeaEventBuilder {
    event: Option<TeaEvent>,
}

impl TeaEventBuilder {
    pub fn new() -> Self {
        Self { event: None }
    }
    
    /// Build a conversation created event
    pub fn conversation_created(conversation_id: EntityId) -> Self {
        Self {
            event: Some(TeaEvent::ConversationCreated {
                conversation_id,
                timestamp: Utc::now(),
            })
        }
    }
    
    /// Build a message added event
    pub fn message_added(conversation_id: EntityId, content: String) -> Self {
        Self {
            event: Some(TeaEvent::MessageAdded {
                conversation_id,
                message_content: content,
                timestamp: Utc::now(),
            })
        }
    }
    
    /// Build a Claude response event
    pub fn claude_response(conversation_id: EntityId, response: String) -> Self {
        Self {
            event: Some(TeaEvent::ClaudeResponseReceived {
                conversation_id,
                response_content: response,
                timestamp: Utc::now(),
            })
        }
    }
    
    /// Build an error event
    pub fn error(error_message: String) -> Self {
        Self {
            event: Some(TeaEvent::ErrorOccurred {
                error: error_message,
                timestamp: Utc::now(),
            })
        }
    }
    
    /// Build a status change event
    pub fn status_changed(
        conversation_id: EntityId, 
        old_status: ConversationStatus, 
        new_status: ConversationStatus
    ) -> Self {
        Self {
            event: Some(TeaEvent::ConversationStatusChanged {
                conversation_id,
                old_status,
                new_status,
                timestamp: Utc::now(),
            })
        }
    }
    
    /// Build a search results event
    pub fn search_results(query: String, results: Vec<ConversationSearchResult>) -> Self {
        let total_count = results.len();
        Self {
            event: Some(TeaEvent::SearchResultsReady {
                query,
                results,
                total_count,
                timestamp: Utc::now(),
            })
        }
    }
    
    /// Build the event
    pub fn build(self) -> Option<TeaEvent> {
        self.event
    }
}

impl Default for TeaEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Event filter for selective event processing
pub struct EventFilter {
    event_types: HashSet<String>,
    conversation_ids: HashSet<EntityId>,
    min_priority: u8,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            event_types: HashSet::new(),
            conversation_ids: HashSet::new(),
            min_priority: 0,
        }
    }
    
    pub fn with_event_types(mut self, types: Vec<&str>) -> Self {
        self.event_types = types.into_iter().map(String::from).collect();
        self
    }
    
    pub fn with_conversation_ids(mut self, ids: Vec<EntityId>) -> Self {
        self.conversation_ids = ids.into_iter().collect();
        self
    }
    
    pub fn with_min_priority(mut self, priority: u8) -> Self {
        self.min_priority = priority;
        self
    }
    
    pub fn matches(&self, event: &TeaEvent) -> bool {
        // Check priority
        if event.priority() < self.min_priority {
            return false;
        }
        
        // Check event type filter
        if !self.event_types.is_empty() && !self.event_types.contains(event.event_type()) {
            return false;
        }
        
        // Check conversation ID filter
        if !self.conversation_ids.is_empty() {
            if let Some(conv_id) = event.conversation_id() {
                if !self.conversation_ids.contains(&conv_id) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Event aggregator for batching related events
pub struct EventAggregator {
    pending_events: Vec<TeaEvent>,
    last_flush: DateTime<Utc>,
    flush_interval: chrono::Duration,
}

impl EventAggregator {
    pub fn new(flush_interval_ms: i64) -> Self {
        Self {
            pending_events: Vec::new(),
            last_flush: Utc::now(),
            flush_interval: chrono::Duration::milliseconds(flush_interval_ms),
        }
    }
    
    pub fn add_event(&mut self, event: TeaEvent) {
        self.pending_events.push(event);
    }
    
    pub fn should_flush(&self) -> bool {
        if self.pending_events.is_empty() {
            return false;
        }
        
        let elapsed = Utc::now() - self.last_flush;
        elapsed >= self.flush_interval || self.pending_events.len() >= 10
    }
    
    pub fn flush(&mut self) -> Vec<TeaEvent> {
        let events = std::mem::take(&mut self.pending_events);
        self.last_flush = Utc::now();
        events
    }
    
    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }
}