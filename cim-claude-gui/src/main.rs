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

/// Global NATS subscription - runs at application level, sends TEA Messages
fn nats_global_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run_with_id(
        "global-nats-subscription",
        stream::unfold(None, |nats_client| async move {
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
                            return Some((Message::ConnectionError(format!("NATS failed: {}", e)), None));
                        }
                    }
                }
            };

            // Subscribe to SAGE subjects
            let sage_result = client.subscribe("sage.response.*").await;
            let status_result = client.subscribe("sage.status.response").await;

            match (sage_result, status_result) {
                (Ok(mut sage_sub), Ok(mut status_sub)) => {
                    // Wait for any message and convert to TEA Message
                    tokio::select! {
                        Some(msg) = sage_sub.next() => {
                            match serde_json::from_slice::<cim_claude_gui::sage_client::SageResponse>(&msg.payload) {
                                Ok(response) => {
                                    Some((Message::SageResponseReceived(response), Some(client)))
                                }
                                Err(e) => {
                                    Some((Message::Error(format!("SAGE parse error: {}", e)), Some(client)))
                                }
                            }
                        }
                        Some(msg) = status_sub.next() => {
                            match serde_json::from_slice::<cim_claude_gui::sage_client::SageStatus>(&msg.payload) {
                                Ok(status) => {
                                    Some((Message::SageStatusReceived(status), Some(client)))
                                }
                                Err(e) => {
                                    Some((Message::Error(format!("Status parse error: {}", e)), Some(client)))
                                }
                            }
                        }
                    }
                }
                _ => {
                    Some((Message::ConnectionError("Failed to subscribe to SAGE".to_string()), Some(client)))
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