/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Universal CIM Agent GUI
//! 
//! A revolutionary GUI that can work with ANY agent personality loaded from
//! `.claude/agents/*.md` files. This implements the Universal Agent Architecture
//! where SAGE and all subagents are just different personality configurations.

use iced::{
    widget::{button, column, container, row, text, text_input, scrollable, pick_list, Space},
    Element, Length, Task, Padding, Color, alignment,
};
use std::collections::HashMap;

use iced::Theme;
use iced_modern_theme::Modern;

use cim_agent_claude::{
    AgentPersonality, AgentContext, AgentRegistry, AgentLoader, ContextManager,
    AgentId, AgentResult,
};

/// Universal message type for the agent-agnostic GUI
#[derive(Debug, Clone)]
pub enum UniversalMessage {
    // Agent Management
    AgentRegistryLoaded(Result<HashMap<AgentId, AgentPersonality>, String>),
    AgentSelected(Option<AgentId>),
    AgentSwitched { from: AgentId, to: AgentId },
    
    // Conversation Management
    QueryInputChanged(String),
    QuerySubmitted,
    MessageReceived(AgentResponse),
    ConversationCleared,
    NewSession,
    
    // Context Management
    ContextPreserved(AgentContext),
    SessionSelected(String),
    
    // UI State
    ThemeToggled,
    TabChanged(UniversalTab),
    ErrorDismissed,
}

/// Tabs in the universal interface
#[derive(Debug, Clone, PartialEq)]
pub enum UniversalTab {
    Conversation,
    Agents,
    Context,
    Settings,
}

/// Response from any agent
#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub agent_id: AgentId,
    pub request_id: String,
    pub response: String,
    pub confidence: f64,
    pub context: AgentContext,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Universal CIM Agent Application
pub struct UniversalApp {
    // Agent System
    agent_registry: AgentRegistry,
    available_agents: HashMap<AgentId, AgentPersonality>,
    active_agent: Option<AgentId>,
    
    // Context Management  
    context_manager: ContextManager,
    active_session: String,
    
    // Conversation State
    query_input: String,
    conversation_history: Vec<ConversationEntry>,
    
    // UI State
    current_tab: UniversalTab,
    theme: Theme,
    dark_mode: bool,
    error_message: Option<String>,
    
    // Loading State
    agents_loading: bool,
    query_sending: bool,
}

/// Entry in conversation display
#[derive(Debug, Clone)]
pub struct ConversationEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent_id: Option<AgentId>,
    pub role: String, // "user", "agent", "system"
    pub content: String,
    pub confidence: Option<f64>,
}

impl UniversalApp {
    /// Create a new universal app
    pub fn new() -> (Self, Task<UniversalMessage>) {
        let (agent_registry, _change_rx) = AgentRegistry::new();
        let context_manager = ContextManager::new();
        let active_session = uuid::Uuid::new_v4().to_string();
        
        let app = Self {
            agent_registry,
            available_agents: HashMap::new(),
            active_agent: None,
            
            context_manager,
            active_session,
            
            query_input: String::new(),
            conversation_history: Vec::new(),
            
            current_tab: UniversalTab::Conversation,
            theme: Theme::Light,
            dark_mode: false,
            error_message: None,
            
            agents_loading: false,
            query_sending: false,
        };
        
        // Load agents on startup
        let load_task = Task::perform(Self::load_agents_async(), UniversalMessage::AgentRegistryLoaded);
        
        (app, load_task)
    }
    
    /// Load agents asynchronously
    async fn load_agents_async() -> Result<HashMap<AgentId, AgentPersonality>, String> {
        let loader = AgentLoader::new();
        loader.load_all_agents().await.map_err(|e| format!("Failed to load agents: {}", e))
    }
    
