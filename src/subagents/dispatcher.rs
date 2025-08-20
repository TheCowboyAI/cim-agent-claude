/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Subagent Dispatcher
//!
//! Handles the execution of queries by dispatching them to the appropriate subagent(s)
//! based on routing decisions. Manages conversation state, error handling, and response
//! aggregation for multi-agent scenarios.

use super::{SubagentQuery, SubagentResponse, SubagentError, Subagent};
use super::registry::SubagentRegistry;
use super::router::{SubagentRouter, SubjectResolution, ResolutionStrategy, RouteDecision, ExecutionStrategy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{info, warn, error, debug};

/// Result of dispatching a query to subagents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResult {
    pub query_id: String,
    pub primary_response: SubagentResponse,
    pub secondary_responses: Vec<SubagentResponse>,
    pub routing_decision: RouteDecision,
    pub execution_time: Duration,
    pub status: DispatchStatus,
    pub aggregated_confidence: f64,
    pub next_recommendations: Vec<super::SubagentRecommendation>,
}

/// Status of query dispatch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DispatchStatus {
    Success,
    PartialSuccess,
    Failed,
    Timeout,
    NoAgentAvailable,
}

/// Configuration for the dispatcher
#[derive(Debug, Clone)]
pub struct DispatcherConfig {
    pub default_timeout: Duration,
    pub max_parallel_agents: usize,
    pub retry_attempts: u32,
    pub enable_response_aggregation: bool,
    pub confidence_threshold: f64,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_parallel_agents: 3,
            retry_attempts: 2,
            enable_response_aggregation: true,
            confidence_threshold: 0.6,
        }
    }
}

/// Dispatcher for executing subagent queries
pub struct SubagentDispatcher {
    registry: Arc<SubagentRegistry>,
    router: Arc<SubagentRouter>,
    config: DispatcherConfig,
    active_queries: Arc<tokio::sync::RwLock<HashMap<String, QueryExecution>>>,
}

/// Tracks execution state of a query
#[derive(Debug, Clone)]
struct QueryExecution {
    query: SubagentQuery,
    routing_decision: RouteDecision,
    started_at: Instant,
    status: DispatchStatus,
    responses: Vec<SubagentResponse>,
}

