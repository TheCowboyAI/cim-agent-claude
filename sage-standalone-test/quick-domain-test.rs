//! Quick domain business question test
use anyhow::Result;
use async_nats::Client;
use serde::{Deserialize, Serialize};
use tokio;
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

#[tokio::main]
async fn main() -> Result<()> {
    let nats_client = async_nats::connect("nats://localhost:4222").await?;
    let request_id = Uuid::new_v4().to_string();
    
    let response_subject = format!("sage.response.{}", request_id);
    let mut response_subscriber = nats_client.subscribe(response_subject).await?;
    
    let request = SageRequest {
        request_id: request_id.clone(),
        query: "How does defining a business domain help my company?".to_string(),
        expert: None,
        timestamp: Utc::now(),
    };
    
    let request_json = serde_json::to_vec(&request)?;
    nats_client.publish("sage.request", request_json.into()).await?;
    
    println!("🎯 Asked SAGE: How does defining a business domain help my company?");
    println!("📡 Communication: Your Question → NATS → SAGE → Claude API → NATS → Response");
    println!("⏳ Waiting for SAGE response...\n");
    
    if let Some(msg) = response_subscriber.next().await {
        let response: SageResponse = serde_json::from_slice(&msg.payload)?;
        
        println!("📋 SAGE RESPONSE:");
        println!("================");
        println!("🔥 Expert Used: {:?}", response.expert_agents_used);
        println!("📊 Confidence: {:.1}% ", response.confidence_score * 100.0);
        println!("\n💬 Response:\n{}", response.response);
    }
    
    Ok(())
}