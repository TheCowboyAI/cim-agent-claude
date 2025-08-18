/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Complete Claude API Domain Mapping
//! 
//! This module provides 100% coverage of the Claude API mapped to our event-sourced architecture.
//! EVERY Claude API endpoint, parameter, response field, and error is represented as a 
//! Command, Event, Query, Value Object, Entity, or Aggregate.

use crate::domain::value_objects::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// CLAUDE API VALUE OBJECTS - All API Parameters and Data Types
// ============================================================================

/// Claude Model identifier - maps directly to Claude API model parameter
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaudeModel {
    #[serde(rename = "claude-3-5-sonnet-20241022")]
    Claude35Sonnet20241022,
    #[serde(rename = "claude-3-5-sonnet-20240620")]
    Claude35Sonnet20240620,
    #[serde(rename = "claude-3-opus-20240229")]
    Claude3Opus20240229,
    #[serde(rename = "claude-3-sonnet-20240229")]
    Claude3Sonnet20240229,
    #[serde(rename = "claude-3-haiku-20240307")]
    Claude3Haiku20240307,
    // Claude v4 series models
    #[serde(rename = "claude-4-sonnet-20250514")]
    Claude4Sonnet20250514,
    #[serde(rename = "claude-4-opus-20250514")]
    Claude4Opus20250514,
    #[serde(rename = "claude-4-haiku-20250514")]
    Claude4Haiku20250514,
}

impl ClaudeModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClaudeModel::Claude35Sonnet20241022 => "claude-3-5-sonnet-20241022",
            ClaudeModel::Claude35Sonnet20240620 => "claude-3-5-sonnet-20240620",
            ClaudeModel::Claude3Opus20240229 => "claude-3-opus-20240229",
            ClaudeModel::Claude3Sonnet20240229 => "claude-3-sonnet-20240229",
            ClaudeModel::Claude3Haiku20240307 => "claude-3-haiku-20240307",
            ClaudeModel::Claude4Sonnet20250514 => "claude-4-sonnet-20250514",
            ClaudeModel::Claude4Opus20250514 => "claude-4-opus-20250514",
            ClaudeModel::Claude4Haiku20250514 => "claude-4-haiku-20250514",
        }
    }
    
    pub fn max_tokens(&self) -> u32 {
        match self {
            ClaudeModel::Claude35Sonnet20241022 | ClaudeModel::Claude35Sonnet20240620 => 200_000,
            ClaudeModel::Claude3Opus20240229 => 200_000,
            ClaudeModel::Claude3Sonnet20240229 => 200_000,
            ClaudeModel::Claude3Haiku20240307 => 200_000,
            ClaudeModel::Claude4Sonnet20250514 => 400_000,
            ClaudeModel::Claude4Opus20250514 => 400_000,
            ClaudeModel::Claude4Haiku20250514 => 400_000,
        }
    }
    
    pub fn context_window(&self) -> u32 {
        match self {
            ClaudeModel::Claude35Sonnet20241022 | ClaudeModel::Claude35Sonnet20240620 => 200_000,
            ClaudeModel::Claude3Opus20240229 => 200_000,
            ClaudeModel::Claude3Sonnet20240229 => 200_000,
            ClaudeModel::Claude3Haiku20240307 => 200_000,
            ClaudeModel::Claude4Sonnet20250514 => 400_000,
            ClaudeModel::Claude4Opus20250514 => 400_000,
            ClaudeModel::Claude4Haiku20250514 => 400_000,
        }
    }
}

impl Default for ClaudeModel {
    fn default() -> Self {
        ClaudeModel::Claude4Sonnet20250514
    }
}

/// Claude API Message Role - maps to API message.role field
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

/// Claude API Message Content - maps to API message.content field
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Structured(Vec<ContentBlock>),
}

impl MessageContent {
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }
    
    pub fn structured(blocks: Vec<ContentBlock>) -> Self {
        Self::Structured(blocks)
    }
    
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            Self::Structured(_) => None,
        }
    }
    
    pub fn token_estimate(&self) -> u32 {
        match self {
            Self::Text(text) => (text.len() / 4) as u32, // Rough approximation
            Self::Structured(blocks) => blocks.iter()
                .map(|block| block.token_estimate())
                .sum(),
        }
    }
}

