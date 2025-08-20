/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Subagent Management System
//!
//! This module provides a comprehensive system for loading, managing, and orchestrating
//! Claude expert subagents from the .claude/agents/ directory. It enables dynamic
//! routing of queries to appropriate domain experts based on the request context.

pub mod registry;
pub mod router;
pub mod dispatcher;
pub mod expert_definitions;
pub mod sage;

pub use registry::{SubagentRegistry, SubagentInfo, SubagentCapability};
pub use router::{SubagentRouter, SubjectResolution, ResolutionStrategy, RouteDecision, ExecutionStrategy, DomainType, DomainContext};
pub use dispatcher::{SubagentDispatcher, DispatchResult};
pub use expert_definitions::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// Core trait that all subagents must implement
#[async_trait]
pub trait Subagent: Send + Sync {
    /// Get the unique identifier for this subagent
    fn id(&self) -> &str;
    
    /// Get the display name for this subagent
    fn name(&self) -> &str;
    
    /// Get the description of this subagent's capabilities
    fn description(&self) -> &str;
    
    /// Get the list of tools this subagent has access to
    fn available_tools(&self) -> Vec<String>;
    
    /// Get the list of capabilities this subagent provides
    fn capabilities(&self) -> Vec<SubagentCapability>;
    
    /// Process a query and return a response
    async fn process_query(&self, query: SubagentQuery) -> Result<SubagentResponse, SubagentError>;
    
    /// Check if this subagent can handle a specific query
    fn can_handle(&self, query: &SubagentQuery) -> bool;
    
    /// Get the priority score for handling a query (higher = more suitable)
    fn priority_score(&self, query: &SubagentQuery) -> u32;
}

/// Query sent to a subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentQuery {
    pub id: String,
    pub user_id: String,
    pub conversation_id: Option<String>,
    pub query_text: String,
    pub context: SubagentContext,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Context information for subagent queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentContext {
    pub domain: Option<String>,
    pub task_type: TaskType,
    pub complexity: ComplexityLevel,
    pub requires_collaboration: bool,
    pub referenced_files: Vec<String>,
    pub keywords: Vec<String>,
}

/// Types of tasks that subagents can handle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Architecture,
    DomainModeling,
    EventStorming,
    Infrastructure,
    NetworkDesign,
    DomainCreation,
    Orchestration,
    Analysis,
    Implementation,
    Debugging,
}

/// Complexity levels for tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
    MultiExpert,
}

/// Response from a subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentResponse {
    pub query_id: String,
    pub subagent_id: String,
    pub response_text: String,
    pub confidence_score: f64,
    pub recommendations: Vec<SubagentRecommendation>,
    pub next_actions: Vec<NextAction>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Recommendations from subagents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentRecommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub priority: Priority,
    pub estimated_effort: Option<String>,
    pub dependencies: Vec<String>,
}

/// Types of recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    NextStep,
    ExpertConsultation,
    ResourceReference,
    ToolUsage,
    BestPractice,
    Warning,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Actions recommended after query completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextAction {
    pub action_type: ActionType,
    pub description: String,
    pub target_subagent: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Types of next actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    InvokeSubagent,
    CreateWorkflow,
    UpdateContext,
    GenerateArtifact,
    ValidateOutput,
}

/// Errors that can occur during subagent operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubagentError {
    NotFound(String),
    InvalidQuery(String),
    ProcessingFailed(String),
    TimeoutError(String),
    ConfigurationError(String),
    ValidationError(String),
}

impl std::fmt::Display for SubagentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubagentError::NotFound(msg) => write!(f, "Subagent not found: {}", msg),
            SubagentError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
            SubagentError::ProcessingFailed(msg) => write!(f, "Processing failed: {}", msg),
            SubagentError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            SubagentError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            SubagentError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for SubagentError {}