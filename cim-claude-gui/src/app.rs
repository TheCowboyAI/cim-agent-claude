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
    nats_client::nats_subscription,
    sage_client,
};

/// Main CIM Manager Application State - Pure UI State Only
pub struct CimManagerApp {
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
    pub fn new() -> (Self, Task<Message>) {
        // NATS is already initialized in main - no connection needed here
        let app = Self {
            nats_url: "nats://localhost:4222".to_string(),
            connected: true, // NATS initialized at startup
            connection_error: None,
            
            current_tab: Tab::Dashboard,
            prompt_input: String::new(),
            session_id_input: uuid::Uuid::new_v4().to_string(),
            selected_conversation: None,
            error_message: None,
            
            theme: Theme::Light,
            dark_mode: false,
            
            conversations: HashMap::new(),
            health_status: HealthStatus::default(),
            system_metrics: SystemMetrics::default(),
            
            // SAGE initialization  
            sage_client: crate::sage_client::SageClient::new(),
            sage_query_input: String::new(),
            sage_selected_expert: None,
            sage_status: None,
            sage_responses: Vec::new(),
        };
        
        // NATS already connected at startup
        (app, Task::none())
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
        nats_subscription()
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Connect(_url) => {
                // NATS connection handled at startup - no runtime connection needed
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
                let request = if let Some(ref expert) = self.sage_selected_expert {
                    self.sage_client.create_expert_request(self.sage_query_input.clone(), expert.clone())
                } else {
                    self.sage_client.create_request(self.sage_query_input.clone())
                };
                Task::perform(sage_client::nats_commands::send_sage_request(request), |msg| msg)
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
                Task::perform(sage_client::nats_commands::request_sage_status(), |msg| msg)
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
        
        let mut content_items = Vec::new();
        
        // SAGE Header
        content_items.push(row![
            text("🎭 SAGE Orchestrator").size(24),
            Space::with_width(Length::Fill),
            button("Clear Conversation")
                .on_press(Message::SageClearConversation)
                .style(Modern::secondary_button()),
            button("New Session")
                .on_press(Message::SageNewSession)  
                .style(Modern::primary_button()),
            button("Status")
                .on_press(Message::SageStatusRequested)
                .style(Modern::secondary_button()),
        ].spacing(10).into());
        
        // Status Display
        if let Some(ref status) = self.sage_status {
            content_items.push(column![
                text(format!("🧠 Consciousness: {} (Level {:.1})", 
                    if status.is_conscious { "Active" } else { "Inactive" },
                    status.consciousness_level
                )),
                text(format!("👥 Available Experts: {} | 📊 Orchestrations: {}", 
                    status.available_agents, status.total_orchestrations
                )),
                text(format!("🧬 Patterns Learned: {} | 💾 Memory: {}", 
                    status.patterns_learned, status.memory_health
                )),
            ].spacing(5).into());
        } else {
            content_items.push(text("Status: Disconnected from SAGE").into());
        }
        
        // Expert Selection - simplified for now
        content_items.push(row![
            text("Expert:"),
            text_input("Expert name (optional)", self.sage_selected_expert.as_ref().unwrap_or(&String::new()))
                .on_input(|expert| Message::SageExpertSelected(Some(expert))),
            text("Auto-route to appropriate expert").size(12),
        ].spacing(10).into());
        
        // Query Input
        content_items.push(row![
            text_input("Ask SAGE anything...", &self.sage_query_input)
                .on_input(Message::SageQueryInputChanged)
                .on_submit(Message::SageSendQuery),
            button("Send")
                .on_press(Message::SageSendQuery)
                .style(Modern::primary_button()),
        ].spacing(10).into());
        
        // Conversation History
        content_items.push(scrollable(
            column(
                self.sage_responses.iter().enumerate().map(|(i, response)| {
                    column![
                        // Query (reconstruct from context)
                        container(
                            text(format!("You ({})", i + 1)).size(14)
                        ).padding(5),
                        
                        // SAGE Response
                        container(
                            column![
                                row![
                                    text("🎭 SAGE").size(14),
                                    Space::with_width(Length::Fill),
                                    text(format!("Experts: {:?}", response.expert_agents_used)).size(10),
                                    text(format!("Confidence: {:.0}%", response.confidence_score * 100.0)).size(10),
                                ].spacing(5),
                                
                                text(&response.response).size(12),
                                    
                                text(format!("🕒 Session: {}", response.updated_context.session_id.as_ref().unwrap_or(&"Unknown".to_string()))).size(10),
                            ].spacing(5)
                        ).padding(10),
                        
                        Space::with_height(10),
                    ].into()
                }).collect::<Vec<_>>()
            ).spacing(5)
        ).height(Length::Fill).into());
        
        // Conversation Summary
        if !self.sage_responses.is_empty() {
            content_items.push(container(
                text(self.sage_client.get_conversation_summary()).size(11)
            ).padding(10).into());
        } else {
            content_items.push(text("Start a conversation with SAGE by typing your question above.").size(12).into());
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
    
    // Legacy CIM Expert helper methods (removed - functionality replaced by SAGE)
}

impl Default for CimManagerApp {
    fn default() -> Self {
        let (app, _task) = Self::new();
        app
    }
}