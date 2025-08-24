//! Dialog Management System
//!
//! Manages conversation context, history, and state preservation using NATS

use crate::error::LlmAdapterError;
use async_nats::{Client, jetstream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Dialog context for maintaining conversation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogContext {
    pub session_id: String,
    pub user_id: Option<String>,
    pub conversation_history: Vec<ConversationEntry>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DialogContext {
    /// Create new dialog context
    pub fn new(session_id: String) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            user_id: None,
            conversation_history: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Add message to conversation history
    pub fn add_message(&mut self, role: String, content: String, provider: Option<String>) {
        let entry = ConversationEntry {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            provider,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        
        self.conversation_history.push(entry);
        self.updated_at = Utc::now();
    }
    
    /// Get recent messages limited by count
    pub fn recent_messages(&self, limit: usize) -> Vec<&ConversationEntry> {
        let start = if self.conversation_history.len() > limit {
            self.conversation_history.len() - limit
        } else {
            0
        };
        
        self.conversation_history[start..].iter().collect()
    }
    
    /// Get messages for LLM provider format
    pub fn to_provider_messages(&self, include_system: bool) -> Vec<crate::providers::Message> {
        self.conversation_history
            .iter()
            .filter(|entry| include_system || entry.role != "system")
            .map(|entry| crate::providers::Message {
                role: entry.role.clone(),
                content: entry.content.clone(),
                metadata: Some(entry.metadata.clone()),
            })
            .collect()
    }
}

/// Individual conversation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub id: String,
    pub role: String, // "user", "assistant", "system"
    pub content: String,
    pub provider: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Dialog Manager for handling conversation state
pub struct DialogManager {
    nats_client: Client,
    jetstream: jetstream::Context,
    domain: Option<String>,
}

impl DialogManager {
    /// Create new dialog manager
    pub async fn new(
        nats_client: Client,
        jetstream: jetstream::Context,
        domain: Option<String>,
    ) -> Result<Self, LlmAdapterError> {
        let manager = Self {
            nats_client,
            jetstream,
            domain,
        };
        
        // Initialize KV store for dialog contexts
        manager.initialize_kv_store().await?;
        
        info!("💬 Dialog Manager initialized");
        
        Ok(manager)
    }
    
    /// Initialize KV store for dialog contexts
    async fn initialize_kv_store(&self) -> Result<(), LlmAdapterError> {
        let bucket_name = if let Some(ref domain) = self.domain {
            format!("LLM_{}_DIALOGS", domain.to_uppercase().replace("-", "_"))
        } else {
            "LLM_DIALOGS".to_string()
        };
        
        info!("🗃️  Creating KV bucket: {}", bucket_name);
        
        let _kv_store = self.jetstream.create_key_value(jetstream::kv::Config {
            bucket: bucket_name,
            description: "LLM Dialog contexts and conversation history".to_string(),
            history: 10, // Keep 10 versions of each context
            ..Default::default()
        }).await.map_err(|e| LlmAdapterError::DialogManagement(
            format!("Failed to create KV store: {}", e)
        ))?;
        
        info!("✅ Dialog KV store initialized");
        Ok(())
    }
    
    /// Get or create dialog context
    pub async fn get_or_create_context(&self, session_id: &str) -> Result<DialogContext, LlmAdapterError> {
        let bucket_name = if let Some(ref domain) = self.domain {
            format!("LLM_{}_DIALOGS", domain.to_uppercase().replace("-", "_"))
        } else {
            "LLM_DIALOGS".to_string()
        };
        
        let kv_store = self.jetstream.get_key_value(&bucket_name).await
            .map_err(|e| LlmAdapterError::DialogManagement(
                format!("Failed to get KV store: {}", e)
            ))?;
        
        // Try to get existing context
        match kv_store.get(session_id).await {
            Ok(Some(entry)) => {
                // Deserialize existing context
                let context: DialogContext = serde_json::from_slice(&entry)
                    .map_err(|e| LlmAdapterError::Deserialization(e.to_string()))?;
                
                info!("📖 Retrieved dialog context for session: {}", session_id);
                Ok(context)
            }
            Ok(None) | Err(_) => {
                // Create new context
                let context = DialogContext::new(session_id.to_string());
                
                // Store it
                self.save_context(&context).await?;
                
                info!("📝 Created new dialog context for session: {}", session_id);
                Ok(context)
            }
        }
    }
    
    /// Save dialog context
    pub async fn save_context(&self, context: &DialogContext) -> Result<(), LlmAdapterError> {
        let bucket_name = if let Some(ref domain) = self.domain {
            format!("LLM_{}_DIALOGS", domain.to_uppercase().replace("-", "_"))
        } else {
            "LLM_DIALOGS".to_string()
        };
        
        let kv_store = self.jetstream.get_key_value(&bucket_name).await
            .map_err(|e| LlmAdapterError::DialogManagement(
                format!("Failed to get KV store: {}", e)
            ))?;
        
        let context_json = serde_json::to_vec(context)
            .map_err(|e| LlmAdapterError::Serialization(e.to_string()))?;
        
        kv_store.put(&context.session_id, context_json.into()).await
            .map_err(|e| LlmAdapterError::DialogManagement(
                format!("Failed to save context: {}", e)
            ))?;
        
        info!("💾 Saved dialog context for session: {}", context.session_id);
        Ok(())
    }
    
    /// Record dialog event in event store
    pub async fn record_dialog_event(
        &self,
        event: DialogEvent,
    ) -> Result<(), LlmAdapterError> {
        let event_subject = if let Some(ref domain) = self.domain {
            format!("{}.events.llm.dialog", domain)
        } else {
            "events.llm.dialog".to_string()
        };
        
        let event_json = serde_json::to_vec(&event)
            .map_err(|e| LlmAdapterError::Serialization(e.to_string()))?;
        
        self.nats_client.publish(event_subject, event_json.into()).await
            .map_err(|e| LlmAdapterError::NatsConnection(e.to_string()))?;
        
        info!("📝 Recorded dialog event: {}", event.event_type);
        Ok(())
    }
    
    /// Clean up old contexts (maintenance function)
    pub async fn cleanup_old_contexts(&self, older_than_days: u64) -> Result<usize, LlmAdapterError> {
        // Implementation for cleaning up old dialog contexts
        // This would scan the KV store and remove contexts older than the specified days
        info!("🧹 Dialog cleanup not implemented yet");
        Ok(0)
    }
}

/// Dialog event types for event sourcing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogEvent {
    pub event_id: String,
    pub event_type: String, // "dialog_started", "message_sent", "context_updated"
    pub session_id: String,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, serde_json::Value>,
}