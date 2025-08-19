/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Claude Adapter Management Interface
//! 
//! Iced GUI application using The Elm Architecture (TEA) for NATS-driven management.
//! Can be deployed as native desktop app or WebAssembly static site.

pub mod app;
pub mod messages;
pub mod nats_client;
pub mod views;

pub use app::CimManagerApp;
pub use messages::Message;