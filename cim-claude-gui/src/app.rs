/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use iced::{
    widget::{button, column, container, row, text, text_input, Space},
    Element, Length, Task, Padding,
};
use std::collections::HashMap;

// Use modern theme system for beautiful styling
use iced::Theme;
use iced_modern_theme::Modern;

use cim_claude_adapter::{
    domain::{commands::Command as DomainCommand, events::EventEnvelope, value_objects::{SessionId, CorrelationId, ConversationContext}, ConversationAggregate},
};
use crate::{
    messages::{Message, Tab, HealthStatus, SystemMetrics, CimExpertConversation, CimExpertMessage, CimExpertMessageRole},
    nats_client::GuiNatsClient,
};
use cim_claude_adapter::CimExpertTopic;

/// Main CIM Manager Application State
pub struct CimManagerApp {
    // Connection State
    nats_client: GuiNatsClient,
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
    
    // Domain State
    conversations: HashMap<String, ConversationAggregate>,
    recent_events: Vec<EventEnvelope>,
    health_status: HealthStatus,
    system_metrics: SystemMetrics,
    
    // CIM Expert State
    cim_expert_conversation: Option<CimExpertConversation>,
    cim_expert_message_input: String,
    cim_expert_context_input: String,
    cim_expert_selected_topic: CimExpertTopic,
}

impl CimManagerApp {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            nats_client: GuiNatsClient::new(),
            nats_url: "nats://localhost:4222".to_string(),
            connected: false,
            connection_error: None,
            
            current_tab: Tab::Dashboard,
            prompt_input: String::new(),
            session_id_input: SessionId::new().as_uuid().to_string(),
            selected_conversation: None,
            error_message: None,
            
            theme: Theme::Light,
            dark_mode: false,
            
            conversations: HashMap::new(),
            recent_events: Vec::new(),
            health_status: HealthStatus::default(),
            system_metrics: SystemMetrics::default(),
            
