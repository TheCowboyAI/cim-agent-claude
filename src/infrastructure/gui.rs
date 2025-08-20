/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! GUI Configuration Module

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    pub enabled: bool,
    pub desktop_enabled: bool,
    pub web_enabled: bool,
    pub web_port: u16,
    pub web_host: String,
    pub websocket_url: String,
}

impl GuiConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let config = GuiConfig {
            enabled: env::var("GUI_ENABLED").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
            desktop_enabled: env::var("GUI_DESKTOP_ENABLED").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
            web_enabled: env::var("GUI_WEB_ENABLED").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
            web_port: env::var("GUI_WEB_PORT").unwrap_or_else(|_| "8081".to_string()).parse().unwrap_or(8081),
            web_host: env::var("GUI_WEB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            websocket_url: env::var("GUI_WEBSOCKET_URL").unwrap_or_else(|_| "ws://localhost:8081/nats-ws".to_string()),
        };
        
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.web_port == 0 {
            return Err("Web port must be greater than 0".into());
        }
        
        if self.web_host.is_empty() {
            return Err("Web host cannot be empty".into());
        }
        
        if self.websocket_url.is_empty() {
            return Err("WebSocket URL cannot be empty".into());
        }
        
        Ok(())
    }
}