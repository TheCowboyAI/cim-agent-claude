/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Strong type for Event IDs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct EventId(Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn is_nil(&self) -> bool {
        self.0.is_nil()
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Strong type for Conversation IDs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct ConversationId(Uuid);

impl ConversationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ConversationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Strong type for Session IDs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Strong type for Correlation IDs (CIM standard)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt value object with validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    content: String,
    character_count: usize,
}

impl Prompt {
    pub const MAX_LENGTH: usize = 50_000;

    pub fn new(content: String) -> Result<Self, String> {
        if content.trim().is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }

        let character_count = content.chars().count();
        if character_count > Self::MAX_LENGTH {
            return Err(format!(
                "Prompt exceeds maximum length of {} characters: {}",
                Self::MAX_LENGTH,
                character_count
            ));
        }

        Ok(Self {
            content: content.trim().to_string(),
            character_count,
        })
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn character_count(&self) -> usize {
        self.character_count
    }
}

/// Claude response value object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaudeResponse {
    content: String,
    usage: TokenUsage,
    finish_reason: String,
    model: String,
}

impl ClaudeResponse {
    pub fn new(content: String, usage: TokenUsage, finish_reason: String, model: String) -> Self {
        Self {
            content,
            usage,
            finish_reason,
            model,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn usage(&self) -> &TokenUsage {
        &self.usage
    }

    pub fn finish_reason(&self) -> &str {
        &self.finish_reason
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenUsage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
}

impl TokenUsage {
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }

    pub fn input_tokens(&self) -> u32 {
        self.input_tokens
    }

    pub fn output_tokens(&self) -> u32 {
        self.output_tokens
    }

    pub fn total_tokens(&self) -> u32 {
        self.total_tokens
    }
}

impl Default for TokenUsage {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Conversation context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationContext {
    max_tokens: Option<u32>,
    temperature: Option<f64>,
    system_prompt: Option<String>,
    metadata: HashMap<String, String>,
}

impl ConversationContext {
    pub fn new() -> Self {
        Self {
            max_tokens: None,
            temperature: None,
            system_prompt: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_system_prompt(mut self, system_prompt: String) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    pub fn temperature(&self) -> Option<f64> {
        self.temperature
    }

    pub fn system_prompt(&self) -> Option<&str> {
        self.system_prompt.as_deref()
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Claude request metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaudeRequestMetadata {
    model: String,
    max_tokens: u32,
    temperature: f64,
    request_timestamp: DateTime<Utc>,
}

impl ClaudeRequestMetadata {
    pub fn new(model: String, max_tokens: u32, temperature: f64) -> Self {
        Self {
            model,
            max_tokens,
            temperature,
            request_timestamp: Utc::now(),
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn max_tokens(&self) -> u32 {
        self.max_tokens
    }

    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    pub fn request_timestamp(&self) -> DateTime<Utc> {
        self.request_timestamp
    }
}

impl Default for ClaudeRequestMetadata {
    fn default() -> Self {
        Self::new("claude-3-sonnet-20240229".to_string(), 4000, 0.7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_validation() {
        // Valid prompt
        let prompt = Prompt::new("Hello Claude".to_string());
        assert!(prompt.is_ok());
        assert_eq!(prompt.unwrap().character_count(), 12);

        // Empty prompt
        let empty_prompt = Prompt::new("".to_string());
        assert!(empty_prompt.is_err());

        // Whitespace only prompt
        let whitespace_prompt = Prompt::new("   ".to_string());
        assert!(whitespace_prompt.is_err());

        // Too long prompt
        let long_content = "x".repeat(Prompt::MAX_LENGTH + 1);
        let long_prompt = Prompt::new(long_content);
        assert!(long_prompt.is_err());
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.input_tokens(), 100);
        assert_eq!(usage.output_tokens(), 50);
        assert_eq!(usage.total_tokens(), 150);
    }

    #[test]
    fn test_conversation_context_builder() {
        let context = ConversationContext::new()
            .with_max_tokens(4000)
            .with_temperature(0.8)
            .with_system_prompt("You are a helpful assistant".to_string())
            .with_metadata("user_id".to_string(), "12345".to_string());

        assert_eq!(context.max_tokens(), Some(4000));
        assert_eq!(context.temperature(), Some(0.8));
        assert_eq!(context.system_prompt(), Some("You are a helpful assistant"));
        assert_eq!(
            context.metadata().get("user_id"),
            Some(&"12345".to_string())
        );
    }
}
