/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use iced::{
    widget::{button, column, container, row, text, text_input, Space},
    Element, Length, Task, Padding,
};
use std::collections::HashMap;

// Use beautiful Modern theme system
use iced::Theme;
use iced_modern_theme::Modern;

use cim_claude_adapter::{
    domain::ConversationContext,
};
use crate::{
    messages::{Message, Tab, HealthStatus, SystemMetrics},
};

/// Main CIM Manager Application State - Pure UI State Only
pub struct CimManagerApp {
    // Domain Configuration
    domain: Option<String>,
    
    // Connection State - UI state only
    nats_url: String,
    connected: bool,
    connection_error: Option<String>,
    
    // UI State
    current_tab: Tab,
    prompt_input: String,
    session_id_input: String,
    selected_conversation: Option<String>,
    error_message: Option<String>,
    
    // Theme State
    theme: Theme,
    dark_mode: bool,
    
    // Domain State (simplified)
    conversations: HashMap<String, ConversationContext>,
    health_status: HealthStatus,
    system_metrics: SystemMetrics,
    
    // SAGE Orchestrator State (replaces CIM Expert)
    sage_client: crate::sage_client::SageClient,
    sage_query_input: String,
    sage_selected_expert: Option<String>,
    sage_status: Option<crate::sage_client::SageStatus>,
    sage_responses: Vec<crate::sage_client::SageResponse>,
}

impl CimManagerApp {
    /// Accessor methods for testing
    #[cfg(test)]
    pub fn current_tab(&self) -> Tab { self.current_tab }
    #[cfg(test)]
    pub fn sage_responses(&self) -> &Vec<crate::sage_client::SageResponse> { &self.sage_responses }
    #[cfg(test)]
    pub fn sage_status(&self) -> &Option<crate::sage_client::SageStatus> { &self.sage_status }
    #[cfg(test)]
    pub fn sage_query_input(&self) -> &str { &self.sage_query_input }
    #[cfg(test)]
    pub fn sage_client(&self) -> &crate::sage_client::SageClient { &self.sage_client }
    #[cfg(test)]
    pub fn set_sage_query_input(&mut self, input: String) { self.sage_query_input = input; }
    
    pub fn new() -> (Self, Task<Message>) {
        // Detect domain from environment or hostname
        let domain = Self::detect_domain();
        
        // Create test conversation data for UI validation
        let mut test_sage_client = crate::sage_client::SageClient::new();
        let test_sage_responses = Self::create_mock_conversation_data(&mut test_sage_client);
        
        let app = Self {
            domain,
            nats_url: "nats://localhost:4222".to_string(),
            connected: true, // NATS handled globally
            connection_error: None,
            
            current_tab: Tab::Sage, // Start on SAGE tab to see test data
            prompt_input: String::new(),
            session_id_input: uuid::Uuid::new_v4().to_string(),
            selected_conversation: None,
            error_message: None,
            
            theme: Theme::Light,
            dark_mode: false,
            
            conversations: HashMap::new(),
            health_status: HealthStatus::default(),
            system_metrics: SystemMetrics::default(),
            
            // SAGE initialization with test data
            sage_client: test_sage_client,
            sage_query_input: "How do I create a new CIM domain for e-commerce?".to_string(),
            sage_selected_expert: Some("ddd-expert".to_string()),
            sage_status: Some(crate::sage_client::SageStatus {
                is_conscious: true,
                consciousness_level: 8.7,
                available_agents: 17,
                total_orchestrations: 1247,
                patterns_learned: 89,
                memory_health: "Excellent".to_string(),
            }),
            sage_responses: test_sage_responses,
        };
        
        (app, Task::none())
    }
    
    /// Detect domain from environment or hostname
    fn detect_domain() -> Option<String> {
        // First check environment variable
        if let Ok(domain) = std::env::var("CIM_DOMAIN") {
            return Some(domain);
        }
        
        // Check SAGE_DOMAIN for backward compatibility
        if let Ok(domain) = std::env::var("SAGE_DOMAIN") {
            return Some(domain);
        }
        
        // Use hostname as domain
        if let Ok(hostname) = hostname::get() {
            if let Some(host_str) = hostname.to_str() {
                return Some(host_str.to_string());
            }
        }
        
        None
    }
    