            // CIM Expert initialization
            cim_expert_conversation: None,
            cim_expert_message_input: String::new(),
            cim_expert_context_input: String::new(),
            cim_expert_selected_topic: CimExpertTopic::Architecture,
        };
        
        (app, Task::none())
    }
    
    pub fn title(&self) -> String {
        "CIM Claude Adapter Manager".to_string()
    }
    
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Connect(url) => {
                self.nats_url = url.clone();
                self.connection_error = None;
                
                let stream = self.nats_client.connect(url);
                Task::run(stream, |message| message)
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
                    let session_id = match uuid::Uuid::parse_str(&session_id) {
                        Ok(uuid) => SessionId::from_uuid(uuid),
                        Err(_) => {
                            self.error_message = Some("Invalid session ID format".to_string());
                            return Task::none();
                        }
                    };
                    
                    let prompt = match cim_claude_adapter::domain::value_objects::Prompt::new(initial_prompt) {
                        Ok(p) => p,
                        Err(e) => {
                            self.error_message = Some(format!("Invalid prompt: {}", e));
                            return Task::none();
                        }
                    };
                    
                    let correlation_id = CorrelationId::new();
                    let command = DomainCommand::StartConversation {
                        session_id: session_id.clone(),
                        initial_prompt: prompt,
                        context: ConversationContext::default(),
                        correlation_id: correlation_id.clone(),
                    };
                    
                    let command_envelope = command.with_metadata(correlation_id);
                    
                    let client = self.nats_client.clone();
                    Task::perform(
                        async move { client.send_command(command_envelope).await },
                        |result| match result {
                            Ok(_) => Message::Connected, // Placeholder
                            Err(e) => Message::ErrorOccurred(e),
                        }
                    )
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
            
            Message::ConversationEvent(event_envelope) => {
                // Update local state based on received events
                self.recent_events.insert(0, event_envelope.clone());
                if self.recent_events.len() > 100 {
                    self.recent_events.truncate(100);
                }
                
                // Update conversation state if needed
                // Note: Event handling can be extended here based on specific event types
                
                Task::none()
            }
            
            Message::ConversationUpdated(aggregate) => {
                let conversation_id = aggregate.session_id().as_uuid().to_string();
                self.conversations.insert(conversation_id, aggregate);
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
            
            // CIM Expert message handling
            Message::CimExpertTabSelected => {
                self.current_tab = Tab::CimExpert;
                Task::none()
            }
            
            Message::CimExpertStartConversation => {
                self.cim_expert_start_conversation()
            }
            
            Message::CimExpertSendMessage(message) => {
                self.cim_expert_send_message(message)
            }
            
            Message::CimExpertMessageInputChanged(input) => {
                self.cim_expert_message_input = input;
                Task::none()
            }
            
            Message::CimExpertTopicSelected(topic) => {
                self.cim_expert_selected_topic = topic;
                Task::none()
            }
            
            Message::CimExpertContextChanged(context) => {
                self.cim_expert_context_input = context;
                Task::none()
            }
            
            Message::CimExpertConversationReceived(conversation) => {
                self.cim_expert_conversation = Some(conversation);
                Task::none()
            }
            
            Message::CimExpertResponseReceived(_message_id, response) => {
                if let Some(ref mut conversation) = self.cim_expert_conversation {
                    conversation.messages.push(CimExpertMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        role: CimExpertMessageRole::Expert,
                        content: response,
                        timestamp: chrono::Utc::now(),
                        topic: Some(self.cim_expert_selected_topic.clone()),
                    });
                    conversation.last_activity = chrono::Utc::now();
                }
                Task::none()
            }
            
            Message::ThemeToggled => {
                self.dark_mode = !self.dark_mode;
                self.theme = if self.dark_mode { Theme::Dark } else { Theme::Light };
                Task::none()
            }
            
            // Health and Monitoring Message Handlers
            Message::HealthCheckRequested => {
                if self.connected {
                    // Trigger a health check request to the CIM system
                    let client = self.nats_client.clone();
                    Task::perform(
                        async move { client.request_health_check().await },
                        |result| match result {
                            Ok(health) => Message::HealthCheckReceived(health),
                            Err(e) => Message::ErrorOccurred(format!("Health check failed: {}", e)),
                        }
                    )
                } else {
                    self.error_message = Some("Not connected to NATS".to_string());
                    Task::none()
                }
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
                            .on_press(Message::ErrorDismissed)
                            .style(Modern::secondary_button()),
                    ]
                    .align_y(iced::alignment::Vertical::Center)
                )
                .padding(12)
                .style(Modern::card_container())
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
            .style(Modern::floating_container())
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
                .style(Modern::blue_tinted_button()),
            Space::with_width(16),
            self.view_connection_controls(),
        ]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(12);
        
        container(header_content)
            .padding(Padding::from([16, 24]))
            .width(Length::Fill)
            .style(Modern::card_container())
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
                .style(Modern::secondary_button())
        } else {
            button(text("Connect"))
                .on_press(Message::Connect(self.nats_url.clone()))
                .style(Modern::primary_button())
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
        .style(Modern::card_container())
        .into()
    }
    
    fn view_tabs(&self) -> Element<'_, Message> {
        row![
            if self.current_tab == Tab::Dashboard {
                button("Dashboard")
                    .on_press(Message::TabSelected(Tab::Dashboard))
                    .style(Modern::primary_button())
            } else {
                button("Dashboard")
                    .on_press(Message::TabSelected(Tab::Dashboard))
                    .style(Modern::blue_tinted_button())
            },
            if self.current_tab == Tab::Conversations {
                button("Conversations")
                    .on_press(Message::TabSelected(Tab::Conversations))
                    .style(Modern::primary_button())
            } else {
                button("Conversations")
                    .on_press(Message::TabSelected(Tab::Conversations))
                    .style(Modern::blue_tinted_button())
            },
            if self.current_tab == Tab::Events {
                button("Events")
                    .on_press(Message::TabSelected(Tab::Events))
                    .style(Modern::primary_button())
            } else {
                button("Events")
                    .on_press(Message::TabSelected(Tab::Events))
                    .style(Modern::blue_tinted_button())
            },
            if self.current_tab == Tab::Monitoring {
                button("Monitoring")
                    .on_press(Message::TabSelected(Tab::Monitoring))
                    .style(Modern::primary_button())
            } else {
                button("Monitoring")
                    .on_press(Message::TabSelected(Tab::Monitoring))
                    .style(Modern::blue_tinted_button())
            },
            if self.current_tab == Tab::CimExpert {
                button("CIM Expert")
                    .on_press(Message::TabSelected(Tab::CimExpert))
                    .style(Modern::primary_button())
            } else {
                button("CIM Expert")
                    .on_press(Message::TabSelected(Tab::CimExpert))
                    .style(Modern::blue_tinted_button())
            },
            if self.current_tab == Tab::Settings {
                button("Settings")
                    .on_press(Message::TabSelected(Tab::Settings))
                    .style(Modern::primary_button())
            } else {
                button("Settings")
                    .on_press(Message::TabSelected(Tab::Settings))
                    .style(Modern::blue_tinted_button())
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
            Tab::CimExpert => self.view_cim_expert(),
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
                        .style(Modern::primary_button()),
                    button("📊 Refresh Health")
                        .on_press(Message::HealthCheckRequested)
                        .style(Modern::secondary_button()),
                    button("🔄 Reconnect NATS")
                        .on_press(Message::Connect(self.nats_url.clone()))
                        .style(Modern::secondary_button()),
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
                button(text(format!("{} - {:?}", id, aggregate.state())))
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
        let events: Vec<Element<Message>> = self.recent_events
            .iter()
            .take(20)
            .map(|event_envelope| {
                text(format!(
                    "[{}] {:?} - {}",
                    event_envelope.timestamp.format("%H:%M:%S"),
                    event_envelope.event.event_type(),
                    event_envelope.correlation_id.as_uuid()
                ))
                .size(12)
                .into()
            })
            .collect();
        
        column![
            text("Recent Events").size(20),
            column(events).spacing(2),
        ]
        .spacing(10)
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
    
    fn view_cim_expert(&self) -> Element<'_, Message> {
        let topic_text = format!("{:?}", self.cim_expert_selected_topic);
        
        if let Some(ref conversation) = self.cim_expert_conversation {
            // Show existing conversation
            let messages_view = conversation.messages.iter().enumerate().fold(
                column![],
                |col, (_i, msg)| {
                    let role_text = match msg.role {
                        CimExpertMessageRole::User => "You",
                        CimExpertMessageRole::Expert => "CIM Expert",
                        CimExpertMessageRole::System => "System",
                    };
                    
                    col.push(
                        container(
                            column![
                                text(format!("{}: {}", role_text, msg.timestamp.format("%H:%M:%S"))).size(12),
                                text(&msg.content).size(14),
                            ].spacing(5)
                        )
                        .padding(10)
                    )
                }
            );
            
            column![
                text("CIM Expert - Conversation").size(20),
                
                // Topic selector
                row![
                    text("Topic:"),
                    button(text(topic_text.clone()))
                        .on_press(Message::CimExpertTopicSelected(
                            match self.cim_expert_selected_topic {
                                CimExpertTopic::Architecture => CimExpertTopic::MathematicalFoundations,
                                CimExpertTopic::MathematicalFoundations => CimExpertTopic::NatsPatterns,
                                CimExpertTopic::NatsPatterns => CimExpertTopic::EventSourcing,
                                CimExpertTopic::EventSourcing => CimExpertTopic::DomainModeling,
                                CimExpertTopic::DomainModeling => CimExpertTopic::Implementation,
                                CimExpertTopic::Implementation => CimExpertTopic::Troubleshooting,
                                CimExpertTopic::Troubleshooting => CimExpertTopic::Architecture,
                                _ => CimExpertTopic::Architecture,
                            }
                        )),
                ].spacing(10),
                
                // Messages area
                container(messages_view).height(Length::Fixed(400.0)),
                
                // Message input
                row![
                    text_input("Ask the CIM Expert...", &self.cim_expert_message_input)
                        .on_input(Message::CimExpertMessageInputChanged)
                        .on_submit(Message::CimExpertSendMessage(self.cim_expert_message_input.clone())),
                    button("Send")
                        .on_press(Message::CimExpertSendMessage(self.cim_expert_message_input.clone())),
                ].spacing(10),
                
                button("End Conversation")
                    .on_press(Message::CimExpertConversationReceived(
                        CimExpertConversation {
                            id: "".to_string(),
                            created_at: chrono::Utc::now(),
                            last_activity: chrono::Utc::now(),
                            messages: vec![],
                            context: None,
                            user_id: None,
                        }
                    )),
            ]
            .spacing(15)
            .into()
        } else {
            // Show conversation starter
            column![
                text("CIM Expert").size(24),
                text("Start a conversation with the CIM Expert to learn about CIM's cognitive architecture, mathematical foundations, and implementation patterns.").size(14),
                
                // Context input
                column![
                    text("Context (optional):"),
                    text_input("Enter domain context for your questions...", &self.cim_expert_context_input)
                        .on_input(Message::CimExpertContextChanged),
                ].spacing(5),
                
                // Topic selector
                column![
                    text("Initial Topic:"),
                    row![
                        button(text(topic_text.clone()))
                            .on_press(Message::CimExpertTopicSelected(
                                match self.cim_expert_selected_topic {
                                    CimExpertTopic::Architecture => CimExpertTopic::MathematicalFoundations,
                                    CimExpertTopic::MathematicalFoundations => CimExpertTopic::NatsPatterns,
                                    CimExpertTopic::NatsPatterns => CimExpertTopic::EventSourcing,
                                    CimExpertTopic::EventSourcing => CimExpertTopic::DomainModeling,
                                    CimExpertTopic::DomainModeling => CimExpertTopic::Implementation,
                                    CimExpertTopic::Implementation => CimExpertTopic::Troubleshooting,
                                    CimExpertTopic::Troubleshooting => CimExpertTopic::Architecture,
                                    _ => CimExpertTopic::Architecture,
                                }
                            )),
                        text("(click to cycle through topics)"),
                    ].spacing(10),
                ].spacing(5),
                
                button("Start Conversation")
                    .on_press(Message::CimExpertStartConversation),
                    
                // Available topics list
                column![
                    text("Available Topics:"),
                    text("• Architecture - CIM system design and structure"),
                    text("• Mathematical Foundations - Category Theory, Graph Theory"),
                    text("• NATS Patterns - Event streaming and messaging"),
                    text("• Event Sourcing - Immutable event-driven systems"),
                    text("• Domain Modeling - DDD and aggregate design"),
                    text("• Implementation - Practical development guidance"),
                    text("• Troubleshooting - Problem solving and debugging"),
                ].spacing(5),
            ]
            .spacing(20)
            .into()
        }
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
    
    // CIM Expert helper methods
    fn cim_expert_start_conversation(&mut self) -> Task<Message> {
        let conversation_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        let conversation = CimExpertConversation {
            id: conversation_id.clone(),
            created_at: now,
            last_activity: now,
            messages: vec![
                CimExpertMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    role: CimExpertMessageRole::System,
                    content: "Welcome to CIM Expert! I'm here to help you understand CIM's cognitive architecture - including Conceptual Spaces, memory engram patterns, graph-based workflows, and emergent intelligence. What would you like to explore?".to_string(),
                    timestamp: now,
                    topic: None,
                }
            ],
            context: if self.cim_expert_context_input.is_empty() { 
                None 
            } else { 
                Some(self.cim_expert_context_input.clone()) 
            },
            user_id: None,
        };
        
        self.cim_expert_conversation = Some(conversation);
        self.cim_expert_context_input.clear();
        
        Task::none()
    }
    
    fn cim_expert_send_message(&mut self, message: String) -> Task<Message> {
        if let Some(ref mut conversation) = self.cim_expert_conversation {
            // Add user message
            conversation.messages.push(CimExpertMessage {
                id: uuid::Uuid::new_v4().to_string(),
                role: CimExpertMessageRole::User,
                content: message.clone(),
                timestamp: chrono::Utc::now(),
                topic: Some(self.cim_expert_selected_topic.clone()),
            });
            
            // Clear input
            self.cim_expert_message_input.clear();
            
            // Simulate expert response (in a real implementation, this would call the CimExpertService)
            let response_id = uuid::Uuid::new_v4().to_string();
            let mock_response = self.generate_mock_expert_response(&message);
            
            Task::perform(
                async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    (response_id, mock_response)
                },
                |(msg_id, response)| Message::CimExpertResponseReceived(msg_id, response)
            )
        } else {
            Task::none()
        }
    }
    
    fn generate_mock_expert_response(&self, user_message: &str) -> String {
        // This is a mock response generator - in real implementation, this would use CimExpertService
        match self.cim_expert_selected_topic {
            CimExpertTopic::Architecture => {
                format!("Regarding CIM architecture and your question about '{}': CIM is built on a foundation of Category Theory and Graph Theory, enabling composable information processing through structure-preserving mappings and type-safe transformations.", user_message)
            },
            CimExpertTopic::MathematicalFoundations => {
                format!("From a mathematical perspective on '{}': CIM leverages Category Theory for composable transformations, Topology for continuous mappings, and Information Theory for optimal encoding strategies.", user_message)
            },
            CimExpertTopic::NatsPatterns => {
                format!("Regarding NATS patterns and '{}': CIM uses NATS JetStream for persistent event sourcing, with subject hierarchies that mirror domain boundaries and enable efficient message routing.", user_message)
            },
            CimExpertTopic::EventSourcing => {
                format!("On event sourcing and '{}': CIM treats all state changes as immutable events, enabling perfect audit trails, time-travel debugging, and deterministic replay of system state.", user_message)
            },
            CimExpertTopic::DomainModeling => {
                format!("For domain modeling with '{}': CIM uses DDD principles with event-driven aggregates, ensuring each domain maintains clear boundaries while enabling cross-domain communication through well-defined interfaces.", user_message)
            },
            _ => {
                format!("Thank you for your question about '{}'. This is a complex topic that touches on CIM's cognitive architecture. Let me explain the key concepts and how they relate to your inquiry...", user_message)
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