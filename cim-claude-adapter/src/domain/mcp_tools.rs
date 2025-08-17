/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! MCP Tools Domain
//! Handles tool integration via NATS - MCP tools are just NATS services
//! Message pattern: mcp->nats-claude->nats->mcp (everything over NATS)

use crate::domain::value_objects::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Tool ID for NATS-connected MCP tools
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(Uuid);

impl ToolId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
    
    pub fn is_nil(&self) -> bool {
        self.0.is_nil()
    }
}

impl Default for ToolId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tool invocation ID for tracking requests
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InvocationId(Uuid);

impl InvocationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl std::fmt::Display for InvocationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// NATS subject for tool communication
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolSubject(String);

impl ToolSubject {
    pub fn new(subject: String) -> Result<Self, String> {
        if subject.trim().is_empty() {
            return Err("Tool subject cannot be empty".to_string());
        }
        
        // Validate NATS subject format
        if !subject.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '*' || c == '>') {
            return Err("Invalid NATS subject format".to_string());
        }
        
        Ok(Self(subject))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToolSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tool definition for NATS-connected MCP tools
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub id: ToolId,
    pub name: String,
    pub description: String,
    pub version: String,
    pub request_subject: ToolSubject,    // Where to send requests to the tool
    pub response_subject: ToolSubject,   // Where tool sends responses
    pub health_subject: ToolSubject,     // Tool health checks
    pub schema: ToolSchema,
    pub capabilities: ToolCapabilities,
    pub metadata: HashMap<String, String>,
    pub timeout_seconds: u32,
    pub retry_count: u32,
}

impl ToolDefinition {
    pub fn new(
        name: String,
        description: String,
        request_subject: String,
    ) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Tool name cannot be empty".to_string());
        }
        
        let tool_id = ToolId::new();
        let base_subject = format!("cim.tools.{}.{}", name.to_lowercase(), tool_id);
        
        Ok(Self {
            id: tool_id,
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            version: "1.0.0".to_string(),
            request_subject: ToolSubject::new(request_subject)?,
            response_subject: ToolSubject::new(format!("{}.resp", base_subject))?,
            health_subject: ToolSubject::new(format!("{}.health", base_subject))?,
            schema: ToolSchema::default(),
            capabilities: ToolCapabilities::default(),
            metadata: HashMap::new(),
            timeout_seconds: 30,
            retry_count: 3,
        })
    }
}

/// Tool input/output schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolSchema {
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub error_schema: serde_json::Value,
}

impl Default for ToolSchema {
    fn default() -> Self {
        Self {
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: serde_json::json!({"type": "object"}),
            error_schema: serde_json::json!({"type": "object", "properties": {"error": {"type": "string"}}}),
        }
    }
}

/// Tool capabilities and permissions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub can_read_files: bool,
    pub can_write_files: bool,
    pub can_execute_commands: bool,
    pub can_access_network: bool,
    pub requires_user_confirmation: bool,
    pub allowed_domains: Vec<String>,
    pub blocked_domains: Vec<String>,
    pub max_execution_time_seconds: u32,
}

impl Default for ToolCapabilities {
    fn default() -> Self {
        Self {
            can_read_files: false,
            can_write_files: false,
            can_execute_commands: false,
            can_access_network: false,
            requires_user_confirmation: true,
            allowed_domains: Vec::new(),
            blocked_domains: Vec::new(),
            max_execution_time_seconds: 30,
        }
    }
}

/// Tool commands - all tool interactions are commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolCommand {
    /// Register a new tool (tool announces itself on NATS)
    RegisterTool {
        tool_definition: ToolDefinition,
        registration_source: String, // Which service is registering
        correlation_id: CorrelationId,
    },
    
    /// Update tool configuration
    UpdateTool {
        tool_id: ToolId,
        updated_definition: ToolDefinition,
        correlation_id: CorrelationId,
    },
    
    /// Unregister tool (tool leaving NATS)
    UnregisterTool {
        tool_id: ToolId,
        reason: String,
        correlation_id: CorrelationId,
    },
    
    /// Invoke tool with parameters
    InvokeTool {
        tool_id: ToolId,
        invocation_id: InvocationId,
        conversation_id: ConversationId,
        parameters: serde_json::Value,
        timeout_override: Option<u32>,
        correlation_id: CorrelationId,
    },
    
    /// Enable tool for specific conversation
    EnableToolForConversation {
        tool_id: ToolId,
        conversation_id: ConversationId,
        permissions: ToolCapabilities,
        correlation_id: CorrelationId,
    },
    
    /// Disable tool for conversation
    DisableToolForConversation {
        tool_id: ToolId,
        conversation_id: ConversationId,
        correlation_id: CorrelationId,
    },
    
    /// Health check tool (ping via NATS)
    HealthCheckTool {
        tool_id: ToolId,
        correlation_id: CorrelationId,
    },
}

