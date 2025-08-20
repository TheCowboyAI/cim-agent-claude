/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Claude API Configuration Module

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_seconds: u64,
}

impl ClaudeConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("CLAUDE_API_KEY")
            .map_err(|_| "CLAUDE_API_KEY environment variable is required")?;
        
        let config = ClaudeConfig {
            api_key,
            base_url: env::var("CLAUDE_BASE_URL").unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
            model: env::var("CLAUDE_MODEL").unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            max_tokens: env::var("CLAUDE_MAX_TOKENS").unwrap_or_else(|_| "4096".to_string()).parse().unwrap_or(4096),
            temperature: env::var("CLAUDE_TEMPERATURE").unwrap_or_else(|_| "0.7".to_string()).parse().unwrap_or(0.7),
            timeout_seconds: env::var("CLAUDE_TIMEOUT_SECONDS").unwrap_or_else(|_| "30".to_string()).parse().unwrap_or(30),
        };
        
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.api_key.is_empty() {
            return Err("Claude API key cannot be empty".into());
        }
        
        if self.base_url.is_empty() {
            return Err("Claude base URL cannot be empty".into());
        }
        
        if self.model.is_empty() {
            return Err("Claude model cannot be empty".into());
        }
        
        if self.max_tokens == 0 {
            return Err("Max tokens must be greater than 0".into());
        }
        
        if !(0.0..=2.0).contains(&self.temperature) {
            return Err("Temperature must be between 0.0 and 2.0".into());
        }
        
        Ok(())
    }
}