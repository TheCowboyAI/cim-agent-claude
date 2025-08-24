/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Agent Personality Loader
//! 
//! Parses agent personalities from `.claude/agents/*.md` files using 
//! markdown frontmatter and content analysis.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs;
use regex::Regex;

use crate::agent_system::{
    AgentPersonality, InvocationPattern, CompositionRule, CompositionType,
    AgentId, Capability, AgentError, AgentResult,
};

/// Agent personality loader that parses markdown configurations
#[derive(Debug)]
pub struct AgentLoader {
    agents_directory: PathBuf,
    frontmatter_regex: Regex,
    capability_regex: Regex,
}

impl AgentLoader {
    /// Create a new agent loader
    pub fn new() -> Self {
        Self {
            agents_directory: PathBuf::from(".claude/agents"),
            frontmatter_regex: Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n").unwrap(),
            capability_regex: Regex::new(r"@([a-zA-Z-]+)").unwrap(),
        }
    }
    
    /// Create agent loader with custom directory
    pub fn with_directory<P: AsRef<Path>>(directory: P) -> Self {
        Self {
            agents_directory: directory.as_ref().to_path_buf(),
            frontmatter_regex: Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n").unwrap(),
            capability_regex: Regex::new(r"@([a-zA-Z-]+)").unwrap(),
        }
    }
    
