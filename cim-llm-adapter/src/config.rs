//! Configuration system for LLM Adapter
//!
//! Manages provider configurations and service settings

use crate::providers::ProviderConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration for LLM Adapter service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAdapterConfig {
    pub service: ServiceConfig,
    pub providers: HashMap<String, ProviderConfig>,
    pub default_provider: String,
    pub dialog_settings: DialogSettings,
}

/// Service-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub domain: Option<String>,
    pub nats_url: String,
    pub health_check_interval_seconds: u64,
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
}

/// Dialog management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogSettings {
    pub max_history_length: usize,
    pub context_retention_days: u64,
    pub auto_cleanup: bool,
    pub system_prompt_template: Option<String>,
}

impl Default for LlmAdapterConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        
        // Default Claude provider configuration
        providers.insert("claude".to_string(), ProviderConfig {
            name: "claude".to_string(),
            base_url: Some("https://api.anthropic.com".to_string()),
            api_key: None, // Should be set via environment variable
            model: "claude-3-5-sonnet-20241022".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            rate_limit_per_minute: Some(60),
            custom_headers: HashMap::new(),
        });
        
        // Default OpenAI provider configuration
        providers.insert("openai".to_string(), ProviderConfig {
            name: "openai".to_string(),
            base_url: Some("https://api.openai.com".to_string()),
            api_key: None, // Should be set via environment variable
            model: "gpt-4-turbo-preview".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            rate_limit_per_minute: Some(60),
            custom_headers: HashMap::new(),
        });
        
        // Default Ollama provider configuration
        providers.insert("ollama".to_string(), ProviderConfig {
            name: "ollama".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
            api_key: None, // Not needed for Ollama
            model: "mistral:latest".to_string(),
            timeout_seconds: 60,
            retry_attempts: 2,
            rate_limit_per_minute: None, // No rate limit for local
            custom_headers: HashMap::new(),
        });
        
        Self {
            service: ServiceConfig {
                name: "llm-adapter".to_string(),
                domain: None,
                nats_url: "nats://localhost:4222".to_string(),
                health_check_interval_seconds: 60,
                max_concurrent_requests: 100,
                request_timeout_seconds: 45,
            },
            providers,
            default_provider: "claude".to_string(),
            dialog_settings: DialogSettings {
                max_history_length: 100,
                context_retention_days: 30,
                auto_cleanup: true,
                system_prompt_template: None,
            },
        }
    }
}

impl LlmAdapterConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Service configuration from environment
        if let Ok(domain) = std::env::var("CIM_DOMAIN") {
            config.service.domain = Some(domain);
        }
        
        if let Ok(nats_url) = std::env::var("NATS_URL") {
            config.service.nats_url = nats_url;
        }
        
        // Provider configurations from environment
        if let Ok(claude_api_key) = std::env::var("ANTHROPIC_API_KEY") {
            if let Some(claude_config) = config.providers.get_mut("claude") {
                claude_config.api_key = Some(claude_api_key);
            }
        }
        
        if let Ok(openai_api_key) = std::env::var("OPENAI_API_KEY") {
            config.providers.insert("openai".to_string(), ProviderConfig {
                name: "openai".to_string(),
                base_url: Some("https://api.openai.com".to_string()),
                api_key: Some(openai_api_key),
                model: "gpt-4".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
                rate_limit_per_minute: Some(60),
                custom_headers: HashMap::new(),
            });
        }
        
        config
    }
    
    /// Get available provider names
    pub fn available_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
    
    /// Get provider configuration by name
    pub fn get_provider_config(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check if default provider exists
        if !self.providers.contains_key(&self.default_provider) {
            return Err(format!(
                "Default provider '{}' not found in providers list", 
                self.default_provider
            ));
        }
        
        // Check if at least one provider has an API key
        let has_configured_provider = self.providers.values()
            .any(|config| config.api_key.is_some());
        
        if !has_configured_provider {
            return Err("No provider has an API key configured".to_string());
        }
        
        Ok(())
    }
}