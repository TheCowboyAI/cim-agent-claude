/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! NATS JetStream Configuration for Claude Adapter
//! 
//! Defines all streams, object stores, and KV stores using the CIM subject algebra

use serde::{Deserialize, Serialize};

/// Complete NATS JetStream configuration for Claude adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfiguration {
    pub streams: Vec<StreamDefinition>,
    pub object_stores: Vec<ObjectStoreDefinition>,
    pub kv_stores: Vec<KvStoreDefinition>,
    pub consumers: Vec<ConsumerDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDefinition {
    pub name: String,
    pub subjects: Vec<String>,
    pub description: String,
    pub retention: RetentionPolicy,
    pub max_consumers: Option<i32>,
    pub max_msgs: Option<i64>,
    pub max_bytes: Option<i64>,
    pub max_age: Option<String>, // Duration string like "24h", "30d"
    pub max_msg_size: Option<i32>,
    pub storage: StorageType,
    pub num_replicas: Option<u32>,
    pub no_ack: Option<bool>,
    pub discard: Option<DiscardPolicy>,
    pub duplicate_window: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStoreDefinition {
    pub name: String,
    pub description: String,
    pub max_bytes: Option<i64>,
    pub storage: StorageType,
    pub num_replicas: Option<u32>,
    pub ttl: Option<String>,
    pub compression: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvStoreDefinition {
    pub name: String,
    pub description: String,
    pub max_bytes: Option<i64>,
    pub history: Option<u8>,
    pub ttl: Option<String>,
    pub storage: StorageType,
    pub num_replicas: Option<u32>,
    pub compression: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerDefinition {
    pub name: String,
    pub stream: String,
    pub description: String,
    pub filter_subject: Option<String>,
    pub deliver_policy: DeliverPolicy,
    pub ack_policy: AckPolicy,
    pub ack_wait: Option<String>,
    pub max_deliver: Option<i32>,
    pub replay_policy: Option<ReplayPolicy>,
    pub sample_freq: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RetentionPolicy {
    Limits,
    Interest,
    Workqueue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    File,
    Memory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscardPolicy {
    Old,
    New,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliverPolicy {
    All,
    Last,
    New,
    ByStartSequence,
    ByStartTime,
    LastPerSubject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AckPolicy {
    None,
    All,
    Explicit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplayPolicy {
    Instant,
    Original,
}

impl NatsConfiguration {
    /// Create the complete NATS configuration for Claude adapter
    pub fn for_claude_adapter() -> Self {
        Self {
            streams: Self::create_streams(),
            object_stores: Self::create_object_stores(),
            kv_stores: Self::create_kv_stores(),
            consumers: Self::create_consumers(),
        }
    }

    fn create_streams() -> Vec<StreamDefinition> {
        vec![
            // Claude conversation commands
            StreamDefinition {
                name: "CIM_CLAUDE_CONV_CMD".to_string(),
                subjects: vec![
                    "cim.claude.conv.cmd.>".to_string(),
                ],
                description: "Claude conversation commands (start, send, end)".to_string(),
                retention: RetentionPolicy::Workqueue, // Commands are processed once
                max_consumers: Some(10),
                max_msgs: Some(1_000_000),
                max_bytes: Some(10 * 1024 * 1024 * 1024), // 10GB
                max_age: Some("30d".to_string()),
                max_msg_size: Some(64 * 1024 * 1024), // 64MB for large prompts
                storage: StorageType::File,
                num_replicas: Some(3),
                no_ack: Some(false),
                discard: Some(DiscardPolicy::Old),
                duplicate_window: Some("2m".to_string()),
            },

            // Claude conversation events (permanent audit trail)
            StreamDefinition {
                name: "CIM_CLAUDE_CONV_EVT".to_string(),
                subjects: vec![
                    "cim.claude.conv.evt.>".to_string(),
                ],
                description: "Claude conversation events (permanent audit trail)".to_string(),
                retention: RetentionPolicy::Limits, // Keep all events
                max_consumers: Some(50),
                max_msgs: Some(10_000_000),
                max_bytes: Some(100 * 1024 * 1024 * 1024), // 100GB
                max_age: Some("2y".to_string()), // Keep for 2 years
                max_msg_size: Some(16 * 1024 * 1024), // 16MB for responses
                storage: StorageType::File,
                num_replicas: Some(3),
                no_ack: Some(false),
                discard: Some(DiscardPolicy::Old),
                duplicate_window: Some("1h".to_string()),
            },

            // Attachment commands
            StreamDefinition {
                name: "CIM_CLAUDE_ATTACH_CMD".to_string(),
                subjects: vec![
                    "cim.claude.attach.cmd.>".to_string(),
                ],
                description: "Attachment upload and processing commands".to_string(),
                retention: RetentionPolicy::Workqueue,
                max_consumers: Some(5),
                max_msgs: Some(100_000),
                max_bytes: Some(5 * 1024 * 1024 * 1024), // 5GB
                max_age: Some("7d".to_string()),
                max_msg_size: Some(256 * 1024 * 1024), // 256MB for large files
                storage: StorageType::File,
                num_replicas: Some(3),
                no_ack: Some(false),
                discard: Some(DiscardPolicy::New), // Drop new if full
                duplicate_window: Some("5m".to_string()),
            },

            // Attachment events
            StreamDefinition {
                name: "CIM_CLAUDE_ATTACH_EVT".to_string(),
                subjects: vec![
                    "cim.claude.attach.evt.>".to_string(),
                ],
                description: "Attachment processing events".to_string(),
                retention: RetentionPolicy::Limits,
                max_consumers: Some(20),
                max_msgs: Some(1_000_000),
                max_bytes: Some(20 * 1024 * 1024 * 1024), // 20GB
                max_age: Some("1y".to_string()),
                max_msg_size: Some(4 * 1024 * 1024), // 4MB for metadata
                storage: StorageType::File,
                num_replicas: Some(3),
                no_ack: Some(false),
                discard: Some(DiscardPolicy::Old),
                duplicate_window: Some("30m".to_string()),
            },

            // Query requests (request/reply pattern)
            StreamDefinition {
                name: "CIM_CLAUDE_CONV_QRY".to_string(),
                subjects: vec![
                    "cim.claude.conv.qry.>".to_string(),
                    "cim.claude.attach.qry.>".to_string(),
                ],
                description: "Query requests for conversation and attachment data".to_string(),
                retention: RetentionPolicy::Workqueue,
                max_consumers: Some(20),
                max_msgs: Some(500_000),
                max_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB
                max_age: Some("24h".to_string()), // Queries are short-lived
                max_msg_size: Some(1024 * 1024), // 1MB for queries
                storage: StorageType::Memory, // Fast access for queries
                num_replicas: Some(3),
                no_ack: Some(false),
                discard: Some(DiscardPolicy::Old),
                duplicate_window: Some("1m".to_string()),
            },

            // System monitoring and health
            StreamDefinition {
                name: "CIM_SYS_HEALTH_EVT".to_string(),
                subjects: vec![
                    "cim.sys.health.evt.>".to_string(),
                    "cim.sys.metrics.evt.>".to_string(),
                ],
                description: "System health and metrics events".to_string(),
                retention: RetentionPolicy::Limits,
                max_consumers: Some(10),
                max_msgs: Some(10_000_000),
                max_bytes: Some(10 * 1024 * 1024 * 1024), // 10GB
                max_age: Some("90d".to_string()),
                max_msg_size: Some(1024 * 1024), // 1MB for metrics
                storage: StorageType::File,
                num_replicas: Some(2), // Less critical, fewer replicas
                no_ack: Some(false),
                discard: Some(DiscardPolicy::Old),
                duplicate_window: Some("10s".to_string()),
            },
        ]
    }

    fn create_object_stores() -> Vec<ObjectStoreDefinition> {
        vec![
            // Image attachments (screenshots, photos)
            ObjectStoreDefinition {
                name: "CIM_CLAUDE_ATTACH_OBJ_IMG".to_string(),
                description: "Image attachments and screenshots".to_string(),
                max_bytes: Some(500 * 1024 * 1024 * 1024), // 500GB
                storage: StorageType::File,
                num_replicas: Some(3),
                ttl: Some("2y".to_string()), // Keep images for 2 years
                compression: Some(false), // Images are already compressed
            },

            // Document attachments (PDFs, text, etc.)
            ObjectStoreDefinition {
                name: "CIM_CLAUDE_ATTACH_OBJ_DOC".to_string(),
                description: "Document attachments (PDF, text, office docs)".to_string(),
                max_bytes: Some(200 * 1024 * 1024 * 1024), // 200GB
                storage: StorageType::File,
                num_replicas: Some(3),
                ttl: Some("3y".to_string()), // Keep documents longer
                compression: Some(true), // Compress documents
            },

            // Code and text files
            ObjectStoreDefinition {
                name: "CIM_CLAUDE_ATTACH_OBJ_CODE".to_string(),
                description: "Code files and text attachments".to_string(),
                max_bytes: Some(50 * 1024 * 1024 * 1024), // 50GB
                storage: StorageType::File,
                num_replicas: Some(2),
                ttl: Some("1y".to_string()),
                compression: Some(true), // Highly compressible
            },

            // Audio attachments
            ObjectStoreDefinition {
                name: "CIM_CLAUDE_ATTACH_OBJ_AUDIO".to_string(),
                description: "Audio file attachments".to_string(),
                max_bytes: Some(100 * 1024 * 1024 * 1024), // 100GB
                storage: StorageType::File,
                num_replicas: Some(2),
                ttl: Some("1y".to_string()),
                compression: Some(false), // Audio already compressed
            },

            // Video attachments
            ObjectStoreDefinition {
                name: "CIM_CLAUDE_ATTACH_OBJ_VIDEO".to_string(),
                description: "Video file attachments".to_string(),
                max_bytes: Some(1000 * 1024 * 1024 * 1024), // 1TB
                storage: StorageType::File,
                num_replicas: Some(2),
                ttl: Some("6m".to_string()), // Videos expire faster
                compression: Some(false), // Videos already compressed
            },

            // Binary and archive files
            ObjectStoreDefinition {
                name: "CIM_CLAUDE_ATTACH_OBJ_BIN".to_string(),
                description: "Binary and archive file attachments".to_string(),
                max_bytes: Some(100 * 1024 * 1024 * 1024), // 100GB
                storage: StorageType::File,
                num_replicas: Some(2),
                ttl: Some("90d".to_string()), // Short-lived binaries
                compression: Some(true), // May help with some binary formats
            },
        ]
    }

    fn create_kv_stores() -> Vec<KvStoreDefinition> {
        vec![
            // Conversation metadata and state
            KvStoreDefinition {
                name: "CIM_CLAUDE_CONV_KV".to_string(),
                description: "Conversation metadata, state, and quick lookups".to_string(),
                max_bytes: Some(20 * 1024 * 1024 * 1024), // 20GB
                history: Some(10), // Keep last 10 versions
                ttl: Some("2y".to_string()),
                storage: StorageType::File,
                num_replicas: Some(3),
                compression: Some(true),
            },

            // Attachment metadata and references
            KvStoreDefinition {
                name: "CIM_CLAUDE_ATTACH_KV".to_string(),
                description: "Attachment metadata, references, and indexing data".to_string(),
                max_bytes: Some(5 * 1024 * 1024 * 1024), // 5GB
                history: Some(5), // Keep last 5 versions
                ttl: Some("2y".to_string()),
                storage: StorageType::File,
                num_replicas: Some(3),
                compression: Some(true),
            },

            // User sessions and preferences
            KvStoreDefinition {
                name: "CIM_CLAUDE_SESSION_KV".to_string(),
                description: "User session data and preferences".to_string(),
                max_bytes: Some(10 * 1024 * 1024 * 1024), // 10GB
                history: Some(3), // Keep last 3 versions
                ttl: Some("30d".to_string()), // Sessions expire monthly
                storage: StorageType::Memory, // Fast session access
                num_replicas: Some(3),
                compression: Some(true),
            },

            // Configuration and settings
            KvStoreDefinition {
                name: "CIM_CLAUDE_CONFIG_KV".to_string(),
                description: "Configuration settings and feature flags".to_string(),
                max_bytes: Some(1024 * 1024 * 1024), // 1GB
                history: Some(20), // Keep more history for config
                ttl: None, // Config doesn't expire
                storage: StorageType::File,
                num_replicas: Some(3),
                compression: Some(true),
            },

            // Usage metrics and analytics (aggregated)
            KvStoreDefinition {
                name: "CIM_CLAUDE_METRICS_KV".to_string(),
                description: "Aggregated usage metrics and analytics data".to_string(),
                max_bytes: Some(50 * 1024 * 1024 * 1024), // 50GB
                history: Some(30), // Keep monthly snapshots
                ttl: Some("3y".to_string()), // Keep metrics for analysis
                storage: StorageType::File,
                num_replicas: Some(2),
                compression: Some(true),
            },
            
            // Active configuration state (separate from general config)
            KvStoreDefinition {
                name: "CIM_CLAUDE_CONFIG_ACTIVE_KV".to_string(),
                description: "Active configuration state (system prompts, model parameters)".to_string(),
                max_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB
                history: Some(50), // Keep extensive configuration history
                ttl: None, // Configuration doesn't expire
                storage: StorageType::File,
                num_replicas: Some(3),
                compression: Some(true),
            },
            
            // MCP tool registry
            KvStoreDefinition {
                name: "CIM_CLAUDE_MCP_TOOLS_KV".to_string(),
                description: "MCP tool registry and configuration".to_string(),
                max_bytes: Some(5 * 1024 * 1024 * 1024), // 5GB
                history: Some(20), // Keep tool version history
                ttl: Some("1y".to_string()),
                storage: StorageType::File,
                num_replicas: Some(3),
                compression: Some(true),
            },
            
            // Conversation control state
            KvStoreDefinition {
                name: "CIM_USER_CONV_CONTROL_KV".to_string(),
                description: "Conversation control state and metadata".to_string(),
                max_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB
                history: Some(15), // Keep control history
                ttl: Some("1y".to_string()),
                storage: StorageType::File,
                num_replicas: Some(3),
                compression: Some(true),
            },
        ]
    }

    fn create_consumers() -> Vec<ConsumerDefinition> {
        vec![
            // Command processors
            ConsumerDefinition {
                name: "claude_conversation_commands".to_string(),
                stream: "CIM_CLAUDE_CONV_CMD".to_string(),
                description: "Process conversation commands".to_string(),
                filter_subject: Some("cim.claude.conv.cmd.>".to_string()),
                deliver_policy: DeliverPolicy::All,
                ack_policy: AckPolicy::Explicit,
                ack_wait: Some("30s".to_string()),
                max_deliver: Some(5),
                replay_policy: Some(ReplayPolicy::Instant),
                sample_freq: None,
            },

            ConsumerDefinition {
                name: "claude_attachment_commands".to_string(),
                stream: "CIM_CLAUDE_ATTACH_CMD".to_string(),
                description: "Process attachment commands".to_string(),
                filter_subject: Some("cim.claude.attach.cmd.>".to_string()),
                deliver_policy: DeliverPolicy::All,
                ack_policy: AckPolicy::Explicit,
                ack_wait: Some("5m".to_string()), // Longer for file processing
                max_deliver: Some(3),
                replay_policy: Some(ReplayPolicy::Instant),
                sample_freq: None,
            },

            // Event processors (for analytics, logging, etc.)
            ConsumerDefinition {
                name: "claude_event_logger".to_string(),
                stream: "CIM_CLAUDE_CONV_EVT".to_string(),
                description: "Log conversation events for audit".to_string(),
                filter_subject: Some("cim.claude.conv.evt.>".to_string()),
                deliver_policy: DeliverPolicy::All,
                ack_policy: AckPolicy::Explicit,
                ack_wait: Some("1m".to_string()),
                max_deliver: Some(10),
                replay_policy: Some(ReplayPolicy::Instant),
                sample_freq: None,
            },

            ConsumerDefinition {
                name: "claude_metrics_collector".to_string(),
                stream: "CIM_CLAUDE_CONV_EVT".to_string(),
                description: "Collect metrics from conversation events".to_string(),
                filter_subject: Some("cim.claude.conv.evt.>".to_string()),
                deliver_policy: DeliverPolicy::All,
                ack_policy: AckPolicy::Explicit,
                ack_wait: Some("30s".to_string()),
                max_deliver: Some(5),
                replay_policy: Some(ReplayPolicy::Instant),
                sample_freq: Some("10%".to_string()), // Sample for performance
            },

            // Query processors
            ConsumerDefinition {
                name: "claude_query_processor".to_string(),
                stream: "CIM_CLAUDE_CONV_QRY".to_string(),
                description: "Process query requests".to_string(),
                filter_subject: Some("cim.claude.*.qry.>".to_string()),
                deliver_policy: DeliverPolicy::All,
                ack_policy: AckPolicy::Explicit,
                ack_wait: Some("10s".to_string()), // Fast query processing
                max_deliver: Some(3),
                replay_policy: Some(ReplayPolicy::Instant),
                sample_freq: None,
            },

            // Health monitoring
            ConsumerDefinition {
                name: "system_health_monitor".to_string(),
                stream: "CIM_SYS_HEALTH_EVT".to_string(),
                description: "Monitor system health events".to_string(),
                filter_subject: Some("cim.sys.health.evt.>".to_string()),
                deliver_policy: DeliverPolicy::New, // Only new health events
                ack_policy: AckPolicy::Explicit,
                ack_wait: Some("5s".to_string()),
                max_deliver: Some(2),
                replay_policy: Some(ReplayPolicy::Instant),
                sample_freq: None,
            },
        ]
    }

    /// Generate the complete YAML configuration for NATS
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Generate configuration for a specific environment
    pub fn for_environment(env: &str) -> Self {
        let mut config = Self::for_claude_adapter();
        
        match env {
            "development" => {
                // Reduce resources for development
                for stream in &mut config.streams {
                    stream.max_bytes = stream.max_bytes.map(|b| b / 10); // 10x smaller
                    stream.max_msgs = stream.max_msgs.map(|m| m / 10);
                    stream.num_replicas = Some(1); // Single replica for dev
                }
                
                for os in &mut config.object_stores {
                    os.max_bytes = os.max_bytes.map(|b| b / 100); // 100x smaller
                    os.num_replicas = Some(1);
                    os.ttl = Some("7d".to_string()); // Short TTL for dev
                }
                
                for kv in &mut config.kv_stores {
                    kv.max_bytes = kv.max_bytes.map(|b| b / 10);
                    kv.num_replicas = Some(1);
                    if kv.name.contains("SESSION") {
                        kv.ttl = Some("1d".to_string()); // Very short sessions
                    }
                }
            },
            
            "staging" => {
                // Medium resources for staging
                for stream in &mut config.streams {
                    stream.max_bytes = stream.max_bytes.map(|b| b / 2); // Half size
                    stream.num_replicas = Some(2); // Dual replica
                }
                
                for os in &mut config.object_stores {
                    os.max_bytes = os.max_bytes.map(|b| b / 5); // 5x smaller
                    os.num_replicas = Some(2);
                }
                
                for kv in &mut config.kv_stores {
                    kv.max_bytes = kv.max_bytes.map(|b| b / 2);
                    kv.num_replicas = Some(2);
                }
            },
            
            "production" => {
                // Full resources - already configured above
            },
            
            _ => {
                // Unknown environment, use production settings
            }
        }
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_creation() {
        let config = NatsConfiguration::for_claude_adapter();
        assert!(!config.streams.is_empty());
        assert!(!config.object_stores.is_empty());
        assert!(!config.kv_stores.is_empty());
        assert!(!config.consumers.is_empty());
    }

    #[test]
    fn test_environment_configuration() {
        let dev_config = NatsConfiguration::for_environment("development");
        let prod_config = NatsConfiguration::for_environment("production");
        
        // Development should have smaller limits
        assert_eq!(dev_config.streams[0].num_replicas, Some(1));
        assert_eq!(prod_config.streams[0].num_replicas, Some(3));
    }

    #[test]
    fn test_yaml_serialization() {
        let config = NatsConfiguration::for_environment("development");
        let yaml = config.to_yaml().unwrap();
        assert!(yaml.contains("CIM_CLAUDE_CONV_CMD"));
        assert!(yaml.contains("CIM_CLAUDE_ATTACH_OBJ_IMG"));
        assert!(yaml.contains("CIM_CLAUDE_CONV_KV"));
    }
}