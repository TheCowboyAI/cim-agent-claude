#!/usr/bin/env -S cargo +stable --quiet -Zscript
//! Demo: Ask SAGE about how domain helps business

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
    println!("🎯 Asking SAGE: How does a domain help my business?");
    println!("=================================================");
    
    let nats_client = async_nats::connect("nats://localhost:4222").await?;
    let request_id = Uuid::new_v4().to_string();
    
    // Subscribe to response
    let response_subject = format!("sage.response.{}", request_id);
    let mut response_subscriber = nats_client.subscribe(response_subject).await?;
    
    // Send domain business question
    let request = SageRequest {
        request_id: request_id.clone(),
        query: "How does defining a business domain help my company achieve better software architecture and business outcomes?".to_string(),
        expert: None, // Let SAGE auto-route to ddd-expert
        timestamp: Utc::now(),
    };
    
    let request_json = serde_json::to_vec(&request)?;
    nats_client.publish("sage.request", request_json.into()).await?;
    
    println!("📤 Sent to SAGE via NATS...");
    println!("⏳ Waiting for intelligent response...\n");
    
    // Wait for response
    if let Some(msg) = response_subscriber.next().await {
        let response: SageResponse = serde_json::from_slice(&msg.payload)?;
        
        println!("🎭 SAGE Response (via Claude API):");
        println!("==================================");
        println!("Expert: {:?}", response.expert_agents_used);
        println!("Confidence: {:.2}", response.confidence_score);
        println!("\n{}\n", response.response);
    }
    
    Ok(())
}