/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Claude Adapter WebAssembly GUI
//! 
//! WebAssembly build for deploying as static website.
//! Connects to NATS server from the browser.

#[cfg(target_arch = "wasm32")]
use cim_claude_adapter::gui::CimManagerApp;
#[cfg(target_arch = "wasm32")]
use iced::{Application, Settings};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better debugging
    console_error_panic_hook::set_once();
    
    // Initialize logging for WASM
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    
    // Configure settings for web
    let settings = Settings {
        window: iced::window::Settings {
            size: (1200, 800),
            min_size: Some((800, 600)),
            ..Default::default()
        },
        ..Default::default()
    };
    
    // Run the application in WASM context
    CimManagerApp::run(settings).expect("Failed to run CIM GUI");
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    panic!("This binary is only for WebAssembly builds");
}