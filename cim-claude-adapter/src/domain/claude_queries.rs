/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Complete Claude API Query Mapping
//! 
//! Every possible data retrieval operation maps to a Query in our event-sourced system.
//! This provides structured access to all conversation data, usage statistics, and system state.

use crate::domain::claude_api::*;
use crate::domain::claude_commands::*;
use crate::domain::claude_events::*;
use crate::domain::value_objects::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Query ID for tracking data retrieval requests
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClaudeQueryId(uuid::Uuid);

impl ClaudeQueryId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    
    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl std::fmt::Display for ClaudeQueryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Complete Claude API Queries - Every data retrieval operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClaudeApiQuery {
    /// Get conversation details and current state
    GetConversation {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        include_messages: bool,
        include_usage_stats: bool,
        include_tool_definitions: bool,
    },
    
    /// Get conversation message history
    GetConversationHistory {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        limit: Option<u32>,
        offset: Option<u32>,
        include_tool_use: bool,
        include_system_messages: bool,
        message_roles: Option<Vec<MessageRole>>,
    },
    
    /// Get specific message by ID
    GetMessage {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        message_id: ClaudeMessageId,
        include_metadata: bool,
    },
    
    /// Search messages by content
    SearchMessages {
        query_id: ClaudeQueryId,
        conversation_id: Option<ConversationId>,
        search_query: String,
        search_options: MessageSearchOptions,
        limit: Option<u32>,
        offset: Option<u32>,
    },
    
    /// Get conversation usage statistics
    GetUsageStatistics {
        query_id: ClaudeQueryId,
        conversation_id: Option<ConversationId>,
        time_range: Option<TimeRange>,
        group_by: UsageGroupBy,
        include_cost_breakdown: bool,
    },
    
    /// Get cost analysis
    GetCostAnalysis {
        query_id: ClaudeQueryId,
        conversation_id: Option<ConversationId>,
        time_range: Option<TimeRange>,
        cost_breakdown: CostBreakdownOptions,
    },
    
    /// Get current model configuration
    GetModelConfiguration {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        include_history: bool,
    },
    
    /// Get system prompt history
    GetSystemPromptHistory {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        limit: Option<u32>,
        include_metadata: bool,
    },
    
    /// Get available tools for conversation
    GetConversationTools {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        include_schemas: bool,
        include_usage_stats: bool,
    },
    
    /// Get tool usage history
    GetToolUsageHistory {
        query_id: ClaudeQueryId,
        conversation_id: Option<ConversationId>,
        tool_name: Option<String>,
        time_range: Option<TimeRange>,
        limit: Option<u32>,
        offset: Option<u32>,
    },
    
    /// Get tool execution details
    GetToolExecution {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        tool_use_id: String,
        include_full_input: bool,
        include_full_output: bool,
    },
    
    /// Get conversation events (audit trail)
    GetConversationEvents {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        event_types: Option<Vec<ClaudeApiEventType>>,
        time_range: Option<TimeRange>,
        limit: Option<u32>,
        offset: Option<u32>,
    },
    
    /// Get error history
    GetErrorHistory {
        query_id: ClaudeQueryId,
        conversation_id: Option<ConversationId>,
        error_types: Option<Vec<ClaudeErrorType>>,
        time_range: Option<TimeRange>,
        limit: Option<u32>,
        include_retry_attempts: bool,
    },
    
    /// Get rate limit status
    GetRateLimitStatus {
        query_id: ClaudeQueryId,
        session_id: Option<SessionId>,
        limit_types: Option<Vec<RateLimitType>>,
    },
    
    /// Get API health status
    GetApiHealthStatus {
        query_id: ClaudeQueryId,
        include_response_times: bool,
        include_error_rates: bool,
        time_range: Option<TimeRange>,
    },
    
    /// Get performance metrics
    GetPerformanceMetrics {
        query_id: ClaudeQueryId,
        conversation_id: Option<ConversationId>,
        metric_types: Vec<PerformanceMetricType>,
        time_range: Option<TimeRange>,
        aggregation: MetricAggregation,
    },
    
    /// Get streaming session details
    GetStreamingSession {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        command_id: ClaudeCommandId,
        include_chunks: bool,
        include_timing: bool,
    },
    
    /// Get conversation validation results
    GetValidationResults {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        validation_types: Option<Vec<ValidationRuleType>>,
        include_history: bool,
    },
    
    /// Get quota usage
    GetQuotaUsage {
        query_id: ClaudeQueryId,
        quota_type: QuotaType,
        time_range: Option<TimeRange>,
        include_projections: bool,
    },
    
    /// Get conversation analytics
    GetConversationAnalytics {
        query_id: ClaudeQueryId,
        conversation_id: Option<ConversationId>,
        analytics_types: Vec<AnalyticsType>,
        time_range: Option<TimeRange>,
    },
    
    /// Get export data
    GetExportData {
        query_id: ClaudeQueryId,
        conversation_id: ConversationId,
        export_format: ExportFormat,
        export_options: ExportOptions,
    },
    
    /// Get conversation comparison
    CompareConversations {
        query_id: ClaudeQueryId,
        conversation_ids: Vec<ConversationId>,
        comparison_criteria: Vec<ComparisonCriterion>,
    },
    
    /// Get session summary
    GetSessionSummary {
        query_id: ClaudeQueryId,
        session_id: SessionId,
        include_all_conversations: bool,
        include_aggregated_stats: bool,
    },
    
    /// Get configuration templates
    GetConfigurationTemplates {
        query_id: ClaudeQueryId,
        template_category: Option<TemplateCategory>,
        include_usage_stats: bool,
    },
    
    /// Search conversations
    SearchConversations {
        query_id: ClaudeQueryId,
        search_criteria: ConversationSearchCriteria,
        sort_options: ConversationSortOptions,
        limit: Option<u32>,
        offset: Option<u32>,
    },
}