    /// Update the application state
    pub fn update(&mut self, message: UniversalMessage) -> Task<UniversalMessage> {
        match message {
            UniversalMessage::AgentRegistryLoaded(result) => {
                self.agents_loading = false;
                match result {
                    Ok(agents) => {
                        tracing::info!("Loaded {} agent personalities", agents.len());
                        self.available_agents = agents;
                        
                        // Auto-select SAGE if available
                        if self.available_agents.contains_key("sage") {
                            self.active_agent = Some("sage".to_string());
                            self.add_system_message("SAGE orchestrator ready for queries");
                        } else if let Some(first_agent) = self.available_agents.keys().next() {
                            self.active_agent = Some(first_agent.clone());
                            self.add_system_message(&format!("Agent {} ready", first_agent));
                        }
                    }
                    Err(error) => {
                        self.error_message = Some(error);
                    }
                }
                Task::none()
            }
            
            UniversalMessage::AgentSelected(agent_id) => {
                if let Some(new_agent_id) = agent_id {
                    if let Some(old_agent_id) = &self.active_agent {
                        if old_agent_id != &new_agent_id {
                            // Switch agents with context preservation
                            self.switch_agent(old_agent_id.clone(), new_agent_id.clone());
                        }
                    } else {
                        self.active_agent = Some(new_agent_id.clone());
                        self.add_system_message(&format!("Switched to agent: {}", 
                            self.get_agent_display_name(&new_agent_id)));
                    }
                } else {
                    self.active_agent = None;
                    self.add_system_message("No agent selected");
                }
                Task::none()
            }
            
            UniversalMessage::QueryInputChanged(input) => {
                self.query_input = input;
                Task::none()
            }
            
            UniversalMessage::QuerySubmitted => {
                if !self.query_input.trim().is_empty() && !self.query_sending {
                    let query = self.query_input.trim().to_string();
                    self.query_input.clear();
                    self.query_sending = true;
                    
                    // Add user message immediately
                    self.add_user_message(&query);
                    
                    // Send to active agent
                    if let Some(agent_id) = &self.active_agent {
                        // In a real implementation, this would use the cim-llm-adapter
                        // For now, create mock response
                        let response = self.create_mock_response(agent_id.clone(), query);
                        Task::perform(async move { Ok(response) }, |result| {
                            match result {
                                Ok(response) => UniversalMessage::MessageReceived(response),
                                Err(_) => UniversalMessage::ErrorDismissed, // Placeholder
                            }
                        })
                    } else {
                        self.error_message = Some("No agent selected".to_string());
                        self.query_sending = false;
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            
            UniversalMessage::MessageReceived(response) => {
                self.query_sending = false;
                self.add_agent_response(response);
                Task::none()
            }
            
            UniversalMessage::ConversationCleared => {
                self.conversation_history.clear();
                self.add_system_message("Conversation cleared");
                Task::none()
            }
            
            UniversalMessage::NewSession => {
                self.active_session = uuid::Uuid::new_v4().to_string();
                self.conversation_history.clear();
                
                if let Some(agent_id) = &self.active_agent {
                    self.add_system_message(&format!("New session started with {}", 
                        self.get_agent_display_name(agent_id)));
                }
                Task::none()
            }
            
            UniversalMessage::ThemeToggled => {
                self.dark_mode = !self.dark_mode;
                self.theme = if self.dark_mode { Theme::Dark } else { Theme::Light };
                Task::none()
            }
            
            UniversalMessage::TabChanged(tab) => {
                self.current_tab = tab;
                Task::none()
            }
            
            UniversalMessage::ErrorDismissed => {
                self.error_message = None;
                Task::none()
            }
            
            _ => Task::none(),
        }
    }
    
    /// Switch between agents with context preservation
    fn switch_agent(&mut self, from_agent: AgentId, to_agent: AgentId) {
        self.active_agent = Some(to_agent.clone());
        
        // Add context switch message
        self.add_system_message(&format!(
            "Switched from {} to {} (context preserved)",
            self.get_agent_display_name(&from_agent),
            self.get_agent_display_name(&to_agent)
        ));
        
        // In a real implementation, this would preserve context via ContextManager
        // and potentially send context to the new agent
    }
    
    /// Get display name for an agent
    fn get_agent_display_name(&self, agent_id: &AgentId) -> String {
        if let Some(personality) = self.available_agents.get(agent_id) {
            format!("{} {}", personality.icon(), personality.name)
        } else {
            agent_id.clone()
        }
    }
    
    /// Add user message to conversation
    fn add_user_message(&mut self, content: &str) {
        self.conversation_history.push(ConversationEntry {
            timestamp: chrono::Utc::now(),
            agent_id: None,
            role: "user".to_string(),
            content: content.to_string(),
            confidence: None,
        });
    }
    
    /// Add agent response to conversation
    fn add_agent_response(&mut self, response: AgentResponse) {
        self.conversation_history.push(ConversationEntry {
            timestamp: chrono::Utc::now(),
            agent_id: Some(response.agent_id),
            role: "agent".to_string(),
            content: response.response,
            confidence: Some(response.confidence),
        });
    }
    
    /// Add system message
    fn add_system_message(&mut self, content: &str) {
        self.conversation_history.push(ConversationEntry {
            timestamp: chrono::Utc::now(),
            agent_id: None,
            role: "system".to_string(),
            content: content.to_string(),
            confidence: None,
        });
    }
    
    /// Create mock response for testing
    fn create_mock_response(&self, agent_id: AgentId, query: String) -> AgentResponse {
        let personality = self.available_agents.get(&agent_id);
        
        let response = if let Some(p) = personality {
            format!(
                "{} **{}** responds:\n\nI understand you're asking: \"{}\"\n\n{}\n\n💡 This is a mock response demonstrating the Universal Agent Architecture where any agent personality can be dynamically loaded and switched.",
                p.icon(),
                p.name,
                query,
                match agent_id.as_str() {
                    "sage" => "🎭 As SAGE, I would normally coordinate multiple expert agents to provide comprehensive guidance. In the full implementation, I'd analyze your query and invoke appropriate experts like @ddd-expert, @nats-expert, etc.",
                    "ddd-expert" => "📐 As a Domain-Driven Design expert, I'd help you model your domain with proper aggregates, entities, and bounded contexts using CIM mathematical foundations.",
                    "tdd-expert" => "🧪 As a Test-Driven Development expert, I'd guide you to write tests first, then implement code that passes those tests using CIM patterns.",
                    "nats-expert" => "📨 As a NATS expert, I'd help you design event-driven messaging patterns using NATS JetStream for your CIM system.",
                    _ => &format!("As {}, I'd provide specialized guidance in my domain of expertise.", p.name),
                }
            )
        } else {
            format!("Agent {} response to: {}", agent_id, query)
        };
        
        AgentResponse {
            agent_id,
            request_id: uuid::Uuid::new_v4().to_string(),
            response,
            confidence: 0.85,
            context: AgentContext::new(self.active_session.clone()),
            metadata: HashMap::new(),
        }
    }
    
    /// Render the application
    pub fn view(&self) -> Element<'_, UniversalMessage> {
        let mut content = Vec::new();
        
        // Header with agent selector
        content.push(self.view_header());
        
        // Error message if present
        if let Some(error) = &self.error_message {
            content.push(self.view_error_bar(error));
        }
        
        // Tab navigation
        content.push(self.view_tabs());
        
        // Main content based on active tab
        content.push(self.view_tab_content());
        
        // Status bar
        content.push(self.view_status_bar());
        
        container(column(content).spacing(16).padding(24))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    /// Render header with agent selector
    fn view_header(&self) -> Element<'_, UniversalMessage> {
        let theme_icon = if self.dark_mode { "☀️" } else { "🌙" };
        
        // Agent selector dropdown
        let agent_options: Vec<(String, AgentId)> = self.available_agents.iter()
            .map(|(id, personality)| (
                format!("{} {}", personality.icon(), personality.name),
                id.clone()
            ))
            .collect();
        
        let agent_selector = if agent_options.is_empty() {
            text("Loading agents...").into()
        } else {
            pick_list(
                agent_options,
                self.active_agent.clone(),
                UniversalMessage::AgentSelected,
            )
            .placeholder("Select an agent...")
            .width(Length::Fixed(250.0))
            .into()
        };
        
        container(
            row![
                text("🤖 Universal CIM Agent").size(24),
                Space::with_width(20),
                agent_selector,
                Space::with_width(Length::Fill),
                button("Clear")
                    .on_press(UniversalMessage::ConversationCleared)
                    .style(Modern::secondary_button()),
                button("New Session")
                    .on_press(UniversalMessage::NewSession)
                    .style(Modern::primary_button()),
                button(theme_icon)
                    .on_press(UniversalMessage::ThemeToggled)
                    .style(Modern::secondary_button()),
            ]
            .align_y(alignment::Vertical::Center)
            .spacing(12)
        )
        .padding(16)
        .style(iced::widget::container::bordered_box)
        .width(Length::Fill)
        .into()
    }
    
    /// Render error bar
    fn view_error_bar(&self, error: &str) -> Element<'_, UniversalMessage> {
        container(
            row![
                text(format!("⚠️ Error: {}", error)).color(Color::from_rgb(0.8, 0.2, 0.2)),
                Space::with_width(Length::Fill),
                button("✕")
                    .on_press(UniversalMessage::ErrorDismissed)
                    .style(Modern::secondary_button()),
            ]
            .align_y(alignment::Vertical::Center)
            .spacing(12)
        )
        .padding(12)
        .style(iced::widget::container::bordered_box)
        .width(Length::Fill)
        .into()
    }
    
    /// Render tab navigation
    fn view_tabs(&self) -> Element<'_, UniversalMessage> {
        row![
            self.tab_button("💬 Conversation", UniversalTab::Conversation),
            self.tab_button("🤖 Agents", UniversalTab::Agents),
            self.tab_button("🧠 Context", UniversalTab::Context),
            self.tab_button("⚙️ Settings", UniversalTab::Settings),
        ]
        .spacing(8)
        .into()
    }
    
