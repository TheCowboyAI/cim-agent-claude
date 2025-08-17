/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Subject Algebra for Claude Adapter
//! 
//! Implements the standardized CIM subject hierarchy using type-safe enums
//! for all NATS JetStream subjects, streams, object stores, and KV stores.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Top-level CIM domain identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CimDomain {
    /// Core CIM infrastructure
    Core,
    /// Claude AI adapter domain
    Claude,
    /// User interaction domain
    User,
    /// System administration domain
    System,
    /// Security and authentication domain  
    Security,
    /// Monitoring and observability domain
    Monitor,
}

impl fmt::Display for CimDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CimDomain::Core => write!(f, "core"),
            CimDomain::Claude => write!(f, "claude"),
            CimDomain::User => write!(f, "user"),
            CimDomain::System => write!(f, "sys"),
            CimDomain::Security => write!(f, "sec"),
            CimDomain::Monitor => write!(f, "mon"),
        }
    }
}

/// Service identifiers within domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CimService {
    /// Conversation management service
    Conversation,
    /// Attachment handling service
    Attachment,
    /// Event processing service
    Event,
    /// Configuration service
    Config,
    /// Health monitoring service
    Health,
    /// Metrics collection service
    Metrics,
    /// Authentication service
    Auth,
    /// Storage management service
    Storage,
}

impl fmt::Display for CimService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CimService::Conversation => write!(f, "conv"),
            CimService::Attachment => write!(f, "attach"),
            CimService::Event => write!(f, "event"),
            CimService::Config => write!(f, "config"),
            CimService::Health => write!(f, "health"),
            CimService::Metrics => write!(f, "metrics"),
            CimService::Auth => write!(f, "auth"),
            CimService::Storage => write!(f, "store"),
        }
    }
}

/// Operation types for message routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CimOperation {
    /// Command operations (imperative)
    Cmd,
    /// Query operations (read-only)
    Query,
    /// Event notifications (past-tense facts)
    Event,
    /// Request-reply operations
    Req,
    /// Response operations
    Resp,
    /// Stream data operations
    Stream,
}

impl fmt::Display for CimOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CimOperation::Cmd => write!(f, "cmd"),
            CimOperation::Query => write!(f, "qry"),
            CimOperation::Event => write!(f, "evt"),
            CimOperation::Req => write!(f, "req"),
            CimOperation::Resp => write!(f, "resp"),
            CimOperation::Stream => write!(f, "str"),
        }
    }
}

/// Specific command types for Claude adapter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaudeCommand {
    /// Start new conversation
    Start,
    /// Send prompt to conversation
    Send,
    /// End conversation
    End,
    /// Upload attachment
    Upload,
    /// Process screenshot
    Screenshot,
    /// Retrieve conversation history
    History,
}

/// Configuration management commands (separate from Claude API commands)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigCommand {
    /// Update system prompt
    UpdateSystemPrompt,
    /// Update model parameters (temperature, max_tokens, etc.)
    UpdateModelParams,
    /// Update conversation settings
    UpdateConversationSettings,
    /// Reset configuration to defaults
    ResetConfig,
    /// Import configuration from file/template
    ImportConfig,
    /// Export current configuration
    ExportConfig,
}

/// NATS-connected tool commands (MCP tools communicate via NATS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NatsToolCommand {
    /// Register new tool on NATS
    RegisterTool,
    /// Unregister tool from NATS
    UnregisterTool,
    /// Update tool configuration
    UpdateTool,
    /// Enable tool for conversation
    EnableTool,
    /// Disable tool for conversation
    DisableTool,
    /// Invoke tool with parameters (via NATS request-reply)
    InvokeTool,
    /// Health check tool (ping via NATS)
    HealthCheckTool,
}

/// Conversation control commands (separate from content commands)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConversationControlCommand {
    /// Pause conversation processing
    Pause,
    /// Resume conversation processing
    Resume,
    /// Set conversation priority
    SetPriority,
    /// Add conversation tags/labels
    AddTags,
    /// Remove conversation tags/labels
    RemoveTags,
    /// Archive conversation
    Archive,
    /// Restore archived conversation
    Restore,
    /// Transfer conversation to different session
    Transfer,
    /// Fork conversation (create branch)
    Fork,
    /// Merge conversation branches
    Merge,
}

