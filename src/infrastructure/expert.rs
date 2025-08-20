/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Expert Configuration Module

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertConfig {
    pub enabled: bool,
    pub knowledge_base_path: String,
    pub max_context_length: usize,
    pub response_timeout_seconds: u64,
    pub cache_enabled: bool,
    pub cache_ttl_seconds: u64,
}

impl ExpertConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let config = ExpertConfig {
            enabled: env::var("EXPERT_ENABLED").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
            knowledge_base_path: env::var("EXPERT_KNOWLEDGE_BASE_PATH").unwrap_or_else(|_| "./knowledge".to_string()),
            max_context_length: env::var("EXPERT_MAX_CONTEXT_LENGTH").unwrap_or_else(|_| "8192".to_string()).parse().unwrap_or(8192),
            response_timeout_seconds: env::var("EXPERT_RESPONSE_TIMEOUT_SECONDS").unwrap_or_else(|_| "60".to_string()).parse().unwrap_or(60),
            cache_enabled: env::var("EXPERT_CACHE_ENABLED").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
            cache_ttl_seconds: env::var("EXPERT_CACHE_TTL_SECONDS").unwrap_or_else(|_| "3600".to_string()).parse().unwrap_or(3600),
        };
        
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.knowledge_base_path.is_empty() {
            return Err("Knowledge base path cannot be empty".into());
        }
        
        if self.max_context_length == 0 {
            return Err("Max context length must be greater than 0".into());
        }
        
        if self.response_timeout_seconds == 0 {
            return Err("Response timeout must be greater than 0".into());
        }
        
        Ok(())
    }
}