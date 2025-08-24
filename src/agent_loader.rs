//! Agent Personality Loader
//! 
//! Loads agent personalities from markdown files in .claude/agents/
//! Parses YAML frontmatter and extracts system prompts

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn, error};

/// Agent metadata from YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub name: String,
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub tools: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

/// Deserialize either a single string or a vector of strings
fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    
    struct StringOrVec;
    
    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or list of strings")
        }
        
        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // Split comma-separated string into vec
            Ok(s.split(',').map(|s| s.trim().to_string()).collect())
        }
        
        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            Vec::deserialize(de::value::SeqAccessDeserializer::new(seq))
        }
    }
    
    deserializer.deserialize_any(StringOrVec)
}

/// Complete agent personality with metadata and prompt
#[derive(Debug, Clone)]
pub struct AgentPersonality {
    pub id: String,
    pub metadata: AgentMetadata,
    pub system_prompt: String,
    pub file_path: PathBuf,
}

/// Agent loader for reading personality files
pub struct AgentLoader {
    agents_dir: PathBuf,
    loaded_agents: HashMap<String, AgentPersonality>,
}

impl AgentLoader {
    /// Create new agent loader
    pub fn new(agents_dir: impl AsRef<Path>) -> Self {
        Self {
            agents_dir: agents_dir.as_ref().to_path_buf(),
            loaded_agents: HashMap::new(),
        }
    }
    
    /// Load all agent personalities from directory
    pub async fn load_all_agents(&mut self) -> Result<Vec<AgentPersonality>> {
        info!("Loading agent personalities from: {:?}", self.agents_dir);
        
        // Read all .md files in directory
        let entries = fs::read_dir(&self.agents_dir)
            .context("Failed to read agents directory")?;
        
        let mut agents = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Skip non-markdown files
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            
            // Load agent from file
            match self.load_agent_from_file(&path).await {
                Ok(agent) => {
                    info!("Loaded agent: {} from {:?}", agent.id, path);
                    self.loaded_agents.insert(agent.id.clone(), agent.clone());
                    agents.push(agent);
                }
                Err(e) => {
                    error!("Failed to load agent from {:?}: {}", path, e);
                }
            }
        }
        
        info!("Successfully loaded {} agent personalities", agents.len());
        Ok(agents)
    }
    
    /// Load a single agent from a markdown file
    pub async fn load_agent_from_file(&self, path: &Path) -> Result<AgentPersonality> {
        // Read file content
        let content = fs::read_to_string(path)
            .context(format!("Failed to read agent file: {:?}", path))?;
        
        // Parse frontmatter and content
        let (metadata, system_prompt) = self.parse_agent_markdown(&content)?;
        
        // Extract agent ID from filename
        let id = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid agent filename"))?
            .to_string();
        
        Ok(AgentPersonality {
            id,
            metadata,
            system_prompt,
            file_path: path.to_path_buf(),
        })
    }
    
    /// Parse markdown content with YAML frontmatter
    fn parse_agent_markdown(&self, content: &str) -> Result<(AgentMetadata, String)> {
        // Check for frontmatter delimiters
        if !content.starts_with("---\n") {
            return Err(anyhow::anyhow!("Agent file missing YAML frontmatter"));
        }
        
        // Find end of frontmatter
        let end_marker = "\n---\n";
        let end_pos = content[4..].find(end_marker)
            .ok_or_else(|| anyhow::anyhow!("Invalid YAML frontmatter format"))?;
        
        // Extract YAML content
        let yaml_content = &content[4..4 + end_pos];
        
        // Parse YAML metadata
        let metadata: AgentMetadata = serde_yaml::from_str(yaml_content)
            .context("Failed to parse agent metadata")?;
        
        // Extract system prompt (everything after frontmatter)
        let prompt_start = 4 + end_pos + end_marker.len();
        let system_prompt = content[prompt_start..].trim().to_string();
        
        Ok((metadata, system_prompt))
    }
    
    /// Get loaded agent by ID
    pub fn get_agent(&self, id: &str) -> Option<&AgentPersonality> {
        self.loaded_agents.get(id)
    }
    
    /// Get all loaded agents
    pub fn get_all_agents(&self) -> Vec<&AgentPersonality> {
        self.loaded_agents.values().collect()
    }
    
    /// Find agents matching keywords
    pub fn find_agents_by_keywords(&self, query: &str) -> Vec<&AgentPersonality> {
        let query_lower = query.to_lowercase();
        
        self.loaded_agents.values()
            .filter(|agent| {
                // Check if query matches agent keywords
                agent.metadata.keywords.iter()
                    .any(|kw| query_lower.contains(&kw.to_lowercase())) ||
                // Check if query matches agent name
                query_lower.contains(&agent.id.replace('-', " ")) ||
                // Check if query matches capabilities
                agent.metadata.capabilities.iter()
                    .any(|cap| query_lower.contains(&cap.to_lowercase()))
            })
            .collect()
    }
    
    /// Reload a specific agent from disk
    pub async fn reload_agent(&mut self, id: &str) -> Result<AgentPersonality> {
        let path = self.agents_dir.join(format!("{}.md", id));
        let agent = self.load_agent_from_file(&path).await?;
        self.loaded_agents.insert(id.to_string(), agent.clone());
        Ok(agent)
    }
}

