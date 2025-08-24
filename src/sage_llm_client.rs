//! SAGE LLM Client
//! 
//! Client for SAGE to communicate with the LLM Adapter service via NATS
//! This replaces direct Claude API calls with NATS-based messaging

use anyhow::Result;
use async_nats::Client;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::Duration;
use tracing::{info, error};
use uuid::Uuid;
use chrono::Utc;

/// LLM request structure matching the adapter's expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub request_id: String,
    pub provider: String,
    pub messages: Vec<serde_json::Value>,
    pub context: DialogContext,
    pub options: Option<serde_json::Value>,
}

/// Dialog context for maintaining conversation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogContext {
    pub session_id: String,
    pub user_id: Option<String>,
    pub conversation_history: Vec<ConversationEntry>,
    pub metadata: serde_json::Map<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Conversation entry for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub id: String,
    pub role: String,
    pub content: String,
    pub provider: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

/// LLM response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub request_id: String,
    pub response: String,
    pub provider_used: String,
    pub token_count: Option<usize>,
    pub updated_context: DialogContext,
}

/// Client for communicating with the LLM Adapter service
pub struct SageLlmClient {
    nats_client: Client,
    domain: Option<String>,
}

impl SageLlmClient {
    /// Create new LLM client
    pub fn new(nats_client: Client, domain: Option<String>) -> Self {
        Self {
            nats_client,
            domain,
        }
    }
    
    /// Send a request to the LLM adapter and wait for response
    pub async fn send_llm_request(
        &self,
        messages: Vec<serde_json::Value>,
        session_id: String,
        system_prompt: Option<&str>,
    ) -> Result<String> {
        let request_id = Uuid::new_v4().to_string();
        
        // Build messages including system prompt if provided
        let mut full_messages = Vec::new();
        if let Some(system) = system_prompt {
            full_messages.push(serde_json::json!({
                "role": "system",
                "content": system
            }));
        }
        full_messages.extend(messages);
        
        // Create dialog context
        let context = DialogContext {
            session_id: session_id.clone(),
            user_id: None,
            conversation_history: Vec::new(), // Will be managed by adapter
            metadata: serde_json::Map::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Create LLM request
        let request = LlmRequest {
            request_id: request_id.clone(),
            provider: "claude".to_string(),
            messages: full_messages,
            context,
            options: Some(serde_json::json!({
                "max_tokens": 4096,
                "temperature": 0.7,
            })),
        };
        
        // Build request subject - always use cim.llm prefix for adapter
        let request_subject = "cim.llm.commands.request".to_string();
        
        // Build response subject - always use cim.llm prefix for adapter
        let response_subject = format!("cim.llm.events.response.{}", request_id);
        
        info!("📤 Sending LLM request: {} to {}", request_id, request_subject);
        
        // Subscribe to response before sending request
        let mut response_sub = self.nats_client
            .subscribe(response_subject.clone())
            .await?;
        
        // Send request
        let request_bytes = serde_json::to_vec(&request)?;
        self.nats_client
            .publish(request_subject, request_bytes.into())
            .await?;
        
        // Wait for response with timeout
        let response_timeout = Duration::from_secs(30);
        let response_msg = tokio::time::timeout(
            response_timeout,
            response_sub.next()
        ).await?;
        
        // Parse response
        let response: LlmResponse = match response_msg {
            Some(msg) => serde_json::from_slice(&msg.payload)?,
            None => return Err(anyhow::anyhow!("No response received from LLM adapter")),
        };
        
        info!("📥 Received LLM response: {} (tokens: {:?})", 
            response.request_id, response.token_count);
        
        Ok(response.response)
    }
    
    /// Send a simple query without maintaining context
    pub async fn query(&self, query: &str) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": query
        })];
        
        self.send_llm_request(messages, session_id, None).await
    }
    
    /// Send a query with a specific agent personality
    pub async fn query_with_agent(
        &self,
        query: &str,
        agent_prompt: &str,
        session_id: Option<String>,
    ) -> Result<String> {
        let session_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": query
        })];
        
        self.send_llm_request(messages, session_id, Some(agent_prompt)).await
    }
}