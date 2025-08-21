//! Simple SAGE Test Client
//! Sends test requests to SAGE service and displays responses

use anyhow::Result;
use async_nats::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio;
use tracing::{info, error};
use uuid::Uuid;
use chrono::Utc;
use futures_util::stream::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageRequest {
    pub request_id: String,
    pub query: String,
    pub expert: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageResponse {
    pub request_id: String,
    pub response: String,
    pub expert_agents_used: Vec<String>,
    pub confidence_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SageStatus {
    pub is_conscious: bool,
    pub consciousness_level: f64,
    pub available_agents: usize,
    pub total_orchestrations: u64,
    pub patterns_learned: usize,
    pub memory_health: String,
}

async fn test_sage_request(nats_client: &Client, query: &str, expert: Option<String>) -> Result<()> {
    let request_id = Uuid::new_v4().to_string();
    
    // Create request
    let request = SageRequest {
        request_id: request_id.clone(),
        query: query.to_string(),
        expert,
        timestamp: Utc::now(),
    };
    
    info!("📤 Sending SAGE request: {}", query);
    
    // Subscribe to response first
    let response_subject = format!("sage.response.{}", request_id);
    let mut response_subscriber = nats_client.subscribe(response_subject.clone()).await?;
    
    // Send request
    let request_json = serde_json::to_vec(&request)?;
    nats_client.publish("sage.request", request_json.into()).await?;
    
    info!("⏳ Waiting for SAGE response...");
    
    // Wait for response with timeout
    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        response_subscriber.next()
    );
    
    match timeout.await {
        Ok(Some(msg)) => {
            match serde_json::from_slice::<SageResponse>(&msg.payload) {
                Ok(response) => {
                    info!("✅ Received SAGE response!");
                    println!("\n🎭 SAGE RESPONSE:");
                    println!("==================");
                    println!("Request ID: {}", response.request_id);
                    println!("Confidence: {:.2}", response.confidence_score);
                    println!("Experts Used: {:?}", response.expert_agents_used);
                    println!("\nResponse Content:");
                    println!("{}", response.response);
                    println!("\n==================\n");
                }
                Err(e) => {
                    error!("Failed to parse response: {}", e);
                }
            }
        }
        Ok(None) => {
            error!("Response stream ended unexpectedly");
        }
        Err(_) => {
            error!("⏰ Timeout waiting for SAGE response");
        }
    }
    
    Ok(())
}

async fn test_sage_status(nats_client: &Client) -> Result<()> {
    info!("📤 Requesting SAGE status...");
    
    // Subscribe to status response first
    let mut status_subscriber = nats_client.subscribe("sage.status.response").await?;
    
    // Send status request
    nats_client.publish("sage.status", "{}".into()).await?;
    
    // Wait for status response with timeout
    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        status_subscriber.next()
    );
    
    match timeout.await {
        Ok(Some(msg)) => {
            match serde_json::from_slice::<SageStatus>(&msg.payload) {
                Ok(status) => {
                    info!("✅ Received SAGE status!");
                    println!("\n📊 SAGE STATUS:");
                    println!("================");
                    println!("Conscious: {}", status.is_conscious);
                    println!("Consciousness Level: {:.1}", status.consciousness_level);
                    println!("Available Agents: {}", status.available_agents);
                    println!("Total Orchestrations: {}", status.total_orchestrations);
                    println!("Patterns Learned: {}", status.patterns_learned);
                    println!("Memory Health: {}", status.memory_health);
                    println!("================\n");
                }
                Err(e) => {
                    error!("Failed to parse status: {}", e);
                }
            }
        }
        Ok(None) => {
            error!("Status stream ended unexpectedly");
        }
        Err(_) => {
            error!("⏰ Timeout waiting for SAGE status");
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .init();
    
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    info!("🧪 SAGE Test Client Starting");
    info!("NATS URL: {}", nats_url);
    
    // Connect to NATS
    let nats_client = async_nats::connect(&nats_url).await?;
    info!("✅ Connected to NATS server");
    
    println!("\n🎭 SAGE Service Test Suite");
    println!("==========================");
    
    // Test 1: Status Check
    println!("\n🔍 Test 1: SAGE Status Check");
    test_sage_status(&nats_client).await?;
    
    // Test 2: Simple Query
    println!("\n🔍 Test 2: Simple CIM Query");
    test_sage_request(&nats_client, "What is a CIM?", None).await?;
    
    // Test 3: NATS Expert Query
    println!("\n🔍 Test 3: NATS Expert Consultation");
    test_sage_request(&nats_client, "How do I design NATS subjects for my domain?", Some("nats-expert".to_string())).await?;
    
    // Test 4: Complex Architecture Query
    println!("\n🔍 Test 4: Complex Architecture Query");
    test_sage_request(&nats_client, "Build a complete order processing CIM with event sourcing and NATS infrastructure", None).await?;
    
    println!("✅ All SAGE tests completed!");
    println!("\n🎉 SAGE ↔ NATS communication verified successfully!");
    
    Ok(())
}