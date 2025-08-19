/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Infrastructure module
//! External dependencies and configuration for NATS, object stores, and Claude API

pub mod config;
pub mod subjects;
pub mod nats_config;
pub mod nats_client;
pub mod claude_client;

pub use config::AdapterConfig;
