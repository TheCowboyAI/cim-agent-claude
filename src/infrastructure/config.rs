/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Configuration
//!
//! Centralized configuration for the entire CIM system.
//! Follows the principle that configuration is part of the composition.

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub nats: NatsConfig,
    pub claude: claude::ClaudeConfig,
    pub gui: gui::GuiConfig,
    pub expert: expert::ExpertConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    pub url: String,
    pub subject_prefix: String,
    pub jetstream: JetstreamConfig,
    pub websocket: WebSocketConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetstreamConfig {
    pub enabled: bool,
    pub store_dir: String,
    pub max_memory: String,
    pub max_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub enabled: bool,
    pub port: u16,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub log_level: String,
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub tracing_enabled: bool,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config {
            nats: NatsConfig {
                url: env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string()),
                subject_prefix: env::var("NATS_SUBJECT_PREFIX").unwrap_or_else(|_| "cim.claude".to_string()),
                jetstream: JetstreamConfig {
                    enabled: env::var("JETSTREAM_ENABLED").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
                    store_dir: env::var("JETSTREAM_STORE_DIR").unwrap_or_else(|_| "/tmp/jetstream".to_string()),
                    max_memory: env::var("JETSTREAM_MAX_MEMORY").unwrap_or_else(|_| "1GB".to_string()),
                    max_file: env::var("JETSTREAM_MAX_FILE").unwrap_or_else(|_| "10GB".to_string()),
                },
                websocket: WebSocketConfig {
                    enabled: env::var("NATS_WEBSOCKET_ENABLED").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
                    port: env::var("NATS_WEBSOCKET_PORT").unwrap_or_else(|_| "8222".to_string()).parse().unwrap_or(8222),
                    allowed_origins: env::var("NATS_WEBSOCKET_ORIGINS")
                        .unwrap_or_else(|_| "http://localhost:8081".to_string())
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect(),
                },
            },
            claude: crate::infrastructure::claude::ClaudeConfig::from_env()?,
            gui: crate::infrastructure::gui::GuiConfig::from_env()?,
            expert: crate::infrastructure::expert::ExpertConfig::from_env()?,
            observability: ObservabilityConfig {
                log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string()),
                metrics_enabled: env::var("METRICS_ENABLED").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
                metrics_port: env::var("METRICS_PORT").unwrap_or_else(|_| "9090".to_string()).parse().unwrap_or(9090),
                tracing_enabled: env::var("TRACING_ENABLED").unwrap_or_else(|_| "false".to_string()).parse().unwrap_or(false),
            },
        };
        
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate NATS URL
        if self.nats.url.is_empty() {
            return Err("NATS URL cannot be empty".into());
        }
        
        // Validate Claude configuration
        self.claude.validate()?;
        
        // Validate GUI configuration
        self.gui.validate()?;
        
        // Validate expert configuration
        self.expert.validate()?;
        
        // Validate observability configuration
        if !["TRACE", "DEBUG", "INFO", "WARN", "ERROR"].contains(&self.observability.log_level.as_str()) {
            return Err("Invalid log level. Must be one of: TRACE, DEBUG, INFO, WARN, ERROR".into());
        }
        
        Ok(())
    }
}