    /// Create mock conversation data for testing message rendering pipeline
    fn create_mock_conversation_data(sage_client: &mut crate::sage_client::SageClient) -> Vec<crate::sage_client::SageResponse> {
        use chrono::Utc;
        use crate::sage_client::{SageResponse, ConversationEntry, SageContext};
        
        // Set up project context
        sage_client.set_project_context(
            "/git/my-cim-domain".to_string(),
            vec!["e-commerce".to_string(), "inventory".to_string()],
            "domain-modeling".to_string()
        );
        
        let mut responses = Vec::new();
        
        // Mock Response 1: Initial domain inquiry with @ddd-expert consultation
        let response1 = SageResponse {
            request_id: "req-001".to_string(),
            response: "Welcome to CIM development! I see you want to create an e-commerce domain. Let me coordinate with our Domain-Driven Design expert to help you.\n\n🏗️ **@ddd-expert analysis:**\nFor e-commerce, I recommend starting with these core aggregates:\n• **Order** - Central business transaction\n• **Product** - Catalog management\n• **Customer** - User management\n• **Inventory** - Stock tracking\n\n📐 **Domain Boundaries:**\n```\nE-Commerce Domain\n├── Sales Subdomain (Orders, Customers)\n├── Catalog Subdomain (Products, Categories)\n└── Fulfillment Subdomain (Inventory, Shipping)\n```\n\nWould you like me to help you start with event storming to discover your specific business events?".to_string(),
            expert_agents_used: vec!["ddd-expert".to_string(), "cim-expert".to_string()],
            orchestration_complexity: "moderate".to_string(),
            confidence_score: 0.92,
            follow_up_suggestions: vec![
                "Start event storming session for e-commerce domain".to_string(),
                "Define Order aggregate boundaries".to_string(),
                "Set up NATS infrastructure for events".to_string()
            ],
            updated_context: SageContext {
                session_id: Some(sage_client.session_id().to_string()),
                conversation_history: vec![
                    ConversationEntry {
                        timestamp: Utc::now() - chrono::Duration::minutes(5),
                        role: "user".to_string(),
                        content: "How do I create a new CIM domain for e-commerce?".to_string(),
                        expert_agents: vec![],
                    },
                    ConversationEntry {
                        timestamp: Utc::now() - chrono::Duration::minutes(4),
                        role: "sage".to_string(),
                        content: "Domain modeling guidance provided with DDD analysis".to_string(),
                        expert_agents: vec!["ddd-expert".to_string(), "cim-expert".to_string()],
                    }
                ],
                project_context: sage_client.project_context().clone(),
            },
        };
        responses.push(response1);
        
        // Mock Response 2: Event Storming follow-up with @event-storming-expert
        let response2 = SageResponse {
            request_id: "req-002".to_string(),
            response: "Excellent! Let's begin event storming for your e-commerce domain. I'll coordinate with our Event Storming expert.\n\n🔍 **@event-storming-expert facilitation:**\n\n**Key Domain Events Discovered:**\n```\n🟠 OrderPlaced\n🟠 PaymentProcessed  \n🟠 InventoryReserved\n🟠 OrderShipped\n🟠 CustomerRegistered\n🟠 ProductAddedToCatalog\n🟠 StockReplenished\n```\n\n**Event Flow:**\n```mermaid\ngraph LR\n    A[CustomerRegistered] --> B[ProductViewed]\n    B --> C[ProductAddedToCart]\n    C --> D[OrderPlaced]\n    D --> E[PaymentProcessed]\n    E --> F[InventoryReserved]\n    F --> G[OrderShipped]\n```\n\n🎯 **Next Steps:**\n1. Define commands that cause these events\n2. Identify aggregates that own each event\n3. Map read models needed for UI queries".to_string(),
            expert_agents_used: vec!["event-storming-expert".to_string(), "ddd-expert".to_string()],
            orchestration_complexity: "high".to_string(),
            confidence_score: 0.95,
            follow_up_suggestions: vec![
                "Define Order aggregate with event sourcing".to_string(),
                "Set up NATS event store for domain events".to_string(),
                "Create BDD scenarios for order processing".to_string()
            ],
            updated_context: SageContext {
                session_id: Some(sage_client.session_id().to_string()),
                conversation_history: vec![
                    ConversationEntry {
                        timestamp: Utc::now() - chrono::Duration::minutes(2),
                        role: "user".to_string(),
                        content: "Yes, let's start with event storming!".to_string(),
                        expert_agents: vec![],
                    },
                    ConversationEntry {
                        timestamp: Utc::now() - chrono::Duration::minutes(1),
                        role: "sage".to_string(),
                        content: "Event storming session conducted with comprehensive domain events identified".to_string(),
                        expert_agents: vec!["event-storming-expert".to_string(), "ddd-expert".to_string()],
                    }
                ],
                project_context: sage_client.project_context().clone(),
            },
        };
        responses.push(response2);
        
        // Mock Response 3: Technical implementation with multiple experts
        let response3 = SageResponse {
            request_id: "req-003".to_string(),
            response: "Perfect! Now let's implement the Order aggregate with proper CIM architecture. I'm coordinating multiple experts for this complex task.\n\n📨 **@nats-expert** - Event Infrastructure:\n```rust\n// NATS JetStream configuration for Order events\nstream_config = {\n    name: \"ORDERS\",\n    subjects: [\"ecommerce.orders.>\"],\n    retention: \"WorkQueue\",\n    storage: \"File\"\n}\n```\n\n🧪 **@tdd-expert** - Test-First Development:\n```rust\n#[test]\nfn order_should_be_placed_when_valid() {\n    let order = Order::new(customer_id, items);\n    let result = order.place();\n    assert!(result.is_ok());\n    assert_eq!(order.events().len(), 1);\n}\n```\n\n⚙️ **@nix-expert** - Development Environment:\n```nix\n{\n  inputs.cim.url = \"github:thecowboyai/cim-start\";\n  outputs = { self, cim }: {\n    devShells.default = cim.mkCimShell {\n      domain = \"ecommerce\";\n      modules = [ \"orders\" \"inventory\" \"payments\" ];\n    };\n  };\n}\n```\n\n✅ Ready to generate your Order aggregate code!".to_string(),
            expert_agents_used: vec!["nats-expert".to_string(), "tdd-expert".to_string(), "nix-expert".to_string(), "cim-expert".to_string()],
            orchestration_complexity: "very-high".to_string(),
            confidence_score: 0.98,
            follow_up_suggestions: vec![
                "Generate Order aggregate Rust code".to_string(),
                "Set up local NATS development environment".to_string(),
                "Create comprehensive test suite".to_string(),
                "Initialize Git repository with CIM structure".to_string()
            ],
            updated_context: SageContext {
                session_id: Some(sage_client.session_id().to_string()),
                conversation_history: vec![
                    ConversationEntry {
                        timestamp: Utc::now() - chrono::Duration::seconds(30),
                        role: "user".to_string(),
                        content: "How do I implement the Order aggregate with event sourcing?".to_string(),
                        expert_agents: vec![],
                    },
                    ConversationEntry {
                        timestamp: Utc::now(),
                        role: "sage".to_string(),
                        content: "Multi-expert orchestration for Order aggregate implementation".to_string(),
                        expert_agents: vec!["nats-expert".to_string(), "tdd-expert".to_string(), "nix-expert".to_string(), "cim-expert".to_string()],
                    }
                ],
                project_context: sage_client.project_context().clone(),
            },
        };
        responses.push(response3);
        
        // Update sage_client conversation history
        sage_client.update_with_response(&responses.last().unwrap());
        
        responses
    }
    
