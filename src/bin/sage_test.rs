//! Minimal SAGE Test Service
//! 
//! A minimal SAGE service for testing basic NATS communication
//! without depending on the complex existing codebase that has compilation errors.

use anyhow::Result;
use async_nats::{Client, jetstream};
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio;
use tracing::{info, error};
use uuid::Uuid;
use chrono::Utc;

/// Minimal SAGE Request for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageRequest {
    pub request_id: String,
    pub query: String,
    pub expert: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Minimal SAGE Response for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageResponse {
    pub request_id: String,
    pub response: String,
    pub expert_agents_used: Vec<String>,
    pub confidence_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Minimal SAGE Service for testing
pub struct SageTestService {
    nats_client: Client,
}

impl SageTestService {
    /// Create new minimal SAGE test service
    pub async fn new(nats_url: &str) -> Result<Self> {
        info!("🎭 Connecting to NATS server at: {}", nats_url);
        let nats_client = async_nats::connect(nats_url).await?;
        info!("✅ Connected to NATS server successfully");
        
        Ok(Self {
            nats_client,
        })
    }
    
    /// Start minimal SAGE test service
    pub async fn start_service(&mut self) -> Result<()> {
        info!("🎭 SAGE Test Service Starting - Minimal NATS Testing");
        
        // Subscribe to SAGE requests
        let mut subscriber = self.nats_client.subscribe("sage.request").await?;
        info!("📨 Subscribed to sage.request subject");
        
        // Process requests
        while let Some(msg) = subscriber.next().await {
            info!("📥 Received message on sage.request");
            
            match serde_json::from_slice::<SageRequest>(&msg.payload) {
                Ok(request) => {
                    info!("✅ Parsed SAGE request: {}", request.request_id);
                    
                    // Create test response
                    let response = SageResponse {
                        request_id: request.request_id.clone(),
                        response: format!(
                            "🎭 SAGE Test Response for query: \"{}\"\n\n\
                            This is a minimal test response demonstrating NATS communication.\n\
                            In production, this would coordinate expert agents and provide intelligent guidance.\n\n\
                            Expert: {}\n\
                            Processing time: {:?}",
                            request.query,
                            request.expert.unwrap_or("sage-orchestrator".to_string()),
                            std::time::SystemTime::now()
                        ),
                        expert_agents_used: vec!["sage-test".to_string()],
                        confidence_score: 0.9,
                        timestamp: Utc::now(),
                    };
                    
                    // Publish response
                    let response_subject = format!("sage.response.{}", request.request_id);
                    match serde_json::to_vec(&response) {
                        Ok(response_json) => {
                            match self.nats_client.publish(response_subject.clone(), response_json.into()).await {
                                Ok(_) => {
                                    info!("✅ Published response to: {}", response_subject);
                                }
                                Err(e) => {
                                    error!("❌ Failed to publish response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("❌ Failed to serialize response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to parse SAGE request: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();
    
    // Get NATS URL from environment or use default
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    info!("🎭 SAGE Test Service Starting...");
    info!("NATS URL: {}", nats_url);
    
    // Create and start minimal SAGE test service
    let mut sage_service = SageTestService::new(&nats_url).await?;
    
    // Handle shutdown signals gracefully
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("🛑 SAGE Test Service received shutdown signal");
    };
    
    // Run service until shutdown
    tokio::select! {
        result = sage_service.start_service() => {
            if let Err(e) = result {
                error!("SAGE Test Service error: {}", e);
                std::process::exit(1);
            }
        }
        _ = shutdown_signal => {
            info!("🎭 SAGE Test Service shutting down gracefully...");
        }
    }
    
    info!("✅ SAGE Test Service stopped");
    Ok(())
}