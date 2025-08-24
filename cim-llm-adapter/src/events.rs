//! Event types for LLM Adapter
//!
//! Event-driven architecture support with comprehensive event modeling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Main LLM event wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmEvent {
    pub event_id: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub domain: String,
    pub data: serde_json::Value,
    pub metadata: EventMetadata,
}

/// Event metadata for tracing and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub source: String,
    pub version: String,
    pub cim_event: bool,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

impl LlmEvent {
    /// Create new LLM event
    pub fn new(
        event_type: String,
        aggregate_id: String,
        data: serde_json::Value,
        session_id: Option<String>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            aggregate_id,
            correlation_id: session_id.clone(),
            causation_id: None,
            timestamp: Utc::now(),
            domain: "llm-adapter".to_string(),
            data,
            metadata: EventMetadata {
                source: "llm-adapter-service".to_string(),
                version: "1.0".to_string(),
                cim_event: true,
                user_id: None,
                session_id,
            },
        }
    }
    
    /// Create completion request event
    pub fn completion_requested(
        session_id: String,
        provider: String,
        model: String,
        message_count: usize,
    ) -> Self {
        let data = serde_json::json!({
            "provider": provider,
            "model": model,
            "message_count": message_count,
            "request_time": Utc::now().to_rfc3339()
        });
        
        Self::new(
            "CompletionRequested".to_string(),
            session_id.clone(),
            data,
            Some(session_id),
        )
    }
    
    /// Create completion completed event
    pub fn completion_completed(
        session_id: String,
        provider: String,
        model: String,
        token_count: Option<u32>,
        duration_ms: u64,
    ) -> Self {
        let data = serde_json::json!({
            "provider": provider,
            "model": model,
            "token_count": token_count,
            "duration_ms": duration_ms,
            "completion_time": Utc::now().to_rfc3339()
        });
        
        Self::new(
            "CompletionCompleted".to_string(),
            session_id.clone(),
            data,
            Some(session_id),
        )
    }
    
    /// Create completion failed event
    pub fn completion_failed(
        session_id: String,
        provider: String,
        error_message: String,
        duration_ms: u64,
    ) -> Self {
        let data = serde_json::json!({
            "provider": provider,
            "error_message": error_message,
            "duration_ms": duration_ms,
            "failure_time": Utc::now().to_rfc3339()
        });
        
        Self::new(
            "CompletionFailed".to_string(),
            session_id.clone(),
            data,
            Some(session_id),
        )
    }
}

/// Dialog-specific events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogEvent {
    pub event_id: String,
    pub event_type: String, 
    pub session_id: String,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub data: DialogEventData,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Dialog event data types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DialogEventData {
    DialogStarted {
        initial_message: String,
        provider: String,
    },
    MessageAdded {
        role: String,
        content: String,
        provider: Option<String>,
    },
    ContextUpdated {
        history_length: usize,
        metadata_changes: HashMap<String, serde_json::Value>,
    },
    DialogEnded {
        reason: String,
        message_count: usize,
        duration_seconds: u64,
    },
}

impl DialogEvent {
    /// Create new dialog event
    pub fn new(
        event_type: String,
        session_id: String,
        data: DialogEventData,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            session_id,
            user_id: None,
            timestamp: Utc::now(),
            data,
            metadata: HashMap::new(),
        }
    }
    
    /// Create dialog started event
    pub fn dialog_started(
        session_id: String,
        initial_message: String,
        provider: String,
    ) -> Self {
        Self::new(
            "DialogStarted".to_string(),
            session_id,
            DialogEventData::DialogStarted {
                initial_message,
                provider,
            },
        )
    }
    
    /// Create message added event
    pub fn message_added(
        session_id: String,
        role: String,
        content: String,
        provider: Option<String>,
    ) -> Self {
        Self::new(
            "MessageAdded".to_string(),
            session_id,
            DialogEventData::MessageAdded {
                role,
                content,
                provider,
            },
        )
    }
}

/// Provider health events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealthEvent {
    pub event_id: String,
    pub provider_name: String,
    pub timestamp: DateTime<Utc>,
    pub health_status: String, // "healthy", "degraded", "unhealthy"
    pub latency_ms: Option<u64>,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ProviderHealthEvent {
    pub fn new(
        provider_name: String,
        health_status: String,
        latency_ms: Option<u64>,
        error_message: Option<String>,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            provider_name,
            timestamp: Utc::now(),
            health_status,
            latency_ms,
            error_message,
            metadata: HashMap::new(),
        }
    }
}