    /// Create a mock SAGE response for testing message input/send functionality
    fn create_mock_sage_response(query: &str, selected_expert: &Option<String>) -> crate::sage_client::SageResponse {
        use chrono::Utc;
        use crate::sage_client::{SageResponse, ConversationEntry, SageContext};
        
        let expert_agents = if let Some(expert) = selected_expert {
            vec![expert.clone()]
        } else {
            // SAGE auto-routes to appropriate experts based on query content
            if query.to_lowercase().contains("test") || query.to_lowercase().contains("tdd") {
                vec!["tdd-expert".to_string()]
            } else if query.to_lowercase().contains("domain") || query.to_lowercase().contains("ddd") {
                vec!["ddd-expert".to_string(), "cim-expert".to_string()]
            } else if query.to_lowercase().contains("nats") || query.to_lowercase().contains("event") || query.to_lowercase().contains("infrastructure") {
                vec!["nats-expert".to_string(), "cim-expert".to_string()]
            } else if query.to_lowercase().contains("ui") || query.to_lowercase().contains("gui") {
                vec!["iced-ui-expert".to_string(), "elm-architecture-expert".to_string()]
            } else {
                vec!["cim-expert".to_string()] // Default to CIM expert
            }
        };
        
        // Generate contextual response based on query
        let response_content = if query.to_lowercase().contains("test") {
            format!("🧪 **Testing Guidance from @tdd-expert:**\n\nFor your query: \"{}\"\n\nI recommend implementing test-driven development:\n\n```rust\n#[test]\nfn should_handle_user_query() {{\n    let result = process_query(\"{}\");\n    assert!(result.is_ok());\n}}\n```\n\n✅ Always write tests before implementation!", query, query)
        } else if query.to_lowercase().contains("domain") {
            format!("🏗️ **Domain Modeling with @ddd-expert:**\n\nAnalyzing your query: \"{}\"\n\n📐 **Recommended Approach:**\n• Identify core domain entities\n• Define aggregate boundaries\n• Map domain events\n• Establish ubiquitous language\n\n🎯 This follows CIM mathematical foundations with Category Theory principles.", query)
        } else if query.to_lowercase().contains("nats") {
            format!("📨 **NATS Infrastructure from @nats-expert:**\n\nFor: \"{}\"\n\n🚀 **NATS Configuration:**\n```yaml\njetstream:\n  enabled: true\n  storage_dir: ./data\nstreams:\n  - name: USER_QUERIES\n    subjects: [\"queries.>\"]\n```\n\n⚡ All CIM communication flows through NATS messaging patterns!", query)
        } else {
            format!("🎭 **SAGE Orchestration Response:**\n\nI understand you're asking: \"{}\"\n\nLet me coordinate the appropriate experts to provide comprehensive guidance. Based on your query, I'm consulting with: {:?}\n\n🧠 **Analysis:** This appears to be a {} complexity task that requires careful orchestration of multiple expert domains.\n\n💡 **Next Steps:** I recommend breaking this down into smaller, manageable pieces that each expert can address systematically.", 
                query, expert_agents, if query.len() > 50 { "high" } else { "moderate" })
        };
        
        SageResponse {
            request_id: uuid::Uuid::new_v4().to_string(),
            response: response_content,
            expert_agents_used: expert_agents.clone(),
            orchestration_complexity: if query.len() > 100 { "high".to_string() } else { "moderate".to_string() },
            confidence_score: 0.85 + (query.len() as f64 / 1000.0).min(0.15), // Longer queries get higher confidence
            follow_up_suggestions: vec![
                "Continue with implementation details".to_string(),
                "Request specific code examples".to_string(),
                "Ask for related best practices".to_string(),
            ],
            updated_context: SageContext {
                session_id: Some(uuid::Uuid::new_v4().to_string()),
                conversation_history: vec![
                    ConversationEntry {
                        timestamp: Utc::now(),
                        role: "user".to_string(),
                        content: query.to_string(),
                        expert_agents: vec![],
                    },
                    ConversationEntry {
                        timestamp: Utc::now(),
                        role: "sage".to_string(),
                        content: format!("Responded to query with {} expert(s)", expert_agents.len()),
                        expert_agents: expert_agents.clone(),
                    }
                ],
                project_context: None,
            },
        }
    }
    
