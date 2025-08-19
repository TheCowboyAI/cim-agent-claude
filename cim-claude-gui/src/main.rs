/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Claude GUI Manager
//! 
//! Native desktop application for managing Claude conversations via NATS.
//! Built with Iced using The Elm Architecture (TEA).

use cim_claude_gui::CimManagerApp;

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    iced::application(
        "CIM Claude GUI Manager", 
        CimManagerApp::update,
        CimManagerApp::view
    )
    .subscription(CimManagerApp::subscription)
    .theme(CimManagerApp::theme)
    .run_with(CimManagerApp::new)
}