    /// Load all agent personalities from the directory
    pub async fn load_all_agents(&self) -> AgentResult<HashMap<AgentId, AgentPersonality>> {
        let mut agents = HashMap::new();
        
        let mut entries = match fs::read_dir(&self.agents_directory).await {
            Ok(entries) => entries,
            Err(e) => return Err(AgentError::LoadError(
                format!("Failed to read agents directory: {}", e)
            )),
        };
        
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                match self.load_from_file(&path).await {
                    Ok(personality) => {
                        agents.insert(personality.id.clone(), personality);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load agent from {:?}: {}", path, e);
                        // Continue loading other agents
                    }
                }
            }
        }
        
        Ok(agents)
    }
    
    /// Load agent personality from a specific markdown file
    pub async fn load_from_file<P: AsRef<Path>>(&self, path: P) -> AgentResult<AgentPersonality> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).await
            .map_err(|e| AgentError::LoadError(format!("Failed to read file {:?}: {}", path, e)))?;
        
        self.parse_agent_from_content(&content, path)
    }
    
    /// Parse agent personality from markdown content
    pub fn parse_agent_from_content(&self, content: &str, path: &Path) -> AgentResult<AgentPersonality> {
        // Extract agent ID from filename
        let agent_id = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| AgentError::InvalidConfiguration("Invalid filename".to_string()))?
            .to_string();
        
        // Parse frontmatter if present
        let (frontmatter, main_content) = self.extract_frontmatter(content);
        
        // Create base personality
        let mut personality = AgentPersonality::new(agent_id.clone(), agent_id.clone());
        
        // Apply frontmatter configuration
        if let Some(fm) = frontmatter {
            self.apply_frontmatter(&mut personality, &fm)?;
        }
        
        // Parse main content
        self.parse_main_content(&mut personality, &main_content)?;
        
        // Extract capabilities from content
        self.extract_capabilities(&mut personality, content);
        
        // Generate invocation patterns
        self.generate_invocation_patterns(&mut personality, content);
        
        // Set default composition rules
        self.set_default_composition_rules(&mut personality);
        
        Ok(personality)
    }
    
    /// Extract YAML frontmatter from markdown
    fn extract_frontmatter(&self, content: &str) -> (Option<HashMap<String, serde_yaml::Value>>, String) {
        if let Some(captures) = self.frontmatter_regex.captures(content) {
            let frontmatter_str = captures.get(1).unwrap().as_str();
            let main_content = content[captures.get(0).unwrap().end()..].to_string();
            
            match serde_yaml::from_str(frontmatter_str) {
                Ok(fm) => (Some(fm), main_content),
                Err(_) => (None, content.to_string()),
            }
        } else {
            (None, content.to_string())
        }
    }
    
    /// Apply frontmatter configuration to personality
    fn apply_frontmatter(
        &self, 
        personality: &mut AgentPersonality, 
        frontmatter: &HashMap<String, serde_yaml::Value>
    ) -> AgentResult<()> {
        if let Some(name) = frontmatter.get("name").and_then(|v| v.as_str()) {
            personality.name = name.to_string();
        }
        
        if let Some(description) = frontmatter.get("description").and_then(|v| v.as_str()) {
            personality.description = description.to_string();
        }
        
        if let Some(icon) = frontmatter.get("icon").and_then(|v| v.as_str()) {
            personality.icon = icon.to_string();
        }
        
        if let Some(capabilities) = frontmatter.get("capabilities").and_then(|v| v.as_sequence()) {
            for cap in capabilities {
                if let Some(cap_str) = cap.as_str() {
                    personality.capabilities.insert(cap_str.to_string());
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse main markdown content
    fn parse_main_content(&self, personality: &mut AgentPersonality, content: &str) -> AgentResult<()> {
        // Find the main system prompt section
        let lines: Vec<&str> = content.lines().collect();
        let mut system_prompt_lines = Vec::new();
        let mut in_system_prompt = false;
        
        for line in lines {
            if line.trim().starts_with("# ") {
                // Extract name from title if not set
                if personality.name == personality.id {
                    let title = line.trim_start_matches("# ").trim();
                    if !title.is_empty() {
                        personality.name = title.to_string();
                    }
                }
            } else if line.trim().starts_with("## ") {
                in_system_prompt = line.to_lowercase().contains("system") || 
                                 line.to_lowercase().contains("instruction") ||
                                 line.to_lowercase().contains("prompt");
            } else if line.trim().starts_with("# ") && in_system_prompt {
                in_system_prompt = false;
            } else if in_system_prompt {
                system_prompt_lines.push(line);
            }
        }
        
        // If no explicit system prompt section, use entire content
        if system_prompt_lines.is_empty() {
            personality.system_prompt = content.to_string();
        } else {
            personality.system_prompt = system_prompt_lines.join("\n");
        }
        
        Ok(())
    }
    
    /// Extract capabilities from agent references in content
    fn extract_capabilities(&self, personality: &mut AgentPersonality, content: &str) {
        // Find @agent-name references
        for cap in self.capability_regex.captures_iter(content) {
            if let Some(agent_name) = cap.get(1) {
                let capability = agent_name.as_str().to_string();
                personality.capabilities.insert(capability);
            }
        }
        
        // Add domain-specific capabilities based on content
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("domain") || content_lower.contains("ddd") {
            personality.capabilities.insert("domain-modeling".to_string());
        }
        if content_lower.contains("test") || content_lower.contains("tdd") || content_lower.contains("bdd") {
            personality.capabilities.insert("testing".to_string());
        }
        if content_lower.contains("nats") || content_lower.contains("event") {
            personality.capabilities.insert("messaging".to_string());
        }
        if content_lower.contains("ui") || content_lower.contains("gui") || content_lower.contains("iced") {
            personality.capabilities.insert("user-interface".to_string());
        }
        if content_lower.contains("nix") || content_lower.contains("infrastructure") {
            personality.capabilities.insert("infrastructure".to_string());
        }
    }
    
    /// Generate invocation patterns based on content analysis
    fn generate_invocation_patterns(&self, personality: &mut AgentPersonality, content: &str) {
        let _content_lower = content.to_lowercase();
        
        // Extract key terms for pattern matching
        let mut keywords = Vec::new();
        
        // Add agent ID as primary keyword
        keywords.push(personality.id.clone());
        
        // Add capability-based keywords
        for capability in &personality.capabilities {
            keywords.push(capability.clone());
            
            // Add related terms
            match capability.as_str() {
                "domain-modeling" => {
                    keywords.extend(vec!["domain".to_string(), "aggregate".to_string(), "entity".to_string(), "ddd".to_string()]);
                }
                "testing" => {
                    keywords.extend(vec!["test".to_string(), "tdd".to_string(), "bdd".to_string(), "scenario".to_string()]);
                }
                "messaging" => {
                    keywords.extend(vec!["nats".to_string(), "event".to_string(), "stream".to_string(), "message".to_string()]);
                }
                "user-interface" => {
                    keywords.extend(vec!["ui".to_string(), "gui".to_string(), "interface".to_string(), "iced".to_string()]);
                }
                "infrastructure" => {
                    keywords.extend(vec!["nix".to_string(), "deploy".to_string(), "config".to_string(), "system".to_string()]);
                }
                _ => {}
            }
        }
        
        // Create invocation pattern
        let pattern = InvocationPattern {
            keywords,
            context_requirements: Vec::new(),
            confidence_threshold: 0.7,
        };
        
        personality.invocation_patterns.push(pattern);
    }
    
    /// Set default composition rules for all agents
    fn set_default_composition_rules(&self, personality: &mut AgentPersonality) {
        // All agents can be orchestrated by SAGE
        let sage_rule = CompositionRule {
            compatible_agents: vec!["sage".to_string()],
            composition_type: CompositionType::Hierarchical,
            orchestration_pattern: "sage-orchestration".to_string(),
        };
        personality.composition_rules.push(sage_rule);
        
        // Domain experts can collaborate
        if personality.capabilities.contains("domain-modeling") {
            let collaboration_rule = CompositionRule {
                compatible_agents: vec![
                    "ddd-expert".to_string(),
                    "event-storming-expert".to_string(),
                    "domain-expert".to_string(),
                ],
                composition_type: CompositionType::Collaborative,
                orchestration_pattern: "domain-collaboration".to_string(),
            };
            personality.composition_rules.push(collaboration_rule);
        }
        
        // Technical experts can work sequentially
        if personality.capabilities.contains("infrastructure") || 
           personality.capabilities.contains("messaging") {
            let sequential_rule = CompositionRule {
                compatible_agents: vec![
                    "nats-expert".to_string(),
                    "nix-expert".to_string(),
                    "network-expert".to_string(),
                ],
                composition_type: CompositionType::Sequential,
                orchestration_pattern: "infrastructure-pipeline".to_string(),
            };
            personality.composition_rules.push(sequential_rule);
        }
    }
}

impl Default for AgentLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_load_agent_from_content() {
        let content = r#"---
name: "Test Expert"
description: "A test agent for validation"
icon: "🧪"
capabilities: ["testing", "validation"]
---

# Test Expert Agent

You are a test expert specializing in @tdd and @bdd methodologies.

## Instructions

Provide comprehensive testing guidance for CIM development.
"#;
        
        let loader = AgentLoader::new();
        let temp_path = Path::new("test-expert.md");
        let personality = loader.parse_agent_from_content(content, temp_path).unwrap();
        
        assert_eq!(personality.name, "Test Expert");
        assert_eq!(personality.description, "A test agent for validation");
        assert_eq!(personality.icon, "🧪");
        assert!(personality.capabilities.contains("testing"));
        assert!(personality.capabilities.contains("validation"));
        assert!(personality.capabilities.contains("tdd")); // Extracted from @tdd reference
    }
    
    #[tokio::test]
    async fn test_invocation_pattern_generation() {
        let content = r#"# Domain Expert

Specializes in @ddd and domain modeling for CIM systems."#;
        
        let loader = AgentLoader::new();
        let temp_path = Path::new("domain-expert.md");
        let personality = loader.parse_agent_from_content(content, temp_path).unwrap();
        
        let pattern = &personality.invocation_patterns[0];
        assert!(pattern.keywords.contains(&"domain".to_string()));
        assert!(pattern.keywords.contains(&"ddd".to_string()));
        assert_eq!(pattern.confidence_threshold, 0.7);
    }
}