/// Agent selector for choosing appropriate agents
pub struct AgentSelector {
    loader: AgentLoader,
}

impl AgentSelector {
    /// Create new agent selector
    pub fn new(loader: AgentLoader) -> Self {
        Self { loader }
    }
    
    /// Select best agent for a query
    pub fn select_agent(&self, query: &str, requested_agent: Option<&str>) -> Option<&AgentPersonality> {
        // If specific agent requested, use that
        if let Some(agent_id) = requested_agent {
            if let Some(agent) = self.loader.get_agent(agent_id) {
                return Some(agent);
            }
        }
        
        // Otherwise find best matching agent
        let matching_agents = self.loader.find_agents_by_keywords(query);
        
        if matching_agents.is_empty() {
            // Default to SAGE if no specific match
            self.loader.get_agent("sage")
        } else if matching_agents.len() == 1 {
            Some(matching_agents[0])
        } else {
            // Multiple matches - use SAGE to orchestrate
            self.loader.get_agent("sage")
        }
    }
    
    /// Get agent for orchestration (always SAGE)
    pub fn get_orchestrator(&self) -> Option<&AgentPersonality> {
        self.loader.get_agent("sage")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[tokio::test]
    async fn test_load_agent_from_file() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let agent_file = temp_dir.path().join("test-agent.md");
        
        // Write test agent file
        let content = r#"---
name: test-agent
description: Test agent for unit testing
tools: [Read, Write]
capabilities: [testing, validation]
keywords: [test, validate, check]
---

You are a test agent designed for unit testing purposes.
Your primary role is to validate functionality."#;
        
        fs::write(&agent_file, content).unwrap();
        
        // Load agent
        let loader = AgentLoader::new(temp_dir.path());
        let agent = loader.load_agent_from_file(&agent_file).await.unwrap();
        
        // Verify loaded data
        assert_eq!(agent.id, "test-agent");
        assert_eq!(agent.metadata.name, "test-agent");
        assert_eq!(agent.metadata.tools.len(), 2);
        assert!(agent.system_prompt.contains("test agent"));
    }
    
    #[tokio::test]
    async fn test_find_agents_by_keywords() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple test agents
        let ddd_content = r#"---
name: ddd-expert
description: Domain-Driven Design expert
keywords: [domain, ddd, aggregate, bounded context]
---
DDD expert prompt"#;
        
        let nats_content = r#"---
name: nats-expert  
description: NATS messaging expert
keywords: [nats, messaging, events, streaming]
---
NATS expert prompt"#;
        
        fs::write(temp_dir.path().join("ddd-expert.md"), ddd_content).unwrap();
        fs::write(temp_dir.path().join("nats-expert.md"), nats_content).unwrap();
        
        // Load agents
        let mut loader = AgentLoader::new(temp_dir.path());
        loader.load_all_agents().await.unwrap();
        
        // Test keyword matching
        let domain_agents = loader.find_agents_by_keywords("domain modeling");
        assert_eq!(domain_agents.len(), 1);
        assert_eq!(domain_agents[0].id, "ddd-expert");
        
        let messaging_agents = loader.find_agents_by_keywords("event streaming");
        assert_eq!(messaging_agents.len(), 1);
        assert_eq!(messaging_agents[0].id, "nats-expert");
    }
}