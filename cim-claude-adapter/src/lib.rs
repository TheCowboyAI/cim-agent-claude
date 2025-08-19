/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

pub mod adapters;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ports;


pub use domain::*;

// CIM Expert functionality for module integration
pub use application::cim_expert_service::{
    CimExpertService, CimExpertClient, CimExpertQuery, CimExpertResponse, 
    CimExpertTopic, SessionContext, ConsultationMetadata
};
