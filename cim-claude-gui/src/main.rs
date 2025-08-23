/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Claude GUI Manager
//! 
//! Native desktop application for managing Claude conversations via NATS.
//! Built with Iced using The Elm Architecture (TEA).

use cim_claude_gui::{CimManagerApp, Message};
use iced::futures::{stream, StreamExt};

/// Detect domain from environment or hostname
fn detect_domain() -> Option<String> {
    // First check environment variable
    if let Ok(domain) = std::env::var("CIM_DOMAIN") {
        return Some(domain);
    }
    
    // Check SAGE_DOMAIN for backward compatibility
    if let Ok(domain) = std::env::var("SAGE_DOMAIN") {
        return Some(domain);
    }
    
    // Use hostname as domain
    if let Ok(hostname) = hostname::get() {
        if let Some(host_str) = hostname.to_str() {
            return Some(host_str.to_string());
        }
    }
    
    None
}

/// Global NATS subscription - runs at application level, sends TEA Messages
fn nats_global_subscription() -> iced::Subscription<Message> {
    let domain = detect_domain();
    
    iced::Subscription::run_with_id(
        "global-nats-subscription",
        stream::unfold((None, domain), |(nats_client, domain)| async move {
            // Get or create NATS client
            let client = match nats_client {
                Some(client) => client,
                None => {
                    match async_nats::connect("nats://localhost:4222").await {
                        Ok(client) => {
                            tracing::info!("✅ Global NATS connected");
                            client
                        }
                        Err(e) => {
                            tracing::error!("❌ Global NATS connection failed: {}", e);
                            return Some((Message::ConnectionError(format!("NATS failed: {}", e)), (None, domain.clone())));
                        }
                    }
                }
            };

            // Subscribe to SAGE subjects using cim-subject pattern
            // Pattern: {domain}.events.sage.response_*
            let sage_subject = if let Some(ref d) = domain {
                format!("{}.events.sage.response_*", d)
            } else {
                "events.sage.response_*".to_string()
            };
            
            // Pattern: {domain}.events.sage.status_response
            let status_subject = if let Some(ref d) = domain {
                format!("{}.events.sage.status_response", d)
            } else {
                "events.sage.status_response".to_string()
            };
            
            let sage_result = client.subscribe(sage_subject).await;
            let status_result = client.subscribe(status_subject).await;

            match (sage_result, status_result) {
                (Ok(mut sage_sub), Ok(mut status_sub)) => {
                    // Wait for any message and convert to TEA Message
                    tokio::select! {
                        Some(msg) = sage_sub.next() => {
                            match serde_json::from_slice::<cim_claude_gui::sage_client::SageResponse>(&msg.payload) {
                                Ok(response) => {
                                    Some((Message::SageResponseReceived(response), (Some(client), domain.clone())))
                                }
                                Err(e) => {
                                    Some((Message::Error(format!("SAGE parse error: {}", e)), (Some(client), domain.clone())))
                                }
                            }
                        }
                        Some(msg) = status_sub.next() => {
                            match serde_json::from_slice::<cim_claude_gui::sage_client::SageStatus>(&msg.payload) {
                                Ok(status) => {
                                    Some((Message::SageStatusReceived(status), (Some(client), domain.clone())))
                                }
                                Err(e) => {
                                    Some((Message::Error(format!("Status parse error: {}", e)), (Some(client), domain.clone())))
                                }
                            }
                        }
                    }
                }
                _ => {
                    Some((Message::ConnectionError("Failed to subscribe to SAGE".to_string()), (Some(client), domain.clone())))
                }
            }
        })
    )
}

#[tokio::main]
async fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    iced::application(
        "CIM Claude GUI Manager", 
        CimManagerApp::update,
        CimManagerApp::view
    )
    .subscription(|_app| nats_global_subscription()) // Global subscription, not app-level
    .theme(CimManagerApp::theme)
    .run_with(CimManagerApp::new)
}