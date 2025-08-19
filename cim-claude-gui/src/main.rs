/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Claude GUI Manager
//! 
//! Native desktop application for managing Claude conversations via NATS.
//! Built with Iced using The Elm Architecture (TEA).

use cim_claude_gui::{app::CimManagerApp, Message};

fn update(state: &mut CimManagerApp, message: Message) -> iced::Task<Message> {
    state.update(message)
}

fn view(state: &CimManagerApp) -> iced::Element<Message> {
    state.view()
}

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Run the Iced application using the functional API
    iced::run("CIM Claude GUI Manager", update, view)
}