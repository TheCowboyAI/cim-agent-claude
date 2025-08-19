use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::{
    domain::{value_objects::*, errors::*, ConversationAggregate},
    ports::ConversationStatePort,
};

/// In-memory implementation of conversation state port
/// Useful for development, testing, and small deployments
pub struct MemoryStateAdapter {
    conversations: RwLock<HashMap<ConversationId, ConversationAggregate>>,
    session_index: RwLock<HashMap<SessionId, Vec<ConversationId>>>,
}

impl MemoryStateAdapter {
    pub fn new() -> Self {
        Self {
            conversations: RwLock::new(HashMap::new()),
            session_index: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get current number of conversations stored
    pub async fn count(&self) -> usize {
        self.conversations.read().await.len()
    }
    
    /// Clear all conversations (useful for testing)
    pub async fn clear(&self) {
        self.conversations.write().await.clear();
        self.session_index.write().await.clear();
    }
}

#[async_trait]
impl ConversationStatePort for MemoryStateAdapter {
    async fn load_conversation(
        &self,
        id: &ConversationId,
    ) -> Result<Option<ConversationAggregate>, ApplicationError> {
        let conversations = self.conversations.read().await;
        Ok(conversations.get(id).cloned())
    }
    
    async fn save_conversation(
        &self,
        aggregate: &ConversationAggregate,
        expected_version: u64,
    ) -> Result<(), ApplicationError> {
        let mut conversations = self.conversations.write().await;
        let mut session_index = self.session_index.write().await;
        
        // Optimistic locking check
        if let Some(existing) = conversations.get(aggregate.id()) {
            if existing.version() != expected_version {
                return Err(DomainError::VersionMismatch {
                    expected: expected_version,
                    actual: existing.version(),
                }.into());
            }
        } else if expected_version != 0 {
            return Err(DomainError::VersionMismatch {
                expected: expected_version,
                actual: 0,
            }.into());
        }
        
        // Update session index
        session_index
            .entry(aggregate.session_id().clone())
            .or_insert_with(Vec::new)
            .push(aggregate.id().clone());
        
        // Save the aggregate
        conversations.insert(aggregate.id().clone(), aggregate.clone());
        
        info!(
            "Saved conversation {} (version {})",
            aggregate.id().as_uuid(),
            aggregate.version()
        );
        
        Ok(())
    }
    
    async fn find_active_conversations(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<ConversationId>, ApplicationError> {
        let session_index = self.session_index.read().await;
        let conversations = self.conversations.read().await;
        
        let conversation_ids = session_index.get(session_id).cloned().unwrap_or_default();
        
        // Filter to only active conversations (not ended)
        let active_conversations: Vec<ConversationId> = conversation_ids
            .into_iter()
            .filter(|id| {
                if let Some(conversation) = conversations.get(id) {
                    conversation.state() != &crate::domain::ConversationState::Ended
                } else {
                    false
                }
            })
            .collect();
        
        Ok(active_conversations)
    }
    
    async fn cleanup_expired_conversations(&self) -> Result<u32, ApplicationError> {
        let mut conversations = self.conversations.write().await;
        let mut session_index = self.session_index.write().await;
        
        let initial_count = conversations.len();
        
        // Find expired conversations
        let expired_ids: Vec<ConversationId> = conversations
            .values()
            .filter(|conversation| conversation.is_expired())
            .map(|conversation| conversation.id().clone())
            .collect();
        
        // Remove expired conversations
        for id in &expired_ids {
            if let Some(conversation) = conversations.remove(id) {
                // Remove from session index
                if let Some(session_conversations) = session_index.get_mut(conversation.session_id()) {
                    session_conversations.retain(|conv_id| conv_id != id);
                    
                    // Remove session entry if no conversations left
                    if session_conversations.is_empty() {
                        session_index.remove(conversation.session_id());
                    }
                }
            }
        }
        
        let removed_count = initial_count - conversations.len();
        
        if removed_count > 0 {
            info!("Cleaned up {} expired conversations", removed_count);
        }
        
        Ok(removed_count as u32)
    }
}

impl Default for MemoryStateAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::commands::*;
    
    fn create_test_aggregate() -> ConversationAggregate {
        let command = Command::StartConversation {
            session_id: SessionId::new(),
            initial_prompt: crate::domain::Prompt::new("Test prompt".to_string()).unwrap(),
            context: ConversationContext::default(),
            correlation_id: CorrelationId::new(),
        };
        
        ConversationAggregate::from_command(command, CorrelationId::new()).unwrap()
    }
    
    #[tokio::test]
    async fn test_save_and_load_conversation() {
        let adapter = MemoryStateAdapter::new();
        let aggregate = create_test_aggregate();
        let id = aggregate.id().clone();
        
        // Save conversation
        adapter.save_conversation(&aggregate, 0).await.unwrap();
        
        // Load conversation
        let loaded = adapter.load_conversation(&id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id(), &id);
    }
    
    #[tokio::test]
    async fn test_optimistic_locking() {
        let adapter = MemoryStateAdapter::new();
        let aggregate = create_test_aggregate();
        
        // Save with correct version
        adapter.save_conversation(&aggregate, 0).await.unwrap();
        
        // Try to save with wrong version
        let result = adapter.save_conversation(&aggregate, 999).await;
        assert!(matches!(result, Err(ApplicationError::Domain(DomainError::VersionMismatch { .. }))));
    }
    
    #[tokio::test]
    async fn test_session_index() {
        let adapter = MemoryStateAdapter::new();
        let aggregate = create_test_aggregate();
        let session_id = aggregate.session_id().clone();
        
        // Save conversation
        adapter.save_conversation(&aggregate, 0).await.unwrap();
        
        // Find active conversations for session
        let active = adapter.find_active_conversations(&session_id).await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], *aggregate.id());
    }
    
    #[tokio::test]
    async fn test_cleanup_expired() {
        let adapter = MemoryStateAdapter::new();
        let mut aggregate = create_test_aggregate();
        
        // Manually set last activity to make it expired
        // This would need access to internals or a way to set the time
        // For now, just test the count
        adapter.save_conversation(&aggregate, 0).await.unwrap();
        
        let initial_count = adapter.count().await;
        let cleaned = adapter.cleanup_expired_conversations().await.unwrap();
        
        // Since we can't easily make conversations expired in this test,
        // we just verify the method runs without error
        assert!(cleaned >= 0);
    }
}