impl fmt::Display for ClaudeCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClaudeCommand::Start => write!(f, "start"),
            ClaudeCommand::Send => write!(f, "send"),
            ClaudeCommand::End => write!(f, "end"),
            ClaudeCommand::Upload => write!(f, "upload"),
            ClaudeCommand::Screenshot => write!(f, "screenshot"),
            ClaudeCommand::History => write!(f, "history"),
        }
    }
}

impl fmt::Display for ConfigCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigCommand::UpdateSystemPrompt => write!(f, "update_system_prompt"),
            ConfigCommand::UpdateModelParams => write!(f, "update_model_params"),
            ConfigCommand::UpdateConversationSettings => write!(f, "update_conversation_settings"),
            ConfigCommand::ResetConfig => write!(f, "reset_config"),
            ConfigCommand::ImportConfig => write!(f, "import_config"),
            ConfigCommand::ExportConfig => write!(f, "export_config"),
        }
    }
}

impl fmt::Display for NatsToolCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NatsToolCommand::RegisterTool => write!(f, "register_tool"),
            NatsToolCommand::UnregisterTool => write!(f, "unregister_tool"),
            NatsToolCommand::UpdateTool => write!(f, "update_tool"),
            NatsToolCommand::EnableTool => write!(f, "enable_tool"),
            NatsToolCommand::DisableTool => write!(f, "disable_tool"),
            NatsToolCommand::InvokeTool => write!(f, "invoke_tool"),
            NatsToolCommand::HealthCheckTool => write!(f, "health_check_tool"),
        }
    }
}

impl fmt::Display for ConversationControlCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversationControlCommand::Pause => write!(f, "pause"),
            ConversationControlCommand::Resume => write!(f, "resume"),
            ConversationControlCommand::SetPriority => write!(f, "set_priority"),
            ConversationControlCommand::AddTags => write!(f, "add_tags"),
            ConversationControlCommand::RemoveTags => write!(f, "remove_tags"),
            ConversationControlCommand::Archive => write!(f, "archive"),
            ConversationControlCommand::Restore => write!(f, "restore"),
            ConversationControlCommand::Transfer => write!(f, "transfer"),
            ConversationControlCommand::Fork => write!(f, "fork"),
            ConversationControlCommand::Merge => write!(f, "merge"),
        }
    }
}

/// Event types for Claude adapter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaudeEvent {
    /// Conversation lifecycle events
    Started,
    Ended,
    /// Message exchange events
    PromptSent,
    ResponseReceived,
    /// Attachment events
    AttachmentUploaded,
    AttachmentProcessed,
    ScreenshotCaptured,
    ScreenshotAnalyzed,
    /// Error events
    RateLimited,
    ApiError,
    ProcessingError,
}

/// Configuration change events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigEvent {
    /// System prompt updated
    SystemPromptUpdated,
    /// Model parameters changed
    ModelParamsUpdated,
    /// Conversation settings changed
    ConversationSettingsUpdated,
    /// Configuration reset to defaults
    ConfigReset,
    /// Configuration imported
    ConfigImported,
    /// Configuration exported
    ConfigExported,
    /// Configuration validation failed
    ConfigValidationFailed,
    /// Configuration backup created
    ConfigBackupCreated,
}

/// NATS-connected tool events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NatsToolEvent {
    /// Tool registered on NATS successfully
    ToolRegistered,
    /// Tool unregistered from NATS
    ToolUnregistered,
    /// Tool configuration updated
    ToolUpdated,
    /// Tool enabled for conversation
    ToolEnabledForConversation,
    /// Tool disabled for conversation
    ToolDisabledForConversation,
    /// Tool invocation started
    ToolInvocationStarted,
    /// Tool invocation completed successfully
    ToolInvocationCompleted,
    /// Tool invocation failed
    ToolInvocationFailed,
    /// Tool became unavailable (not responding on NATS)
    ToolBecameUnavailable,
    /// Tool became available (responding on NATS)
    ToolBecameAvailable,
    /// Tool health check completed
    ToolHealthCheckCompleted,
}

