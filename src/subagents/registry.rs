/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Subagent Registry
//!
//! Manages the registration, discovery, and lifecycle of Claude expert subagents.
//! Loads agent definitions from .claude/agents/ and provides a central registry
//! for agent lookup and management.

use super::{Subagent, SubagentQuery};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Metadata about a registered subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub capabilities: Vec<SubagentCapability>,
    pub file_path: PathBuf,
    pub version: String,
    pub last_loaded: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Capabilities that subagents can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub enum SubagentCapability {
    Architecture,
    DomainModeling,
    EventStorming,
    Infrastructure,
    NetworkDesign,
    SystemConfiguration,
    MessageBroker,
    DomainCreation,
    MasterOrchestration,
    CategoryTheory,
    GraphTheory,
    EventSourcing,
    CQRS,
    NixConfiguration,
    SecurityHardening,
    Collaboration,
    ProjectManagement,
}

impl From<&str> for SubagentCapability {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "architecture" => SubagentCapability::Architecture,
            "domain-modeling" => SubagentCapability::DomainModeling,
            "event-storming" => SubagentCapability::EventStorming,
            "infrastructure" => SubagentCapability::Infrastructure,
            "network-design" => SubagentCapability::NetworkDesign,
            "system-configuration" => SubagentCapability::SystemConfiguration,
            "message-broker" => SubagentCapability::MessageBroker,
            "domain-creation" => SubagentCapability::DomainCreation,
            "master-orchestration" => SubagentCapability::MasterOrchestration,
            "category-theory" => SubagentCapability::CategoryTheory,
            "graph-theory" => SubagentCapability::GraphTheory,
            "event-sourcing" => SubagentCapability::EventSourcing,
            "cqrs" => SubagentCapability::CQRS,
            "nix-configuration" => SubagentCapability::NixConfiguration,
            "security-hardening" => SubagentCapability::SecurityHardening,
            "collaboration" => SubagentCapability::Collaboration,
            "project-management" => SubagentCapability::ProjectManagement,
            _ => SubagentCapability::Architecture, // Default fallback
        }
    }
}

/// Central registry for managing subagents
#[derive(Clone)]
pub struct SubagentRegistry {
    agents: Arc<RwLock<HashMap<String, Arc<dyn Subagent>>>>,
    agent_info: Arc<RwLock<HashMap<String, SubagentInfo>>>,
    capabilities_index: Arc<RwLock<HashMap<SubagentCapability, Vec<String>>>>,
    agents_directory: PathBuf,
}