// ============================================================================
// QUERY PARAMETERS AND OPTIONS
// ============================================================================

/// Time range for filtering queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, String> {
        if start >= end {
            return Err("Start time must be before end time".to_string());
        }
        Ok(Self { start, end })
    }
    
    pub fn last_hours(hours: u32) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::hours(hours as i64);
        Self { start, end }
    }
    
    pub fn last_days(days: u32) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days as i64);
        Self { start, end }
    }
    
    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        Self { start, end: now }
    }
}

/// Message search options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageSearchOptions {
    pub case_sensitive: bool,
    pub exact_match: bool,
    pub regex_enabled: bool,
    pub search_in_tool_content: bool,
    pub search_in_metadata: bool,
    pub include_context: bool,
    pub context_messages: u32,
}

impl Default for MessageSearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            exact_match: false,
            regex_enabled: false,
            search_in_tool_content: true,
            search_in_metadata: false,
            include_context: false,
            context_messages: 0,
        }
    }
}

/// Usage grouping options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UsageGroupBy {
    Hour,
    Day,
    Week,
    Month,
    Conversation,
    Model,
    Session,
}

/// Cost breakdown options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostBreakdownOptions {
    pub by_model: bool,
    pub by_conversation: bool,
    pub by_session: bool,
    pub by_time_period: Option<UsageGroupBy>,
    pub include_tool_costs: bool,
    pub include_storage_costs: bool,
    pub currency: CostCurrency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CostCurrency {
    USD,
    EUR,
    GBP,
    JPY,
}

/// Claude API event types for filtering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClaudeApiEventType {
    MessageResponse,
    StreamingChunk,
    StreamingComplete,
    ApiError,
    Timeout,
    Cancellation,
    Retry,
    SystemPromptUpdate,
    ModelConfigUpdate,
    ToolsUpdate,
    ToolUse,
    ToolExecution,
    Validation,
    RateLimit,
    UsageThreshold,
    HealthCheck,
}

/// Performance metric types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformanceMetricType {
    ResponseTime,
    TokensPerSecond,
    ErrorRate,
    SuccessRate,
    RetryRate,
    ThroughputRps,
    LatencyP50,
    LatencyP95,
    LatencyP99,
    ConcurrentRequests,
    QueueTime,
    ProcessingTime,
}

/// Metric aggregation methods
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricAggregation {
    Average,
    Sum,
    Min,
    Max,
    Median,
    P95,
    P99,
    Count,
}

/// Quota types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuotaType {
    MonthlyTokens,
    DailyRequests,
    ConcurrentRequests,
    MonthlySpend,
    StorageBytes,
}

