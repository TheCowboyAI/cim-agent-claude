/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Event Flow Management
//!
//! Manages event flows between CIM modules using Category Theory principles.
//! Each event flow is a natural transformation between functors.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFlow {
    pub id: String,
    pub source_module: String,
    pub target_module: String,
    pub subject_mapping: SubjectMapping,
    pub transformation: Option<EventMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectMapping {
    pub source_subject: String,
    pub target_subject: String,
    pub filter: Option<String>, // Optional subject filter
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMapping {
    pub mapping_type: MappingType,
    pub transformation_rules: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MappingType {
    Identity,           // Direct pass-through
    Transform,          // Apply transformation rules
    Filter,             // Filter events based on criteria
    Aggregate,          // Combine multiple events
    Split,              // Split event into multiple events
}

pub struct SubjectGraph {
    flows: Vec<EventFlow>,
    adjacency: HashMap<String, Vec<String>>,
}

impl SubjectGraph {
    pub fn new() -> Self {
        Self {
            flows: Vec::new(),
            adjacency: HashMap::new(),
        }
    }
    
    pub fn add_flow(&mut self, flow: EventFlow) {
        // Add to adjacency list
        self.adjacency
            .entry(flow.source_module.clone())
            .or_default()
            .push(flow.target_module.clone());
        
        self.flows.push(flow);
    }
    
    pub fn validate_acyclic(&self) -> Result<(), String> {
        // TODO: Implement cycle detection using DFS
        Ok(())
    }
    
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        // TODO: Implement topological sort for module startup order
        let modules: Vec<String> = self.adjacency.keys().cloned().collect();
        Ok(modules)
    }
}