/// Conversation control events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConversationControlEvent {
    /// Conversation paused
    Paused,
    /// Conversation resumed
    Resumed,
    /// Priority changed
    PriorityChanged,
    /// Tags added
    TagsAdded,
    /// Tags removed
    TagsRemoved,
    /// Conversation archived
    Archived,
    /// Conversation restored
    Restored,
    /// Conversation transferred
    Transferred,
    /// Conversation forked
    Forked,
    /// Conversations merged
    Merged,
    /// Control action failed
    ControlActionFailed,
}

impl fmt::Display for ClaudeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClaudeEvent::Started => write!(f, "started"),
            ClaudeEvent::Ended => write!(f, "ended"),
            ClaudeEvent::PromptSent => write!(f, "prompt_sent"),
            ClaudeEvent::ResponseReceived => write!(f, "response_received"),
            ClaudeEvent::AttachmentUploaded => write!(f, "attachment_uploaded"),
            ClaudeEvent::AttachmentProcessed => write!(f, "attachment_processed"),
            ClaudeEvent::ScreenshotCaptured => write!(f, "screenshot_captured"),
            ClaudeEvent::ScreenshotAnalyzed => write!(f, "screenshot_analyzed"),
            ClaudeEvent::RateLimited => write!(f, "rate_limited"),
            ClaudeEvent::ApiError => write!(f, "api_error"),
            ClaudeEvent::ProcessingError => write!(f, "processing_error"),
        }
    }
}

impl fmt::Display for ConfigEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigEvent::SystemPromptUpdated => write!(f, "system_prompt_updated"),
            ConfigEvent::ModelParamsUpdated => write!(f, "model_params_updated"),
            ConfigEvent::ConversationSettingsUpdated => write!(f, "conversation_settings_updated"),
            ConfigEvent::ConfigReset => write!(f, "config_reset"),
            ConfigEvent::ConfigImported => write!(f, "config_imported"),
            ConfigEvent::ConfigExported => write!(f, "config_exported"),
            ConfigEvent::ConfigValidationFailed => write!(f, "config_validation_failed"),
            ConfigEvent::ConfigBackupCreated => write!(f, "config_backup_created"),
        }
    }
}

impl fmt::Display for NatsToolEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NatsToolEvent::ToolRegistered => write!(f, "tool_registered"),
            NatsToolEvent::ToolUnregistered => write!(f, "tool_unregistered"),
            NatsToolEvent::ToolUpdated => write!(f, "tool_updated"),
            NatsToolEvent::ToolEnabledForConversation => write!(f, "tool_enabled_for_conversation"),
            NatsToolEvent::ToolDisabledForConversation => write!(f, "tool_disabled_for_conversation"),
            NatsToolEvent::ToolInvocationStarted => write!(f, "tool_invocation_started"),
            NatsToolEvent::ToolInvocationCompleted => write!(f, "tool_invocation_completed"),
            NatsToolEvent::ToolInvocationFailed => write!(f, "tool_invocation_failed"),
            NatsToolEvent::ToolBecameUnavailable => write!(f, "tool_became_unavailable"),
            NatsToolEvent::ToolBecameAvailable => write!(f, "tool_became_available"),
            NatsToolEvent::ToolHealthCheckCompleted => write!(f, "tool_health_check_completed"),
        }
    }
}

