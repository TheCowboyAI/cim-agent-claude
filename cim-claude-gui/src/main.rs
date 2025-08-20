/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Claude GUI Manager
//! 
//! Native desktop application for managing Claude conversations via NATS.
//! Built with Iced using The Elm Architecture (TEA).

use cim_claude_gui::{CimManagerApp, nats_client};

#[tokio::main]
async fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Initialize NATS client before starting GUI - proper TEA architecture
    if let Err(e) = nats_client::initialize_nats().await {
        eprintln!("Failed to initialize NATS: {}", e);
        std::process::exit(1);
    }
    
    iced::application(
        "CIM Claude GUI Manager", 
        CimManagerApp::update,
        CimManagerApp::view
    )
    .subscription(CimManagerApp::subscription)
    .theme(CimManagerApp::theme)
    .run_with(CimManagerApp::new)
}