impl SubagentRegistry {
    /// Create a new subagent registry
    pub fn new(agents_directory: PathBuf) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            agent_info: Arc::new(RwLock::new(HashMap::new())),
            capabilities_index: Arc::new(RwLock::new(HashMap::new())),
            agents_directory,
        }
    }

    /// Initialize the registry by loading all agent definitions
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing subagent registry from {:?}", self.agents_directory);
        
        if !self.agents_directory.exists() {
            warn!("Agents directory {:?} does not exist", self.agents_directory);
            return Ok(());
        }

        let mut dir = tokio::fs::read_dir(&self.agents_directory).await?;
        let mut loaded_count = 0;

        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                match self.load_agent_from_file(&path).await {
                    Ok(agent_id) => {
                        info!("Loaded subagent: {}", agent_id);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        error!("Failed to load agent from {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("Loaded {} subagents successfully", loaded_count);
        Ok(())
    }

    /// Load a single agent from a markdown file
    async fn load_agent_from_file(&self, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let content = tokio::fs::read_to_string(path).await?;
        let agent_def = self.parse_agent_definition(&content, path)?;
        
        // Create the actual subagent instance
        let registry_ref = if agent_def.id == "sage" {
            Some(Arc::new(self.clone())) // SAGE needs registry reference for coordination
        } else {
            None
        };
        let agent = super::expert_definitions::create_agent_from_definition(agent_def.clone(), registry_ref)?;
        
        let agent_id = agent.id().to_string();
        
        // Register the agent
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id.clone(), Arc::from(agent));
        }

        // Store agent info
        {
            let mut agent_info = self.agent_info.write().await;
            agent_info.insert(agent_id.clone(), agent_def);
        }

        // Update capabilities index
        self.refresh_capabilities_index(&agent_id).await?;

        Ok(agent_id)
    }

    /// Parse agent definition from markdown content
    fn parse_agent_definition(&self, content: &str, path: &Path) -> Result<SubagentInfo, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = content.lines().collect();
        
        // Extract YAML frontmatter
        let mut in_frontmatter = false;
        let mut frontmatter_lines = Vec::new();
        let _description_lines: Vec<&str> = Vec::new();
        
        for line in lines {
            if line.trim() == "---" {
                if !in_frontmatter {
                    in_frontmatter = true;
                } else {
                    break;
                }
                continue;
            }
            
            if in_frontmatter {
                frontmatter_lines.push(line);
            }
        }

        // Parse YAML frontmatter
        let frontmatter = frontmatter_lines.join("\n");
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&frontmatter)?;
        
        let name = yaml_value.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'name' field in agent definition")?
            .to_string();
            
        let description = yaml_value.get("description")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'description' field in agent definition")?
            .to_string();
            
        let tools = yaml_value.get("tools")
            .and_then(|v| v.as_str())
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);

        // Extract capabilities from description and content analysis
        let capabilities = self.extract_capabilities(&description, &content);
        
        let agent_info = SubagentInfo {
            id: name.clone(),
            name: name.clone(),
            description,
            tools,
            capabilities,
            file_path: path.to_path_buf(),
            version: "1.0.0".to_string(),
            last_loaded: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        Ok(agent_info)
    }

    /// Extract capabilities from agent description and content
    fn extract_capabilities(&self, description: &str, content: &str) -> Vec<SubagentCapability> {
        let mut capabilities = Vec::new();
        let text = format!("{} {}", description.to_lowercase(), content.to_lowercase());

        // Map keywords to capabilities
        let capability_keywords = vec![
            (SubagentCapability::Architecture, vec!["architecture", "design", "technical", "system design"]),
            (SubagentCapability::DomainModeling, vec!["domain", "ddd", "domain-driven", "modeling", "boundaries"]),
            (SubagentCapability::EventStorming, vec!["event storming", "events", "collaborative", "discovery"]),
            (SubagentCapability::Infrastructure, vec!["infrastructure", "deployment", "operations", "devops"]),
            (SubagentCapability::NetworkDesign, vec!["network", "topology", "routing", "connectivity"]),
            (SubagentCapability::SystemConfiguration, vec!["configuration", "config", "setup", "installation"]),
            (SubagentCapability::MessageBroker, vec!["nats", "messaging", "broker", "jetstream", "streams"]),
            (SubagentCapability::DomainCreation, vec!["domain creation", "graph", "validation", "implementation"]),
            (SubagentCapability::MasterOrchestration, vec!["orchestration", "coordination", "master", "workflow"]),
            (SubagentCapability::CategoryTheory, vec!["category theory", "mathematical", "composition"]),
            (SubagentCapability::GraphTheory, vec!["graph theory", "nodes", "edges", "traversal"]),
            (SubagentCapability::EventSourcing, vec!["event sourcing", "events", "sourcing", "immutable"]),
            (SubagentCapability::CQRS, vec!["cqrs", "command", "query", "separation"]),
            (SubagentCapability::NixConfiguration, vec!["nix", "nixos", "declarative", "functional"]),
            (SubagentCapability::SecurityHardening, vec!["security", "hardening", "secure", "authentication"]),
            (SubagentCapability::Collaboration, vec!["collaboration", "team", "facilitation", "workshop"]),
            (SubagentCapability::ProjectManagement, vec!["project", "management", "planning", "coordination"]),
        ];

        for (capability, keywords) in capability_keywords {
            for keyword in keywords {
                if text.contains(keyword) {
                    if !capabilities.contains(&capability) {
                        capabilities.push(capability);
                    }
                    break;
                }
            }
        }

        // If no capabilities found, default to Architecture
        if capabilities.is_empty() {
            capabilities.push(SubagentCapability::Architecture);
        }

        capabilities
    }

    /// Refresh capabilities index for quick lookup
    async fn refresh_capabilities_index(&self, agent_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let agent_info = {
            let agent_info_lock = self.agent_info.read().await;
            agent_info_lock.get(agent_id).cloned()
        };

        if let Some(info) = agent_info {
            let mut capabilities_index = self.capabilities_index.write().await;
            
            for capability in &info.capabilities {
                capabilities_index
                    .entry(capability.clone())
                    .or_insert_with(Vec::new)
                    .push(agent_id.to_string());
            }
        }

        Ok(())
    }

    /// Get all registered agents
    pub async fn get_all_agents(&self) -> Vec<SubagentInfo> {
        let agent_info = self.agent_info.read().await;
        agent_info.values().cloned().collect()
    }

    /// Get agent by ID
    pub async fn get_agent(&self, agent_id: &str) -> Option<Arc<dyn Subagent>> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Get agent info by ID
    pub async fn get_agent_info(&self, agent_id: &str) -> Option<SubagentInfo> {
        let agent_info = self.agent_info.read().await;
        agent_info.get(agent_id).cloned()
    }

    /// Find agents by capability
    pub async fn find_agents_by_capability(&self, capability: &SubagentCapability) -> Vec<String> {
        let capabilities_index = self.capabilities_index.read().await;
        capabilities_index.get(capability).cloned().unwrap_or_default()
    }

    /// Find best agents for a query
    pub async fn find_best_agents_for_query(&self, query: &SubagentQuery) -> Vec<(String, u32)> {
        let agents = self.agents.read().await;
        let mut scored_agents = Vec::new();

        for (agent_id, agent) in agents.iter() {
            if agent.can_handle(query) {
                let score = agent.priority_score(query);
                scored_agents.push((agent_id.clone(), score));
            }
        }

        // Sort by score descending
        scored_agents.sort_by(|a, b| b.1.cmp(&a.1));
        scored_agents
    }

    /// Register a new agent dynamically
    pub async fn register_agent(&self, agent: Arc<dyn Subagent>) -> Result<(), Box<dyn std::error::Error>> {
        let agent_id = agent.id().to_string();
        
        // Create agent info
        let agent_info = SubagentInfo {
            id: agent_id.clone(),
            name: agent.name().to_string(),
            description: agent.description().to_string(),
            tools: agent.available_tools(),
            capabilities: agent.capabilities(),
            file_path: PathBuf::new(), // Not loaded from file
            version: "1.0.0".to_string(),
            last_loaded: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id.clone(), agent);
        }

        {
            let mut agent_info_lock = self.agent_info.write().await;
            agent_info_lock.insert(agent_id.clone(), agent_info);
        }

        self.refresh_capabilities_index(&agent_id).await?;

        Ok(())
    }

    /// Remove an agent from the registry
    pub async fn unregister_agent(&self, agent_id: &str) -> bool {
        let removed_from_agents = {
            let mut agents = self.agents.write().await;
            agents.remove(agent_id).is_some()
        };

        let removed_from_info = {
            let mut agent_info = self.agent_info.write().await;
            agent_info.remove(agent_id).is_some()
        };

        // Clean up capabilities index
        {
            let mut capabilities_index = self.capabilities_index.write().await;
            for agents_list in capabilities_index.values_mut() {
                agents_list.retain(|id| id != agent_id);
            }
        }

        removed_from_agents && removed_from_info
    }

    /// Get registry statistics
    pub async fn get_stats(&self) -> RegistryStats {
        let agents = self.agents.read().await;
        let capabilities_index = self.capabilities_index.read().await;

        RegistryStats {
            total_agents: agents.len(),
            capabilities_count: capabilities_index.len(),
            agents_by_capability: capabilities_index
                .iter()
                .map(|(cap, agents)| (format!("{:?}", cap), agents.len()))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_agents: usize,
    pub capabilities_count: usize,
    pub agents_by_capability: HashMap<String, usize>,
}