/// Analytics types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnalyticsType {
    MessageLength,
    ResponseTime,
    TokenEfficiency,
    ToolUsagePatterns,
    ErrorPatterns,
    UserBehavior,
    CostOptimization,
    ModelPerformance,
}

/// Export options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_system_prompts: bool,
    pub include_tool_definitions: bool,
    pub include_metadata: bool,
    pub include_usage_stats: bool,
    pub include_error_history: bool,
    pub compress_output: bool,
    pub split_large_files: bool,
    pub max_file_size_mb: Option<u32>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_system_prompts: true,
            include_tool_definitions: true,
            include_metadata: true,
            include_usage_stats: true,
            include_error_history: false,
            compress_output: false,
            split_large_files: false,
            max_file_size_mb: None,
        }
    }
}

/// Comparison criteria for conversation comparison
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonCriterion {
    MessageCount,
    TokenUsage,
    Cost,
    ResponseTimes,
    ErrorRates,
    ToolUsage,
    ModelConfiguration,
    SystemPrompts,
}

/// Template categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemplateCategory {
    SystemPrompts,
    ModelConfigurations,
    ToolSets,
    ConversationSettings,
    ValidationRules,
}

/// Conversation search criteria
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationSearchCriteria {
    pub text_query: Option<String>,
    pub session_ids: Option<Vec<SessionId>>,
    pub models_used: Option<Vec<ClaudeModel>>,
    pub time_range: Option<TimeRange>,
    pub min_messages: Option<u32>,
    pub max_messages: Option<u32>,
    pub has_tools: Option<bool>,
    pub has_errors: Option<bool>,
    pub cost_range: Option<CostRange>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostRange {
    pub min_cost_usd: f64,
    pub max_cost_usd: f64,
}

/// Conversation sort options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationSortOptions {
    pub sort_by: ConversationSortBy,
    pub sort_direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationSortBy {
    CreatedAt,
    LastActivity,
    MessageCount,
    TotalTokens,
    TotalCost,
    ErrorCount,
    Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

// ============================================================================
// QUERY RESULTS
// ============================================================================

/// Query results for conversation details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationDetails {
    pub conversation_id: ConversationId,
    pub session_id: SessionId,
    pub current_model: ClaudeModel,
    pub system_prompt: Option<ClaudeSystemPrompt>,
    pub tool_definitions: Vec<ClaudeToolDefinition>,
    pub message_count: u32,
    pub total_usage: ClaudeUsage,
    pub estimated_cost_usd: f64,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub status: ConversationStatus,
    pub messages: Option<Vec<ClaudeMessage>>,
    pub error_count: u32,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationStatus {
    Active,
    Paused,
    Archived,
    Error,
    RateLimited,
    TokenLimitReached,
}

impl ClaudeApiQuery {
    pub fn query_id(&self) -> &ClaudeQueryId {
        match self {
            Self::GetConversation { query_id, .. } => query_id,
            Self::GetConversationHistory { query_id, .. } => query_id,
            Self::GetMessage { query_id, .. } => query_id,
            Self::SearchMessages { query_id, .. } => query_id,
            Self::GetUsageStatistics { query_id, .. } => query_id,
            Self::GetCostAnalysis { query_id, .. } => query_id,
            Self::GetModelConfiguration { query_id, .. } => query_id,
            Self::GetSystemPromptHistory { query_id, .. } => query_id,
            Self::GetConversationTools { query_id, .. } => query_id,
            Self::GetToolUsageHistory { query_id, .. } => query_id,
            Self::GetToolExecution { query_id, .. } => query_id,
            Self::GetConversationEvents { query_id, .. } => query_id,
            Self::GetErrorHistory { query_id, .. } => query_id,
            Self::GetRateLimitStatus { query_id, .. } => query_id,
            Self::GetApiHealthStatus { query_id, .. } => query_id,
            Self::GetPerformanceMetrics { query_id, .. } => query_id,
            Self::GetStreamingSession { query_id, .. } => query_id,
            Self::GetValidationResults { query_id, .. } => query_id,
            Self::GetQuotaUsage { query_id, .. } => query_id,
            Self::GetConversationAnalytics { query_id, .. } => query_id,
            Self::GetExportData { query_id, .. } => query_id,
            Self::CompareConversations { query_id, .. } => query_id,
            Self::GetSessionSummary { query_id, .. } => query_id,
            Self::GetConfigurationTemplates { query_id, .. } => query_id,
            Self::SearchConversations { query_id, .. } => query_id,
        }
    }
    
    pub fn conversation_id(&self) -> Option<&ConversationId> {
        match self {
            Self::GetConversation { conversation_id, .. } => Some(conversation_id),
            Self::GetConversationHistory { conversation_id, .. } => Some(conversation_id),
            Self::GetMessage { conversation_id, .. } => Some(conversation_id),
            Self::SearchMessages { conversation_id: Some(id), .. } => Some(id),
            Self::GetUsageStatistics { conversation_id: Some(id), .. } => Some(id),
            Self::GetCostAnalysis { conversation_id: Some(id), .. } => Some(id),
            Self::GetModelConfiguration { conversation_id, .. } => Some(conversation_id),
            Self::GetSystemPromptHistory { conversation_id, .. } => Some(conversation_id),
            Self::GetConversationTools { conversation_id, .. } => Some(conversation_id),
            Self::GetToolUsageHistory { conversation_id: Some(id), .. } => Some(id),
            Self::GetToolExecution { conversation_id, .. } => Some(conversation_id),
            Self::GetConversationEvents { conversation_id, .. } => Some(conversation_id),
            Self::GetErrorHistory { conversation_id: Some(id), .. } => Some(id),
            Self::GetPerformanceMetrics { conversation_id: Some(id), .. } => Some(id),
            Self::GetStreamingSession { conversation_id, .. } => Some(conversation_id),
            Self::GetValidationResults { conversation_id, .. } => Some(conversation_id),
            Self::GetConversationAnalytics { conversation_id: Some(id), .. } => Some(id),
            Self::GetExportData { conversation_id, .. } => Some(conversation_id),
            _ => None, // Queries without specific conversation context
        }
    }
    
    pub fn requires_pagination(&self) -> bool {
        matches!(
            self,
            Self::GetConversationHistory { .. } |
            Self::SearchMessages { .. } |
            Self::GetToolUsageHistory { .. } |
            Self::GetConversationEvents { .. } |
            Self::GetErrorHistory { .. } |
            Self::SearchConversations { .. } |
            Self::GetSystemPromptHistory { .. }
        )
    }
    
    pub fn is_expensive_query(&self) -> bool {
        matches!(
            self,
            Self::SearchMessages { .. } |
            Self::GetConversationAnalytics { .. } |
            Self::CompareConversations { .. } |
            Self::SearchConversations { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range_validation() {
        let now = Utc::now();
        let past = now - chrono::Duration::hours(1);
        
        assert!(TimeRange::new(past, now).is_ok());
        assert!(TimeRange::new(now, past).is_err());
    }

    #[test]
    fn test_time_range_convenience_methods() {
        let last_24h = TimeRange::last_hours(24);
        let duration = last_24h.end - last_24h.start;
        assert_eq!(duration.num_hours(), 24);
        
        let last_week = TimeRange::last_days(7);
        let duration = last_week.end - last_week.start;
        assert_eq!(duration.num_days(), 7);
    }

    #[test]
    fn test_query_classification() {
        let query = ClaudeApiQuery::GetConversation {
            query_id: ClaudeQueryId::new(),
            conversation_id: ConversationId::new(),
            include_messages: true,
            include_usage_stats: true,
            include_tool_definitions: false,
        };
        
        assert!(query.conversation_id().is_some());
        assert!(!query.requires_pagination());
        assert!(!query.is_expensive_query());
        
        let search_query = ClaudeApiQuery::SearchMessages {
            query_id: ClaudeQueryId::new(),
            conversation_id: None,
            search_query: "test".to_string(),
            search_options: MessageSearchOptions::default(),
            limit: Some(100),
            offset: None,
        };
        
        assert!(search_query.requires_pagination());
        assert!(search_query.is_expensive_query());
    }

    #[test]
    fn test_cost_range() {
        let range = CostRange {
            min_cost_usd: 0.0,
            max_cost_usd: 10.0,
        };
        
        assert_eq!(range.min_cost_usd, 0.0);
        assert_eq!(range.max_cost_usd, 10.0);
    }
}