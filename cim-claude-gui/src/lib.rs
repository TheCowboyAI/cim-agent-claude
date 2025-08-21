/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Claude GUI Manager
//! 
//! Standalone GUI application for managing Claude conversations via NATS.
//! Built with Iced using The Elm Architecture (TEA).
//! 
//! This module depends on cim-claude-adapter for core functionality
//! but provides the UI layer as a separate concern.

pub mod app;
pub mod messages;
pub mod nats_client;
pub mod sage_client;
pub mod views;

#[cfg(target_arch = "wasm32")]
pub mod nats_websocket;

#[cfg(not(feature = "cim-claude-adapter"))]
pub mod wasm_types;

pub use app::CimManagerApp;
pub use messages::Message;