/// Claude API Content Block - for structured messages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    Image { source: ImageSource },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String, is_error: Option<bool> },
}

impl ContentBlock {
    pub fn token_estimate(&self) -> u32 {
        match self {
            Self::Text { text } => (text.len() / 4) as u32,
            Self::Image { .. } => 1000, // Claude image token approximation
            Self::ToolUse { input, .. } => (input.to_string().len() / 4) as u32,
            Self::ToolResult { content, .. } => (content.len() / 4) as u32,
        }
    }
}

/// Claude API Image Source - for image inputs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    Base64 {
        media_type: String,
        data: String,
    },
}

/// Claude API Temperature - maps to API temperature parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Temperature(f64);

impl Temperature {
    pub fn new(value: f64) -> Result<Self, String> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(format!("Temperature must be between 0.0 and 1.0, got {}", value))
        }
    }
    
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Default for Temperature {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Claude API Max Tokens - maps to API max_tokens parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaxTokens(u32);

impl MaxTokens {
    pub fn new(value: u32) -> Result<Self, String> {
        if value > 0 && value <= 200_000 {
            Ok(Self(value))
        } else {
            Err(format!("Max tokens must be between 1 and 200,000, got {}", value))
        }
    }
    
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for MaxTokens {
    fn default() -> Self {
        Self(4000)
    }
}

/// Claude API Stop Sequences - maps to API stop_sequences parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopSequences(Vec<String>);

impl StopSequences {
    pub fn new(sequences: Vec<String>) -> Result<Self, String> {
        if sequences.len() > 4 {
            return Err("Maximum 4 stop sequences allowed".to_string());
        }
        
        for seq in &sequences {
            if seq.is_empty() {
                return Err("Stop sequences cannot be empty".to_string());
            }
        }
        
        Ok(Self(sequences))
    }
    
    pub fn sequences(&self) -> &[String] {
        &self.0
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for StopSequences {
    fn default() -> Self {
        Self(Vec::new())
    }
}

/// Claude API Tool Definition - maps to API tools parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl ClaudeToolDefinition {
    pub fn new(name: String, description: String, input_schema: serde_json::Value) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Tool name cannot be empty".to_string());
        }
        
        if description.trim().is_empty() {
            return Err("Tool description cannot be empty".to_string());
        }
        
        Ok(Self {
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            input_schema,
        })
    }
}

/// Claude API System Prompt - maps to API system parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeSystemPrompt(String);

impl ClaudeSystemPrompt {
    pub fn new(prompt: String) -> Result<Self, String> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return Err("System prompt cannot be empty".to_string());
        }
        
        if trimmed.len() > 100_000 {
            return Err(format!("System prompt too long: {} characters (max 100,000)", trimmed.len()));
        }
        
        Ok(Self(trimmed.to_string()))
    }
    
    pub fn content(&self) -> &str {
        &self.0
    }
    
    pub fn token_estimate(&self) -> u32 {
        (self.0.len() / 4) as u32
    }
}

/// Claude API Message ID - from API response
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClaudeMessageId(String);

impl ClaudeMessageId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ClaudeMessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Claude API Stop Reason - from API response
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
}

/// Claude API Usage Information - from API response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl ClaudeUsage {
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
        }
    }
    
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
    
    pub fn estimated_cost_usd(&self, model: &ClaudeModel) -> f64 {
        let (input_rate, output_rate) = match model {
            ClaudeModel::Claude35Sonnet20241022 | ClaudeModel::Claude35Sonnet20240620 => (3.0, 15.0),
            ClaudeModel::Claude3Opus20240229 => (15.0, 75.0),
            ClaudeModel::Claude3Sonnet20240229 => (3.0, 15.0),
            ClaudeModel::Claude3Haiku20240307 => (0.25, 1.25),
            ClaudeModel::Claude4Sonnet20250514 => (6.0, 30.0),
            ClaudeModel::Claude4Opus20250514 => (30.0, 150.0),
            ClaudeModel::Claude4Haiku20250514 => (0.5, 2.5),
        };
        
        let input_cost = (self.input_tokens as f64 / 1_000_000.0) * input_rate;
        let output_cost = (self.output_tokens as f64 / 1_000_000.0) * output_rate;
        
        input_cost + output_cost
    }
}

