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
    /// Command subjects
    pub fn command(cmd: ClaudeCommand, conversation_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Conversation, CimOperation::Cmd)
            .resource_type(cmd);
        
        match conversation_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }

    /// Event subjects  
    pub fn event(event: ClaudeEvent, conversation_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Conversation, CimOperation::Event)
            .resource_type(event);
        
        match conversation_id {
            Some(id) => subject.resource_id(id),
            None => subject,
        }
    }

    /// Query subjects
    pub fn query(query: ClaudeQuery, resource_id: Option<&str>) -> CimSubject {
        let subject = CimSubject::new(CimDomain::Claude, CimService::Conversation, CimOperation::Query)
            .resource_type(query);
        
        match resource_id {
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
}