impl fmt::Display for ConversationControlEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversationControlEvent::Paused => write!(f, "paused"),
            ConversationControlEvent::Resumed => write!(f, "resumed"),
            ConversationControlEvent::PriorityChanged => write!(f, "priority_changed"),
            ConversationControlEvent::TagsAdded => write!(f, "tags_added"),
            ConversationControlEvent::TagsRemoved => write!(f, "tags_removed"),
            ConversationControlEvent::Archived => write!(f, "archived"),
            ConversationControlEvent::Restored => write!(f, "restored"),
            ConversationControlEvent::Transferred => write!(f, "transferred"),
            ConversationControlEvent::Forked => write!(f, "forked"),
            ConversationControlEvent::Merged => write!(f, "merged"),
            ConversationControlEvent::ControlActionFailed => write!(f, "control_action_failed"),
        }
    }
}

/// Query types for Claude adapter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaudeQuery {
    /// Get conversation status
    Status,
    /// Get conversation history
    History,
    /// Get attachment metadata
    Attachment,
    /// Get usage metrics
    Usage,
    /// Get health status
    Health,
}

/// Configuration query types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigQuery {
    /// Get current configuration
    GetConfig,
    /// Get configuration history
    GetConfigHistory,
    /// Get default configuration
    GetDefaultConfig,
    /// Validate configuration
    ValidateConfig,
    /// Get configuration schema
    GetConfigSchema,
    /// Get configuration templates
    GetConfigTemplates,
}

/// NATS-connected tool query types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NatsToolQuery {
    /// Get available tools on NATS
    GetAvailableTools,
    /// Get tool details
    GetToolDetails,
    /// Get tools enabled for conversation
    GetConversationTools,
    /// Get tool invocation history
    GetToolInvocationHistory,
    /// Get tool health status
    GetToolHealthStatus,
}

/// Conversation control query types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConversationControlQuery {
    /// Get conversation metadata
    GetMetadata,
    /// Get conversation tags
    GetTags,
    /// Get conversation priority
    GetPriority,
    /// Get conversation status
    GetStatus,
    /// Get conversation branches
    GetBranches,
    /// Get conversation statistics
    GetStatistics,
}

impl fmt::Display for ClaudeQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClaudeQuery::Status => write!(f, "status"),
            ClaudeQuery::History => write!(f, "history"),
            ClaudeQuery::Attachment => write!(f, "attachment"),
            ClaudeQuery::Usage => write!(f, "usage"),
            ClaudeQuery::Health => write!(f, "health"),
        }
    }
}

impl fmt::Display for ConfigQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigQuery::GetConfig => write!(f, "get_config"),
            ConfigQuery::GetConfigHistory => write!(f, "get_config_history"),
            ConfigQuery::GetDefaultConfig => write!(f, "get_default_config"),
            ConfigQuery::ValidateConfig => write!(f, "validate_config"),
            ConfigQuery::GetConfigSchema => write!(f, "get_config_schema"),
            ConfigQuery::GetConfigTemplates => write!(f, "get_config_templates"),
        }
    }
}

impl fmt::Display for NatsToolQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NatsToolQuery::GetAvailableTools => write!(f, "get_available_tools"),
            NatsToolQuery::GetToolDetails => write!(f, "get_tool_details"),
            NatsToolQuery::GetConversationTools => write!(f, "get_conversation_tools"),
            NatsToolQuery::GetToolInvocationHistory => write!(f, "get_tool_invocation_history"),
            NatsToolQuery::GetToolHealthStatus => write!(f, "get_tool_health_status"),
        }
    }
}

impl fmt::Display for ConversationControlQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversationControlQuery::GetMetadata => write!(f, "get_metadata"),
            ConversationControlQuery::GetTags => write!(f, "get_tags"),
            ConversationControlQuery::GetPriority => write!(f, "get_priority"),
            ConversationControlQuery::GetStatus => write!(f, "get_status"),
            ConversationControlQuery::GetBranches => write!(f, "get_branches"),
            ConversationControlQuery::GetStatistics => write!(f, "get_statistics"),
        }
    }
}

/// MCP tool types for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpToolType {
    /// File system operations
    FileSystem,
    /// Web browsing and scraping
    WebBrowser,
    /// Database queries
    Database,
    /// API integrations
    ApiClient,
    /// Code execution
    CodeExecution,
    /// System commands
    SystemCommand,
    /// Custom tools
    Custom,
}

