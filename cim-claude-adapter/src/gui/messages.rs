/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use crate::{
    bridge::{TeaEvent, EntityId},
    domain::{
        events::*, 
        ConversationAggregate,
    },
};
use tokio::sync::mpsc;

/// Bridge-specific messages for TEA-ECS communication
#[derive(Debug)]
pub enum BridgeMessage {
    Connected,
    ConnectionError(String),
    EventReceiverReady(mpsc::UnboundedReceiver<TeaEvent>),
    EventReceived(TeaEvent),
}

// We can't derive Clone for BridgeMessage due to the receiver, but we can implement it manually
impl Clone for BridgeMessage {
    fn clone(&self) -> Self {
        match self {
            Self::Connected => Self::Connected,
            Self::ConnectionError(err) => Self::ConnectionError(err.clone()),
            Self::EventReceiverReady(_) => {
                // Can't clone a receiver, so we'll create a placeholder
                Self::ConnectionError("Receiver cannot be cloned".to_string())
            }
            Self::EventReceived(event) => Self::EventReceived(event.clone()),
        }
    }
}

/// TEA Messages for the CIM Manager with Bridge Integration
#[derive(Debug, Clone)]
pub enum Message {
    // Connection Management
    Connect(String), // NATS URL
    Connected,
    Disconnected,
    ConnectionError(String),
    BridgeStatusChanged { connected: bool, error: Option<String> },
    
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
    
    // Event Handling (from NATS)
    ConversationEvent(EventEnvelope),
    ConversationUpdated(ConversationAggregate),
    
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
    
    // Bridge Integration
    BridgeMessage(BridgeMessage),
    TeaEventReceived(TeaEvent),
    
    // Error Handling
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

/// Enhanced system metrics with bridge statistics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub conversations_total: u32,
    pub conversations_active: u32,
    pub events_published: u64,
    pub events_consumed: u64,
    pub tea_events_received: u64,
    pub bridge_commands_sent: u64,
    pub api_requests_total: u64,
    pub api_requests_failed: u64,
    pub response_time_avg_ms: f64,
    pub bridge_latency_avg_ms: f64,
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
            tea_events_received: 0,
            bridge_commands_sent: 0,
            api_requests_total: 0,
            api_requests_failed: 0,
            response_time_avg_ms: 0.0,
            bridge_latency_avg_ms: 0.0,
        }
    }
}

/// Loading state management
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingState {
    Idle,
    Loading,
    Success,
    Error(String),
}

/// UI component states for reactive updates
#[derive(Debug, Clone)]
pub struct ComponentState {
    pub conversation_list: LoadingState,
    pub message_sending: LoadingState,
    pub bridge_connection: LoadingState,
    pub health_check: LoadingState,
}

impl Default for ComponentState {
    fn default() -> Self {
        Self {
            conversation_list: LoadingState::Idle,
            message_sending: LoadingState::Idle,
            bridge_connection: LoadingState::Idle,
            health_check: LoadingState::Idle,
        }
    }
}

/// Event filtering options for the UI
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub show_errors_only: bool,
    pub show_conversation_events: bool,
    pub show_system_events: bool,
    pub min_priority: u8,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            show_errors_only: false,
            show_conversation_events: true,
            show_system_events: true,
            min_priority: 0,
        }
    }
}