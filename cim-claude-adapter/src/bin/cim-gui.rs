/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Claude Adapter GUI Manager
//! 
//! Native desktop application for managing Claude conversations via NATS.
//! Built with Iced using The Elm Architecture (TEA).

use cim_claude_adapter::gui::app::CimManagerApp;

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Run the Iced application using the functional API
    let (mut app, _task) = CimManagerApp::new();
    
    iced::application("CIM Claude Adapter Manager", 
        |app: &mut CimManagerApp, message| app.update(message),
        |app: &CimManagerApp| app.view()
    ).run()
}