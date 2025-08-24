//! Error types for CIM LLM Adapter

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmAdapterError {
    #[error("NATS connection error: {0}")]
    NatsConnection(String),
    
    #[error("Stream creation error: {0}")]
    StreamCreation(String),
    
    #[error("Provider error: {0}")]
    Provider(String),
    
    #[error("Dialog management error: {0}")]
    DialogManagement(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl From<serde_json::Error> for LlmAdapterError {
    fn from(err: serde_json::Error) -> Self {
        LlmAdapterError::Serialization(err.to_string())
    }
}

impl From<async_nats::Error> for LlmAdapterError {
    fn from(err: async_nats::Error) -> Self {
        LlmAdapterError::NatsConnection(err.to_string())
    }
}