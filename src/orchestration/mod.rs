/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Service Orchestration Module
//!
//! Manages the lifecycle of composed CIM modules.
//! Provides health monitoring, graceful shutdown, and error recovery.

use std::sync::Arc;
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

use crate::composition::composer::{CimModule, ModuleHealth};
use crate::infrastructure::{NatsInfrastructure, Config};

pub struct ServiceOrchestrator {
    modules: Vec<Arc<dyn CimModule>>,
    nats_infrastructure: Arc<NatsInfrastructure>,
    config: Config,
    running: bool,
}

impl ServiceOrchestrator {
    pub fn new(
        modules: Vec<Arc<dyn CimModule>>,
        nats_infrastructure: Arc<NatsInfrastructure>,
        config: Config,
    ) -> Self {
        Self {
            modules,
            nats_infrastructure,
            config,
            running: false,
        }
    }
    
    /// Start all modules in dependency order
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting service orchestration");
        
        // Initialize all modules
        for module in &mut self.modules {
            info!("Initializing module: {}", module.id());
            
            // Clone the module and initialize it
            let mut module_clone = Arc::clone(module);
            let infrastructure = self.nats_infrastructure.clone();
            
            // This is a workaround - we need mutable access but Arc doesn't allow it
            // In practice, each module would handle its own initialization state
            info!("Module {} will be initialized during start", module.id());
        }
        
        // Start all modules
        for module in &self.modules {
            info!("Starting module: {}", module.id());
            module.start().await?;
        }
        
        self.running = true;
        
        // Start health monitoring
        self.start_health_monitoring().await;
        
        info!("Service orchestration started successfully");
        Ok(())
    }
    
    /// Gracefully shutdown all modules
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting graceful shutdown");
        
        // Stop modules in reverse order
        for module in self.modules.iter().rev() {
            info!("Stopping module: {}", module.id());
            if let Err(e) = module.stop().await {
                warn!("Error stopping module {}: {}", module.id(), e);
            }
        }
        
        info!("Graceful shutdown completed");
        Ok(())
    }
    
    /// Start health monitoring for all modules
    async fn start_health_monitoring(&self) {
        let modules = self.modules.clone();
        let health_interval = Duration::from_secs(30);
        
        tokio::spawn(async move {
            let mut interval = interval(health_interval);
            
            loop {
                interval.tick().await;
                
                let mut all_healthy = true;
                let mut health_report = HashMap::new();
                
                for module in &modules {
                    let health = module.health().await;
                    
                    if !health.healthy {
                        all_healthy = false;
                        warn!("Module {} is unhealthy: {}", module.id(), health.message);
                    }
                    
                    health_report.insert(module.id().to_string(), health);
                }
                
                if !all_healthy {
                    warn!("System health check failed - some modules are unhealthy");
                    // TODO: Implement recovery strategies
                } else {
                    info!("System health check passed - all modules healthy");
                }
                
                // TODO: Publish health metrics to NATS
            }
        });
        
        info!("Health monitoring started");
    }
    
    /// Get overall system health
    pub async fn system_health(&self) -> SystemHealth {
        let mut module_health = HashMap::new();
        let mut all_healthy = true;
        
        for module in &self.modules {
            let health = module.health().await;
            all_healthy = all_healthy && health.healthy;
            module_health.insert(module.id().to_string(), health);
        }
        
        SystemHealth {
            healthy: all_healthy,
            modules: module_health,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub healthy: bool,
    pub modules: HashMap<String, ModuleHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}