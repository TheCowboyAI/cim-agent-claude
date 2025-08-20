/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Claude Domain Types
//!
//! Pure domain types for Claude API interactions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Claude API Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeRequest {
    pub messages: Vec<ClaudeMessage>,
    pub system_prompt: Option<String>,
    pub metadata: Option<Value>,
}

/// Claude API Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub content: String,
    pub model: String,
    pub usage: Value,
    pub metadata: Option<Value>,
}

/// Claude Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Message Role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}

/// Conversation Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub id: String,
    pub messages: Vec<ClaudeMessage>,
    pub system_prompt: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ConversationContext {
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            messages: Vec::new(),
            system_prompt: None,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_message(&mut self, message: ClaudeMessage) {
        self.messages.push(message);
        self.updated_at = chrono::Utc::now();
    }

    pub fn to_request(&self) -> ClaudeRequest {
        ClaudeRequest {
            messages: self.messages.clone(),
            system_prompt: self.system_prompt.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

/// Claude Model Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub max_tokens: u32,
    pub context_window: u32,
    pub capabilities: Vec<String>,
}

/// Usage Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

impl From<Value> for Usage {
    fn from(value: Value) -> Self {
        Self {
            input_tokens: value["input_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: value["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: value["input_tokens"].as_u64().unwrap_or(0) as u32
                + value["output_tokens"].as_u64().unwrap_or(0) as u32,
        }
    }
}