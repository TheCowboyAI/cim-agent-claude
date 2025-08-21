/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Composer - Module Composition Engine
//!
//! Implements the core CIM composition patterns using Category Theory principles.
//! Composes modules into a coherent system with proper event flows.

use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use tracing::{info, debug, error};

use crate::infrastructure::{NatsInfrastructure, Config};
use super::{ModuleRegistry, ModuleInfo, ModuleType};

/// CIM Module trait - all modules must implement this
#[async_trait]
pub trait CimModule: Send + Sync {
    /// Module identifier
    fn id(&self) -> &str;
    
    /// Module type
    fn module_type(&self) -> ModuleType;
    
    /// Initialize the module with infrastructure
    async fn initialize(&mut self, infrastructure: Arc<NatsInfrastructure>) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Start the module
    async fn start(&self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Stop the module
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Get module health
    async fn health(&self) -> ModuleHealth;
    
    /// Get event subjects this module subscribes to
    fn input_subjects(&self) -> Vec<String>;
    
    /// Get event subjects this module publishes to
    fn output_subjects(&self) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct ModuleHealth {
    pub healthy: bool,
    pub message: String,
    pub metrics: HashMap<String, serde_json::Value>,
}

/// CIM Composer - orchestrates module composition
pub struct CimComposer {
    nats_infrastructure: Arc<NatsInfrastructure>,
    config: Config,
    registry: ModuleRegistry,
}

impl CimComposer {
    pub fn new(
        nats_infrastructure: Arc<NatsInfrastructure>,
        config: Config,
    ) -> Self {
        Self {
            nats_infrastructure,
            config,
            registry: ModuleRegistry::new(),
        }
    }
    
    /// Compose all enabled modules
    pub async fn compose_modules(&self) -> Result<Vec<Arc<dyn CimModule>>, Box<dyn std::error::Error>> {
        info!("Starting CIM module composition");
        
        let mut modules = Vec::new();
        
        // Claude Adapter Module (always enabled)
        if let Some(claude_module) = self.compose_claude_adapter().await? {
            modules.push(claude_module);
        }
        
        // GUI Module (if enabled)
        if self.config.gui.enabled {
            if let Some(gui_module) = self.compose_gui_module().await? {
                modules.push(gui_module);
            }
        }
        
        // Expert Module (if enabled)  
        if self.config.expert.enabled {
            if let Some(expert_module) = self.compose_expert_module().await? {
                modules.push(expert_module);
            }
        }
        
        // Validate composition
        self.validate_composition(&modules).await?;
        
        info!("CIM composition completed: {} modules", modules.len());
        Ok(modules)
    }
    
    /// Compose Claude Adapter module
    async fn compose_claude_adapter(&self) -> Result<Option<Arc<dyn CimModule>>, Box<dyn std::error::Error>> {
        info!("Composing Claude Adapter module");
        
        let module = ClaudeAdapterModule::new(
            self.config.claude.clone(),
            self.nats_infrastructure.clone(),
        );
        
        let module_arc: Arc<dyn CimModule> = Arc::new(module);
        
        // Register in module registry
        let info = ModuleInfo {
            id: "claude-adapter".to_string(),
            module_type: ModuleType::Adapter,
            version: "1.0.0".to_string(),
            dependencies: vec![],
        };
        
        self.registry.register("claude-adapter", info).await?;
        
        Ok(Some(module_arc))
    }
    
    /// Compose GUI module
    async fn compose_gui_module(&self) -> Result<Option<Arc<dyn CimModule>>, Box<dyn std::error::Error>> {
        info!("Composing GUI module");
        
        let module = GuiModule::new(
            self.config.gui.clone(),
            self.nats_infrastructure.clone(),
        );
        
        let module_arc: Arc<dyn CimModule> = Arc::new(module);
        
        let info = ModuleInfo {
            id: "gui".to_string(),
            module_type: ModuleType::Interface,
            version: "1.0.0".to_string(),
            dependencies: vec!["claude-adapter".to_string()],
        };
        
        self.registry.register("gui", info).await?;
        
        Ok(Some(module_arc))
    }
    
    /// Compose CIM Expert module
    async fn compose_expert_module(&self) -> Result<Option<Arc<dyn CimModule>>, Box<dyn std::error::Error>> {
        info!("Composing CIM Expert module");
        
        let module = CimExpertModule::new(
            self.config.expert.clone(),
            self.nats_infrastructure.clone(),
        );
        
        let module_arc: Arc<dyn CimModule> = Arc::new(module);
        
        let info = ModuleInfo {
            id: "cim-expert".to_string(),
            module_type: ModuleType::Service,
            version: "1.0.0".to_string(),
            dependencies: vec!["claude-adapter".to_string()],
        };
        
        self.registry.register("cim-expert", info).await?;
        
        Ok(Some(module_arc))
    }
    
    /// Validate that the composition is mathematically sound
    async fn validate_composition(&self, modules: &[Arc<dyn CimModule>]) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Validating CIM composition");
        
        // Check for circular dependencies
        self.check_circular_dependencies(modules).await?;
        
        // Validate event flows
        self.validate_event_flows(modules).await?;
        
        // Check resource constraints
        self.check_resource_constraints(modules).await?;
        
        info!("CIM composition validation passed");
        Ok(())
    }
    
    async fn check_circular_dependencies(&self, _modules: &[Arc<dyn CimModule>]) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement dependency graph analysis
        Ok(())
    }
    