impl fmt::Display for McpToolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpToolType::FileSystem => write!(f, "filesystem"),
            McpToolType::WebBrowser => write!(f, "webbrowser"),
            McpToolType::Database => write!(f, "database"),
            McpToolType::ApiClient => write!(f, "apiclient"),
            McpToolType::CodeExecution => write!(f, "codeexec"),
            McpToolType::SystemCommand => write!(f, "syscmd"),
            McpToolType::Custom => write!(f, "custom"),
        }
    }
}

/// Storage types for object stores and KV stores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageType {
    /// Object store for binary data
    Object,
    /// Key-value store for structured data
    Kv,
    /// Stream for ordered events
    Stream,
}

impl fmt::Display for StorageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageType::Object => write!(f, "obj"),
            StorageType::Kv => write!(f, "kv"),
            StorageType::Stream => write!(f, "str"),
        }
    }
}

/// Attachment types for object store categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttachmentType {
    /// Image files (screenshots, photos)
    Image,
    /// Document files (PDF, text, etc.)
    Document,
    /// Audio files
    Audio,
    /// Video files
    Video,
    /// Code files
    Code,
    /// Archive files
    Archive,
    /// Raw binary data
    Binary,
}

impl fmt::Display for AttachmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttachmentType::Image => write!(f, "img"),
            AttachmentType::Document => write!(f, "doc"),
            AttachmentType::Audio => write!(f, "audio"),
            AttachmentType::Video => write!(f, "video"),
            AttachmentType::Code => write!(f, "code"),
            AttachmentType::Archive => write!(f, "arc"),
            AttachmentType::Binary => write!(f, "bin"),
        }
    }
}

/// Complete subject builder for type-safe NATS subjects
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CimSubject {
    domain: CimDomain,
    service: CimService,
    operation: CimOperation,
    resource_type: String,
    resource_id: Option<String>,
}

impl CimSubject {
    /// Create a new subject builder
    pub fn new(domain: CimDomain, service: CimService, operation: CimOperation) -> Self {
        Self {
            domain,
            service,
            operation,
            resource_type: "*".to_string(),
            resource_id: None,
        }
    }

    /// Set the resource type (command, event, query type)
    pub fn resource_type<T: fmt::Display>(mut self, resource_type: T) -> Self {
        self.resource_type = resource_type.to_string();
        self
    }

    /// Set the resource ID (conversation ID, session ID, etc.)
    pub fn resource_id<T: fmt::Display>(mut self, resource_id: T) -> Self {
        self.resource_id = Some(resource_id.to_string());
        self
    }

    /// Build the complete subject string
    pub fn build(&self) -> String {
        match &self.resource_id {
            Some(id) => format!(
                "cim.{}.{}.{}.{}.{}",
                self.domain, self.service, self.operation, self.resource_type, id
            ),
            None => format!(
                "cim.{}.{}.{}.{}",
                self.domain, self.service, self.operation, self.resource_type
            ),
        }
    }

    /// Build a wildcard subject for subscriptions
    pub fn wildcard(&self) -> String {
        match &self.resource_id {
            Some(_) => format!(
                "cim.{}.{}.{}.{}.>",
                self.domain, self.service, self.operation, self.resource_type
            ),
            None => format!(
                "cim.{}.{}.{}.>",
                self.domain, self.service, self.operation
            ),
        }
    }

    /// Create stream name from subject
    pub fn stream_name(&self) -> String {
        format!(
            "CIM_{}_{}_{}", 
            self.domain.to_string().to_uppercase(),
            self.service.to_string().to_uppercase(),
            self.operation.to_string().to_uppercase()
        )
    }

    /// Create object store name
    pub fn object_store_name(&self, attachment_type: AttachmentType) -> String {
        format!(
            "CIM_{}_{}_OBJ_{}", 
            self.domain.to_string().to_uppercase(),
            self.service.to_string().to_uppercase(),
            attachment_type.to_string().to_uppercase()
        )
    }

