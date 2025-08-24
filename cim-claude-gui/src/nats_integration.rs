//! NATS Integration for GUI
//! 
//! Handles NATS communication between GUI and SAGE V2 service
//! Sends requests and receives responses via NATS messaging

use anyhow::Result;
use async_nats::Client;
use futures_util::StreamExt;
use serde_json;
use std::time::Duration;
use tracing::{info, error, warn};

use crate::sage_client::{SageRequest, SageResponse, SageStatus};

/// NATS Integration for SAGE communication
pub struct NatsIntegration {
    client: Client,
    domain: Option<String>,
}

impl NatsIntegration {
    /// Create new NATS integration
    pub async fn new(nats_url: &str) -> Result<Self> {
        let client = async_nats::connect(nats_url).await?;
        let domain = Self::detect_domain();
        
        info!("📡 NATS Integration initialized");
        if let Some(ref d) = domain {
            info!("📍 Domain: {}", d);
        }
        
        Ok(Self {
            client,
            domain,
        })
    }
    
    /// Detect domain from environment or hostname
    fn detect_domain() -> Option<String> {
        if let Ok(domain) = std::env::var("CIM_DOMAIN") {
            return Some(domain);
        }
        
        hostname::get()
            .ok()
            .and_then(|h| h.to_str().map(|s| s.to_string()))
    }
    
    /// Send request to SAGE and wait for response
    pub async fn send_sage_request(&self, request: SageRequest) -> Result<SageResponse> {
        // Build request subject
        let request_subject = if let Some(ref domain) = self.domain {
            format!("{}.commands.sage.request", domain)
        } else {
            "commands.sage.request".to_string()
        };
        
        // Build response subject
        let response_subject = if let Some(ref domain) = self.domain {
            format!("{}.events.sage.response.{}", domain, request.request_id)
        } else {
            format!("events.sage.response.{}", request.request_id)
        };
        
        info!("📤 Sending SAGE request: {} to {}", request.request_id, request_subject);
        
        // Subscribe to response before sending request
        let mut response_sub = self.client
            .subscribe(response_subject.clone())
            .await?;
        
        // Serialize and send request
        let request_bytes = serde_json::to_vec(&request)?;
        self.client
            .publish(request_subject, request_bytes.into())
            .await?;
        
        // Wait for response with timeout
        let response_timeout = Duration::from_secs(30);
        let response_msg = tokio::time::timeout(
            response_timeout,
            response_sub.next()
        ).await?;
        
        // Parse response
        let response: SageResponse = match response_msg {
            Some(msg) => serde_json::from_slice(&msg.payload)?,
            None => return Err(anyhow::anyhow!("No response received from SAGE")),
        };
        
        info!("📥 Received SAGE response: {} (experts: {:?})", 
            response.request_id, response.expert_agents_used);
        
        Ok(response)
    }
    
    /// Query SAGE status
    pub async fn get_sage_status(&self) -> Result<SageStatus> {
        let status_subject = if let Some(ref domain) = self.domain {
            format!("{}.queries.sage.status", domain)
        } else {
            "queries.sage.status".to_string()
        };
        
        let response_subject = if let Some(ref domain) = self.domain {
            format!("{}.events.sage.status_response", domain)
        } else {
            "events.sage.status_response".to_string()
        };
        
        info!("📊 Querying SAGE status on {}", status_subject);
        
        // Subscribe to status response
        let mut response_sub = self.client
            .subscribe(response_subject.clone())
            .await?;
        
        // Send status query
        self.client
            .publish(status_subject, "{}".into())
            .await?;
        
        // Wait for status response
        let response_timeout = Duration::from_secs(5);
        let status_msg = tokio::time::timeout(
            response_timeout,
            response_sub.next()
        ).await?;
        
        // Parse status
        let status: SageStatus = match status_msg {
            Some(msg) => serde_json::from_slice(&msg.payload)?,
            None => return Err(anyhow::anyhow!("No status received from SAGE")),
        };
        
        info!("✅ SAGE Status: conscious={}, level={:.1}, agents={}", 
            status.is_conscious, status.consciousness_level, status.available_agents);
        
        Ok(status)
    }
    
    /// Test NATS connectivity
    pub async fn test_connection(&self) -> bool {
        // Try to flush to verify connection
        self.client.flush().await.is_ok()
    }
    
    /// Get list of available expert agents
    pub async fn get_available_experts(&self) -> Vec<String> {
        // In a full implementation, this would query SAGE for available agents
        // For now, return the known agent list
        vec![
            "sage".to_string(),
            "ddd-expert".to_string(),
            "nats-expert".to_string(),
            "cim-expert".to_string(),
            "nix-expert".to_string(),
            "git-expert".to_string(),
            "bdd-expert".to_string(),
            "tdd-expert".to_string(),
            "qa-expert".to_string(),
            "domain-expert".to_string(),
            "network-expert".to_string(),
            "event-storming-expert".to_string(),
            "subject-expert".to_string(),
            "language-expert".to_string(),
            "ricing-expert".to_string(),
            "cim-domain-expert".to_string(),
            "iced-ui-expert".to_string(),
            "elm-architecture-expert".to_string(),
            "cim-tea-ecs-expert".to_string(),
        ]
    }
}

/// Background service for handling NATS operations
pub struct NatsBackgroundService {
    integration: NatsIntegration,
    tx: tokio::sync::mpsc::Sender<NatsCommand>,
    rx: tokio::sync::mpsc::Receiver<NatsCommand>,
}

/// Commands for the NATS background service
#[derive(Debug)]
pub enum NatsCommand {
    SendSageRequest {
        request: SageRequest,
        response_tx: tokio::sync::oneshot::Sender<Result<SageResponse>>,
    },
    GetSageStatus {
        response_tx: tokio::sync::oneshot::Sender<Result<SageStatus>>,
    },
    TestConnection {
        response_tx: tokio::sync::oneshot::Sender<bool>,
    },
}

impl NatsBackgroundService {
    /// Create new background service
    pub async fn new(nats_url: &str) -> Result<(Self, tokio::sync::mpsc::Sender<NatsCommand>)> {
        let integration = NatsIntegration::new(nats_url).await?;
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let tx_clone = tx.clone();
        
        Ok((
            Self {
                integration,
                tx,
                rx,
            },
            tx_clone,
        ))
    }
    
    /// Run the background service
    pub async fn run(mut self) {
        info!("🚀 NATS Background Service started");
        
        while let Some(cmd) = self.rx.recv().await {
            match cmd {
                NatsCommand::SendSageRequest { request, response_tx } => {
                    let result = self.integration.send_sage_request(request).await;
                    let _ = response_tx.send(result);
                }
                NatsCommand::GetSageStatus { response_tx } => {
                    let result = self.integration.get_sage_status().await;
                    let _ = response_tx.send(result);
                }
                NatsCommand::TestConnection { response_tx } => {
                    let result = self.integration.test_connection().await;
                    let _ = response_tx.send(result);
                }
            }
        }
        
        info!("🛑 NATS Background Service stopped");
    }
}