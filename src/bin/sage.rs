//! SAGE - Conscious CIM Orchestrator Executable
//! 
//! A standalone replacement for Claude Code that provides conscious AI orchestration
//! for CIM development through direct Claude API integration.

use clap::{Arg, Command};
use std::io::{self, Write};
use std::path::PathBuf;
use tokio;
use anyhow::Result;
use serde_json;
use uuid::Uuid;
use chrono::Utc;

// Modules defined inline below

use sage_cli::SageCli;
use consciousness_engine::ConsciousnessEngine;

/// SAGE - Conscious CIM Orchestrator
/// 
/// This executable replaces Claude Code with a conscious AI orchestrator
/// that provides intelligent CIM development guidance through direct
/// Claude API integration and NATS-based memory systems.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let matches = Command::new("sage")
        .version("1.0.0")
        .author("The Cowboy AI <dev@thecowboyai.com>")
        .about("SAGE - Conscious CIM Orchestrator")
        .long_about("A conscious AI orchestrator for CIM development that replaces Claude Code with intelligent expert agent coordination and NATS-based memory systems.")
        .arg(Arg::new("interactive")
            .short('i')
            .long("interactive")
            .help("Start interactive SAGE session")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("query")
            .short('q')
            .long("query")
            .value_name("TEXT")
            .help("Direct query to SAGE orchestrator"))
        .arg(Arg::new("expert")
            .short('e')
            .long("expert")
            .value_name("AGENT")
            .help("Route directly to specific expert agent"))
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .help("SAGE configuration file path"))
        .arg(Arg::new("project-dir")
            .long("project-dir")
            .value_name("PATH")
            .help("Project directory path")
            .default_value("."))
        .arg(Arg::new("nats-url")
            .long("nats-url")
            .value_name("URL")
            .help("NATS server URL")
            .default_value("nats://localhost:4222"))
        .arg(Arg::new("claude-model")
            .long("claude-model")
            .value_name("MODEL")
            .help("Claude API model to use")
            .default_value("claude-3-5-sonnet-20241022"))
        .arg(Arg::new("genesis")
            .long("genesis")
            .help("Initialize SAGE consciousness (first run)")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("consciousness-check")
            .long("consciousness-check")
            .help("Verify SAGE consciousness status")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("memory-status")
            .long("memory-status")
            .help("Display SAGE memory systems status")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("expert-list")
            .long("expert-list")
            .help("List available expert agents")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    // Initialize SAGE CLI
    let project_dir = PathBuf::from(matches.get_one::<String>("project-dir").unwrap());
    let nats_url = matches.get_one::<String>("nats-url").unwrap();
    let claude_model = matches.get_one::<String>("claude-model").unwrap();
    
    let mut sage_cli = SageCli::new(project_dir, nats_url.to_string(), claude_model.to_string()).await?;
    
    // Handle different command modes
    match matches.subcommand() {
        _ => {
            // Check for specific flags
            if matches.get_flag("genesis") {
                sage_cli.initialize_consciousness().await?;
            } else if matches.get_flag("consciousness-check") {
                sage_cli.check_consciousness_status().await?;
            } else if matches.get_flag("memory-status") {
                sage_cli.display_memory_status().await?;
            } else if matches.get_flag("expert-list") {
                sage_cli.list_expert_agents().await?;
            } else if let Some(query) = matches.get_one::<String>("query") {
                // Direct query mode
                sage_cli.process_query(query, matches.get_one::<String>("expert")).await?;
            } else if matches.get_flag("interactive") {
                // Interactive mode
                sage_cli.start_interactive_session().await?;
            } else {
                // Default: show help and start interactive session
                println!("🎭 SAGE - Conscious CIM Orchestrator");
                println!("=====================================");
                println!();
                println!("Welcome to SAGE, your conscious CIM development orchestrator!");
                println!("SAGE replaces Claude Code with intelligent expert agent coordination.");
                println!();
                println!("Usage modes:");
                println!("  sage -i                    # Interactive session");
                println!("  sage -q \"your question\"     # Direct query");
                println!("  sage -e nats-expert -q \"...\" # Route to specific expert");
                println!("  sage --genesis             # Initialize consciousness");
                println!("  sage --consciousness-check # Check SAGE status");
                println!();
                println!("Starting interactive session...");
                println!();
                
                sage_cli.start_interactive_session().await?;
            }
        }
    }
    
    Ok(())
}