    /// Create KV store name
    pub fn kv_store_name(&self) -> String {
        format!(
            "CIM_{}_{}_KV", 
            self.domain.to_string().to_uppercase(),
            self.service.to_string().to_uppercase()
        )
    }
}

/// Pre-defined subject patterns for Claude adapter
pub struct ClaudeSubjects;

impl ClaudeSubjects {
    /// Claude API command subjects (for actual Claude interaction)
    pub fn command(cmd: ClaudeCommand, conversation_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Conversation, CimOperation::Cmd)
            .resource_type(cmd);
        
        match conversation_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }

    /// Claude API event subjects
    pub fn event(event: ClaudeEvent, conversation_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Conversation, CimOperation::Event)
            .resource_type(event);
        
        match conversation_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }

    /// Claude API query subjects
    pub fn query(query: ClaudeQuery, resource_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Conversation, CimOperation::Query)
            .resource_type(query);
        
        match resource_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }
    
    /// Configuration command subjects (separate from Claude API)
    pub fn config_command(cmd: ConfigCommand, config_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Config, CimOperation::Cmd)
            .resource_type(cmd);
        
        match config_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }
    
    /// Configuration event subjects
    pub fn config_event(event: ConfigEvent, config_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Config, CimOperation::Event)
            .resource_type(event);
        
        match config_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }
    
    /// Configuration query subjects
    pub fn config_query(query: ConfigQuery, config_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Config, CimOperation::Query)
            .resource_type(query);
        
        match config_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }
    
    /// NATS tool command subjects (tools are NATS services)
    pub fn nats_tool_command(cmd: NatsToolCommand, tool_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Core, CimService::Event, CimOperation::Cmd) // Using Core domain for infrastructure
            .resource_type(cmd);
        
        match tool_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }
    
    /// NATS tool event subjects
    pub fn nats_tool_event(event: NatsToolEvent, tool_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Core, CimService::Event, CimOperation::Event)
            .resource_type(event);
        
        match tool_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }
    
    /// NATS tool query subjects
    pub fn nats_tool_query(query: NatsToolQuery, tool_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Core, CimService::Event, CimOperation::Query)
            .resource_type(query);
        
        match tool_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }
    
    /// Conversation control command subjects
    pub fn control_command(cmd: ConversationControlCommand, conversation_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::User, CimService::Conversation, CimOperation::Cmd) // Using User domain for control
            .resource_type(cmd);
        
        match conversation_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }
    
    /// Conversation control event subjects
    pub fn control_event(event: ConversationControlEvent, conversation_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::User, CimService::Conversation, CimOperation::Event)
            .resource_type(event);
        
        match conversation_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }
    
    /// Conversation control query subjects
    pub fn control_query(query: ConversationControlQuery, conversation_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::User, CimService::Conversation, CimOperation::Query)
            .resource_type(query);
        
        match conversation_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }

    /// Attachment command subjects
    pub fn attachment_command(cmd: ClaudeCommand, attachment_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Attachment, CimOperation::Cmd)
            .resource_type(cmd);
        
        match attachment_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }

    /// Attachment event subjects
    pub fn attachment_event(event: ClaudeEvent, attachment_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Attachment, CimOperation::Event)
            .resource_type(event);
        
        match attachment_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }

    /// Stream subjects for real-time data
    pub fn stream(conversation_id: &str) -> CimSubject {
        CimSubject::new(CimDomain::Claude, CimService::Conversation, CimOperation::Stream)
            .resource_type("live")
            .resource_id(conversation_id)
    }

    /// Health monitoring subjects
    pub fn health() -> CimSubject {
        CimSubject::new(CimDomain::System, CimService::Health, CimOperation::Event)
            .resource_type("status")
    }

    /// Metrics subjects
    pub fn metrics() -> CimSubject {
        CimSubject::new(CimDomain::System, CimService::Metrics, CimOperation::Event)
            .resource_type("usage")
    }
}

/// JetStream stream configuration builder
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub name: String,
    pub subjects: Vec<String>,
    pub description: String,
    pub max_msgs: i64,
    pub max_bytes: i64,
    pub max_age_days: i64,
    pub replicas: u32,
}

