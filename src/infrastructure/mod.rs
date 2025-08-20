/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Infrastructure Module
//!
//! Provides shared infrastructure services for CIM composition:
//! - NATS messaging infrastructure
//! - Observability (logging, metrics, tracing)
//! - Configuration management
//! - Health monitoring

pub mod config;
pub mod nats;
pub mod observability;
pub mod claude;
pub mod gui;
pub mod expert;

pub use config::Config;
pub use nats::NatsInfrastructure;
pub use observability::ObservabilityInfrastructure;