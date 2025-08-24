//! SAGE Integration Module
//!
//! Provides integration between LLM Adapter and SAGE orchestrator
//! Allows SAGE to use LLM providers through the adapter layer

use crate::{LlmRequest, LlmResponse, LlmAdapterError, dialog::DialogContext};
use async_nats::Client;
use futures_util::stream::StreamExt;
use serde_json;
use std::time::Duration;
use tracing::{info, error, debug};
use uuid::Uuid;

/// SAGE integration client for LLM requests
pub struct SageLlmClient {
    nats_client: Client,
    domain: Option<String>,
    timeout: Duration,
}

impl SageLlmClient {
    /// Create new SAGE LLM client
    pub fn new(nats_client: Client, domain: Option<String>) -> Self {
        Self {
            nats_client,
            domain,
            timeout: Duration::from_secs(30),
        }
    }
    
    /// Send LLM request via NATS to LLM Adapter
    pub async fn request_completion(
        &self,
        provider: &str,
        messages: Vec<serde_json::Value>,
        session_id: Option<String>,
        options: Option<serde_json::Value>,
    ) -> Result<String, LlmAdapterError> {
        let request_id = Uuid::new_v4().to_string();
        let session_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Create dialog context
        let context = DialogContext::new(session_id.clone());
        
        // Build LLM request
        let request = LlmRequest {
            request_id: request_id.clone(),
            provider: provider.to_string(),
            messages,
            context,
            options,
        };
        
        debug!("🔄 SAGE requesting LLM completion: {} via {}", request_id, provider);
        
        // Serialize request
        let request_json = serde_json::to_vec(&request)
            .map_err(|e| LlmAdapterError::Serialization(e.to_string()))?;
        
        // Set up response subscription before sending request
        let response_subject = self.build_response_subject(&request_id);
        let response_subscriber = self.nats_client.subscribe(response_subject.clone()).await
            .map_err(|e| LlmAdapterError::NatsConnection(e.to_string()))?;
        
        // Send request
        let request_subject = self.build_request_subject();
        self.nats_client.publish(request_subject, request_json.into()).await
            .map_err(|e| LlmAdapterError::NatsConnection(e.to_string()))?;
        
        info!("📤 SAGE sent LLM request: {} to {}", request_id, provider);
        
        // Wait for response with timeout
        let response = tokio::time::timeout(
            self.timeout,
            self.wait_for_response(response_subscriber, request_id.clone())
        ).await;
        
        match response {
            Ok(Ok(llm_response)) => {
                info!("📥 SAGE received LLM response: {}", request_id);
                Ok(llm_response.response)
            }
            Ok(Err(e)) => {
                error!("❌ SAGE LLM request failed: {}", e);
                Err(e)
            }
            Err(_) => {
                error!("⏰ SAGE LLM request timeout: {}", request_id);
                Err(LlmAdapterError::Provider(
                    format!("Request timeout after {}s", self.timeout.as_secs())
                ))
            }
        }
    }
    
    /// Wait for response from LLM Adapter
    async fn wait_for_response(
        &self,
        mut subscriber: async_nats::Subscriber,
        request_id: String,
    ) -> Result<LlmResponse, LlmAdapterError> {
        while let Some(msg) = subscriber.next().await {
            // Deserialize response
            match serde_json::from_slice::<LlmResponse>(&msg.payload) {
                Ok(response) => {
                    if response.request_id == request_id {
                        return Ok(response);
                    }
                }
                Err(e) => {
                    error!("Failed to deserialize LLM response: {}", e);
                }
            }
        }
        
        Err(LlmAdapterError::Provider("No response received".to_string()))
    }
    
    /// Build request subject using cim-subject patterns
    fn build_request_subject(&self) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.commands.llm.request", domain)
        } else {
            "commands.llm.request".to_string()
        }
    }
    
    /// Build response subject for specific request
    fn build_response_subject(&self, request_id: &str) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.events.llm.response.{}", domain, request_id)
        } else {
            format!("events.llm.response.{}", request_id)
        }
    }
    
    /// Quick Claude completion helper for SAGE
    pub async fn claude_completion(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        session_id: Option<String>,
    ) -> Result<String, LlmAdapterError> {
        let mut messages = Vec::new();
        
        if let Some(system) = system_prompt {
            messages.push(serde_json::json!({
                "role": "system",
                "content": system
            }));
        }
        
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));
        
        let options = Some(serde_json::json!({
            "max_tokens": 4096,
            "temperature": 0.7,
            "system_prompt": system_prompt
        }));
        
        self.request_completion("claude", messages, session_id, options).await
    }
    
    /// Get available providers from LLM Adapter
    pub async fn get_available_providers(&self) -> Result<Vec<String>, LlmAdapterError> {
        // This would query the LLM adapter for available providers
        // For now, return default providers
        Ok(vec!["claude".to_string()])
    }
    
    /// Health check for LLM Adapter service
    pub async fn health_check(&self) -> Result<bool, LlmAdapterError> {
        // This would send a health check request to LLM adapter
        // For now, assume it's healthy if we can connect to NATS
        Ok(true)
    }
}