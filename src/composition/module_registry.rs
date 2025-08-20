/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Module Registry
//!
//! Manages the registry of available CIM modules and their metadata.

use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub id: String,
    pub module_type: ModuleType,
    pub version: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModuleType {
    Adapter,      // External API adapters
    Service,      // Business logic services
    Interface,    // User interfaces
    Infrastructure, // Infrastructure components
}

pub struct ModuleRegistry {
    modules: RwLock<HashMap<String, ModuleInfo>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn register(&self, id: &str, info: ModuleInfo) -> Result<(), Box<dyn std::error::Error>> {
        let mut modules = self.modules.write().unwrap();
        modules.insert(id.to_string(), info);
        Ok(())
    }
    
    pub async fn get(&self, id: &str) -> Option<ModuleInfo> {
        let modules = self.modules.read().unwrap();
        modules.get(id).cloned()
    }
    
    pub async fn list(&self) -> Vec<ModuleInfo> {
        let modules = self.modules.read().unwrap();
        modules.values().cloned().collect()
    }
    
    pub async fn dependencies(&self, id: &str) -> Vec<String> {
        let modules = self.modules.read().unwrap();
        modules.get(id)
            .map(|info| info.dependencies.clone())
            .unwrap_or_default()
    }
}