impl SubagentDispatcher {
    /// Create a new subagent dispatcher
    pub fn new(
        registry: Arc<SubagentRegistry>,
        router: Arc<SubagentRouter>,
        config: DispatcherConfig,
    ) -> Self {
        Self {
            registry,
            router,
            config,
            active_queries: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Dispatch a query to the appropriate subagent(s)
    pub async fn dispatch_query(&self, query: SubagentQuery) -> Result<DispatchResult, SubagentError> {
        let start_time = Instant::now();
        let query_id = query.id.clone();
        
        info!("Dispatching query: {} ({})", query.query_text.chars().take(50).collect::<String>(), query_id);

        // Step 1: Route the query to determine execution strategy
        let routing_decision = self.router.route_query(&query).await?;
        debug!("Routing decision: {:?}", routing_decision);

        // Step 2: Track query execution
        {
            let mut active_queries = self.active_queries.write().await;
            active_queries.insert(query_id.clone(), QueryExecution {
                query: query.clone(),
                routing_decision: routing_decision.clone(),
                started_at: start_time,
                status: DispatchStatus::Success,
                responses: Vec::new(),
            });
        }

        // Step 3: Execute based on routing strategy
        let result = match routing_decision.execution_strategy {
            ExecutionStrategy::Sequential => {
                self.execute_sequential(&query, &routing_decision).await
            },
            ExecutionStrategy::Parallel => {
                self.execute_parallel(&query, &routing_decision).await
            },
            ExecutionStrategy::Adaptive => {
                self.execute_adaptive(&query, &routing_decision).await
            },
        };

        // Step 4: Clean up tracking
        {
            let mut active_queries = self.active_queries.write().await;
            active_queries.remove(&query_id);
        }

        // Step 5: Handle result and create dispatch result
        match result {
            Ok(mut dispatch_result) => {
                dispatch_result.execution_time = start_time.elapsed();
                info!("Query {} completed successfully in {:?}", query_id, dispatch_result.execution_time);
                Ok(dispatch_result)
            },
            Err(e) => {
                error!("Query {} failed: {}", query_id, e);
                // Create failed dispatch result
                Ok(DispatchResult {
                    query_id: query_id.clone(),
                    primary_response: self.build_error_response(&query, &e),
                    secondary_responses: Vec::new(),
                    routing_decision,
                    execution_time: start_time.elapsed(),
                    status: DispatchStatus::Failed,
                    aggregated_confidence: 0.0,
                    next_recommendations: Vec::new(),
                })
            }
        }
    }

    /// Execute query with a single agent
    async fn execute_single_agent(&self, query: &SubagentQuery, decision: &RouteDecision) -> Result<DispatchResult, SubagentError> {
        let agent = self.registry.get_agent(&decision.primary_agent).await
            .ok_or_else(|| SubagentError::NotFound(format!("Agent '{}' not found", decision.primary_agent)))?;

        let response = self.execute_with_timeout(agent, query.clone()).await?;

        Ok(DispatchResult {
            query_id: query.id.clone(),
            primary_response: response,
            secondary_responses: Vec::new(),
            routing_decision: decision.clone(),
            execution_time: Duration::from_secs(0), // Will be set by caller
            status: DispatchStatus::Success,
            aggregated_confidence: decision.confidence_score,
            next_recommendations: Vec::new(),
        })
    }

    /// Execute query with multiple agents sequentially
    async fn execute_sequential(&self, query: &SubagentQuery, decision: &RouteDecision) -> Result<DispatchResult, SubagentError> {
        let mut responses = Vec::new();
        let mut current_query = query.clone();

        // Execute primary agent first
        let primary_agent = self.registry.get_agent(&decision.primary_agent).await
            .ok_or_else(|| SubagentError::NotFound(format!("Agent '{}' not found", decision.primary_agent)))?;
        
        let primary_response = self.execute_with_timeout(primary_agent, current_query.clone()).await?;
        
        // Update context for subsequent agents based on primary response
        current_query.context.keywords.extend(
            self.extract_keywords_from_response(&primary_response)
        );

        // Execute secondary agents
        for secondary_agent_id in &decision.secondary_agents {
            if let Some(agent) = self.registry.get_agent(secondary_agent_id).await {
                match self.execute_with_timeout(agent, current_query.clone()).await {
                    Ok(response) => {
                        responses.push(response);
                        // Update query context for next agent
                        current_query.context.keywords.extend(
                            self.extract_keywords_from_response(responses.last().unwrap())
                        );
                    },
                    Err(e) => {
                        warn!("Secondary agent {} failed: {}", secondary_agent_id, e);
                    }
                }
            }
        }

        let aggregated_confidence = self.calculate_aggregated_confidence(&primary_response, &responses);
        let next_recommendations = self.aggregate_recommendations(&primary_response, &responses);

        Ok(DispatchResult {
            query_id: query.id.clone(),
            primary_response,
            secondary_responses: responses,
            routing_decision: decision.clone(),
            execution_time: Duration::from_secs(0),
            status: DispatchStatus::Success,
            aggregated_confidence,
            next_recommendations,
        })
    }

    /// Execute query with multiple agents in parallel
    async fn execute_parallel(&self, query: &SubagentQuery, decision: &RouteDecision) -> Result<DispatchResult, SubagentError> {
        let mut handles = Vec::new();

        // Execute primary agent
        let primary_agent = self.registry.get_agent(&decision.primary_agent).await
            .ok_or_else(|| SubagentError::NotFound(format!("Agent '{}' not found", decision.primary_agent)))?;
        
        let primary_query = query.clone();
        let primary_handle = tokio::spawn(async move {
            primary_agent.process_query(primary_query).await
        });
        
        // Execute secondary agents in parallel (limited by config)
        let secondary_agents = decision.secondary_agents.iter()
            .take(self.config.max_parallel_agents.saturating_sub(1)) // Reserve one slot for primary
            .collect::<Vec<_>>();

        for secondary_agent_id in secondary_agents {
            if let Some(agent) = self.registry.get_agent(secondary_agent_id).await {
                let agent_clone = agent.clone();
                let query_clone = query.clone();
                let handle = tokio::spawn(async move {
                    agent_clone.process_query(query_clone).await
                });
                handles.push(handle);
            }
        }

        // Wait for all agents to complete
        let primary_response = timeout(self.config.default_timeout, primary_handle).await
            .map_err(|_| SubagentError::TimeoutError("Primary agent timeout".to_string()))?
            .map_err(|e| SubagentError::ProcessingFailed(format!("Primary agent failed: {}", e)))?
            .map_err(|e| e)?;

        let mut secondary_responses = Vec::new();
        for handle in handles {
            match timeout(self.config.default_timeout, handle).await {
                Ok(Ok(Ok(response))) => secondary_responses.push(response),
                Ok(Ok(Err(e))) => warn!("Secondary agent failed: {}", e),
                Ok(Err(e)) => warn!("Secondary agent task failed: {}", e),
                Err(_) => warn!("Secondary agent timeout"),
            }
        }

        let aggregated_confidence = self.calculate_aggregated_confidence(&primary_response, &secondary_responses);
        let next_recommendations = self.aggregate_recommendations(&primary_response, &secondary_responses);

        Ok(DispatchResult {
            query_id: query.id.clone(),
            primary_response,
            secondary_responses,
            routing_decision: decision.clone(),
            execution_time: Duration::from_secs(0),
            status: DispatchStatus::Success,
            aggregated_confidence,
            next_recommendations,
        })
    }

    /// Execute orchestrated query (via SAGE)
    async fn execute_adaptive(&self, query: &SubagentQuery, decision: &RouteDecision) -> Result<DispatchResult, SubagentError> {
        // For orchestrated queries, we route through SAGE which manages the workflow
        let sage_agent = self.registry.get_agent("sage").await
            .ok_or_else(|| SubagentError::NotFound("SAGE orchestrator not available".to_string()))?;

        let response = self.execute_with_timeout(sage_agent, query.clone()).await?;

        Ok(DispatchResult {
            query_id: query.id.clone(),
            primary_response: response,
            secondary_responses: Vec::new(),
            routing_decision: decision.clone(),
            execution_time: Duration::from_secs(0),
            status: DispatchStatus::Success,
            aggregated_confidence: decision.confidence_score,
            next_recommendations: Vec::new(),
        })
    }

    /// Execute collaborative query (multiple agents working together)
    async fn execute_collaborative(&self, query: &SubagentQuery, decision: &RouteDecision) -> Result<DispatchResult, SubagentError> {
        // For collaborative queries, we use event-storming-expert to facilitate
        let facilitator = self.registry.get_agent("event-storming-expert").await
            .ok_or_else(|| SubagentError::NotFound("Event storming facilitator not available".to_string()))?;

        // Create a collaborative query context
        let mut collaborative_query = query.clone();
        collaborative_query.metadata.insert(
            "collaboration_mode".to_string(),
            serde_json::Value::Bool(true)
        );
        collaborative_query.metadata.insert(
            "participating_agents".to_string(),
            serde_json::Value::Array(
                decision.secondary_agents.iter()
                    .map(|id| serde_json::Value::String(id.clone()))
                    .collect()
            )
        );

        let response = self.execute_with_timeout(facilitator, collaborative_query).await?;

        Ok(DispatchResult {
            query_id: query.id.clone(),
            primary_response: response,
            secondary_responses: Vec::new(),
            routing_decision: decision.clone(),
            execution_time: Duration::from_secs(0),
            status: DispatchStatus::Success,
            aggregated_confidence: decision.confidence_score,
            next_recommendations: Vec::new(),
        })
    }

    /// Execute an agent with timeout
    async fn execute_with_timeout(&self, agent: Arc<dyn Subagent>, query: SubagentQuery) -> Result<SubagentResponse, SubagentError> {
        timeout(self.config.default_timeout, agent.process_query(query)).await
            .map_err(|_| SubagentError::TimeoutError("Agent execution timeout".to_string()))?
            .map_err(|e| e)
    }

    /// Extract keywords from a response for context enhancement
    fn extract_keywords_from_response(&self, response: &SubagentResponse) -> Vec<String> {
        response.response_text
            .split_whitespace()
            .filter(|word| word.len() > 4)
            .take(10)
            .map(|word| word.to_lowercase())
            .collect()
    }

    /// Calculate aggregated confidence from multiple responses
    fn calculate_aggregated_confidence(&self, primary: &SubagentResponse, secondary: &[SubagentResponse]) -> f64 {
        if secondary.is_empty() {
            return primary.confidence_score;
        }

        let total_confidence: f64 = secondary.iter()
            .map(|r| r.confidence_score)
            .sum::<f64>() + primary.confidence_score;

        total_confidence / (secondary.len() as f64 + 1.0)
    }

    /// Aggregate recommendations from multiple responses
    fn aggregate_recommendations(&self, primary: &SubagentResponse, secondary: &[SubagentResponse]) -> Vec<super::SubagentRecommendation> {
        let mut all_recommendations = primary.recommendations.clone();
        
        for response in secondary {
            all_recommendations.extend(response.recommendations.clone());
        }

        // Remove duplicates and sort by priority
        all_recommendations.sort_by(|a, b| {
            b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        all_recommendations.into_iter().take(5).collect() // Limit to top 5
    }

    /// Build an error response when agent execution fails
    fn build_error_response(&self, query: &SubagentQuery, error: &SubagentError) -> SubagentResponse {
        SubagentResponse {
            query_id: query.id.clone(),
            subagent_id: "system".to_string(),
            response_text: format!("Error processing query: {}", error),
            confidence_score: 0.0,
            recommendations: Vec::new(),
            next_actions: Vec::new(),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get statistics about active queries
    pub async fn get_active_queries_count(&self) -> usize {
        self.active_queries.read().await.len()
    }

    /// Get detailed information about active queries
    pub async fn get_active_queries_info(&self) -> Vec<(String, Duration)> {
        let active_queries = self.active_queries.read().await;
        active_queries
            .iter()
            .map(|(id, execution)| (id.clone(), execution.started_at.elapsed()))
            .collect()
    }
}