/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Composition Module
//!
//! Implements CIM composition patterns for module orchestration.
//! This module follows Category Theory principles where:
//! - Each module is a morphism in the CIM category
//! - Composition is associative and has identity
//! - Event flows are natural transformations

pub mod composer;
pub mod module_registry;
pub mod event_flows;

pub use composer::CimComposer;
pub use module_registry::{ModuleRegistry, ModuleInfo, ModuleType};
pub use event_flows::{EventFlow, EventMapping, SubjectGraph};