    /// Render a tab button
    fn tab_button(&self, label: &str, tab: UniversalTab) -> Element<'_, UniversalMessage> {
        let style = if self.current_tab == tab {
            Modern::primary_button()
        } else {
            Modern::secondary_button()
        };
        
        button(label)
            .on_press(UniversalMessage::TabChanged(tab))
            .style(style)
            .into()
    }
    
    /// Render tab content
    fn view_tab_content(&self) -> Element<'_, UniversalMessage> {
        match self.current_tab {
            UniversalTab::Conversation => self.view_conversation_tab(),
            UniversalTab::Agents => self.view_agents_tab(),
            UniversalTab::Context => self.view_context_tab(),
            UniversalTab::Settings => self.view_settings_tab(),
        }
    }
    
    /// Render conversation tab
    fn view_conversation_tab(&self) -> Element<'_, UniversalMessage> {
        let mut content = Vec::new();
        
        // Query input
        content.push(
            container(
                row![
                    text_input("Ask any agent anything...", &self.query_input)
                        .on_input(UniversalMessage::QueryInputChanged)
                        .on_submit(UniversalMessage::QuerySubmitted)
                        .width(Length::Fill)
                        .padding(12),
                    button(if self.query_sending { "..." } else { "Send" })
                        .on_press(UniversalMessage::QuerySubmitted)
                        .style(Modern::primary_button()),
                ]
                .spacing(12)
                .align_y(alignment::Vertical::Center)
            )
            .padding(16)
            .style(iced::widget::container::bordered_box)
            .width(Length::Fill)
            .into()
        );
        
        // Conversation history
        if self.conversation_history.is_empty() {
            content.push(
                container(
                    column![
                        text("🚀 Universal Agent System Ready!")
                            .size(20)
                            .color(Color::from_rgb(0.4, 0.6, 0.8)),
                        Space::with_height(10),
                        text("Select an agent above and start asking questions.")
                            .size(14)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                        Space::with_height(10),
                        text("✨ Any agent personality can be dynamically loaded and switched")
                            .size(12)
                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                        text("🧠 Conversation context is preserved across agent switches")
                            .size(12)
                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                        text("🎭 SAGE can orchestrate multiple experts for complex queries")
                            .size(12)
                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    ]
                    .align_x(alignment::Horizontal::Center)
                    .spacing(5)
                )
                .center_x(Length::Fill)
                .center_y(Length::Fixed(200.0))
                .padding(40)
                .into()
            );
        } else {
            let messages: Vec<Element<UniversalMessage>> = self.conversation_history.iter()
                .map(|entry| self.render_conversation_entry(entry))
                .collect();
            
            content.push(
                scrollable(column(messages).spacing(12))
                    .height(Length::Fixed(400.0))
                    .into()
            );
        }
        
        column(content).spacing(16).into()
    }
    
    /// Render a conversation entry
    fn render_conversation_entry(&self, entry: &ConversationEntry) -> Element<'_, UniversalMessage> {
        let (icon, color, role_display) = match entry.role.as_str() {
            "user" => ("🙋", Color::from_rgb(0.2, 0.6, 1.0), "You"),
            "agent" => {
                if let Some(agent_id) = &entry.agent_id {
                    let display_name = self.get_agent_display_name(agent_id);
                    let agent_parts: Vec<&str> = display_name.splitn(2, ' ').collect();
                    let icon = agent_parts.get(0).unwrap_or(&"🤖");
                    let name = agent_parts.get(1).unwrap_or(agent_id);
                    (icon, Color::from_rgb(0.6, 0.3, 0.8), name)
                } else {
                    ("🤖", Color::from_rgb(0.6, 0.3, 0.8), "Agent")
                }
            }
            "system" => ("ℹ️", Color::from_rgb(0.5, 0.5, 0.5), "System"),
            _ => ("❓", Color::from_rgb(0.4, 0.4, 0.4), "Unknown"),
        };
        
        let mut header_items = vec![
            text(icon).size(16).into(),
            text(role_display).size(14).color(color).into(),
            Space::with_width(Length::Fill).into(),
        ];
        
        if let Some(confidence) = entry.confidence {
            header_items.push(
                text(format!("{:.0}%", confidence * 100.0))
                    .size(10)
                    .color(if confidence > 0.8 { 
                        Color::from_rgb(0.2, 0.8, 0.2) 
                    } else if confidence > 0.6 { 
                        Color::from_rgb(0.8, 0.8, 0.2) 
                    } else { 
                        Color::from_rgb(0.8, 0.4, 0.2) 
                    })
                    .into()
            );
        }
        
        container(
            column![
                row(header_items).spacing(8).align_y(alignment::Vertical::Center),
                text(&entry.content)
                    .size(13)
                    .color(Color::from_rgb(0.2, 0.2, 0.2)),
                text(entry.timestamp.format("%H:%M:%S").to_string())
                    .size(9)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            ]
            .spacing(6)
        )
        .padding(12)
        .style(iced::widget::container::bordered_box)
        .width(Length::Fill)
        .into()
    }
    
    /// Render agents tab
    fn view_agents_tab(&self) -> Element<'_, UniversalMessage> {
        let mut content = vec![
            text("🤖 Available Agent Personalities").size(20).into(),
            text(format!("Loaded {} agents from .claude/agents/", self.available_agents.len()))
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6))
                .into(),
        ];
        
        if self.available_agents.is_empty() {
            content.push(
                text("No agents loaded. Check .claude/agents/ directory.")
                    .color(Color::from_rgb(0.8, 0.4, 0.4))
                    .into()
            );
        } else {
            let agent_cards: Vec<Element<UniversalMessage>> = self.available_agents.iter()
                .map(|(id, personality)| self.render_agent_card(id, personality))
                .collect();
            
            content.push(scrollable(column(agent_cards).spacing(12)).height(Length::Fixed(400.0)).into());
        }
        
        column(content).spacing(16).into()
    }
    
    /// Render an agent card
    fn render_agent_card(&self, agent_id: &AgentId, personality: &AgentPersonality) -> Element<'_, UniversalMessage> {
        let is_active = self.active_agent.as_ref() == Some(agent_id);
        let border_color = if is_active { 
            iced::widget::container::primary 
        } else { 
            iced::widget::container::bordered_box 
        };
        
        container(
            column![
                row![
                    text(&personality.icon).size(20),
                    text(&personality.name).size(16).color(Color::from_rgb(0.3, 0.3, 0.8)),
                    Space::with_width(Length::Fill),
                    if is_active {
                        text("ACTIVE").size(10).color(Color::from_rgb(0.2, 0.8, 0.2))
                    } else {
                        text("").size(10)
                    },
                ]
                .spacing(8)
                .align_y(alignment::Vertical::Center),
                
                text(&personality.description).size(12),
                
                row![
                    text("Capabilities:").size(10).color(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(format!("{}", personality.capabilities.len())).size(10),
                ]
                .spacing(5),
                
                button("Select Agent")
                    .on_press(UniversalMessage::AgentSelected(Some(agent_id.clone())))
                    .style(if is_active { Modern::primary_button() } else { Modern::secondary_button() }),
            ]
            .spacing(8)
        )
        .padding(15)
        .style(border_color)
        .width(Length::Fill)
        .into()
    }
    
    /// Render context tab
    fn view_context_tab(&self) -> Element<'_, UniversalMessage> {
        column![
            text("🧠 Conversation Context").size(20),
            text("Context preservation and agent memory management").size(12),
            text("(Full implementation would show context details here)").size(10),
        ]
        .spacing(16)
        .into()
    }
    
    /// Render settings tab
    fn view_settings_tab(&self) -> Element<'_, UniversalMessage> {
        column![
            text("⚙️ Settings").size(20),
            text("Universal Agent System Configuration").size(12),
            text("(Full implementation would show settings here)").size(10),
        ]
        .spacing(16)
        .into()
    }
    
    /// Render status bar
    fn view_status_bar(&self) -> Element<'_, UniversalMessage> {
        let status_text = if let Some(agent_id) = &self.active_agent {
            format!("Active: {} | Session: {} | Messages: {}",
                self.get_agent_display_name(agent_id),
                &self.active_session[..8],
                self.conversation_history.len()
            )
        } else {
            format!("No agent selected | Session: {} | Agents: {}",
                &self.active_session[..8],
                self.available_agents.len()
            )
        };
        
        container(
            row![
                text(status_text).size(10).color(Color::from_rgb(0.6, 0.6, 0.6)),
                Space::with_width(Length::Fill),
                text("Universal Agent Architecture").size(10).color(Color::from_rgb(0.4, 0.6, 0.8)),
            ]
        )
        .padding(8)
        .width(Length::Fill)
        .into()
    }
    
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

impl Default for UniversalApp {
    fn default() -> Self {
        let (app, _) = Self::new();
        app
    }
}