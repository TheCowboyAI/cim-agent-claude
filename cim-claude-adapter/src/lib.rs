/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Claude Adapter
//!
//! A pure Claude API integration adapter for CIM systems.
//! This adapter provides a clean interface to Claude's API
//! without any orchestration or infrastructure concerns.
//!
//! Responsibilities:
//! - Claude API client implementation
//! - Request/response mapping
//! - Error handling for Claude-specific errors
//! - Rate limiting and retry logic
//!
//! Does NOT include:
//! - NATS integration (handled by the parent CIM)
//! - GUI components (separate cim-claude-gui module)
//! - Service orchestration (handled by the parent CIM)

pub mod client;
pub mod domain;
pub mod error;

pub use client::ClaudeClient;
pub use domain::*;
pub use error::ClaudeError;