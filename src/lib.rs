/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Agent Claude
//!
//! A Composable Information Machine (CIM) that orchestrates Claude AI integration
//! through event-driven architecture using NATS messaging.
//!
//! This CIM follows mathematical foundations based on:
//! - Category Theory: Modules as morphisms, composition as functors
//! - Graph Theory: Event flows as directed acyclic graphs
//! - IPLD: Content-addressed data structures
//! - Event Sourcing: Immutable event streams via NATS JetStream
//!
//! Architecture:
//! ```
//! CIM Agent Claude (Root CIM)
//! ├── Composition Layer    # Module orchestration
//! ├── Infrastructure Layer # NATS, observability, config
//! ├── Orchestration Layer  # Service lifecycle management
//! └── Modules
//!     ├── cim-claude-adapter  # Pure Claude API integration
//!     ├── cim-claude-gui      # Management interfaces
//!     └── cim-expert          # Domain expertise module
//! ```

pub mod composition;
pub mod infrastructure;
pub mod orchestration;
pub mod subagents;

// Re-exports for public API
pub use composition::composer::{CimModule, ModuleHealth};
pub use composition::{CimComposer, ModuleRegistry, ModuleType};
pub use infrastructure::{Config, NatsInfrastructure, ObservabilityInfrastructure};
pub use orchestration::{ServiceOrchestrator, SystemHealth};
pub use subagents::{SubagentRegistry, SubagentRouter, SubagentDispatcher, SubjectResolution, DomainType};

/// CIM Agent Claude version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// CIM Agent Claude description
pub const DESCRIPTION: &str = "Event-driven Claude AI integration CIM using NATS messaging";

/// Initialize a new CIM Agent Claude instance
pub async fn initialize() -> Result<CimAgentClaude, Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    config.validate()?;
    
    CimAgentClaude::new(config).await
}

/// Main CIM Agent Claude structure
pub struct CimAgentClaude {
    config: Config,
    orchestrator: Option<ServiceOrchestrator>,
}

impl CimAgentClaude {
    /// Create a new CIM Agent Claude instance
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            orchestrator: None,
        })
    }
    
    /// Start the CIM
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize infrastructure
        let _observability = ObservabilityInfrastructure::initialize(&self.config.observability).await?;
        let nats_infrastructure = NatsInfrastructure::initialize(&self.config.nats).await?;
        
        // Compose modules
        let composer = CimComposer::new(nats_infrastructure.clone(), self.config.clone());
        let modules = composer.compose_modules().await?;
        
        // Start orchestration
        let mut orchestrator = ServiceOrchestrator::new(modules, nats_infrastructure, self.config.clone());
        orchestrator.start().await?;
        
        self.orchestrator = Some(orchestrator);
        Ok(())
    }
    
    /// Stop the CIM
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(orchestrator) = &self.orchestrator {
            orchestrator.shutdown().await?;
        }
        Ok(())
    }
    
    /// Get system health
    pub async fn health(&self) -> Option<SystemHealth> {
        if let Some(orchestrator) = &self.orchestrator {
            Some(orchestrator.system_health().await)
        } else {
            None
        }
    }
    
    /// Get configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}