/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Agent Claude - Main CIM Service
//! 
//! This is the primary CIM (Composable Information Machine) that orchestrates
//! Claude AI integration through event-driven architecture using NATS.
//!
//! The CIM composes multiple modules:
//! - cim-claude-adapter: Pure Claude API integration
//! - cim-claude-gui: Management interface
//! - Infrastructure: NATS, observability, deployment
//!
//! Architecture follows CIM patterns:
//! - Event Sourcing via NATS JetStream
//! - CQRS pattern for commands/queries  
//! - Module composition over inheritance
//! - Infrastructure as composition

use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

mod composition;
mod infrastructure;
mod orchestration;

use composition::CimComposer;
use infrastructure::{NatsInfrastructure, ObservabilityInfrastructure, Config};
use orchestration::ServiceOrchestrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load CIM configuration
    let config = Config::from_env()?;
    config.validate()?;
    
    info!("Starting CIM Agent Claude");
    info!("Configuration: NATS={}, Modules enabled", config.nats.url);
    
    // Initialize infrastructure
    let observability = ObservabilityInfrastructure::initialize(&config.observability).await?;
    let nats_infrastructure = NatsInfrastructure::initialize(&config.nats).await?;
    
    info!("Infrastructure initialized");
    
    // Compose CIM modules
    let composer = CimComposer::new(
        nats_infrastructure.clone(),
        config.clone(),
    );
    
    let modules = composer.compose_modules().await?;
    info!("CIM modules composed: {} modules", modules.len());
    
    // Orchestrate services
    let mut orchestrator = ServiceOrchestrator::new(
        modules,
        nats_infrastructure,
        observability,
        config,
    );
    
    orchestrator.start().await?;
    info!("CIM Agent Claude started successfully");
    
    // Wait for shutdown
    wait_for_shutdown().await;
    
    info!("Shutting down CIM Agent Claude");
    orchestrator.shutdown().await?;
    
    Ok(())
}

async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}