/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Configuration Domain
//! Handles system prompt updates, model parameter changes, and conversation settings.
//! Everything is event-sourced - all configuration changes are Commands and Events.

use crate::domain::value_objects::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Configuration aggregate ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigurationId(Uuid);

impl ConfigurationId {
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

impl Default for ConfigurationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ConfigurationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// System prompt configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemPrompt {
    pub content: String,
    pub version: u32,
    pub metadata: HashMap<String, String>,
}

impl SystemPrompt {
    pub fn new(content: String) -> Result<Self, String> {
        if content.trim().is_empty() {
            return Err("System prompt cannot be empty".to_string());
        }
        
        if content.len() > 100_000 {
            return Err("System prompt too long (max 100k characters)".to_string());
        }
        
        Ok(Self {
            content: content.trim().to_string(),
            version: 1,
            metadata: HashMap::new(),
        })
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn update_content(&mut self, new_content: String) -> Result<(), String> {
        if new_content.trim().is_empty() {
            return Err("System prompt cannot be empty".to_string());
        }
        
        if new_content.len() > 100_000 {
            return Err("System prompt too long (max 100k characters)".to_string());
        }
        
        self.content = new_content.trim().to_string();
        self.version += 1;
        Ok(())
    }
}

/// Model parameters for Claude API
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelParameters {
    pub model_name: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub stop_sequences: Vec<String>,
    pub stream: bool,
}

impl ModelParameters {
    pub fn new(model_name: String) -> Result<Self, String> {
        if model_name.trim().is_empty() {
            return Err("Model name cannot be empty".to_string());
        }
        
        Ok(Self {
            model_name: model_name.trim().to_string(),
            temperature: 0.7,
            max_tokens: 4000,
            top_p: None,
            top_k: None,
            stop_sequences: Vec::new(),
            stream: false,
        })
    }
    
    pub fn with_temperature(mut self, temperature: f64) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&temperature) {
            return Err("Temperature must be between 0.0 and 1.0".to_string());
        }
        self.temperature = temperature;
        Ok(self)
    }
    
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Result<Self, String> {
        if max_tokens == 0 || max_tokens > 200_000 {
            return Err("Max tokens must be between 1 and 200,000".to_string());
        }
        self.max_tokens = max_tokens;
        Ok(self)
    }
    
    pub fn with_top_p(mut self, top_p: f64) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&top_p) {
            return Err("Top-p must be between 0.0 and 1.0".to_string());
        }
        self.top_p = Some(top_p);
        Ok(self)
    }
    
    pub fn with_streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            model_name: "claude-3-sonnet-20240229".to_string(),
            temperature: 0.7,
            max_tokens: 4000,
            top_p: None,
            top_k: None,
            stop_sequences: Vec::new(),
            stream: false,
        }
    }
}

/// Conversation-specific settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationSettings {
    pub max_exchanges: u32,
    pub context_window_tokens: u32,
    pub enable_attachments: bool,
    pub enable_tools: bool,
    pub auto_save_interval: Option<chrono::Duration>,
    pub tags: Vec<String>,
}

impl ConversationSettings {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_max_exchanges(mut self, max_exchanges: u32) -> Self {
        self.max_exchanges = max_exchanges;
        self
    }
    
    pub fn with_attachments(mut self, enable: bool) -> Self {
        self.enable_attachments = enable;
        self
    }
    
    pub fn with_tools(mut self, enable: bool) -> Self {
        self.enable_tools = enable;
        self
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }
}

impl Default for ConversationSettings {
    fn default() -> Self {
        Self {
            max_exchanges: 100,
            context_window_tokens: 150_000,
            enable_attachments: true,
            enable_tools: true,
            auto_save_interval: Some(chrono::Duration::minutes(5)),
            tags: Vec::new(),
        }
    }
}

