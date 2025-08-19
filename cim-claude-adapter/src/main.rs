use cim_claude_adapter::{
    infrastructure::AdapterConfig,
    adapters::{NatsAdapter, ClaudeApiAdapter},
    application::ConversationService,
};
use std::{sync::Arc, time::Duration};
use tokio::signal;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AdapterConfig::from_env()?;
    config.validate()?;
    
    // Initialize observability
    setup_observability(&config).await?;
    
    info!("Starting CIM Claude Adapter");
    info!("Configuration loaded: NATS={}, Claude API configured", config.nats.url);
    
    // Initialize adapters
    let nats_adapter = Arc::new(
        NatsAdapter::new(&config.nats.url)
            .await
            .map_err(|e| format!("Failed to initialize NATS adapter: {}", e))?
    );
    
    let claude_adapter = Arc::new(
        ClaudeApiAdapter::new(
            config.claude.api_key.clone(),
            Some(config.claude.base_url.clone()),
        )
    );
    
    // Create conversation service (state managed via NATS JetStream)
    let conversation_service = Arc::new(ConversationService::new(
        nats_adapter.clone(),
        nats_adapter.clone(), // Use NATS adapter for state as well  
        claude_adapter.clone(),
    ));
    
    // Start the service
    conversation_service.start().await?;
    
    // Start background tasks
    start_background_tasks(conversation_service.clone(), &config).await;
    
    // Health monitoring (replaced by GUI interface)
    info!("Health monitoring available via GUI interface");
    
    info!("CIM Claude Adapter started successfully");
    info!("Management GUI available: cargo run --bin cim-gui");
    info!("WebAssembly build: cargo build --target wasm32-unknown-unknown --bin wasm-gui --features wasm");
    
    // Wait for shutdown signal
    wait_for_shutdown().await;
    
    info!("Shutting down CIM Claude Adapter");
    Ok(())
}

/// Setup observability (logging, metrics, tracing)
async fn setup_observability(config: &AdapterConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Parse log level
    let log_level = config.observability.log_level.parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);
    
    // Setup tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("cim_claude_adapter={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
    
    info!("Observability initialized with log level: {}", log_level);
    
    // TODO: Initialize metrics and tracing if enabled
    if config.observability.metrics_enabled {
        info!("Metrics enabled on port {}", config.observability.metrics_port);
    }
    
    if config.observability.tracing_enabled {
        info!("Distributed tracing enabled");
    }
    
    Ok(())
}

/// Start background tasks
async fn start_background_tasks(
    conversation_service: Arc<ConversationService>,
    config: &AdapterConfig,
) {
    let service = conversation_service.clone();
    let cleanup_interval = Duration::from_secs(config.server.cleanup_interval_seconds);
    
    // Cleanup task
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(cleanup_interval);
        loop {
            interval.tick().await;
            match service.cleanup_expired_conversations().await {
                Ok(count) => {
                    if count > 0 {
                        info!("Background cleanup removed {} expired conversations", count);
                    }
                }
                Err(e) => {
                    error!("Background cleanup failed: {}", e);
                }
            }
        }
    });
    
    let service = conversation_service.clone();
    let health_interval = Duration::from_secs(config.server.health_check_interval_seconds);
    
    // Health check task
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(health_interval);
        loop {
            interval.tick().await;
            match service.health_check().await {
                Ok(health) => {
                    if !health.is_healthy {
                        warn!(
                            "Service health check failed: NATS={}, Claude={}",
                            health.conversation_port_healthy,
                            health.claude_api_available
                        );
                    }
                }
                Err(e) => {
                    error!("Health check failed: {}", e);
                }
            }
        }
    });
    
    info!("Background tasks started");
}

// Health server removed - health checks now available through GUI

/// Wait for shutdown signal
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

// Clone implementation removed - we now pass Arc<NatsAdapter> directly