impl StreamConfig {
    pub fn for_claude_commands() -> Self {
        Self {
            name: "CIM_CLAUDE_CONV_CMD".to_string(),
            subjects: vec![
                "cim.claude.conv.cmd.>".to_string(),
            ],
            description: "Claude conversation commands".to_string(),
            max_msgs: 1_000_000,
            max_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            max_age_days: 30,
            replicas: 3,
        }
    }

    pub fn for_claude_events() -> Self {
        Self {
            name: "CIM_CLAUDE_CONV_EVT".to_string(),
            subjects: vec![
                "cim.claude.conv.evt.>".to_string(),
            ],
            description: "Claude conversation events".to_string(),
            max_msgs: 10_000_000,
            max_bytes: 50 * 1024 * 1024 * 1024, // 50GB
            max_age_days: 365, // Keep events for 1 year
            replicas: 3,
        }
    }

    pub fn for_attachment_commands() -> Self {
        Self {
            name: "CIM_CLAUDE_ATTACH_CMD".to_string(),
            subjects: vec![
                "cim.claude.attach.cmd.>".to_string(),
            ],
            description: "Claude attachment commands".to_string(),
            max_msgs: 100_000,
            max_bytes: 1024 * 1024 * 1024, // 1GB
            max_age_days: 7,
            replicas: 3,
        }
    }

    pub fn for_attachment_events() -> Self {
        Self {
            name: "CIM_CLAUDE_ATTACH_EVT".to_string(),
            subjects: vec![
                "cim.claude.attach.evt.>".to_string(),
            ],
            description: "Claude attachment events".to_string(),
            max_msgs: 1_000_000,
            max_bytes: 5 * 1024 * 1024 * 1024, // 5GB
            max_age_days: 90,
            replicas: 3,
        }
    }
}

/// Object store configuration builder
#[derive(Debug, Clone)]
pub struct ObjectStoreConfig {
    pub name: String,
    pub description: String,
    pub max_bytes: i64,
    pub replicas: u32,
    pub ttl_days: Option<i64>,
}

impl ObjectStoreConfig {
    pub fn for_images() -> Self {
        Self {
            name: "CIM_CLAUDE_ATTACH_OBJ_IMG".to_string(),
            description: "Image attachments and screenshots".to_string(),
            max_bytes: 100 * 1024 * 1024 * 1024, // 100GB
            replicas: 3,
            ttl_days: Some(365), // Keep images for 1 year
        }
    }

    pub fn for_documents() -> Self {
        Self {
            name: "CIM_CLAUDE_ATTACH_OBJ_DOC".to_string(),
            description: "Document attachments".to_string(),
            max_bytes: 50 * 1024 * 1024 * 1024, // 50GB
            replicas: 3,
            ttl_days: Some(730), // Keep documents for 2 years
        }
    }

    pub fn for_binary() -> Self {
        Self {
            name: "CIM_CLAUDE_ATTACH_OBJ_BIN".to_string(),
            description: "Binary attachments".to_string(),
            max_bytes: 20 * 1024 * 1024 * 1024, // 20GB
            replicas: 2,
            ttl_days: Some(90), // Keep binary files for 90 days
        }
    }
}

/// KV store configuration builder
#[derive(Debug, Clone)]
pub struct KvStoreConfig {
    pub name: String,
    pub description: String,
    pub max_bytes: i64,
    pub history: u8,
    pub replicas: u32,
    pub ttl_days: Option<i64>,
}

impl KvStoreConfig {
    pub fn for_conversations() -> Self {
        Self {
            name: "CIM_CLAUDE_CONV_KV".to_string(),
            description: "Conversation state and metadata".to_string(),
            max_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            history: 10, // Keep last 10 revisions
            replicas: 3,
            ttl_days: Some(365),
        }
    }

    pub fn for_attachments() -> Self {
        Self {
            name: "CIM_CLAUDE_ATTACH_KV".to_string(),
            description: "Attachment metadata and references".to_string(),
            max_bytes: 1024 * 1024 * 1024, // 1GB
            history: 5,
            replicas: 3,
            ttl_days: Some(365),
        }
    }

