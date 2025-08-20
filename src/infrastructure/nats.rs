/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! NATS Infrastructure Module

use std::sync::Arc;
use async_nats::{Client, jetstream};
use tracing::{info, error};

use super::config::NatsConfig;

pub struct NatsInfrastructure {
    client: Client,
    jetstream: jetstream::Context,
    config: NatsConfig,
}

impl NatsInfrastructure {
    pub async fn initialize(config: &NatsConfig) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        info!("Initializing NATS infrastructure at {}", config.url);
        
        // Connect to NATS
        let client = async_nats::connect(&config.url).await
            .map_err(|e| format!("Failed to connect to NATS: {}", e))?;
        
        // Initialize JetStream
        let jetstream = jetstream::new(client.clone());
        
        // Setup streams if enabled
        if config.jetstream.enabled {
            Self::setup_streams(&jetstream, config).await?;
        }
        
        let infrastructure = Arc::new(Self {
            client,
            jetstream,
            config: config.clone(),
        });
        
        info!("NATS infrastructure initialized successfully");
        Ok(infrastructure)
    }
    
    async fn setup_streams(js: &jetstream::Context, config: &NatsConfig) -> Result<(), Box<dyn std::error::Error>> {
        info!("Setting up JetStream streams");
        
        // Claude Events Stream
        let events_stream = format!("{}_EVENTS", config.subject_prefix.replace(".", "_").to_uppercase());
        js.create_stream(jetstream::stream::Config {
            name: events_stream,
            subjects: vec![format!("{}.event.>", config.subject_prefix)],
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        }).await.or_else(|e| {
            if e.to_string().contains("already exists") {
                Ok(js.get_stream(&format!("{}_EVENTS", config.subject_prefix.replace(".", "_").to_uppercase())).await?)
            } else {
                Err(e)
            }
        })?;
        
        // Claude Commands Stream  
        let commands_stream = format!("{}_COMMANDS", config.subject_prefix.replace(".", "_").to_uppercase());
        js.create_stream(jetstream::stream::Config {
            name: commands_stream,
            subjects: vec![format!("{}.cmd.>", config.subject_prefix)],
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        }).await.or_else(|e| {
            if e.to_string().contains("already exists") {
                Ok(js.get_stream(&format!("{}_COMMANDS", config.subject_prefix.replace(".", "_").to_uppercase())).await?)
            } else {
                Err(e)
            }
        })?;
        
        // Conversation State KV
        let kv_name = format!("{}_STATE", config.subject_prefix.replace(".", "_").to_uppercase());
        js.create_key_value(jetstream::kv::Config {
            bucket: kv_name,
            ..Default::default()
        }).await.or_else(|e| {
            if e.to_string().contains("already exists") {
                Ok(js.get_key_value(&format!("{}_STATE", config.subject_prefix.replace(".", "_").to_uppercase())).await?)
            } else {
                Err(e)
            }
        })?;
        
        info!("JetStream streams created successfully");
        Ok(())
    }
    
    pub fn client(&self) -> &Client {
        &self.client
    }
    
    pub fn jetstream(&self) -> &jetstream::Context {
        &self.jetstream
    }
    
    pub fn config(&self) -> &NatsConfig {
        &self.config
    }
}