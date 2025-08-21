/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

#[cfg(feature = "cim-claude-adapter")]
use cim_claude_adapter;

#[cfg(not(feature = "cim-claude-adapter"))]
use crate::wasm_types::{
    SimpleEventEnvelope as EventEnvelope,
    SimpleConversationAggregate as ConversationAggregate,
    CimExpertTopic,
    HealthStatus,
    SystemMetrics,
    CimExpertConversation,
    CimExpertMessage,
    CimExpertMessageRole,
};


/// TEA Messages for the CIM Manager
#[derive(Debug, Clone)]
pub enum Message {
    // Connection Management
    Connect(String), // NATS URL
    Connected,
    Disconnected,
    ConnectionError(String),
    
    // Conversation Management
    StartConversation { 
        session_id: String,
        initial_prompt: String,
    },
    SendPrompt {
        conversation_id: String,
        prompt: String,
    },
    EndConversation {
        conversation_id: String,
        reason: String,
    },
    
    // Event Handling (simplified)
    ConversationUpdated(cim_claude_adapter::domain::ConversationContext),
    CommandSent,
    
    // UI State Changes
    TabSelected(Tab),
    ConversationSelected(String),
    PromptInputChanged(String),
    SessionIdChanged(String),
    NatsUrlChanged(String),
    
    // Health and Monitoring
    HealthCheckRequested,
    HealthCheckReceived(HealthStatus),
    MetricsReceived(SystemMetrics),
    
    // Legacy CIM Expert Messages (deprecated - use SAGE instead)
    CimExpertTabSelected,
    CimExpertStartConversation,
    CimExpertSendMessage(String),
    CimExpertMessageInputChanged(String),
    CimExpertTopicSelected(String), // Simplified - topic as string
    CimExpertContextChanged(String),
    CimExpertConversationReceived(CimExpertConversation),
    CimExpertResponseReceived(String, String), // message_id, response
    
    // Theme Management
    ThemeToggled,
    
    // SAGE Orchestrator Messages
    SageRequestSent(String), // request_id
    SageResponseReceived(crate::sage_client::SageResponse),
    SageStatusRequested,
    SageStatusReceived(crate::sage_client::SageStatus),
    SageQueryInputChanged(String),
    SageExpertSelected(Option<String>),
    SageSendQuery,
    SageClearConversation,
    SageNewSession,
    
    // Error Handling
    Error(String),
    ErrorOccurred(String),
    ErrorDismissed,
}

/// Tab navigation for the management interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Conversations,
    Events,
    Monitoring,
    Sage,
    Settings,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Dashboard
    }
}

/// Health status for the entire system
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub nats_connected: bool,
    pub claude_api_available: bool,
    pub active_conversations: u32,
    pub events_processed: u64,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// System metrics for monitoring
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub conversations_total: u32,
    pub conversations_active: u32,
    pub events_published: u64,
    pub events_consumed: u64,
    pub api_requests_total: u64,
    pub api_requests_failed: u64,
    pub response_time_avg_ms: f64,
}

/// CIM Expert conversation session for GUI
#[derive(Debug, Clone)]
pub struct CimExpertConversation {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub messages: Vec<CimExpertMessage>,
    pub context: Option<String>,
    pub user_id: Option<String>,
}

/// Message in a CIM Expert conversation
#[derive(Debug, Clone)]
pub struct CimExpertMessage {
    pub id: String,
    pub role: CimExpertMessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub topic: Option<String>, // Simplified - topic as string
}

/// Role of a message in CIM Expert conversation
#[derive(Debug, Clone)]
pub enum CimExpertMessageRole {
    User,
    Expert,
    System,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            nats_connected: false,
            claude_api_available: false,
            active_conversations: 0,
            events_processed: 0,
            last_check: chrono::Utc::now(),
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            conversations_total: 0,
            conversations_active: 0,
            events_published: 0,
            events_consumed: 0,
            api_requests_total: 0,
            api_requests_failed: 0,
            response_time_avg_ms: 0.0,
        }
    }
}