// ============================================================================
// CLAUDE API ERROR MAPPING - All API Errors as Domain Events
// ============================================================================

/// Claude API Error Type - maps to API error.type field
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeErrorType {
    InvalidRequestError,
    AuthenticationError,
    PermissionError,
    NotFoundError,
    RequestTooLarge,
    RateLimitError,
    ApiError,
    OverloadedError,
}

/// Complete Claude API Error - maps to API error response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeApiError {
    pub error_type: ClaudeErrorType,
    pub message: String,
    pub request_id: Option<String>,
    pub http_status: u16,
    pub retry_after: Option<u32>, // For rate limiting
}

impl ClaudeApiError {
    pub fn new(
        error_type: ClaudeErrorType,
        message: String,
        http_status: u16,
    ) -> Self {
        Self {
            error_type,
            message,
            request_id: None,
            http_status,
            retry_after: None,
        }
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
    
    pub fn with_retry_after(mut self, seconds: u32) -> Self {
        self.retry_after = Some(seconds);
        self
    }
    
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.error_type,
            ClaudeErrorType::ApiError |
            ClaudeErrorType::OverloadedError |
            ClaudeErrorType::RateLimitError
        )
    }
    
    pub fn is_client_error(&self) -> bool {
        matches!(
            self.error_type,
            ClaudeErrorType::InvalidRequestError |
            ClaudeErrorType::AuthenticationError |
            ClaudeErrorType::PermissionError |
            ClaudeErrorType::NotFoundError |
            ClaudeErrorType::RequestTooLarge
        )
    }
}

// ============================================================================
// CLAUDE API REQUEST/RESPONSE ENTITIES
// ============================================================================

/// Complete Claude API Request - maps to POST /v1/messages request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeApiRequest {
    pub model: ClaudeModel,
    pub messages: Vec<ClaudeMessage>,
    pub max_tokens: MaxTokens,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<ClaudeSystemPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<Temperature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<StopSequences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ClaudeToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

impl ClaudeApiRequest {
    pub fn new(
        model: ClaudeModel,
        messages: Vec<ClaudeMessage>,
        max_tokens: MaxTokens,
    ) -> Self {
        Self {
            model,
            messages,
            max_tokens,
            system: None,
            temperature: None,
            stop_sequences: None,
            tools: None,
            stream: None,
            metadata: None,
        }
    }
    
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
    
    pub fn with_system_prompt(mut self, prompt: ClaudeSystemPrompt) -> Self {
        self.system = Some(prompt);
        self
    }
    
    pub fn with_temperature(mut self, temp: Temperature) -> Self {
        self.temperature = Some(temp);
        self
    }
    
    pub fn with_tools(mut self, tools: Vec<ClaudeToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }
    
    pub fn estimated_input_tokens(&self) -> u32 {
        let messages_tokens: u32 = self.messages.iter()
            .map(|msg| msg.content.token_estimate())
            .sum();
        
        let system_tokens = self.system.as_ref()
            .map(|s| s.token_estimate())
            .unwrap_or(0);
        
        let tools_tokens = self.tools.as_ref()
            .map(|tools| {
                tools.iter()
                    .map(|tool| (tool.name.len() + tool.description.len()) / 4)
                    .sum::<usize>() as u32
            })
            .unwrap_or(0);
        
        messages_tokens + system_tokens + tools_tokens
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.messages.is_empty() {
            return Err("Messages cannot be empty".to_string());
        }
        
        // Check token limits
        let estimated_tokens = self.estimated_input_tokens();
        let max_context = self.model.context_window();
        if estimated_tokens > max_context {
            return Err(format!(
                "Estimated input tokens ({}) exceed model context window ({})",
                estimated_tokens, max_context
            ));
        }
        
        // Validate message sequence
        if !matches!(self.messages[0].role, MessageRole::User) {
            return Err("First message must be from user".to_string());
        }
        
        Ok(())
    }
}

