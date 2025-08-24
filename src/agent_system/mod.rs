/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Universal Agent System
//! 
//! This module implements the revolutionary Universal Agent Architecture where
//! SAGE and all subagents are dynamic personality configurations loaded from
//! `.claude/agents/*.md` files.

pub mod personality;
pub mod registry;
pub mod loader;
pub mod composition;
pub mod context;

#[cfg(test)]
pub mod tests;

pub use personality::*;
pub use registry::*;
pub use loader::*;
pub use composition::*;
pub use context::*;

/// Core agent system types
pub type AgentId = String;
pub type Capability = String;

/// Error types for the agent system
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Failed to load agent configuration: {0}")]
    LoadError(String),
    
    #[error("Invalid agent configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Agent invocation failed: {0}")]
    InvocationError(String),
    
    #[error("Context preservation failed: {0}")]
    ContextError(String),
}

pub type AgentResult<T> = Result<T, AgentError>;