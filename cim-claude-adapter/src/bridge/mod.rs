// TEA-ECS Bridge Module
// 
// This module implements the critical bridge between:
// - TEA Display Layer: Synchronous Model-View-Update for UI rendering
// - ECS Communication Layer: Asynchronous Entity-Component-System for message bus operations
//
// Key Principles:
// - Model = Entity[Components] - shared state representation
// - View = Display Projection of Events - synchronous rendering
// - Update = Synchronous Functions bound to Model - immediate UI updates
// - Systems = Asynchronous Commands/Queries to MessageBus - state machine driven

pub mod bridge;
pub mod entities;
pub mod systems;
pub mod sync;
pub mod commands;
pub mod events;

pub use bridge::*;
pub use entities::*;
pub use systems::*;
pub use sync::*;
pub use commands::*;
pub use events::*;

use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;

/// Entity ID for all bridge entities
pub type EntityId = Uuid;

/// Bridge error types
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Entity not found: {0}")]
    EntityNotFound(EntityId),
    
    #[error("Component missing: {component} for entity {entity}")]
    ComponentMissing { entity: EntityId, component: String },
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Sync error: {0}")]
    SyncError(String),
    
    #[error("Command dispatch error: {0}")]
    CommandDispatchError(String),
}