/// Configuration commands (all configuration changes start as commands)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigurationCommand {
    /// Update system prompt
    UpdateSystemPrompt {
        configuration_id: ConfigurationId,
        new_prompt: SystemPrompt,
        reason: String,
        correlation_id: CorrelationId,
    },
    
    /// Update model parameters
    UpdateModelParameters {
        configuration_id: ConfigurationId,
        new_parameters: ModelParameters,
        reason: String,
        correlation_id: CorrelationId,
    },
    
    /// Update conversation settings
    UpdateConversationSettings {
        configuration_id: ConfigurationId,
        new_settings: ConversationSettings,
        reason: String,
        correlation_id: CorrelationId,
    },
    
    /// Reset configuration to defaults
    ResetConfiguration {
        configuration_id: ConfigurationId,
        reset_scope: ResetScope,
        correlation_id: CorrelationId,
    },
    
    /// Import configuration from template or file
    ImportConfiguration {
        configuration_id: ConfigurationId,
        import_source: ImportSource,
        merge_strategy: MergeStrategy,
        correlation_id: CorrelationId,
    },
    
    /// Export current configuration
    ExportConfiguration {
        configuration_id: ConfigurationId,
        export_format: ConfigurationExportFormat,
        export_scope: ExportScope,
        correlation_id: CorrelationId,
    },
    
    /// Create configuration backup
    CreateBackup {
        configuration_id: ConfigurationId,
        backup_name: String,
        correlation_id: CorrelationId,
    },
    
    /// Restore from backup
    RestoreFromBackup {
        configuration_id: ConfigurationId,
        backup_name: String,
        correlation_id: CorrelationId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResetScope {
    SystemPrompt,
    ModelParameters,
    ConversationSettings,
    All,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportSource {
    Template(String),
    File(String),
    Url(String),
    Raw(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergeStrategy {
    Replace,
    Merge,
    Append,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigurationExportFormat {
    Json,
    Yaml,
    Toml,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportScope {
    SystemPrompt,
    ModelParameters,
    ConversationSettings,
    All,
}

/// Configuration events (result of processing commands)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigurationEvent {
    /// System prompt was updated
    SystemPromptUpdated {
        configuration_id: ConfigurationId,
        old_prompt: SystemPrompt,
        new_prompt: SystemPrompt,
        reason: String,
        updated_by: SessionId,
        updated_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Model parameters were updated
    ModelParametersUpdated {
        configuration_id: ConfigurationId,
        old_parameters: ModelParameters,
        new_parameters: ModelParameters,
        reason: String,
        updated_by: SessionId,
        updated_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Conversation settings were updated
    ConversationSettingsUpdated {
        configuration_id: ConfigurationId,
        old_settings: ConversationSettings,
        new_settings: ConversationSettings,
        reason: String,
        updated_by: SessionId,
        updated_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Configuration was reset
    ConfigurationReset {
        configuration_id: ConfigurationId,
        reset_scope: ResetScope,
        previous_state: ConfigurationSnapshot,
        reset_by: SessionId,
        reset_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Configuration was imported
    ConfigurationImported {
        configuration_id: ConfigurationId,
        import_source: ImportSource,
        merge_strategy: MergeStrategy,
        changes_summary: Vec<String>,
        imported_by: SessionId,
        imported_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Configuration was exported
    ConfigurationExported {
        configuration_id: ConfigurationId,
        export_format: ConfigurationExportFormat,
        export_scope: ExportScope,
        export_location: String,
        exported_by: SessionId,
        exported_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Configuration backup was created
    BackupCreated {
        configuration_id: ConfigurationId,
        backup_name: String,
        backup_size_bytes: u64,
        created_by: SessionId,
        created_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Configuration was restored from backup
    RestoredFromBackup {
        configuration_id: ConfigurationId,
        backup_name: String,
        previous_state: ConfigurationSnapshot,
        restored_by: SessionId,
        restored_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Configuration validation failed
    ConfigurationValidationFailed {
        configuration_id: ConfigurationId,
        validation_errors: Vec<String>,
        attempted_command: String,
        failed_at: chrono::DateTime<chrono::Utc>,
    },
}

/// Complete configuration state snapshot
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigurationSnapshot {
    pub system_prompt: SystemPrompt,
    pub model_parameters: ModelParameters,
    pub conversation_settings: ConversationSettings,
    pub snapshot_version: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ConfigurationSnapshot {
    pub fn new(
        system_prompt: SystemPrompt,
        model_parameters: ModelParameters,
        conversation_settings: ConversationSettings,
    ) -> Self {
        Self {
            system_prompt,
            model_parameters,
            conversation_settings,
            snapshot_version: 1,
            created_at: chrono::Utc::now(),
        }
    }
}

/// Configuration query types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigurationQuery {
    /// Get current configuration
    GetCurrentConfiguration {
        configuration_id: ConfigurationId,
    },
    
    /// Get configuration history
    GetConfigurationHistory {
        configuration_id: ConfigurationId,
        limit: Option<u32>,
        offset: Option<u32>,
    },
    
    /// Get default configuration
    GetDefaultConfiguration,
    
    /// Validate configuration
    ValidateConfiguration {
        configuration: ConfigurationSnapshot,
    },
    
    /// Get configuration schema
    GetConfigurationSchema,
    
    /// Get available templates
    GetAvailableTemplates,
    
    /// Get available backups
    GetAvailableBackups {
        configuration_id: ConfigurationId,
    },
}

/// Configuration aggregate
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigurationAggregate {
    pub id: ConfigurationId,
    pub current_state: ConfigurationSnapshot,
    pub version: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ConfigurationAggregate {
    pub fn new(id: ConfigurationId) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            current_state: ConfigurationSnapshot::new(
                SystemPrompt::new("You are Claude, a helpful AI assistant.".to_string()).unwrap(),
                ModelParameters::default(),
                ConversationSettings::default(),
            ),
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn apply_event(&mut self, event: &ConfigurationEvent) {
        match event {
            ConfigurationEvent::SystemPromptUpdated { new_prompt, updated_at, .. } => {
                self.current_state.system_prompt = new_prompt.clone();
                self.updated_at = *updated_at;
                self.version += 1;
            }
            ConfigurationEvent::ModelParametersUpdated { new_parameters, updated_at, .. } => {
                self.current_state.model_parameters = new_parameters.clone();
                self.updated_at = *updated_at;
                self.version += 1;
            }
            ConfigurationEvent::ConversationSettingsUpdated { new_settings, updated_at, .. } => {
                self.current_state.conversation_settings = new_settings.clone();
                self.updated_at = *updated_at;
                self.version += 1;
            }
            ConfigurationEvent::ConfigurationReset { reset_at, .. } => {
                // Reset to defaults
                self.current_state = ConfigurationSnapshot::new(
                    SystemPrompt::new("You are Claude, a helpful AI assistant.".to_string()).unwrap(),
                    ModelParameters::default(),
                    ConversationSettings::default(),
                );
                self.updated_at = *reset_at;
                self.version += 1;
            }
            ConfigurationEvent::RestoredFromBackup { restored_at, .. } => {
                // Restoration logic would be implemented here
                self.updated_at = *restored_at;
                self.version += 1;
            }
            _ => {
                // Other events don't change state directly
                self.version += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_validation() {
        assert!(SystemPrompt::new("".to_string()).is_err());
        assert!(SystemPrompt::new("  ".to_string()).is_err());
        assert!(SystemPrompt::new("Valid prompt".to_string()).is_ok());
        
        let long_prompt = "x".repeat(200_000);
        assert!(SystemPrompt::new(long_prompt).is_err());
    }

    #[test]
    fn test_model_parameters_validation() {
        let params = ModelParameters::new("claude-3-sonnet".to_string())
            .unwrap()
            .with_temperature(0.5)
            .unwrap()
            .with_max_tokens(8000)
            .unwrap();
            
        assert_eq!(params.temperature, 0.5);
        assert_eq!(params.max_tokens, 8000);
        
        // Invalid temperature
        assert!(ModelParameters::default().with_temperature(1.5).is_err());
        assert!(ModelParameters::default().with_max_tokens(0).is_err());
    }

    #[test]
    fn test_configuration_aggregate() {
        let id = ConfigurationId::new();
        let mut config = ConfigurationAggregate::new(id.clone());
        
        let new_prompt = SystemPrompt::new("New system prompt".to_string()).unwrap();
        let event = ConfigurationEvent::SystemPromptUpdated {
            configuration_id: id,
            old_prompt: config.current_state.system_prompt.clone(),
            new_prompt: new_prompt.clone(),
            reason: "Testing".to_string(),
            updated_by: SessionId::new(),
            updated_at: chrono::Utc::now(),
        };
        
        config.apply_event(&event);
        assert_eq!(config.current_state.system_prompt.content, "New system prompt");
        assert_eq!(config.version, 2);
    }
}