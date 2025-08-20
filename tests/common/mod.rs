//! Common test utilities and fixtures
//!
//! This module provides shared functionality for all integration tests including
//! test fixtures, mock NATS servers, and test data generators.

use std::time::Duration;
use async_nats::jetstream;
use serde_json::{json, Value};
use tokio::sync::OnceCell;
use uuid::Uuid;

pub mod fixtures;
pub mod mock_nats;
pub mod test_config;

/// Global test NATS server instance
static TEST_NATS: OnceCell<TestNatsServer> = OnceCell::const_new();

/// Test NATS server wrapper
#[derive(Debug)]
pub struct TestNatsServer {
    pub url: String,
    pub jetstream: jetstream::Context,
    pub client: async_nats::Client,
}

impl TestNatsServer {
    /// Get or create the global test NATS instance
    pub async fn global() -> &'static TestNatsServer {
        TEST_NATS.get_or_init(|| async {
            Self::start().await
        }).await
    }
    
    /// Start a new test NATS server
    pub async fn start() -> Self {
        // For integration tests, we'll use embedded NATS server
        // In real deployment, this would connect to actual NATS cluster
        let url = "nats://127.0.0.1:4222"; // Assume NATS server running for tests
        
        let client = async_nats::connect(&url).await
            .expect("Failed to connect to test NATS server");
            
        let jetstream = jetstream::new(client.clone());
        
        Self {
            url: url.to_string(),
            jetstream,
            client,
        }
    }
    
    /// Clean up test streams and KV stores
    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clean up any test-specific streams
        let stream_prefix = "TEST_";
        // Implementation would list and delete test streams
        Ok(())
    }
}

/// Generate a unique test ID for isolation
pub fn test_id() -> String {
    format!("test_{}", Uuid::new_v4().simple())
}

/// Generate test configuration for CIM system
pub fn test_cim_config() -> Value {
    json!({
        "nats": {
            "url": "nats://127.0.0.1:4222",
            "subject_prefix": format!("test.cim.{}", test_id()),
            "jetstream": {
                "enabled": true,
                "store_dir": "/tmp/jetstream-test",
                "max_memory": "100MB",
                "max_file": "1GB"
            }
        },
        "claude": {
            "api_key": "test-api-key",
            "base_url": "http://localhost:8080", // Mock server
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 1024,
            "timeout": 5
        },
        "gui": {
            "enabled": true,
            "web_port": 8081,
            "websocket_url": "ws://localhost:8081/nats-ws"
        },
        "observability": {
            "log_level": "DEBUG",
            "metrics_enabled": true
        }
    })
}

/// Wait for condition with timeout
pub async fn wait_for_condition<F, Fut>(
    condition: F,
    timeout: Duration,
    interval: Duration,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout {
        if condition().await {
            return Ok(());
        }
        tokio::time::sleep(interval).await;
    }
    
    Err("Condition timeout exceeded".into())
}

/// Assertion helpers for event-driven testing
pub mod assertions {
    use super::*;
    
    /// Assert that an event was published to NATS
    pub async fn assert_event_published(
        client: &async_nats::Client,
        subject: &str,
        timeout: Duration,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let mut subscriber = client.subscribe(subject).await?;
        
        tokio::time::timeout(timeout, subscriber.next())
            .await?
            .ok_or("No event received")?
            .payload
            .as_ref()
            .try_into()
            .map_err(|e| format!("Failed to parse event JSON: {}", e).into())
    }
    
    /// Assert system reaches ready state
    pub async fn assert_system_ready(
        client: &async_nats::Client,
        subject_prefix: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ready_subject = format!("{}.system.evt.system_ready", subject_prefix);
        let _event = assert_event_published(client, &ready_subject, Duration::from_secs(10)).await?;
        Ok(())
    }
}