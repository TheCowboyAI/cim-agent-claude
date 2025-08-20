// ECS Commands - Async operations dispatched from TEA layer
//
// Commands represent actions that the TEA display layer wants to perform
// in the asynchronous ECS communication layer. These are fire-and-forget
// operations that don't block the UI.

use super::*;

/// Commands sent from TEA display layer to ECS communication layer
#[derive(Debug, Clone)]
pub enum EcsCommand {
    /// Send a message to Claude via NATS
    SendMessageToNats {
        conversation_id: EntityId,
        content: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Create a new conversation
    CreateConversation {
        title: String,
    },
    
    /// Update application configuration
    UpdateConfiguration {
        config_key: String,
        value: serde_json::Value,
    },
    
    /// Invoke a tool via NATS
    InvokeTool {
        tool_id: String,
        parameters: serde_json::Value,
        conversation_id: EntityId,
    },
    
    /// Archive a conversation
    ArchiveConversation {
        conversation_id: EntityId,
        reason: Option<String>,
    },
    
    /// Pause a conversation
    PauseConversation {
        conversation_id: EntityId,
    },
    
    /// Resume a conversation
    ResumeConversation {
        conversation_id: EntityId,
    },
    
    /// Save conversation to persistent storage
    SaveConversation {
        conversation_id: EntityId,
    },
    
    /// Load conversation from persistent storage
    LoadConversation {
        conversation_id: EntityId,
    },
    
    /// Search conversations
    SearchConversations {
        query: String,
        limit: Option<usize>,
    },
    
    /// Update user preferences
    UpdateUserPreferences {
        preferences: UserPreferences,
    },
    
    /// Export conversation data
    ExportConversation {
        conversation_id: EntityId,
        format: ExportFormat,
    },
    
    /// Import conversation data
    ImportConversation {
        data: Vec<u8>,
        format: ImportFormat,
    },
    
    /// Health check for external systems
    HealthCheck,
    
    /// Cleanup expired data
    Cleanup {
        older_than: DateTime<Utc>,
    },
}

impl EcsCommand {
    /// Get command type as string for logging and metrics
    pub fn command_type(&self) -> &'static str {
        match self {
            Self::SendMessageToNats { .. } => "send_message",
            Self::CreateConversation { .. } => "create_conversation",
            Self::UpdateConfiguration { .. } => "update_config",
            Self::InvokeTool { .. } => "invoke_tool",
            Self::ArchiveConversation { .. } => "archive_conversation",
            Self::PauseConversation { .. } => "pause_conversation",
            Self::ResumeConversation { .. } => "resume_conversation",
            Self::SaveConversation { .. } => "save_conversation",
            Self::LoadConversation { .. } => "load_conversation",
            Self::SearchConversations { .. } => "search_conversations",
            Self::UpdateUserPreferences { .. } => "update_preferences",
            Self::ExportConversation { .. } => "export_conversation",
            Self::ImportConversation { .. } => "import_conversation",
            Self::HealthCheck => "health_check",
            Self::Cleanup { .. } => "cleanup",
        }
    }
    
    /// Get conversation ID if applicable
    pub fn conversation_id(&self) -> Option<EntityId> {
        match self {
            Self::SendMessageToNats { conversation_id, .. } |
            Self::InvokeTool { conversation_id, .. } |
            Self::ArchiveConversation { conversation_id, .. } |
            Self::PauseConversation { conversation_id } |
            Self::ResumeConversation { conversation_id } |
            Self::SaveConversation { conversation_id } |
            Self::LoadConversation { conversation_id } |
            Self::ExportConversation { conversation_id, .. } => Some(*conversation_id),
            _ => None,
        }
    }
    
    /// Check if command requires immediate response
    pub fn requires_immediate_response(&self) -> bool {
        matches!(self, 
            Self::LoadConversation { .. } |
            Self::SearchConversations { .. } |
            Self::HealthCheck
        )
    }
    
