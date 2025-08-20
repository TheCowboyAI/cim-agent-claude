//! CIM Agent Claude Integration Tests
//!
//! This module contains integration tests that verify the complete CIM system
//! functionality according to the user stories defined in USER-STORIES.md.
//!
//! Test Organization:
//! - `system_integration/` - Story 1.x: CIM Composition and Orchestration 
//! - `claude_integration/` - Story 2.x: Claude API Integration
//! - `gui_integration/` - Story 3.x: GUI Management
//! - `configuration/` - Story 4.x: System Configuration
//! - `observability/` - Story 5.x: Monitoring and Observability
//! - `development/` - Story 6.x: Development and Testing

pub mod common;
pub mod system_integration;
pub mod claude_integration; 
pub mod gui_integration;
pub mod configuration;
pub mod observability;
pub mod development;
pub mod sage_orchestration;
pub mod nats_operations;