    pub fn for_sessions() -> Self {
        Self {
            name: "CIM_CLAUDE_SESSION_KV".to_string(),
            description: "User session data".to_string(),
            max_bytes: 5 * 1024 * 1024 * 1024, // 5GB
            history: 3,
            replicas: 3,
            ttl_days: Some(30), // Sessions expire after 30 days
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_building() {
        let subject = ClaudeSubjects::command(ClaudeCommand::Start, Some("conv-123"));
        assert_eq!(subject.build(), "cim.claude.conv.cmd.start.conv-123");

        let wildcard = subject.wildcard();
        assert_eq!(wildcard, "cim.claude.conv.cmd.start.>");
    }
    
    #[test]
    fn test_config_subjects() {
        let subject = ClaudeSubjects::config_command(ConfigCommand::UpdateSystemPrompt, Some("config-456"));
        assert_eq!(subject.build(), "cim.claude.config.cmd.update_system_prompt.config-456");
        
        let event = ClaudeSubjects::config_event(ConfigEvent::SystemPromptUpdated, Some("config-456"));
        assert_eq!(event.build(), "cim.claude.config.evt.system_prompt_updated.config-456");
    }
    
    #[test]
    fn test_nats_tool_subjects() {
        let subject = ClaudeSubjects::nats_tool_command(NatsToolCommand::RegisterTool, Some("tool-789"));
        assert_eq!(subject.build(), "cim.core.event.cmd.register_tool.tool-789");
        
        let event = ClaudeSubjects::nats_tool_event(NatsToolEvent::ToolRegistered, Some("tool-789"));
        assert_eq!(event.build(), "cim.core.event.evt.tool_registered.tool-789");
    }
    
    #[test]
    fn test_control_subjects() {
        let subject = ClaudeSubjects::control_command(ConversationControlCommand::Pause, Some("conv-123"));
        assert_eq!(subject.build(), "cim.user.conv.cmd.pause.conv-123");
        
        let event = ClaudeSubjects::control_event(ConversationControlEvent::Paused, Some("conv-123"));
        assert_eq!(event.build(), "cim.user.conv.evt.paused.conv-123");
    }

    #[test]
    fn test_stream_naming() {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Conversation, CimOperation::Cmd);
        assert_eq!(subject.stream_name(), "CIM_CLAUDE_CONV_CMD");
    }

    #[test]
    fn test_object_store_naming() {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Attachment, CimOperation::Cmd);
        assert_eq!(
            subject.object_store_name(AttachmentType::Image), 
            "CIM_CLAUDE_ATTACH_OBJ_IMG"
        );
    }

    #[test]
    fn test_attachment_subjects() {
        let subject = ClaudeSubjects::attachment_event(
            ClaudeEvent::AttachmentUploaded, 
            Some("attach-456")
        );
        assert_eq!(subject.build(), "cim.claude.attach.evt.attachment_uploaded.attach-456");
    }
    
    #[test]
    fn test_subject_separation() {
        // Claude API commands should be separate from config commands
        let claude_cmd = ClaudeSubjects::command(ClaudeCommand::Send, Some("conv-123"));
        let config_cmd = ClaudeSubjects::config_command(ConfigCommand::UpdateSystemPrompt, Some("conv-123"));
        
        assert_ne!(claude_cmd.build(), config_cmd.build());
        assert!(claude_cmd.build().contains("conv.cmd"));
        assert!(config_cmd.build().contains("config.cmd"));
        
        // NATS tool commands should be separate from both
        let nats_tool_cmd = ClaudeSubjects::nats_tool_command(NatsToolCommand::InvokeTool, Some("tool-456"));
        assert_ne!(claude_cmd.build(), nats_tool_cmd.build());
        assert_ne!(config_cmd.build(), nats_tool_cmd.build());
        assert!(nats_tool_cmd.build().contains("core.event.cmd"));
    }
}