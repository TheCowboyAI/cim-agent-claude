/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use std::sync::Arc;
use iced::{Subscription, subscription};
use tokio::sync::mpsc;

use crate::{
    bridge::{TeaEcsBridge, TeaEvent},
    gui::messages::{Message, BridgeMessage},
};

/// Subscription manager for TEA-ECS bridge events
pub struct BridgeSubscription {
    bridge: Arc<TeaEcsBridge>,
}

impl BridgeSubscription {
    pub fn new(bridge: Arc<TeaEcsBridge>) -> Self {
        Self { bridge }
    }
    
    /// Create a subscription for bridge events
    pub fn subscription(&self) -> Subscription<Message> {
        subscription::channel(
            std::any::TypeId::of::<TeaEvent>(),
            100, // Channel capacity
            {
                let bridge = Arc::clone(&self.bridge);
                move |mut output| {
                    let bridge = Arc::clone(&bridge);
                    async move {
                        // Get the event receiver from the bridge
                        if let Some(mut receiver) = bridge.take_event_receiver().await {
                            loop {
                                tokio::select! {
                                    // Receive TEA events from bridge
                                    event = receiver.recv() => {
                                        match event {
                                            Some(tea_event) => {
                                                let message = Message::TeaEventReceived(tea_event);
                                                if output.send(message).await.is_err() {
                                                    break; // Channel closed
                                                }
                                            }
                                            None => {
                                                // Bridge disconnected
                                                let message = Message::BridgeMessage(
                                                    BridgeMessage::ConnectionError(
                                                        "Bridge event stream closed".to_string()
                                                    )
                                                );
                                                let _ = output.send(message).await;
                                                break;
                                            }
                                        }
                                    }
                                    
                                    // Periodic health check
                                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                                        // Could send health check events here
                                        continue;
                                    }
                                }
                            }
                        } else {
                            // Failed to get receiver
                            let message = Message::BridgeMessage(
                                BridgeMessage::ConnectionError(
                                    "Failed to get bridge event receiver".to_string()
                                )
                            );
                            let _ = output.send(message).await;
                        }
                    }
                }
            }
        )
    }
}

/// Health check subscription for monitoring system status
pub struct HealthCheckSubscription;

impl HealthCheckSubscription {
    pub fn subscription() -> Subscription<Message> {
        subscription::unfold(
            std::any::TypeId::of::<HealthCheckSubscription>(),
            (),
            |_state| async {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                
                // Perform health checks
                let health_status = crate::gui::messages::HealthStatus {
                    nats_connected: true, // Would check actual NATS status
                    claude_api_available: true, // Would check Claude API
                    active_conversations: 0, // Would get actual count
                    events_processed: 0, // Would get actual count
                    last_check: chrono::Utc::now(),
                };
                
                (Message::HealthCheckReceived(health_status), ())
            }
        )
    }
}

/// Connection status subscription for monitoring NATS connectivity
pub struct ConnectionStatusSubscription {
    bridge: Arc<TeaEcsBridge>,
}

impl ConnectionStatusSubscription {
    pub fn new(bridge: Arc<TeaEcsBridge>) -> Self {
        Self { bridge }
    }
    
    pub fn subscription(&self) -> Subscription<Message> {
        subscription::unfold(
            std::any::TypeId::of::<ConnectionStatusSubscription>(),
            Arc::clone(&self.bridge),
            |bridge| async {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                
                // Check bridge connection status
                // In a real implementation, we'd query the bridge for status
                let connected = true; // Placeholder
                
                let message = Message::BridgeStatusChanged {
                    connected,
                    error: None,
                };
                
                (message, bridge)
            }
        )
    }
}

/// Event aggregator subscription for batching related events
pub struct EventAggregatorSubscription {
    events: Arc<tokio::sync::Mutex<Vec<TeaEvent>>>,
}

impl EventAggregatorSubscription {
    pub fn new() -> Self {
        Self {
            events: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
    
    pub fn add_event(&self, event: TeaEvent) {
        let events = Arc::clone(&self.events);
        tokio::spawn(async move {
            let mut events = events.lock().await;
            events.push(event);
        });
    }
    
    pub fn subscription(&self) -> Subscription<Message> {
        subscription::unfold(
            std::any::TypeId::of::<EventAggregatorSubscription>(),
            Arc::clone(&self.events),
            |events| async {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                let mut events_guard = events.lock().await;
                if !events_guard.is_empty() {
                    let batch = events_guard.drain(..).collect::<Vec<_>>();
                    drop(events_guard);
                    
                    // Process batch of events
                    for event in batch {
                        // Would yield each event
                    }
                }
                
                // Return a placeholder message
                (Message::HealthCheckRequested, events)
            }
        )
    }
}

/// Error recovery subscription for handling bridge disconnections
pub struct ErrorRecoverySubscription {
    bridge: Arc<TeaEcsBridge>,
    retry_count: usize,
    max_retries: usize,
}

impl ErrorRecoverySubscription {
    pub fn new(bridge: Arc<TeaEcsBridge>, max_retries: usize) -> Self {
        Self {
            bridge,
            retry_count: 0,
            max_retries,
        }
    }
    
    pub fn subscription(&self) -> Subscription<Message> {
        subscription::unfold(
            std::any::TypeId::of::<ErrorRecoverySubscription>(),
            (Arc::clone(&self.bridge), self.retry_count, self.max_retries),
            |(bridge, retry_count, max_retries)| async {
                // Wait before attempting recovery
                let delay = std::cmp::min(5000 * (retry_count + 1), 30000); // Exponential backoff
                tokio::time::sleep(tokio::time::Duration::from_millis(delay as u64)).await;
                
                // Attempt to restart bridge
                if retry_count < max_retries {
                    match bridge.start().await {
                        Ok(_) => {
                            // Recovery successful
                            (
                                Message::BridgeMessage(BridgeMessage::Connected),
                                (bridge, 0, max_retries) // Reset retry count
                            )
                        }
                        Err(e) => {
                            // Recovery failed
                            (
                                Message::BridgeMessage(BridgeMessage::ConnectionError(
                                    format!("Recovery attempt {} failed: {}", retry_count + 1, e)
                                )),
                                (bridge, retry_count + 1, max_retries)
                            )
                        }
                    }
                } else {
                    // Max retries exceeded
                    (
                        Message::BridgeMessage(BridgeMessage::ConnectionError(
                            "Maximum recovery attempts exceeded".to_string()
                        )),
                        (bridge, retry_count, max_retries)
                    )
                }
            }
        )
    }
}

/// Metrics collection subscription for system monitoring
pub struct MetricsSubscription;

impl MetricsSubscription {
    pub fn subscription() -> Subscription<Message> {
        subscription::unfold(
            std::any::TypeId::of::<MetricsSubscription>(),
            (),
            |_state| async {
                tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
                
                // Collect system metrics
                let metrics = crate::gui::messages::SystemMetrics {
                    conversations_total: 0, // Would get from bridge
                    conversations_active: 0, // Would get from bridge
                    events_published: 0, // Would get from bridge
                    events_consumed: 0, // Would get from bridge
                    tea_events_received: 0, // Would get from bridge
                    bridge_commands_sent: 0, // Would get from bridge
                    api_requests_total: 0, // Would get from API client
                    api_requests_failed: 0, // Would get from API client
                    response_time_avg_ms: 0.0, // Would calculate from metrics
                    bridge_latency_avg_ms: 0.0, // Would calculate from bridge metrics
                };
                
                (Message::MetricsReceived(metrics), ())
            }
        )
    }
}