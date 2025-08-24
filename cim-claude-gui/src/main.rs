/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Claude GUI Manager
//! 
//! Native desktop application for managing Claude conversations via NATS.
//! Built with Iced using The Elm Architecture (TEA).

use cim_claude_gui::{CimManagerApp, Message};
use cim_claude_gui::nats_client_fixed;

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
    // Use the fixed NATS client with proper event stream
    nats_client_fixed::nats_subscription()
}

#[tokio::main]
async fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Initialize NATS client before starting GUI
    if let Err(e) = nats_client_fixed::initialize_nats().await {
        tracing::error!("Failed to initialize NATS: {}", e);
        // Continue with GUI but show connection error
    }
    
    iced::application(
        "CIM Claude GUI Manager", 
        CimManagerApp::update,
        CimManagerApp::view
    )
    .subscription(|_app| nats_global_subscription()) // Global subscription, not app-level
    .theme(CimManagerApp::theme)
    .run_with(CimManagerApp::new)
}