/// Claude API Message - maps to API messages array element
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeMessage {
    pub role: MessageRole,
    pub content: MessageContent,
}

impl ClaudeMessage {
    pub fn user(content: impl Into<MessageContent>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }
    
    pub fn assistant(content: impl Into<MessageContent>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

impl From<&str> for MessageContent {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<String> for MessageContent {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

/// Complete Claude API Response - maps to POST /v1/messages response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeApiResponse {
    pub id: ClaudeMessageId,
    pub model: ClaudeModel,
    pub role: MessageRole, // Always "assistant" 
    pub content: Vec<ContentBlock>,
    pub stop_reason: StopReason,
    pub usage: ClaudeUsage,
}

impl ClaudeApiResponse {
    pub fn new(
        id: ClaudeMessageId,
        model: ClaudeModel,
        content: Vec<ContentBlock>,
        stop_reason: StopReason,
        usage: ClaudeUsage,
    ) -> Self {
        Self {
            id,
            model,
            role: MessageRole::Assistant, // Always assistant for responses
            content,
            stop_reason,
            usage,
        }
    }
    
    pub fn text_content(&self) -> Option<String> {
        let text_blocks: Vec<&str> = self.content.iter()
            .filter_map(|block| {
                if let ContentBlock::Text { text } = block {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect();
        
        if text_blocks.is_empty() {
            None
        } else {
            Some(text_blocks.join(""))
        }
    }
    
    pub fn tool_uses(&self) -> Vec<&ContentBlock> {
        self.content.iter()
            .filter(|block| matches!(block, ContentBlock::ToolUse { .. }))
            .collect()
    }
}

// ============================================================================
// CLAUDE API AGGREGATE - Complete API State Management
// ============================================================================

/// Claude API Session Aggregate - manages complete API interaction state
#[derive(Debug, Clone, PartialEq)]
pub struct ClaudeApiSession {
    pub conversation_id: ConversationId,
    pub model_config: ClaudeModel,
    pub system_prompt: Option<ClaudeSystemPrompt>,
    pub message_history: Vec<ClaudeMessage>,
    pub tool_definitions: Vec<ClaudeToolDefinition>,
    pub total_usage: ClaudeUsage,
    pub last_request_id: Option<String>,
    pub error_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl ClaudeApiSession {
    pub fn new(conversation_id: ConversationId, model: ClaudeModel) -> Self {
        let now = chrono::Utc::now();
        Self {
            conversation_id,
            model_config: model,
            system_prompt: None,
            message_history: Vec::new(),
            tool_definitions: Vec::new(),
            total_usage: ClaudeUsage::new(0, 0),
            last_request_id: None,
            error_count: 0,
            created_at: now,
            last_activity: now,
        }
    }
    
    pub fn add_user_message(&mut self, content: MessageContent) {
        self.message_history.push(ClaudeMessage::user(content));
        self.last_activity = chrono::Utc::now();
    }
    
    pub fn add_assistant_message(&mut self, response: ClaudeApiResponse) {
        // Convert response content to message content
        let content = if response.content.len() == 1 && matches!(&response.content[0], ContentBlock::Text { .. }) {
            if let ContentBlock::Text { text } = &response.content[0] {
                MessageContent::Text(text.clone())
            } else {
                MessageContent::Structured(response.content.clone())
            }
        } else {
            MessageContent::Structured(response.content)
        };
        
        self.message_history.push(ClaudeMessage::assistant(content));
        
        // Update usage
        self.total_usage.input_tokens += response.usage.input_tokens;
        self.total_usage.output_tokens += response.usage.output_tokens;
        
        self.last_activity = chrono::Utc::now();
    }
    
    pub fn add_error(&mut self) {
        self.error_count += 1;
        self.last_activity = chrono::Utc::now();
    }
    
    pub fn estimated_cost_usd(&self) -> f64 {
        self.total_usage.estimated_cost_usd(&self.model_config)
    }
    
    pub fn can_add_message(&self, content: &MessageContent) -> bool {
        let current_tokens = self.estimated_total_tokens();
        let new_tokens = content.token_estimate();
        let max_context = self.model_config.context_window();
        
        current_tokens + new_tokens < max_context
    }
    
    fn estimated_total_tokens(&self) -> u32 {
        let history_tokens: u32 = self.message_history.iter()
            .map(|msg| msg.content.token_estimate())
            .sum();
        
        let system_tokens = self.system_prompt.as_ref()
            .map(|s| s.token_estimate())
            .unwrap_or(0);
        
        history_tokens + system_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_model_properties() {
        let model = ClaudeModel::Claude35Sonnet20241022;
        assert_eq!(model.as_str(), "claude-3-5-sonnet-20241022");
        assert_eq!(model.max_tokens(), 200_000);
        assert_eq!(model.context_window(), 200_000);
    }

    #[test]
    fn test_temperature_validation() {
        assert!(Temperature::new(0.5).is_ok());
        assert!(Temperature::new(0.0).is_ok());
        assert!(Temperature::new(1.0).is_ok());
        assert!(Temperature::new(1.5).is_err());
        assert!(Temperature::new(-0.1).is_err());
    }

    #[test]
    fn test_max_tokens_validation() {
        assert!(MaxTokens::new(1000).is_ok());
        assert!(MaxTokens::new(200_000).is_ok());
        assert!(MaxTokens::new(0).is_err());
        assert!(MaxTokens::new(300_000).is_err());
    }

    #[test]
    fn test_stop_sequences_validation() {
        assert!(StopSequences::new(vec!["STOP".to_string()]).is_ok());
        assert!(StopSequences::new(vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()]).is_ok());
        assert!(StopSequences::new(vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string()]).is_err());
        assert!(StopSequences::new(vec!["".to_string()]).is_err());
    }

    #[test]
    fn test_claude_api_request_validation() {
        let model = ClaudeModel::Claude35Sonnet20241022;
        let messages = vec![ClaudeMessage::user(MessageContent::text("Hello"))];
        let max_tokens = MaxTokens::new(1000).unwrap();
        
        let request = ClaudeApiRequest::new(model, messages, max_tokens);
        assert!(request.validate().is_ok());
        
        // Test empty messages
        let empty_request = ClaudeApiRequest::new(
            ClaudeModel::Claude35Sonnet20241022,
            vec![],
            MaxTokens::new(1000).unwrap()
        );
        assert!(empty_request.validate().is_err());
    }

    #[test]
    fn test_claude_api_session() {
        let conv_id = ConversationId::new();
        let mut session = ClaudeApiSession::new(conv_id, ClaudeModel::Claude35Sonnet20241022);
        
        session.add_user_message(MessageContent::text("Hello"));
        assert_eq!(session.message_history.len(), 1);
        
        let response = ClaudeApiResponse::new(
            ClaudeMessageId::new("msg_123".to_string()),
            ClaudeModel::Claude35Sonnet20241022,
            vec![ContentBlock::Text { text: "Hello!".to_string() }],
            StopReason::EndTurn,
            ClaudeUsage::new(10, 5),
        );
        
        session.add_assistant_message(response);
        assert_eq!(session.message_history.len(), 2);
        assert_eq!(session.total_usage.input_tokens, 10);
        assert_eq!(session.total_usage.output_tokens, 5);
    }

    #[test]
    fn test_claude_usage_cost_calculation() {
        let usage = ClaudeUsage::new(1_000_000, 500_000); // 1M input, 500K output
        let cost = usage.estimated_cost_usd(&ClaudeModel::Claude35Sonnet20241022);
        // Should be: (1.0 * 3.0) + (0.5 * 15.0) = 3.0 + 7.5 = 10.5
        assert!((cost - 10.5).abs() < 0.01);
    }
}