    /// Get command priority (higher number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            Self::HealthCheck => 255,
            Self::SendMessageToNats { .. } => 200,
            Self::LoadConversation { .. } => 150,
            Self::CreateConversation { .. } => 100,
            Self::InvokeTool { .. } => 90,
            Self::UpdateConfiguration { .. } => 80,
            Self::SearchConversations { .. } => 70,
            Self::SaveConversation { .. } => 50,
            Self::UpdateUserPreferences { .. } => 40,
            Self::PauseConversation { .. } |
            Self::ResumeConversation { .. } => 30,
            Self::ArchiveConversation { .. } => 20,
            Self::ExportConversation { .. } |
            Self::ImportConversation { .. } => 10,
            Self::Cleanup { .. } => 1,
        }
    }
}

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Yaml,
    Markdown,
    PlainText,
    Csv,
}

/// Import format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportFormat {
    Json,
    Yaml,
    ChatGptExport,
    ClaudeExport,
}

/// Command builder for creating complex commands
pub struct EcsCommandBuilder {
    command: Option<EcsCommand>,
}

impl EcsCommandBuilder {
    pub fn new() -> Self {
        Self { command: None }
    }
    
    /// Build a send message command
    pub fn send_message(conversation_id: EntityId, content: String) -> Self {
        Self {
            command: Some(EcsCommand::SendMessageToNats {
                conversation_id,
                content,
                timestamp: Utc::now(),
            })
        }
    }
    
    /// Build a create conversation command
    pub fn create_conversation(title: String) -> Self {
        Self {
            command: Some(EcsCommand::CreateConversation { title })
        }
    }
    
    /// Build an invoke tool command
    pub fn invoke_tool(tool_id: String, parameters: serde_json::Value, conversation_id: EntityId) -> Self {
        Self {
            command: Some(EcsCommand::InvokeTool {
                tool_id,
                parameters,
                conversation_id,
            })
        }
    }
    
    /// Build a configuration update command
    pub fn update_config<T: Serialize>(key: String, value: T) -> Result<Self, serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        Ok(Self {
            command: Some(EcsCommand::UpdateConfiguration {
                config_key: key,
                value: json_value,
            })
        })
    }
    
    /// Build a search command
    pub fn search_conversations(query: String) -> Self {
        Self {
            command: Some(EcsCommand::SearchConversations {
                query,
                limit: None,
            })
        }
    }
    
    /// Add limit to search command
    pub fn with_limit(mut self, limit: usize) -> Self {
        if let Some(EcsCommand::SearchConversations { ref mut limit: cmd_limit, .. }) = self.command {
            *cmd_limit = Some(limit);
        }
        self
    }
    
    /// Build the command
    pub fn build(self) -> Option<EcsCommand> {
        self.command
    }
}

impl Default for EcsCommandBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Command execution context
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub command_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub issued_by: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub timeout: Option<std::time::Duration>,
}

impl CommandContext {
    pub fn new() -> Self {
        Self {
            command_id: Uuid::new_v4(),
            issued_at: Utc::now(),
            issued_by: None,
            correlation_id: None,
            timeout: None,
        }
    }
    
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn with_issuer(mut self, issuer: String) -> Self {
        self.issued_by = Some(issuer);
        self
    }
}

impl Default for CommandContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Command execution result
#[derive(Debug, Clone)]
pub enum CommandResult {
    Success {
        command_id: Uuid,
        events: Vec<super::TeaEvent>,
        execution_time: std::time::Duration,
    },
    
    Failure {
        command_id: Uuid,
        error: String,
        execution_time: std::time::Duration,
    },
    
    Timeout {
        command_id: Uuid,
        timeout_duration: std::time::Duration,
    },
}

impl CommandResult {
    pub fn command_id(&self) -> Uuid {
        match self {
            Self::Success { command_id, .. } |
            Self::Failure { command_id, .. } |
            Self::Timeout { command_id, .. } => *command_id
        }
    }
    
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }
    
    pub fn execution_time(&self) -> Option<std::time::Duration> {
        match self {
            Self::Success { execution_time, .. } |
            Self::Failure { execution_time, .. } => Some(*execution_time),
            Self::Timeout { timeout_duration, .. } => Some(*timeout_duration),
        }
    }
    
    pub fn events(self) -> Vec<super::TeaEvent> {
        match self {
            Self::Success { events, .. } => events,
            _ => Vec::new(),
        }
    }
}