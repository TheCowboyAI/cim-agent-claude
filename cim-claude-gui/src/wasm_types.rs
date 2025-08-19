/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Simplified types for WASM builds that don't depend on native-only crates

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Simplified conversation event for WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleEventEnvelope {
    pub event_id: String,
    pub correlation_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Simplified conversation aggregate for WASM  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleConversationAggregate {
    pub id: String,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Simplified command for WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleCommand {
    pub command_type: String,
    pub session_id: Option<String>,
    pub conversation_id: Option<String>,
    pub prompt: Option<String>,
    pub correlation_id: String,
}

/// Simplified command envelope for WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleCommandEnvelope {
    pub command: SimpleCommand,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: String,
}

/// Health status compatible with WASM
#[derive(Debug, Clone, Default)]
pub struct HealthStatus {
    pub nats_connected: bool,
    pub claude_api_available: bool,
    pub active_conversations: u32,
    pub events_processed: u64,
    pub last_check: DateTime<Utc>,
}

/// System metrics compatible with WASM  
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    pub conversations_total: u64,
    pub conversations_active: u32,
    pub events_published: u64,
    pub events_consumed: u64,
    pub api_requests_total: u64,
    pub api_requests_failed: u64,
    pub response_time_avg_ms: f64,
}

/// CIM Expert conversation for WASM
#[derive(Debug, Clone)]
pub struct CimExpertConversation {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub messages: Vec<CimExpertMessage>,
    pub context: Option<String>,
    pub user_id: Option<String>,
}

/// CIM Expert message for WASM
#[derive(Debug, Clone)]
pub struct CimExpertMessage {
    pub id: String,
    pub role: CimExpertMessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub topic: Option<CimExpertTopic>,
}

/// Message roles for CIM Expert
#[derive(Debug, Clone, PartialEq)]
pub enum CimExpertMessageRole {
    User,
    Expert,
    System,
}

/// CIM Expert topics for WASM
#[derive(Debug, Clone, PartialEq)]
pub enum CimExpertTopic {
    Architecture,
    MathematicalFoundations,
    NatsPatterns,
    EventSourcing,
    DomainModeling,
    Implementation,
    Troubleshooting,
}