    async fn validate_event_flows(&self, modules: &[Arc<dyn CimModule>]) -> Result<(), Box<dyn std::error::Error>> {
        let mut publishers = HashMap::new();
        let mut subscribers = HashMap::new();
        
        for module in modules {
            // Collect publishers and subscribers
            for subject in module.output_subjects() {
                publishers.entry(subject.clone()).or_insert_with(Vec::new).push(module.id().to_string());
            }
            
            for subject in module.input_subjects() {
                subscribers.entry(subject.clone()).or_insert_with(Vec::new).push(module.id().to_string());
            }
        }
        
        // Check for orphaned subjects
        for (subject, subs) in &subscribers {
            if !publishers.contains_key(subject) {
                error!("Orphaned subject '{}' has subscribers but no publishers: {:?}", subject, subs);
                return Err(format!("Invalid event flow: subject '{}' has no publishers", subject).into());
            }
        }
        
        debug!("Event flow validation passed");
        Ok(())
    }
    
    async fn check_resource_constraints(&self, _modules: &[Arc<dyn CimModule>]) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Check memory, CPU, network constraints
        Ok(())
    }
}

// Module implementations (these would be in separate files in practice)

use crate::infrastructure::claude::ClaudeConfig;

struct ClaudeAdapterModule {
    id: String,
    config: ClaudeConfig,
    infrastructure: Option<Arc<NatsInfrastructure>>,
}

impl ClaudeAdapterModule {
    fn new(config: ClaudeConfig, _infrastructure: Arc<NatsInfrastructure>) -> Self {
        Self {
            id: "claude-adapter".to_string(),
            config,
            infrastructure: None,
        }
    }
}

#[async_trait]
impl CimModule for ClaudeAdapterModule {
    fn id(&self) -> &str { &self.id }
    fn module_type(&self) -> ModuleType { ModuleType::Adapter }
    
    async fn initialize(&mut self, infrastructure: Arc<NatsInfrastructure>) -> Result<(), Box<dyn std::error::Error>> {
        self.infrastructure = Some(infrastructure);
        info!("Claude Adapter module initialized");
        Ok(())
    }
    
    async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Claude Adapter module started");
        Ok(())
    }
    
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Claude Adapter module stopped");
        Ok(())
    }
    
    async fn health(&self) -> ModuleHealth {
        ModuleHealth {
            healthy: true,
            message: "Claude Adapter operational".to_string(),
            metrics: HashMap::new(),
        }
    }
    
    fn input_subjects(&self) -> Vec<String> {
        vec![
            "cim.claude.cmd.>".to_string(),
            "cim.expert.query.>".to_string(),
        ]
    }
    
    fn output_subjects(&self) -> Vec<String> {
        vec![
            "cim.claude.event.>".to_string(),
            "cim.expert.response.>".to_string(),
        ]
    }
}

use crate::infrastructure::gui::GuiConfig;

struct GuiModule {
    id: String,
    config: GuiConfig,
    infrastructure: Option<Arc<NatsInfrastructure>>,
}

impl GuiModule {
    fn new(config: GuiConfig, _infrastructure: Arc<NatsInfrastructure>) -> Self {
        Self {
            id: "gui".to_string(),
            config,
            infrastructure: None,
        }
    }
}

#[async_trait]
impl CimModule for GuiModule {
    fn id(&self) -> &str { &self.id }
    fn module_type(&self) -> ModuleType { ModuleType::Interface }
    
    async fn initialize(&mut self, infrastructure: Arc<NatsInfrastructure>) -> Result<(), Box<dyn std::error::Error>> {
        self.infrastructure = Some(infrastructure);
        info!("GUI module initialized");
        Ok(())
    }
    
    async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("GUI module started");
        Ok(())
    }
    
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("GUI module stopped");
        Ok(())
    }
    
    async fn health(&self) -> ModuleHealth {
        ModuleHealth {
            healthy: true,
            message: "GUI module operational".to_string(),
            metrics: HashMap::new(),
        }
    }
    
    fn input_subjects(&self) -> Vec<String> {
        vec![
            "cim.claude.event.>".to_string(),
            "cim.expert.response.>".to_string(),
        ]
    }
    
    fn output_subjects(&self) -> Vec<String> {
        vec![
            "cim.claude.cmd.>".to_string(),
            "cim.expert.query.>".to_string(),
        ]
    }
}

use crate::infrastructure::expert::ExpertConfig;

struct CimExpertModule {
    id: String,
    config: ExpertConfig,
    infrastructure: Option<Arc<NatsInfrastructure>>,
}

impl CimExpertModule {
    fn new(config: ExpertConfig, _infrastructure: Arc<NatsInfrastructure>) -> Self {
        Self {
            id: "cim-expert".to_string(),
            config,
            infrastructure: None,
        }
    }
}

#[async_trait]
impl CimModule for CimExpertModule {
    fn id(&self) -> &str { &self.id }
    fn module_type(&self) -> ModuleType { ModuleType::Service }
    
    async fn initialize(&mut self, infrastructure: Arc<NatsInfrastructure>) -> Result<(), Box<dyn std::error::Error>> {
        self.infrastructure = Some(infrastructure);
        info!("CIM Expert module initialized");
        Ok(())
    }
    
    async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("CIM Expert module started");
        Ok(())
    }
    
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("CIM Expert module stopped");
        Ok(())
    }
    
    async fn health(&self) -> ModuleHealth {
        ModuleHealth {
            healthy: true,
            message: "CIM Expert module operational".to_string(),
            metrics: HashMap::new(),
        }
    }
    
    fn input_subjects(&self) -> Vec<String> {
        vec![
            "cim.expert.query.>".to_string(),
        ]
    }
    
    fn output_subjects(&self) -> Vec<String> {
        vec![
            "cim.expert.response.>".to_string(),
        ]
    }
}