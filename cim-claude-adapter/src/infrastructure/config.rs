use serde::{Deserialize, Serialize};
use std::env;

/// Configuration for the Claude adapter service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub nats: NatsConfig,
    pub claude: ClaudeConfig,
    pub server: ServerConfig,
    pub observability: ObservabilityConfig,
}

/// NATS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    pub url: String,
    pub credentials_file: Option<String>,
    pub connection_timeout_ms: u64,
    pub reconnect_attempts: u32,
}

/// Claude API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub rate_limit_per_minute: u32,
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub health_check_interval_seconds: u64,
    pub cleanup_interval_seconds: u64,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout_duration_seconds: u64,
    pub half_open_max_calls: u32,
    pub half_open_success_threshold: u32,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub log_level: String,
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub tracing_enabled: bool,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            nats: NatsConfig::default(),
            claude: ClaudeConfig::default(),
            server: ServerConfig::default(),
            observability: ObservabilityConfig::default(),
        }
    }
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".to_string(),
            credentials_file: None,
            connection_timeout_ms: 5000,
            reconnect_attempts: 10,
        }
    }
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            api_key: "".to_string(), // Must be provided via env var
            base_url: "https://api.anthropic.com".to_string(),
            timeout_seconds: 30,
            rate_limit_per_minute: 50,
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            health_check_interval_seconds: 30,
            cleanup_interval_seconds: 3600, // 1 hour
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout_duration_seconds: 60,
            half_open_max_calls: 3,
            half_open_success_threshold: 2,
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            metrics_enabled: true,
            metrics_port: 9090,
            tracing_enabled: true,
        }
    }
}

impl AdapterConfig {
    /// Load configuration from environment variables and defaults
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        
        // NATS configuration
        if let Ok(url) = env::var("NATS_URL") {
            config.nats.url = url;
        }
        if let Ok(creds) = env::var("NATS_CREDENTIALS_FILE") {
            config.nats.credentials_file = Some(creds);
        }
        
        // Claude configuration
        config.claude.api_key = env::var("CLAUDE_API_KEY")
            .map_err(|_| ConfigError::MissingRequired("CLAUDE_API_KEY".to_string()))?;
        
        if let Ok(url) = env::var("CLAUDE_BASE_URL") {
            config.claude.base_url = url;
        }
        
        if let Ok(timeout) = env::var("CLAUDE_TIMEOUT_SECONDS") {
            config.claude.timeout_seconds = timeout.parse()
                .map_err(|_| ConfigError::InvalidValue("CLAUDE_TIMEOUT_SECONDS".to_string()))?;
        }
        
        // Server configuration
        if let Ok(host) = env::var("SERVER_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            config.server.port = port.parse()
                .map_err(|_| ConfigError::InvalidValue("SERVER_PORT".to_string()))?;
        }
        
        // Observability configuration
        if let Ok(level) = env::var("LOG_LEVEL") {
            config.observability.log_level = level;
        }
        if let Ok(enabled) = env::var("METRICS_ENABLED") {
            config.observability.metrics_enabled = enabled.parse()
                .map_err(|_| ConfigError::InvalidValue("METRICS_ENABLED".to_string()))?;
        }
        
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.claude.api_key.is_empty() {
            return Err(ConfigError::MissingRequired("Claude API key".to_string()));
        }
        
        if self.nats.url.is_empty() {
            return Err(ConfigError::MissingRequired("NATS URL".to_string()));
        }
        
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue("Server port cannot be 0".to_string()));
        }
        
        Ok(())
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),
    
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    
    #[error("Configuration file error: {0}")]
    FileError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_default_config() {
        let config = AdapterConfig::default();
        assert_eq!(config.nats.url, "nats://localhost:4222");
        assert_eq!(config.claude.base_url, "https://api.anthropic.com");
        assert_eq!(config.server.port, 8080);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = AdapterConfig::default();
        
        // Should fail without API key
        assert!(config.validate().is_err());
        
        // Should pass with API key
        config.claude.api_key = "test-key".to_string();
        assert!(config.validate().is_ok());
        
        // Should fail with invalid port
        config.server.port = 0;
        assert!(config.validate().is_err());
    }
    
    #[tokio::test]
    async fn test_env_config_loading() {
        // Set test environment variables
        env::set_var("CLAUDE_API_KEY", "test-api-key");
        env::set_var("NATS_URL", "nats://test:4222");
        env::set_var("SERVER_PORT", "9999");
        
        let config = AdapterConfig::from_env().unwrap();
        assert_eq!(config.claude.api_key, "test-api-key");
        assert_eq!(config.nats.url, "nats://test:4222");
        assert_eq!(config.server.port, 9999);
        
        // Clean up
        env::remove_var("CLAUDE_API_KEY");
        env::remove_var("NATS_URL");
        env::remove_var("SERVER_PORT");
    }
}