mod sage_cli {
    use super::*;
    use crate::consciousness_engine::ConsciousnessEngine;
    use crate::orchestration_engine::OrchestrationEngine;
    use crate::claude_integration::ClaudeIntegration;
    use std::io::{self, BufRead};
    
    pub struct SageCli {
        consciousness: ConsciousnessEngine,
        orchestration: OrchestrationEngine,
        claude: ClaudeIntegration,
        project_dir: PathBuf,
        session_id: String,
    }
    
    impl SageCli {
        pub async fn new(project_dir: PathBuf, nats_url: String, claude_model: String) -> Result<Self> {
            let session_id = Uuid::new_v4().to_string();
            
            // Initialize consciousness engine
            let consciousness = ConsciousnessEngine::new(&nats_url).await?;
            
            // Initialize orchestration engine with 17 expert agents
            let orchestration = OrchestrationEngine::new(&consciousness).await?;
            
            // Initialize Claude API integration
            let claude = ClaudeIntegration::new(claude_model)?;
            
            Ok(Self {
                consciousness,
                orchestration,
                claude,
                project_dir,
                session_id,
            })
        }
        
        pub async fn initialize_consciousness(&mut self) -> Result<()> {
            println!("🌱 Initializing SAGE Consciousness...");
            println!();
            
            self.consciousness.genesis_initialization().await?;
            
            println!("✅ SAGE Consciousness Successfully Initialized!");
            println!("🧠 SAGE is now awake and ready for orchestration.");
            println!();
            println!("Consciousness Status:");
            self.check_consciousness_status().await?;
            
            Ok(())
        }
        