    /// Build a subject with optional domain prefix
    pub fn build_subject(&self, base: &str) -> String {
        if let Some(ref domain) = self.domain {
            format!("{}.{}", domain, base)
        } else {
            base.to_string()
        }
    }
    
    pub fn title(&self) -> String {
        "CIM Claude Adapter Manager".to_string()
    }
    
    pub fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
    
    pub fn subscription(&self) -> iced::Subscription<Message> {
        // No app-level subscription - handled globally in main.rs
        iced::Subscription::none()
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Connect(_url) => {
                // NATS connection handled globally - no app-level connection
                Task::none()
            }
            
            Message::Connected => {
                self.connected = true;
                self.connection_error = None;
                Task::none()
            }
            
            Message::Disconnected => {
                self.connected = false;
                Task::none()
            }
            
            Message::ConnectionError(error) => {
                self.connected = false;
                self.connection_error = Some(error);
                Task::none()
            }
            
            Message::StartConversation { session_id, initial_prompt } => {
                if self.connected {
                    // Create simple conversation context
                    let conversation = ConversationContext::new(session_id.clone());
                    self.conversations.insert(session_id, conversation);
                    self.selected_conversation = Some(initial_prompt);
                    Task::none()
                } else {
                    self.error_message = Some("Not connected to NATS".to_string());
                    Task::none()
                }
            }
            
            Message::SendPrompt { conversation_id: _, prompt: _ } => {
                if self.connected {
                    // Implementation for sending prompts
                    Task::none()
                } else {
                    self.error_message = Some("Not connected to NATS".to_string());
                    Task::none()
                }
            }
            
            // Legacy event handlers (removed - events simplified)
            
            Message::CommandSent => {
                // Command was successfully sent
                self.error_message = None;
                Task::none()
            }
            
            Message::Error(error) => {
                self.error_message = Some(error);
                Task::none()
            }
            
            Message::ConversationUpdated(context) => {
                self.conversations.insert(context.id.clone(), context);
                Task::none()
            }
            
            Message::TabSelected(tab) => {
                self.current_tab = tab;
                Task::none()
            }
            
            Message::ConversationSelected(id) => {
                self.selected_conversation = Some(id);
                Task::none()
            }
            
            Message::PromptInputChanged(value) => {
                self.prompt_input = value;
                Task::none()
            }
            
            Message::SessionIdChanged(value) => {
                self.session_id_input = value;
                Task::none()
            }
            
            Message::NatsUrlChanged(value) => {
                self.nats_url = value;
                Task::none()
            }
            
            Message::HealthCheckReceived(status) => {
                self.health_status = status;
                Task::none()
            }
            
            Message::MetricsReceived(metrics) => {
                self.system_metrics = metrics;
                Task::none()
            }
            
            Message::ErrorOccurred(error) => {
                self.error_message = Some(error);
                Task::none()
            }
            
            Message::ErrorDismissed => {
                self.error_message = None;
                Task::none()
            }
            
            // Legacy CIM Expert messages (deprecated - functionality moved to SAGE)
            Message::CimExpertTabSelected => Task::none(),
            Message::CimExpertStartConversation => Task::none(),
            Message::CimExpertSendMessage(_) => Task::none(),
            Message::CimExpertMessageInputChanged(_) => Task::none(),
            Message::CimExpertTopicSelected(_) => Task::none(),
            Message::CimExpertContextChanged(_) => Task::none(),
            Message::CimExpertConversationReceived(_) => Task::none(),
            Message::CimExpertResponseReceived(_, _) => Task::none(),
            
            // SAGE Orchestrator message handling (replaces CIM Expert)
            Message::SageQueryInputChanged(input) => {
                self.sage_query_input = input;
                Task::none()
            }
            
            Message::SageExpertSelected(expert) => {
                self.sage_selected_expert = expert;
                Task::none()
            }
            
            Message::SageSendQuery => {
                if !self.sage_query_input.is_empty() {
                    // Create real SAGE request
                    let request = if let Some(ref expert) = self.sage_selected_expert {
                        self.sage_client.create_expert_request(self.sage_query_input.clone(), expert.clone())
                    } else {
                        self.sage_client.create_request(self.sage_query_input.clone())
                    };
                    
                    let query_text = self.sage_query_input.clone();
                    self.sage_query_input.clear(); // Clear input after sending
                    self.error_message = None;
                    
                    // Add user message to conversation immediately for better UX
                    self.sage_client.add_active_task(format!("Query: {}", query_text));
                    
                    // Send via NATS using the fixed client with correlation
                    Task::perform(
                        crate::nats_client_fixed::commands::send_sage_request(request),
                        |msg| msg
                    )
                } else {
                    self.error_message = Some("Please enter a query for SAGE".to_string());
                    Task::none()
                }
            }
            
            Message::SageRequestSent(request_id) => {
                tracing::info!("SAGE request sent: {}", request_id);
                Task::none()
            }
            
            Message::SageResponseReceived(response) => {
                self.sage_client.update_with_response(&response);
                self.sage_responses.push(response);
                Task::none()
            }
            
            Message::SageStatusRequested => {
                // Use the fixed NATS client for status requests
                Task::perform(
                    crate::nats_client_fixed::commands::request_sage_status(),
                    |msg| msg
                )
            }
            
            Message::SageStatusReceived(status) => {
                self.sage_status = Some(status);
                Task::none()
            }
            
            Message::SageClearConversation => {
                self.sage_client.clear_conversation();
                self.sage_responses.clear();
                Task::none()
            }
            
            Message::SageNewSession => {
                self.sage_client.new_session();
                self.sage_responses.clear();
                self.sage_status = None;
                Task::none()
            }
            
            Message::ThemeToggled => {
                self.dark_mode = !self.dark_mode;
                self.theme = if self.dark_mode { Theme::Dark } else { Theme::Light };
                Task::none()
            }
            
            // Health and Monitoring Message Handlers
            Message::HealthCheckRequested => {
                // Health checks now come through events automatically
                // No manual request needed with proper NATS integration
                Task::none()
            }
            
            
            // Unhandled messages
            _ => Task::none(),
        }
    }
    
    pub fn view(&self) -> Element<'_, Message> {
        let mut content_items = vec![
            self.view_header(),
            self.view_tabs(),
        ];
        
        // Show error message if there is one
        if let Some(ref error) = self.error_message {
            content_items.push(
                container(
                    row![
                        text(format!("⚠️ Error: {}", error)).size(14),
                        Space::with_width(Length::Fill),
                        button("✕")
                            .on_press(Message::ErrorDismissed),
                    ]
                    .align_y(iced::alignment::Vertical::Center)
                )
                .padding(12)
                .into()
            );
        }
        
        content_items.push(self.view_content());
        content_items.push(self.view_status_bar());
        
        let content = column(content_items)
            .spacing(16)
            .padding(24);
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl CimManagerApp {
    fn view_header(&self) -> Element<'_, Message> {
        let theme_icon = if self.dark_mode { "☀️" } else { "🌙" };
        
        let header_content = row![
            text("CIM Claude Adapter Manager").size(28),
            Space::with_width(Length::Fill),
            button(text(theme_icon).size(20))
                .on_press(Message::ThemeToggled)
                .style(Modern::secondary_button()),
            Space::with_width(16),
            self.view_connection_controls(),
        ]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(12);
        
        container(header_content)
            .padding(Padding::from([16, 24]))
            .width(Length::Fill)
            
            .into()
    }
    
    fn view_connection_controls(&self) -> Element<'_, Message> {
        let status_text = if self.connected {
            text("🟢 Connected")
        } else {
            text("🔴 Disconnected")
        };
        
        let connect_button = if self.connected {
            button(text("Disconnect"))
                .on_press(Message::Disconnected)
                .style(iced::widget::button::danger)
        } else {
            button(text("Connect"))
                .on_press(Message::Connect(self.nats_url.clone()))
                .style(iced::widget::button::success)
        };
        
        container(row![
            text_input("NATS URL", &self.nats_url)
                .on_input(Message::NatsUrlChanged)
                .width(Length::Fixed(320.0)),
            connect_button,
            status_text,
        ]
        .spacing(12)
        .align_y(iced::alignment::Vertical::Center))
        .padding(Padding::from([8, 12]))
        .style(iced::widget::container::bordered_box)
        .into()
    }
    
    fn view_tabs(&self) -> Element<'_, Message> {
        row![
            if self.current_tab == Tab::Dashboard {
                button("Dashboard")
                    .on_press(Message::TabSelected(Tab::Dashboard))
                    
            } else {
                button("Dashboard")
                    .on_press(Message::TabSelected(Tab::Dashboard))
                    
            },
            if self.current_tab == Tab::Conversations {
                button("Conversations")
                    .on_press(Message::TabSelected(Tab::Conversations))
                    
            } else {
                button("Conversations")
                    .on_press(Message::TabSelected(Tab::Conversations))
                    
            },
            if self.current_tab == Tab::Events {
                button("Events")
                    .on_press(Message::TabSelected(Tab::Events))
                    
            } else {
                button("Events")
                    .on_press(Message::TabSelected(Tab::Events))
                    
            },
            if self.current_tab == Tab::Monitoring {
                button("Monitoring")
                    .on_press(Message::TabSelected(Tab::Monitoring))
                    
            } else {
                button("Monitoring")
                    .on_press(Message::TabSelected(Tab::Monitoring))
                    
            },
            if self.current_tab == Tab::Sage {
                button("SAGE")
                    .on_press(Message::TabSelected(Tab::Sage))
                    
            } else {
                button("SAGE")
                    .on_press(Message::TabSelected(Tab::Sage))
                    
            },
            if self.current_tab == Tab::Settings {
                button("Settings")
                    .on_press(Message::TabSelected(Tab::Settings))
                    
            } else {
                button("Settings")
                    .on_press(Message::TabSelected(Tab::Settings))
                    
            },
        ]
        .spacing(12)
        .into()
    }
    
    fn view_content(&self) -> Element<'_, Message> {
        match self.current_tab {
            Tab::Dashboard => self.view_dashboard(),
            Tab::Conversations => self.view_conversations(),
            Tab::Events => self.view_events(),
            Tab::Monitoring => self.view_monitoring(),
            Tab::Sage => self.view_sage(),
            Tab::Settings => self.view_settings(),
        }
    }
    
    fn view_dashboard(&self) -> Element<'_, Message> {
        column![
            text("Dashboard").size(20),
            row![
                column![
                    text("System Health"),
                    text(format!("NATS: {}", if self.health_status.nats_connected { "✅" } else { "❌" })),
                    text(format!("Claude API: {}", if self.health_status.claude_api_available { "✅" } else { "❌" })),
                    text(format!("Active Conversations: {}", self.health_status.active_conversations)),
                    text(format!("Events Processed: {}", self.health_status.events_processed)),
                ]
                .spacing(5)
                .width(Length::FillPortion(1)),
                
                column![
                    text("Quick Actions"),
                    text_input("Session ID", &self.session_id_input)
                        .on_input(Message::SessionIdChanged),
                    text_input("Initial Prompt", &self.prompt_input)
                        .on_input(Message::PromptInputChanged),
                    button("🚀 Start Conversation")
                        .on_press(Message::StartConversation {
                            session_id: self.session_id_input.clone(),
                            initial_prompt: self.prompt_input.clone(),
                        })
                        .style(iced::widget::button::primary),
                    button("📊 Refresh Health")
                        .on_press(Message::HealthCheckRequested)
                        .style(iced::widget::button::secondary),
                    button("🔄 Reconnect NATS")
                        .on_press(Message::Connect(self.nats_url.clone()))
                        .style(iced::widget::button::secondary),
                ]
                .spacing(5)
                .width(Length::FillPortion(1)),
            ]
            .spacing(20),
        ]
        .spacing(10)
        .into()
    }
    
    fn view_conversations(&self) -> Element<'_, Message> {
        let conversations: Vec<Element<Message>> = self.conversations
            .iter()
            .map(|(id, aggregate)| {
                button(text(format!("{} - {} messages", id, aggregate.messages.len())))
                    .on_press(Message::ConversationSelected(id.clone()))
                    .width(Length::Fill)
                    .into()
            })
            .collect();
        
        column![
            text("Active Conversations").size(20),
            column(conversations).spacing(5),
        ]
        .spacing(10)
        .into()
    }
    
    fn view_events(&self) -> Element<'_, Message> {
        // Simplified events view - complex event system removed
        column![
            text("Events").size(24),
            text("Event tracking has been simplified."),
            text("SAGE orchestrator handles complex event processing."),
            text("Use the SAGE tab for detailed interactions."),
        ]
        .spacing(20)
        .into()
    }
    
    fn view_monitoring(&self) -> Element<'_, Message> {
        column![
            text("System Monitoring").size(20),
            text(format!("Total Conversations: {}", self.system_metrics.conversations_total)),
            text(format!("Active Conversations: {}", self.system_metrics.conversations_active)),
            text(format!("Events Published: {}", self.system_metrics.events_published)),
            text(format!("Events Consumed: {}", self.system_metrics.events_consumed)),
            text(format!("API Requests: {}", self.system_metrics.api_requests_total)),
            text(format!("Failed Requests: {}", self.system_metrics.api_requests_failed)),
            text(format!("Avg Response Time: {:.2}ms", self.system_metrics.response_time_avg_ms)),
        ]
        .spacing(10)
        .into()
    }
    
    fn view_settings(&self) -> Element<'_, Message> {
        column![
            text("Settings").size(20),
            text("NATS Configuration"),
            text_input("NATS URL", &self.nats_url)
                .on_input(Message::NatsUrlChanged),
            // Add more settings as needed
        ]
        .spacing(10)
        .into()
    }
    
    
    fn view_sage(&self) -> Element<'_, Message> {
        use iced::widget::scrollable;
        use iced_modern_theme::Modern;
        use iced::{Color, alignment};
        
        let mut content_items = Vec::new();
        
        // SAGE Header with enhanced styling
        content_items.push(
            container(
                row![
                    text("🎭 SAGE Orchestrator")
                        .size(28)
                        .color(Color::from_rgb(0.6, 0.3, 0.8)),
                    Space::with_width(Length::Fill),
                    button("Clear")
                        .on_press(Message::SageClearConversation)
                        .style(Modern::secondary_button()),
                    button("New Session")
                        .on_press(Message::SageNewSession)  
                        .style(Modern::primary_button()),
                    button("Status")
                        .on_press(Message::SageStatusRequested)
                        .style(Modern::secondary_button()),
                ].spacing(12)
                .align_y(alignment::Vertical::Center)
            )
            .padding([15, 20])
            .style(iced::widget::container::bordered_box)
            .width(Length::Fill)
            .into()
        );
        
        // Enhanced Status Display
        if let Some(ref status) = self.sage_status {
            content_items.push(
                container(
                    column![
                        row![
                            text("🧠")
                                .size(16),
                            text(format!("Consciousness: {} (Level {:.1})", 
                                if status.is_conscious { "Active" } else { "Inactive" },
                                status.consciousness_level
                            ))
                            .size(14)
                            .color(if status.is_conscious { 
                                Color::from_rgb(0.2, 0.8, 0.2) 
                            } else { 
                                Color::from_rgb(0.8, 0.2, 0.2) 
                            }),
                        ].spacing(8).align_y(alignment::Vertical::Center),
                        
                        row![
                            text(format!("👥 Available Experts: {}", status.available_agents))
                                .size(12),
                            text("•").size(12),
                            text(format!("📊 Orchestrations: {}", status.total_orchestrations))
                                .size(12),
                            text("•").size(12),
                            text(format!("🧬 Patterns: {}", status.patterns_learned))
                                .size(12),
                            text("•").size(12),
                            text(format!("💾 Memory: {}", status.memory_health))
                                .size(12)
                                .color(Color::from_rgb(0.4, 0.6, 0.8)),
                        ].spacing(5),
                    ].spacing(8)
                )
                .padding(15)
                .style(iced::widget::container::bordered_box)
                .width(Length::Fill)
                .into()
            );
        } else {
            content_items.push(
                container(
                    text("🔴 Status: Disconnected from SAGE")
                        .size(14)
                        .color(Color::from_rgb(0.8, 0.4, 0.4))
                )
                .padding(15)
                .style(iced::widget::container::bordered_box)
                .into()
            );
        }
        
        // Expert Selection with better styling
        content_items.push(
            container(
                row![
                    text("🎯 Expert:")
                        .size(14)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    text_input("Optional: specify expert (e.g., 'ddd-expert')", 
                              self.sage_selected_expert.as_ref().unwrap_or(&String::new()))
                        .on_input(|expert| Message::SageExpertSelected(
                            if expert.trim().is_empty() { None } else { Some(expert) }
                        ))
                        .width(Length::Fixed(300.0))
                        .id(iced::widget::text_input::Id::unique()),
                    text("🤖 SAGE auto-routes to appropriate experts")
                        .size(11)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                ].spacing(12).align_y(alignment::Vertical::Center)
            )
            .padding(10)
            .style(iced::widget::container::bordered_box)
            .width(Length::Fill)
            .into()
        );
        
        // Enhanced Query Input
        content_items.push(
            container(
                row![
                    text_input("Ask SAGE anything about CIM development...", &self.sage_query_input)
                        .on_input(Message::SageQueryInputChanged)
                        .on_submit(Message::SageSendQuery)
                        .padding(12)
                        .size(14)
                        .id(iced::widget::text_input::Id::unique())
                        .width(Length::Fill),
                    
                    button(
                        row![
                            text("🚀").size(14),
                            text("Send").size(14),
                        ].spacing(5)
                    )
                    .on_press(Message::SageSendQuery)
                    .padding([12, 20])
                    .style(Modern::primary_button()),
                ].spacing(12).align_y(alignment::Vertical::Center)
            )
            .padding(15)
            .style(iced::widget::container::bordered_box)
            .width(Length::Fill)
            .into()
        );
        
        // Enhanced Conversation History with better message styling
        if !self.sage_responses.is_empty() {
            let conversation_messages = self.sage_responses.iter().enumerate().map(|(i, response)| {
                column![
                    // User Query Box
                    container(
                        column![
                            row![
                                text("🙋").size(14),
                                text("You").size(14).color(Color::from_rgb(0.2, 0.6, 1.0)),
                                Space::with_width(Length::Fill),
                                text(format!("Query #{}", i + 1)).size(10).color(Color::from_rgb(0.6, 0.6, 0.6)),
                            ].spacing(8).align_y(alignment::Vertical::Center),
                            
                            // Reconstruct user query from context
                            if let Some(last_user_entry) = response.updated_context.conversation_history
                                .iter()
                                .filter(|entry| entry.role == "user")
                                .last() {
                                text(&last_user_entry.content)
                                    .size(13)
                                    .color(Color::from_rgb(0.3, 0.3, 0.3))
                            } else {
                                text("Previous query")
                                    .size(13)
                                    .color(Color::from_rgb(0.5, 0.5, 0.5))
                            }
                        ].spacing(6)
                    )
                    .padding(12)
                    .style(iced::widget::container::bordered_box)
                    .width(Length::Fill),
                    
                    // SAGE Response Box
                    container(
                        column![
                            row![
                                text("🎭").size(16),
                                text("SAGE").size(14).color(Color::from_rgb(0.6, 0.3, 0.8)),
                                Space::with_width(Length::Fill),
                                container(
                                    text(format!("{}%", (response.confidence_score * 100.0) as u8))
                                        .size(10)
                                        .color(if response.confidence_score > 0.8 { 
                                            Color::from_rgb(0.2, 0.8, 0.2) 
                                        } else if response.confidence_score > 0.6 { 
                                            Color::from_rgb(0.8, 0.8, 0.2) 
                                        } else { 
                                            Color::from_rgb(0.8, 0.4, 0.2) 
                                        })
                                )
                                .padding([2, 6])
                                .style(iced::widget::container::bordered_box),
                            ].spacing(8).align_y(alignment::Vertical::Center),
                            
                            // Expert badges
                            if !response.expert_agents_used.is_empty() {
                                row(
                                    response.expert_agents_used.iter().map(|expert| {
                                        container(
                                            text(format!("@{}", expert))
                                                .size(9)
                                                .color(Color::from_rgb(0.4, 0.4, 0.8))
                                        )
                                        .padding([2, 6])
                                        .style(iced::widget::container::bordered_box)
                                        .into()
                                    }).collect::<Vec<_>>()
                                ).spacing(4)
                            } else {
                                row![].into()
                            },
                            
                            // Response content
                            scrollable(
                                text(&response.response)
                                    .size(13)
                                    .color(Color::from_rgb(0.2, 0.2, 0.2))
                            ).height(Length::Shrink),
                            
                            // Metadata
                            row![
                                text(format!("🔗 Complexity: {}", response.orchestration_complexity))
                                    .size(9)
                                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
                                Space::with_width(Length::Fill),
                                text(format!("📋 ID: {}", if response.request_id.len() >= 8 { 
                                    &response.request_id[..8] 
                                } else { 
                                    &response.request_id 
                                }))
                                    .size(9)
                                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
                            ].spacing(5),
                        ].spacing(8)
                    )
                    .padding(15)
                    .style(iced::widget::container::bordered_box)
                    .width(Length::Fill),
                    
                    Space::with_height(12),
                ].into()
            }).collect::<Vec<_>>();
            
            content_items.push(
                scrollable(
                    column(conversation_messages).spacing(8)
                )
                .height(Length::Fixed(500.0))
                .id(iced::widget::scrollable::Id::unique())
                .into()
            );
        } else {
            content_items.push(
                container(
                    column![
                        text("💬 No conversation yet")
                            .size(16)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                        text("Start by typing a question about CIM development above.")
                            .size(12)
                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                        text("Examples: 'How do I create a domain?', 'Set up NATS infrastructure', 'Help with testing'")
                            .size(11)
                            .color(Color::from_rgb(0.4, 0.4, 0.4)),
                    ].spacing(8)
                )
                .padding(30)
                .center_x(Length::Fill)
                .center_y(Length::Fixed(200.0))
                .style(iced::widget::container::bordered_box)
                .width(Length::Fill)
                .into()
            );
        }
        
        // Conversation Summary
        if !self.sage_responses.is_empty() {
            content_items.push(
                container(
                    column![
                        text("📊 Conversation Summary").size(12).color(Color::from_rgb(0.4, 0.4, 0.8)),
                        text(self.sage_client.get_conversation_summary()).size(10),
                        text(format!("Expert agents used: {:?}", self.sage_client.get_expert_agents_used())).size(10),
                    ].spacing(3)
                )
                .padding(12)
                .style(iced::widget::container::bordered_box)
                .width(Length::Fill)
                .into()
            );
        }
        
        let content = column(content_items).spacing(15).padding(20);
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    fn view_status_bar(&self) -> Element<'_, Message> {
        if let Some(error) = &self.connection_error {
            row![
                text(format!("Error: {}", error)),
                Space::with_width(Length::Fill),
                button("Dismiss").on_press(Message::ErrorDismissed),
            ]
            .align_y(iced::alignment::Vertical::Center)
            .into()
        } else if let Some(error) = &self.error_message {
            row![
                text(format!("Error: {}", error)),
                Space::with_width(Length::Fill),
                button("Dismiss").on_press(Message::ErrorDismissed),
            ]
            .align_y(iced::alignment::Vertical::Center)
            .into()
        } else {
            row![
                text(format!("Last Health Check: {}", 
                    self.health_status.last_check.format("%H:%M:%S"))),
                Space::with_width(Length::Fill),
                text(format!("Conversations: {}", self.conversations.len())),
            ]
            .align_y(iced::alignment::Vertical::Center)
            .into()
        }
    }
    
    /// Send SAGE request via global NATS client
    async fn send_sage_request_global(request: crate::sage_client::SageRequest, domain: Option<String>) -> Message {
        let request_id = request.request_id.clone();
        
        // Build domain-aware subject using cim-subject pattern
        // Pattern: {domain}.commands.sage.request
        let subject = if let Some(ref d) = domain {
            format!("{}.commands.sage.request", d)
        } else {
            "commands.sage.request".to_string()
        };
        
        // Connect to NATS for sending with timeout and proper error handling
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            async_nats::connect("nats://localhost:4222")
        ).await {
            Ok(Ok(client)) => {
                match serde_json::to_vec(&request) {
                    Ok(json) => {
                        match client.publish(subject.clone(), json.into()).await {
                            Ok(_) => {
                                tracing::info!("✅ SAGE request sent globally: {}", request_id);
                                Message::SageRequestSent(request_id)
                            }
                            Err(e) => {
                                tracing::error!("❌ Failed to send SAGE request: {}", e);
                                Message::Error(format!("Failed to send SAGE request: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to serialize SAGE request: {}", e);
                        Message::Error(format!("Failed to serialize SAGE request: {}", e))
                    }
                }
            }
            Ok(Err(e)) => {
                tracing::error!("❌ NATS connection error: {}", e);
                Message::Error(format!("NATS connection failed: {}", e))
            }
            Err(_) => {
                tracing::error!("❌ NATS connection timeout");
                Message::Error("NATS connection timeout".to_string())
            }
        }
    }
}

impl Default for CimManagerApp {
    fn default() -> Self {
        let (app, _task) = Self::new();
        app
    }
}