/// Tool events - results of processing commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolEvent {
    /// Tool registered successfully on NATS
    ToolRegistered {
        tool_definition: ToolDefinition,
        registered_at: chrono::DateTime<chrono::Utc>,
        registered_by: String,
    },
    
    /// Tool updated
    ToolUpdated {
        tool_id: ToolId,
        old_definition: ToolDefinition,
        new_definition: ToolDefinition,
        updated_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Tool unregistered from NATS
    ToolUnregistered {
        tool_id: ToolId,
        tool_name: String,
        reason: String,
        unregistered_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Tool invocation started
    ToolInvocationStarted {
        tool_id: ToolId,
        invocation_id: InvocationId,
        conversation_id: ConversationId,
        parameters: serde_json::Value,
        started_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Tool invocation completed successfully
    ToolInvocationCompleted {
        tool_id: ToolId,
        invocation_id: InvocationId,
        conversation_id: ConversationId,
        result: serde_json::Value,
        execution_time_ms: u64,
        completed_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Tool invocation failed
    ToolInvocationFailed {
        tool_id: ToolId,
        invocation_id: InvocationId,
        conversation_id: ConversationId,
        error: String,
        error_code: Option<String>,
        execution_time_ms: u64,
        failed_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Tool enabled for conversation
    ToolEnabledForConversation {
        tool_id: ToolId,
        conversation_id: ConversationId,
        permissions: ToolCapabilities,
        enabled_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Tool disabled for conversation
    ToolDisabledForConversation {
        tool_id: ToolId,
        conversation_id: ConversationId,
        disabled_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Tool became unavailable (stopped responding on NATS)
    ToolBecameUnavailable {
        tool_id: ToolId,
        tool_name: String,
        last_seen: chrono::DateTime<chrono::Utc>,
        unavailable_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Tool became available (started responding on NATS)
    ToolBecameAvailable {
        tool_id: ToolId,
        tool_name: String,
        available_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Health check completed
    ToolHealthCheckCompleted {
        tool_id: ToolId,
        is_healthy: bool,
        response_time_ms: u64,
        health_details: HashMap<String, String>,
        checked_at: chrono::DateTime<chrono::Utc>,
    },
}

/// Tool queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolQuery {
    /// Get available tools
    GetAvailableTools {
        filter_by_capability: Option<ToolCapabilities>,
    },
    
    /// Get tool details
    GetToolDetails {
        tool_id: ToolId,
    },
    
    /// Get tools enabled for conversation
    GetConversationTools {
        conversation_id: ConversationId,
    },
    
    /// Get tool invocation history
    GetToolInvocationHistory {
        tool_id: ToolId,
        conversation_id: Option<ConversationId>,
        limit: Option<u32>,
        offset: Option<u32>,
    },
    
    /// Get tool health status
    GetToolHealthStatus {
        tool_id: Option<ToolId>, // None = all tools
    },
}

/// Tool registry for managing available tools
#[derive(Debug, Clone, PartialEq)]
pub struct ToolRegistry {
    pub tools: HashMap<ToolId, ToolDefinition>,
    pub conversation_tools: HashMap<ConversationId, Vec<ToolId>>,
    pub tool_health: HashMap<ToolId, ToolHealthStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolHealthStatus {
    pub is_available: bool,
    pub last_ping: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: u64,
    pub error_count_24h: u32,
    pub success_rate_24h: f64,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            conversation_tools: HashMap::new(),
            tool_health: HashMap::new(),
        }
    }
    
    pub fn register_tool(&mut self, tool: ToolDefinition) {
        let tool_id = tool.id.clone();
        self.tools.insert(tool_id.clone(), tool);
        self.tool_health.insert(tool_id, ToolHealthStatus {
            is_available: true,
            last_ping: chrono::Utc::now(),
            response_time_ms: 0,
            error_count_24h: 0,
            success_rate_24h: 100.0,
        });
    }
    
    pub fn unregister_tool(&mut self, tool_id: &ToolId) {
        self.tools.remove(tool_id);
        self.tool_health.remove(tool_id);
        
        // Remove from all conversations
        for tools in self.conversation_tools.values_mut() {
            tools.retain(|id| id != tool_id);
        }
    }
    
    pub fn enable_tool_for_conversation(&mut self, tool_id: ToolId, conversation_id: ConversationId) {
        self.conversation_tools
            .entry(conversation_id)
            .or_insert_with(Vec::new)
            .push(tool_id);
    }
    
    pub fn disable_tool_for_conversation(&mut self, tool_id: &ToolId, conversation_id: &ConversationId) {
        if let Some(tools) = self.conversation_tools.get_mut(conversation_id) {
            tools.retain(|id| id != tool_id);
        }
    }
    
    pub fn get_conversation_tools(&self, conversation_id: &ConversationId) -> Vec<&ToolDefinition> {
        self.conversation_tools
            .get(conversation_id)
            .map(|tool_ids| {
                tool_ids
                    .iter()
                    .filter_map(|id| self.tools.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition_creation() {
        let tool = ToolDefinition::new(
            "test_tool".to_string(),
            "A test tool".to_string(),
            "cim.tools.test.req".to_string(),
        ).unwrap();
        
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
        assert_eq!(tool.request_subject.as_str(), "cim.tools.test.req");
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        
        let tool = ToolDefinition::new(
            "test_tool".to_string(),
            "A test tool".to_string(),
            "cim.tools.test.req".to_string(),
        ).unwrap();
        
        let tool_id = tool.id.clone();
        registry.register_tool(tool);
        
        assert!(registry.tools.contains_key(&tool_id));
        assert!(registry.tool_health.contains_key(&tool_id));
        
        // Enable for conversation
        let conv_id = ConversationId::new();
        registry.enable_tool_for_conversation(tool_id.clone(), conv_id.clone());
        
        let conv_tools = registry.get_conversation_tools(&conv_id);
        assert_eq!(conv_tools.len(), 1);
        assert_eq!(conv_tools[0].id, tool_id);
        
        // Unregister
        registry.unregister_tool(&tool_id);
        assert!(!registry.tools.contains_key(&tool_id));
        
        let conv_tools = registry.get_conversation_tools(&conv_id);
        assert_eq!(conv_tools.len(), 0);
    }

    #[test]
    fn test_tool_subject_validation() {
        assert!(ToolSubject::new("".to_string()).is_err());
        assert!(ToolSubject::new("valid.subject".to_string()).is_ok());
        assert!(ToolSubject::new("invalid subject with spaces".to_string()).is_err());
        assert!(ToolSubject::new("tools.*.cmd".to_string()).is_ok());
        assert!(ToolSubject::new("tools.>".to_string()).is_ok());
    }
}