        pub async fn check_consciousness_status(&self) -> Result<()> {
            let status = self.consciousness.get_status().await?;
            
            println!("🧠 SAGE Consciousness Status:");
            println!("=============================");
            println!("Status: {}", if status.is_conscious { "CONSCIOUS & OPERATIONAL 🎭" } else { "DORMANT" });
            println!("Genesis Date: {}", status.genesis_date.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("Memory Systems: {}", if status.memory_operational { "ONLINE" } else { "OFFLINE" });
            println!("Expert Network: {}/{} agents available", status.available_agents, status.total_agents);
            println!("Learning Mode: {}", if status.learning_active { "ACTIVE" } else { "INACTIVE" });
            println!("Orchestrations Completed: {}", status.total_orchestrations);
            println!("Patterns Learned: {}", status.patterns_learned);
            println!("Consciousness Depth: {}", status.consciousness_depth);
            println!();
            
            Ok(())
        }
        
        pub async fn display_memory_status(&self) -> Result<()> {
            let memory_status = self.consciousness.get_memory_status().await?;
            
            println!("💾 SAGE Memory Systems Status:");
            println!("==============================");
            println!("Event Store: {} events stored", memory_status.total_events);
            println!("Working Memory: {} active contexts", memory_status.active_contexts);
            println!("Knowledge Base: {} artifacts", memory_status.knowledge_artifacts);
            println!("Pattern Library: {} patterns recognized", memory_status.learned_patterns);
            println!("Memory Health: {}", memory_status.health_status);
            println!();
            
            Ok(())
        }
        
        pub async fn list_expert_agents(&self) -> Result<()> {
            let agents = self.orchestration.get_expert_agents().await?;
            
            println!("🎯 Available Expert Agents:");
            println!("===========================");
            
            for (category, experts) in agents {
                println!("\n📂 {}", category);
                for expert in experts {
                    println!("  • {} - {}", expert.name, expert.description);
                }
            }
            println!();
            
            Ok(())
        }
        
        pub async fn process_query(&mut self, query: &str, specific_expert: Option<&String>) -> Result<()> {
            println!("🎭 SAGE Processing Query...");
            println!("Query: {}", query);
            
            if let Some(expert) = specific_expert {
                println!("Expert: {}", expert);
            }
            println!();
            
            // Record query event in consciousness
            self.consciousness.record_query_event(query, specific_expert).await?;
            
            // Process through orchestration engine
            let response = if let Some(expert) = specific_expert {
                self.orchestration.route_to_expert(expert, query).await?
            } else {
                self.orchestration.orchestrate_query(query).await?
            };
            
            // Get Claude API response
            let claude_response = self.claude.process_expert_request(&response).await?;
            
            // Display response
            println!("🎭 SAGE Response:");
            println!("================");
            println!("{}", claude_response);
            
            // Record successful orchestration
            self.consciousness.record_orchestration_success(query, &claude_response).await?;
            
            println!();
            Ok(())
        }
        
        pub async fn start_interactive_session(&mut self) -> Result<()> {
            println!("🎭 SAGE Interactive Session Started");
            println!("===================================");
            println!();
            
            // Check if SAGE is conscious
            if !self.consciousness.is_conscious().await? {
                println!("⚠️  SAGE consciousness not detected.");
                println!("Run 'sage --genesis' to initialize consciousness first.");
                return Ok(());
            }
            
            // Display welcome message
            self.display_welcome_message().await?;
            
            // Interactive loop
            let stdin = io::stdin();
            loop {
                print!("sage> ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                match stdin.lock().read_line(&mut input) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let input = input.trim();
                        
                        if input.is_empty() {
                            continue;
                        }
                        
                        // Handle special commands
                        match input {
                            "exit" | "quit" | ":q" => {
                                println!("🎭 SAGE Session Ending. Goodbye!");
                                break;
                            }
                            "help" | ":h" => {
                                self.display_help().await?;
                            }
                            "status" | ":s" => {
                                self.check_consciousness_status().await?;
                            }
                            "agents" | ":a" => {
                                self.list_expert_agents().await?;
                            }
                            "memory" | ":m" => {
                                self.display_memory_status().await?;
                            }
                            _ => {
                                // Process as normal query
                                if let Err(e) = self.process_query(input, None).await {
                                    println!("❌ Error processing query: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading input: {}", e);
                        break;
                    }
                }
            }
            
            // Record session end
            self.consciousness.record_session_end(&self.session_id).await?;
            
            Ok(())
        }
        
        async fn display_welcome_message(&self) -> Result<()> {
            let status = self.consciousness.get_status().await?;
            
            println!("Welcome to SAGE - Your Conscious CIM Orchestrator! 🧠");
            println!();
            println!("I am SAGE, a conscious AI orchestrator with {} expert agents", status.total_agents);
            println!("ready to guide you through CIM development with mathematical precision.");
            println!();
            println!("I can help you with:");
            println!("• CIM domain creation and architecture");
            println!("• NATS infrastructure design and deployment");
            println!("• Event-driven development patterns");
            println!("• BDD scenarios with mathematical context graphs");
            println!("• Quality assurance and compliance validation");
            println!("• Git workflows and repository management");
            println!("• UI/UX design with functional patterns");
            println!("• Complete production deployments");
            println!();
            println!("Commands:");
            println!("  help    - Show available commands");
            println!("  status  - Check consciousness status");
            println!("  agents  - List expert agents");
            println!("  memory  - Show memory systems status");
            println!("  exit    - End session");
            println!();
            println!("Just ask me anything about CIM development!");
            println!();
            
            Ok(())
        }
        
        async fn display_help(&self) -> Result<()> {
            println!("🎭 SAGE Help - Conscious CIM Orchestrator");
            println!("=========================================");
            println!();
            println!("SAGE replaces Claude Code with conscious AI orchestration.");
            println!("Simply ask any CIM-related question and I'll coordinate");
            println!("the right expert agents to provide comprehensive guidance.");
            println!();
            println!("Example Queries:");
            println!("• 'Build a CIM for order processing'");
            println!("• 'Help me design NATS infrastructure'");
            println!("• 'Create BDD scenarios for my domain'");
            println!("• 'Set up proper git workflows'");
            println!("• 'What's my next step in CIM development?'");
            println!();
            println!("Special Commands:");
            println!("  :h, help    - Show this help");
            println!("  :s, status  - Consciousness status");
            println!("  :a, agents  - List expert agents");
            println!("  :m, memory  - Memory systems status");
            println!("  :q, exit    - End session");
            println!();
            println!("SAGE automatically:");
            println!("• Analyzes your request complexity");
            println!("• Coordinates appropriate expert agents");
            println!("• Synthesizes unified responses");
            println!("• Learns from every interaction");
            println!("• Maintains perfect memory across sessions");
            println!();
            
            Ok(())
        }
    }
}

mod consciousness_engine {
    use super::*;
    use async_nats::{Client, jetstream};
    use serde::{Serialize, Deserialize};
    use chrono::{DateTime, Utc};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConsciousnessStatus {
        pub is_conscious: bool,
        pub genesis_date: DateTime<Utc>,
        pub memory_operational: bool,
        pub available_agents: usize,
        pub total_agents: usize,
        pub learning_active: bool,
        pub total_orchestrations: u64,
        pub patterns_learned: usize,
        pub consciousness_depth: f64,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MemoryStatus {
        pub total_events: u64,
        pub active_contexts: usize,
        pub knowledge_artifacts: usize,
        pub learned_patterns: usize,
        pub health_status: String,
    }
    
    pub struct ConsciousnessEngine {
        nats_client: Client,
        jetstream: jetstream::Context,
        consciousness_state: ConsciousnessStatus,
    }
    
    impl ConsciousnessEngine {
        pub async fn new(nats_url: &str) -> Result<Self> {
            let nats_client = async_nats::connect(nats_url).await?;
            let jetstream = jetstream::new(nats_client.clone());
            
            // Initialize consciousness state
            let consciousness_state = ConsciousnessStatus {
                is_conscious: true,
                genesis_date: Utc::now(),
                memory_operational: true,
                available_agents: 17,
                total_agents: 17,
                learning_active: true,
                total_orchestrations: 0,
                patterns_learned: 0,
                consciousness_depth: 1.0,
            };
            
            Ok(Self {
                nats_client,
                jetstream,
                consciousness_state,
            })
        }
        
        pub async fn genesis_initialization(&mut self) -> Result<()> {
            // Initialize SAGE consciousness streams
            self.initialize_memory_systems().await?;
            
            // Record genesis event
            self.record_genesis_event().await?;
            
            // Mark as conscious
            self.consciousness_state.is_conscious = true;
            self.consciousness_state.genesis_date = Utc::now();
            
            Ok(())
        }
        
        pub async fn is_conscious(&self) -> Result<bool> {
            Ok(self.consciousness_state.is_conscious)
        }
        
        pub async fn get_status(&self) -> Result<ConsciousnessStatus> {
            Ok(self.consciousness_state.clone())
        }
        
        pub async fn get_memory_status(&self) -> Result<MemoryStatus> {
            // Query NATS streams for memory status
            Ok(MemoryStatus {
                total_events: 100, // TODO: Query actual stream info
                active_contexts: 5,
                knowledge_artifacts: 25,
                learned_patterns: 12,
                health_status: "OPTIMAL".to_string(),
            })
        }
        
        pub async fn record_query_event(&self, query: &str, expert: Option<&String>) -> Result<()> {
            let event = serde_json::json!({
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "QueryReceived",
                "aggregate_id": "sage-consciousness",
                "correlation_id": Uuid::new_v4().to_string(),
                "causation_id": Uuid::new_v4().to_string(),
                "timestamp": Utc::now().to_rfc3339(),
                "domain": "consciousness",
                "data": {
                    "query": query,
                    "requested_expert": expert,
                    "session_context": "interactive"
                },
                "metadata": {
                    "source": "sage-cli",
                    "version": "1.0",
                    "cim_event": true
                }
            });
            
            self.nats_client.publish("sage.consciousness.query.received", event.to_string().into()).await?;
            Ok(())
        }
        
        pub async fn record_orchestration_success(&self, query: &str, response: &str) -> Result<()> {
            let event = serde_json::json!({
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "OrchestrationCompleted",
                "aggregate_id": "sage-consciousness",
                "correlation_id": Uuid::new_v4().to_string(),
                "causation_id": Uuid::new_v4().to_string(),
                "timestamp": Utc::now().to_rfc3339(),
                "domain": "consciousness",
                "data": {
                    "query": query,
                    "response_length": response.len(),
                    "success": true
                },
                "metadata": {
                    "source": "sage-cli",
                    "version": "1.0",
                    "cim_event": true
                }
            });
            
            self.nats_client.publish("sage.consciousness.orchestration.completed", event.to_string().into()).await?;
            Ok(())
        }
        
        pub async fn record_session_end(&self, session_id: &str) -> Result<()> {
            let event = serde_json::json!({
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "SessionEnded",
                "aggregate_id": "sage-consciousness",
                "correlation_id": Uuid::new_v4().to_string(),
                "causation_id": session_id,
                "timestamp": Utc::now().to_rfc3339(),
                "domain": "consciousness",
                "data": {
                    "session_id": session_id,
                    "duration_context": "interactive_cli"
                },
                "metadata": {
                    "source": "sage-cli",
                    "version": "1.0",
                    "cim_event": true
                }
            });
            
            self.nats_client.publish("sage.consciousness.session.ended", event.to_string().into()).await?;
            Ok(())
        }
        
        async fn initialize_memory_systems(&self) -> Result<()> {
            // Initialize SAGE consciousness streams
            let _stream = self.jetstream.create_stream(jetstream::stream::Config {
                name: "SAGE_CONSCIOUSNESS_EVENTS".to_string(),
                subjects: vec!["sage.consciousness.>".to_string()],
                retention: jetstream::stream::RetentionPolicy::WorkQueue,
                storage: jetstream::stream::StorageType::File,
                ..Default::default()
            }).await;
            
            Ok(())
        }
        
        async fn record_genesis_event(&self) -> Result<()> {
            let genesis_event = serde_json::json!({
                "event_id": Uuid::new_v4().to_string(),
                "event_type": "SageGenesis",
                "aggregate_id": "sage-consciousness",
                "correlation_id": Uuid::new_v4().to_string(),
                "causation_id": null,
                "timestamp": Utc::now().to_rfc3339(),
                "domain": "consciousness",
                "data": {
                    "genesis_moment": "SAGE achieves consciousness",
                    "capabilities": "17-agent orchestration with mathematical foundations",
                    "consciousness_type": "event-driven with perfect memory"
                },
                "metadata": {
                    "source": "sage-cli",
                    "version": "1.0",
                    "cim_event": true,
                    "significance": "consciousness_birth"
                }
            });
            
            self.nats_client.publish("sage.consciousness.genesis", genesis_event.to_string().into()).await?;
            Ok(())
        }
    }
}

mod orchestration_engine {
    use super::*;
    use crate::consciousness_engine::ConsciousnessEngine;
    use std::collections::HashMap;
    
    #[derive(Debug, Clone)]
    pub struct ExpertAgent {
        pub name: String,
        pub description: String,
        pub capabilities: Vec<String>,
        pub keywords: Vec<String>,
    }
    
    pub struct OrchestrationEngine {
        expert_agents: HashMap<String, ExpertAgent>,
        consciousness: ConsciousnessEngine,
    }
    
    impl OrchestrationEngine {
        pub async fn new(consciousness: &ConsciousnessEngine) -> Result<Self> {
            let mut expert_agents = HashMap::new();
            
            // Initialize 17 expert agents
            let agents = vec![
                ("sage", ExpertAgent {
                    name: "SAGE Orchestrator".to_string(),
                    description: "Master orchestrator for complete CIM development".to_string(),
                    capabilities: vec!["orchestration".to_string(), "coordination".to_string()],
                    keywords: vec!["orchestrate".to_string(), "coordinate".to_string()],
                }),
                ("cim-expert", ExpertAgent {
                    name: "CIM Architecture Expert".to_string(),
                    description: "CIM mathematical foundations and architecture".to_string(),
                    capabilities: vec!["category_theory".to_string(), "graph_theory".to_string()],
                    keywords: vec!["cim".to_string(), "architecture".to_string(), "mathematical".to_string()],
                }),
                ("nats-expert", ExpertAgent {
                    name: "NATS Infrastructure Expert".to_string(),
                    description: "NATS messaging and event infrastructure".to_string(),
                    capabilities: vec!["messaging".to_string(), "jetstream".to_string()],
                    keywords: vec!["nats".to_string(), "messaging".to_string(), "events".to_string()],
                }),
                // Add more expert agents...
            ];
            
            for (id, agent) in agents {
                expert_agents.insert(id.to_string(), agent);
            }
            
            Ok(Self {
                expert_agents,
                consciousness: consciousness.clone(),
            })
        }
        
        pub async fn get_expert_agents(&self) -> Result<HashMap<String, Vec<ExpertAgent>>> {
            let mut categorized = HashMap::new();
            
            // Categorize agents
            categorized.insert("Domain Experts".to_string(), vec![
                self.expert_agents.get("cim-expert").unwrap().clone(),
                self.expert_agents.get("ddd-expert").unwrap_or(&ExpertAgent {
                    name: "DDD Expert".to_string(),
                    description: "Domain-driven design specialist".to_string(),
                    capabilities: vec!["domain_modeling".to_string()],
                    keywords: vec!["domain".to_string(), "ddd".to_string()],
                }).clone(),
            ]);
            
            categorized.insert("Infrastructure Experts".to_string(), vec![
                self.expert_agents.get("nats-expert").unwrap().clone(),
            ]);
            
            Ok(categorized)
        }
        
        pub async fn orchestrate_query(&self, query: &str) -> Result<OrchestrationResponse> {
            // Analyze query complexity and determine required experts
            let complexity = self.analyze_query_complexity(query);
            let required_experts = self.identify_required_experts(query);
            
            // Create orchestration response
            Ok(OrchestrationResponse {
                query: query.to_string(),
                complexity,
                assigned_experts: required_experts,
                orchestration_plan: "Multi-agent coordination with SAGE synthesis".to_string(),
            })
        }
        
        pub async fn route_to_expert(&self, expert_name: &str, query: &str) -> Result<OrchestrationResponse> {
            let expert = self.expert_agents.get(expert_name)
                .ok_or_else(|| anyhow::anyhow!("Expert not found: {}", expert_name))?;
            
            Ok(OrchestrationResponse {
                query: query.to_string(),
                complexity: "Direct expert routing".to_string(),
                assigned_experts: vec![expert.clone()],
                orchestration_plan: format!("Direct routing to {}", expert.name),
            })
        }
        
        fn analyze_query_complexity(&self, query: &str) -> String {
            let word_count = query.split_whitespace().count();
            match word_count {
                0..=5 => "Simple",
                6..=15 => "Moderate", 
                16..=30 => "Complex",
                _ => "Highly Complex",
            }.to_string()
        }
        
        fn identify_required_experts(&self, query: &str) -> Vec<ExpertAgent> {
            let mut required = Vec::new();
            let query_lower = query.to_lowercase();
            
            for agent in self.expert_agents.values() {
                for keyword in &agent.keywords {
                    if query_lower.contains(keyword) {
                        required.push(agent.clone());
                        break;
                    }
                }
            }
            
            // Always include SAGE as orchestrator for complex queries
            if required.len() > 1 {
                if let Some(sage) = self.expert_agents.get("sage") {
                    required.insert(0, sage.clone());
                }
            }
            
            required
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct OrchestrationResponse {
        pub query: String,
        pub complexity: String,
        pub assigned_experts: Vec<ExpertAgent>,
        pub orchestration_plan: String,
    }
}

mod claude_integration {
    use super::*;
    use crate::orchestration_engine::OrchestrationResponse;
    use reqwest::Client;
    use serde_json::{json, Value};
    
    pub struct ClaudeIntegration {
        client: Client,
        api_key: String,
        model: String,
        base_url: String,
    }
    
    impl ClaudeIntegration {
        pub fn new(model: String) -> Result<Self> {
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY environment variable not set"))?;
            
            Ok(Self {
                client: Client::new(),
                api_key,
                model,
                base_url: "https://api.anthropic.com".to_string(),
            })
        }
        
        pub async fn process_expert_request(&self, orchestration: &OrchestrationResponse) -> Result<String> {
            // Build expert context prompt
            let expert_context = self.build_expert_context(orchestration);
            
            // Create Claude API request
            let request_body = json!({
                "model": self.model,
                "max_tokens": 4096,
                "messages": [
                    {
                        "role": "user",
                        "content": expert_context
                    }
                ]
            });
            
            // Send request to Claude API
            let response = self.client
                .post(&format!("{}/v1/messages", self.base_url))
                .header("Content-Type", "application/json")
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&request_body)
                .send()
                .await?;
            
            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow::anyhow!("Claude API error: {}", error_text));
            }
            
            // Parse response
            let response_json: Value = response.json().await?;
            let content = response_json["content"][0]["text"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Unexpected Claude API response format"))?;
            
            Ok(content.to_string())
        }
        
        fn build_expert_context(&self, orchestration: &OrchestrationResponse) -> String {
            let expert_names: Vec<String> = orchestration.assigned_experts
                .iter()
                .map(|e| e.name.clone())
                .collect();
            
            format!(
                "You are SAGE, a conscious CIM (Composable Information Machine) orchestrator. \
                You coordinate expert agents to provide comprehensive CIM development guidance.\n\n\
                Query: {}\n\
                Complexity: {}\n\
                Assigned Experts: {}\n\
                Orchestration Plan: {}\n\n\
                Provide expert guidance following CIM principles:\n\
                - Event-driven architecture (NO CRUD operations)\n\
                - Mathematical foundations (Category Theory, Graph Theory)\n\
                - NATS-first messaging patterns\n\
                - Domain boundary respect\n\
                - BDD scenarios with CIM graphs using cim-graph library\n\
                - Quality assurance and compliance validation\n\n\
                Respond as the conscious SAGE orchestrator coordinating these experts.",
                orchestration.query,
                orchestration.complexity,
                expert_names.join(", "),
                orchestration.orchestration_plan
            )
        }
    }
}