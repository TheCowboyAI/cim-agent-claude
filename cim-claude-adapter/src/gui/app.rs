/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

use iced::{
    widget::{button, column, container, row, text, text_input, Space, scrollable},
    Element, Length, Theme, Task, Subscription,
};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::mpsc;

use crate::{
    bridge::{
        TeaEcsBridge, TeaEvent, EcsCommand, EcsCommandBuilder,
        ConversationEntity, EntityManager, EntityId,
    },
    adapters::NatsAdapter,
    domain::{commands::{Command as DomainCommand, *}, events::*, value_objects::*, ConversationAggregate},
    gui::{
        error_boundary::{ErrorInfo, ErrorBoundary, LoadingIndicator, ToastNotification, ErrorSeverity, errors},
        messages::{Message, Tab, HealthStatus, SystemMetrics, BridgeMessage, ComponentState, LoadingState},
        nats_client::GuiNatsClient,
        subscriptions::{BridgeSubscription, HealthCheckSubscription, ConnectionStatusSubscription, MetricsSubscription},
    },
};

/// Main CIM Manager Application State with TEA-ECS Bridge Integration
#[derive(Debug)]
pub struct CimManagerApp {
    // TEA-ECS Bridge Integration
    bridge: Arc<TeaEcsBridge>,
    bridge_connected: bool,
    event_receiver: Option<mpsc::UnboundedReceiver<TeaEvent>>,
    
    // Legacy Connection State (for backward compatibility)
    nats_client: GuiNatsClient,
    nats_url: String,
    connected: bool,
    connection_error: Option<String>,
    
    // Enhanced UI State
    current_tab: Tab,
    prompt_input: String,
    session_id_input: String,
    selected_conversation: Option<EntityId>,
    error_message: Option<String>,
    loading_states: HashMap<String, bool>,
    component_states: ComponentState,
    
    // Subscription managers
    bridge_subscription: Option<BridgeSubscription>,
    health_subscription: HealthCheckSubscription,
    connection_subscription: Option<ConnectionStatusSubscription>,
    metrics_subscription: MetricsSubscription,
    
    // Entity-Component State (Bridge-integrated)
    entity_manager: Arc<tokio::sync::RwLock<EntityManager>>,
    conversations: HashMap<EntityId, ConversationEntity>,
    
    // Error Handling and User Feedback
    error_boundary: ErrorBoundary,
    current_error: Option<ErrorInfo>,
    toast_notifications: Vec<ToastNotification>,
    
    // Legacy Domain State (transitioning to bridge)
    recent_events: Vec<EventEnvelope>,
    tea_events: Vec<TeaEvent>,
    health_status: HealthStatus,
    system_metrics: SystemMetrics,
}

impl CimManagerApp {
    pub fn new() -> (Self, Task<Message>) {
        // Create NATS adapter for bridge
        let nats_adapter = Arc::new(NatsAdapter::new("nats://localhost:4222".to_string()));
        
        // Create TEA-ECS bridge
        let bridge = Arc::new(TeaEcsBridge::new(nats_adapter));
        let entity_manager = bridge.entity_manager();
        
        let app = Self {
            // TEA-ECS Bridge
            bridge: Arc::clone(&bridge),
            bridge_connected: false,
            event_receiver: None,
            
            // Legacy state
            nats_client: GuiNatsClient::new(),
            nats_url: "nats://localhost:4222".to_string(),
            connected: false,
            connection_error: None,
            
            // Enhanced UI state
            current_tab: Tab::Dashboard,
            prompt_input: String::new(),
            session_id_input: uuid::Uuid::new_v4().to_string(),
            selected_conversation: None,
            error_message: None,
            loading_states: HashMap::new(),
            component_states: ComponentState::default(),
            
            // Subscription managers
            bridge_subscription: Some(BridgeSubscription::new(Arc::clone(&bridge))),
            health_subscription: HealthCheckSubscription,
            connection_subscription: Some(ConnectionStatusSubscription::new(Arc::clone(&bridge))),
            metrics_subscription: MetricsSubscription,
            
            // Entity state
            entity_manager,
            conversations: HashMap::new(),
            
            // Error handling
            error_boundary: ErrorBoundary::new(),
            current_error: None,
            toast_notifications: Vec::new(),
            
            // Legacy domain state
            recent_events: Vec::new(),
            tea_events: Vec::new(),
            health_status: HealthStatus::default(),
            system_metrics: SystemMetrics::default(),
        };
        
        // Initialize bridge connection task
        let init_task = Task::perform(
            Self::initialize_bridge(Arc::clone(&bridge)),
            Message::BridgeMessage
        );
        
        (app, init_task)
    }
    
    /// Initialize the TEA-ECS bridge
    async fn initialize_bridge(bridge: Arc<TeaEcsBridge>) -> BridgeMessage {
        match bridge.start().await {
            Ok(_) => BridgeMessage::Connected,
            Err(e) => BridgeMessage::ConnectionError(e.to_string()),
        }
    }
    
    pub fn title(&self) -> String {
        "CIM Claude Adapter Manager".to_string()
    }
    
    /// Get subscription for real-time events
    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();
        
        // Bridge events subscription
        if let Some(bridge_sub) = &self.bridge_subscription {
            subscriptions.push(bridge_sub.subscription());
        }
        
        // Health check subscription
        subscriptions.push(HealthCheckSubscription::subscription());
        
        // Connection status subscription
        if let Some(conn_sub) = &self.connection_subscription {
            subscriptions.push(conn_sub.subscription());
        }
        
        // Metrics collection subscription
        subscriptions.push(MetricsSubscription::subscription());
        
        Subscription::batch(subscriptions)
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Connect(url) => {
                self.nats_url = url.clone();
                self.connection_error = None;
                self.loading_states.insert("connection".to_string(), true);
                
                // Use bridge for connection instead of legacy client
                let bridge = Arc::clone(&self.bridge);
                Task::perform(
                    async move {
                        // Bridge connection logic would go here
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        BridgeMessage::Connected
                    },
                    Message::BridgeMessage
                )
            }
            
            Message::BridgeMessage(bridge_msg) => {
                self.loading_states.remove("connection");
                self.component_states.bridge_connection = LoadingState::Idle;
                
                match bridge_msg {
                    BridgeMessage::Connected => {
                        self.bridge_connected = true;
                        self.connected = true;
                        self.connection_error = None;
                        self.component_states.bridge_connection = LoadingState::Success;
                        Task::none()
                    }
                    
                    BridgeMessage::EventReceiverReady(receiver) => {
                        // The subscription system now handles this
                        Task::none()
                    }
                    
                    BridgeMessage::ConnectionError(error) => {
                        self.bridge_connected = false;
                        self.connected = false;
                        self.connection_error = Some(error.clone());
                        self.component_states.bridge_connection = LoadingState::Error(error);
                        Task::none()
                    }
                    
                    BridgeMessage::EventReceived(tea_event) => {
                        self.handle_tea_event(tea_event);
                        Task::none()
                    }
                }
            }
            
            Message::BridgeStatusChanged { connected, error } => {
                self.bridge_connected = connected;
                self.connected = connected;
                self.connection_error = error;
                
                if connected {
                    self.component_states.bridge_connection = LoadingState::Success;
                } else {
                    self.component_states.bridge_connection = LoadingState::Error(
                        self.connection_error.clone().unwrap_or("Unknown error".to_string())
                    );
                }
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
                if self.bridge_connected {
                    self.loading_states.insert("start_conversation".to_string(), true);
                    self.component_states.conversation_list = LoadingState::Loading;
                    
                    // Use bridge command pattern
                    let create_cmd = EcsCommandBuilder::create_conversation(initial_prompt.clone())
                        .build()
                        .unwrap();
                    
                    let bridge = Arc::clone(&self.bridge);
                    if let Err(e) = bridge.send_command(create_cmd) {
                        self.error_message = Some(format!("Failed to start conversation: {}", e));
                        self.loading_states.remove("start_conversation");
                        self.component_states.conversation_list = LoadingState::Error(e.to_string());
                        return Task::none();
                    }
                    
                    // Clear input after sending
                    self.prompt_input.clear();
                    Task::none()
                } else {
                    self.error_message = Some("Bridge not connected".to_string());
                    self.component_states.conversation_list = LoadingState::Error(
                        "Bridge not connected".to_string()
                    );
                    Task::none()
                }
            }
            
            Message::SendPrompt { conversation_id, prompt } => {
                if self.bridge_connected {
                    if let Ok(conv_id) = uuid::Uuid::parse_str(&conversation_id) {
                        self.loading_states.insert(format!("send_{}", conversation_id), true);
                        self.component_states.message_sending = LoadingState::Loading;
                        
                        let send_cmd = EcsCommandBuilder::send_message(conv_id, prompt.clone())
                            .build()
                            .unwrap();
                        
                        let bridge = Arc::clone(&self.bridge);
                        if let Err(e) = bridge.send_command(send_cmd) {
                            self.error_message = Some(format!("Failed to send message: {}", e));
                            self.loading_states.remove(&format!("send_{}", conversation_id));
                            self.component_states.message_sending = LoadingState::Error(e.to_string());
                        } else {
                            // Clear input after successful send
                            self.prompt_input.clear();
                        }
                    }
                    Task::none()
                } else {
                    self.error_message = Some("Bridge not connected".to_string());
                    self.component_states.message_sending = LoadingState::Error(
                        "Bridge not connected".to_string()
                    );
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
                match &event_envelope.event {
                    DomainEvent::ConversationStarted { conversation_id, .. } => {
                        // Load full conversation state
                        let conv_id = conversation_id.clone();
                        let client = self.nats_client.clone();
                        return Task::perform(
                            async move { client.load_conversation(&conv_id).await },
                            |result| match result {
                                Ok(Some(aggregate)) => Message::ConversationUpdated(aggregate),
                                Ok(None) => Message::ErrorOccurred("Conversation not found".to_string()),
                                Err(e) => Message::ErrorOccurred(e),
                            }
                        );
                    }
                    _ => {}
                }
                
                Task::none()
            }
            
            Message::ConversationUpdated(aggregate) => {
                let conversation_id = aggregate.id().to_string();
                self.conversations.insert(conversation_id, aggregate);
                Task::none()
            }
            
            Message::TabSelected(tab) => {
                self.current_tab = tab;
                Task::none()
            }
            
            Message::ConversationSelected(id) => {
                if let Ok(entity_id) = uuid::Uuid::parse_str(&id) {
                    self.selected_conversation = Some(entity_id);
                } else {
                    self.error_message = Some("Invalid conversation ID".to_string());
                }
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
                self.current_error = None;
                self.error_boundary = ErrorBoundary::new();
                
                // Clear expired toast notifications
                self.toast_notifications.retain(|toast| !toast.is_expired());
                
                Task::none()
            }
            
            Message::TeaEventReceived(tea_event) => {
                self.handle_tea_event(tea_event);
                Task::none()
            }
            
            _ => Task::none(),
        }
    }
    
    /// Handle incoming TEA events from the bridge
    fn handle_tea_event(&mut self, event: TeaEvent) {
        // Add to event history
        self.tea_events.insert(0, event.clone());
        if self.tea_events.len() > 100 {
            self.tea_events.truncate(100);
        }
        
        match event {
            TeaEvent::ConversationCreated { conversation_id, .. } => {
                // Load the full entity from entity manager
                let entity_manager = Arc::clone(&self.entity_manager);
                // In real implementation, we'd update from the bridge
                self.loading_states.remove("start_conversation");
                self.component_states.conversation_list = LoadingState::Success;
                
                // Auto-select the new conversation
                self.selected_conversation = Some(conversation_id);
            }
            
            TeaEvent::ConversationUpdated { conversation_id, entity, .. } => {
                // Update local conversation state
                self.conversations.insert(conversation_id, entity);
                self.component_states.conversation_list = LoadingState::Success;
            }
            
            TeaEvent::MessageAdded { conversation_id, .. } => {
                // Remove loading state for this conversation
                self.loading_states.remove(&format!("send_{}", conversation_id));
                self.component_states.message_sending = LoadingState::Success;
                
                // Update activity for conversation metadata
                if let Some(conv) = self.conversations.get_mut(&conversation_id) {
                    conv.metadata.update_activity();
                }
            }
            
            TeaEvent::ClaudeResponseReceived { conversation_id, response_content, .. } => {
                // Handle Claude response - could trigger UI notification
                if Some(conversation_id) == self.selected_conversation {
                    // Auto-scroll to bottom or highlight new response
                    // Could also show a toast notification
                }
                
                // Update message count and activity
                if let Some(conv) = self.conversations.get_mut(&conversation_id) {
                    conv.metadata.update_activity();
                }
            }
            
            TeaEvent::ToolInvocationCompleted { conversation_id, tool_id, result, .. } => {
                // Handle successful tool execution
                if let Some(conv) = self.conversations.get_mut(&conversation_id) {
                    conv.metadata.update_activity();
                }
            }
            
            TeaEvent::ToolInvocationFailed { conversation_id, tool_id, error, .. } => {
                // Handle tool execution failure
                self.error_message = Some(format!("Tool {} failed: {}", tool_id, error));
            }
            
            TeaEvent::ErrorOccurred { error, .. } => {
                self.error_message = Some(error.clone());
                // Clear all loading states on error
                self.loading_states.clear();
                self.component_states.conversation_list = LoadingState::Error(error.clone());
                self.component_states.message_sending = LoadingState::Error(error);
            }
            
            TeaEvent::ConnectionStatusChanged { service, connected, .. } => {
                if service == "nats" {
                    self.connected = connected;
                    if !connected {
                        self.bridge_connected = false;
                        self.connection_error = Some("NATS connection lost".to_string());
                        self.component_states.bridge_connection = LoadingState::Error(
                            "NATS connection lost".to_string()
                        );
                    } else {
                        self.component_states.bridge_connection = LoadingState::Success;
                    }
                }
            }
            
            TeaEvent::HealthStatusUpdate { status, .. } => {
                // Update health status from bridge events
                self.health_status.nats_connected = matches!(
                    status.overall_status, 
                    crate::bridge::events::HealthStatus::Healthy
                );
                self.health_status.last_check = chrono::Utc::now();
                self.component_states.health_check = LoadingState::Success;
            }
            
            TeaEvent::ConversationStatusChanged { conversation_id, new_status, .. } => {
                // Update conversation status
                if let Some(conv) = self.conversations.get_mut(&conversation_id) {
                    conv.metadata.status = new_status;
                    conv.metadata.update_activity();
                }
            }
            
            TeaEvent::SearchResultsReady { results, .. } => {
                // Handle search results - could update a search view
                // This would be useful for implementing conversation search
            }
            
            TeaEvent::ExportCompleted { conversation_id, format, file_path, .. } => {
                // Handle export completion - show success message
                let message = if let Some(path) = file_path {
                    format!("Conversation exported to: {}", path)
                } else {
                    "Conversation exported successfully".to_string()
                };
                // Could show a success toast here
            }
            
            TeaEvent::ImportCompleted { conversation_id, message_count, .. } => {
                // Handle import completion - refresh conversation list
                let message = format!("Imported {} messages", message_count);
                // Could show a success toast here
                self.component_states.conversation_list = LoadingState::Loading; // Trigger refresh
            }
            
            _ => {
                // Handle other events as needed
            }
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let mut content = column![
            self.view_header(),
            self.view_tabs(),
            self.view_content(),
            self.view_status_bar(),
        ]
        .spacing(10)
        .padding(20);
        
        // Add toast notifications overlay
        let main_content = if !self.toast_notifications.is_empty() {
            column![
                // Toast notifications at the top
                column(
                    self.toast_notifications
                        .iter()
                        .take(3) // Limit to 3 visible toasts
                        .map(|toast| toast.view())
                        .collect()
                )
                .spacing(5),
                
                Space::with_height(10),
                
                // Main content
                content.into(),
            ]
            .into()
        } else {
            content.into()
        };
        
        // Wrap in error boundary if there's an error
        let final_content = if let Some(ref error) = self.current_error {
            let error_boundary = ErrorBoundary::with_error(error.clone());
            error_boundary.view(main_content, None)
        } else {
            main_content
        };
        
        container(final_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl CimManagerApp {
    fn view_header(&self) -> Element<Message> {
        row![
            text("CIM Claude Adapter Manager").size(24),
            Space::with_width(Length::Fill),
            self.view_connection_controls(),
        ]
        .align_y(iced::alignment::Vertical::Center)
        .into()
    }
    
    fn view_connection_controls(&self) -> Element<Message> {
        row![
            text_input("NATS URL", &self.nats_url)
                .on_input(Message::NatsUrlChanged)
                .width(Length::Fixed(300.0)),
            if self.connected {
                button("Disconnect").on_press(Message::Disconnected)
            } else {
                button("Connect").on_press(Message::Connect(self.nats_url.clone()))
            },
            if self.connected {
                text("🟢 Connected")
            } else {
                text("🔴 Disconnected")
            },
        ]
        .spacing(10)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    }
    
    fn view_tabs(&self) -> Element<Message> {
        row![
            button("Dashboard").on_press(Message::TabSelected(Tab::Dashboard)),
            button("Conversations").on_press(Message::TabSelected(Tab::Conversations)),
            button("Events").on_press(Message::TabSelected(Tab::Events)),
            button("Monitoring").on_press(Message::TabSelected(Tab::Monitoring)),
            button("Settings").on_press(Message::TabSelected(Tab::Settings)),
        ]
        .spacing(5)
        .into()
    }
    
    fn view_content(&self) -> Element<Message> {
        match self.current_tab {
            Tab::Dashboard => self.view_dashboard(),
            Tab::Conversations => self.view_conversations(),
            Tab::Events => self.view_events(),
            Tab::Monitoring => self.view_monitoring(),
            Tab::Settings => self.view_settings(),
        }
    }
    
    fn view_dashboard(&self) -> Element<Message> {
        let is_loading = self.loading_states.contains_key("start_conversation");
        let can_start = self.bridge_connected && !self.prompt_input.is_empty() && !is_loading;
        
        column![
            text("Dashboard").size(20),
            row![
                // System Health Panel
                column![
                    text("System Health").size(16),
                    text(format!("Bridge: {}", if self.bridge_connected { "🟢 Connected" } else { "🔴 Disconnected" })),
                    text(format!("NATS: {}", if self.health_status.nats_connected { "✅ Online" } else { "❌ Offline" })),
                    text(format!("Claude API: {}", if self.health_status.claude_api_available { "✅ Available" } else { "❌ Unavailable" })),
                    text(format!("Active Conversations: {}", self.conversations.len())),
                    text(format!("Recent Events: {}", self.tea_events.len())),
                ]
                .spacing(5)
                .padding(10)
                .width(Length::FillPortion(1)),
                
                // Quick Actions Panel
                column![
                    text("Quick Actions").size(16),
                    text_input("Session ID", &self.session_id_input)
                        .on_input(Message::SessionIdChanged)
                        .width(Length::Fill),
                    text_input("Start a conversation...", &self.prompt_input)
                        .on_input(Message::PromptInputChanged)
                        .on_submit(if can_start {
                            Message::StartConversation {
                                session_id: self.session_id_input.clone(),
                                initial_prompt: self.prompt_input.clone(),
                            }
                        } else {
                            Message::ErrorOccurred("Cannot start conversation".to_string())
                        })
                        .width(Length::Fill),
                    
                    if is_loading {
                        button("Starting...").style(iced::widget::button::secondary)
                    } else if can_start {
                        button("Start Conversation")
                            .on_press(Message::StartConversation {
                                session_id: self.session_id_input.clone(),
                                initial_prompt: self.prompt_input.clone(),
                            })
                    } else {
                        button("Start Conversation").style(iced::widget::button::secondary)
                    },
                    
                    Space::with_height(20),
                    
                    if let Some(selected_id) = self.selected_conversation {
                        column![
                            text("Current Conversation").size(14),
                            text(format!("ID: {}", selected_id)).size(12),
                            if let Some(conv) = self.conversations.get(&selected_id) {
                                column![
                                    text(format!("Title: {}", conv.metadata.title)).size(12),
                                    text(format!("Messages: {}", conv.messages.message_count)).size(12),
                                    text(format!("Status: {:?}", conv.metadata.status)).size(12),
                                ].spacing(2).into()
                            } else {
                                text("Loading...").size(12).into()
                            }
                        ].spacing(5).into()
                    } else {
                        text("No conversation selected").size(12).into()
                    },
                ]
                .spacing(5)
                .padding(10)
                .width(Length::FillPortion(1)),
            ]
            .spacing(20),
            
            // Recent TEA Events Preview
            if !self.tea_events.is_empty() {
                column![
                    text("Recent Events").size(14),
                    scrollable(
                        column(
                            self.tea_events.iter().take(5).map(|event| {
                                text(format!(
                                    "[{}] {} - {}",
                                    event.timestamp().format("%H:%M:%S"),
                                    event.event_type(),
                                    if let Some(conv_id) = event.conversation_id() {
                                        format!("Conv: {}", &conv_id.to_string()[..8])
                                    } else {
                                        "System".to_string()
                                    }
                                ))
                                .size(10)
                                .into()
                            }).collect()
                        ).spacing(2)
                    ).height(100)
                ].spacing(5).into()
            } else {
                Space::with_height(0).into()
            },
        ]
        .spacing(10)
        .into()
    }
    
    fn view_conversations(&self) -> Element<Message> {
        // Use loading indicator for conversation list
        let content = self.render_conversation_list();
        LoadingIndicator::view(&self.component_states.conversation_list, content)
    }
    
    fn render_conversation_list(&self) -> Element<Message> {
        let conversations: Vec<Element<Message>> = self.conversations
            .iter()
            .map(|(id, entity)| {
                let is_selected = Some(*id) == self.selected_conversation;
                let is_loading = self.loading_states.contains_key(&format!("send_{}", id));
                
                let style = if is_selected {
                    iced::widget::button::primary
                } else {
                    iced::widget::button::secondary
                };
                
                column![
                    button(
                        row![
                            column![
                                text(&entity.metadata.title).size(14),
                                text(format!(
                                    "Messages: {} | Status: {:?} | Last: {}",
                                    entity.messages.message_count,
                                    entity.metadata.status,
                                    entity.metadata.last_active.format("%H:%M")
                                )).size(10),
                            ]
                            .width(Length::Fill),
                            
                            if is_loading {
                                text("⏳").size(16)
                            } else if is_selected {
                                text("👁️").size(16)
                            } else {
                                Space::with_width(0)
                            }
                        ]
                        .align_y(iced::alignment::Vertical::Center)
                    )
                    .on_press(Message::ConversationSelected(id.to_string()))
                    .width(Length::Fill)
                    .style(style),
                    
                    // Show quick message input for selected conversation
                    if is_selected {
                        row![
                            text_input("Type a message...", &self.prompt_input)
                                .on_input(Message::PromptInputChanged)
                                .on_submit(Message::SendPrompt {
                                    conversation_id: id.to_string(),
                                    prompt: self.prompt_input.clone(),
                                })
                                .width(Length::Fill),
                            
                            if is_loading {
                                button("⏳").style(iced::widget::button::secondary)
                            } else if !self.prompt_input.is_empty() {
                                button("Send")
                                    .on_press(Message::SendPrompt {
                                        conversation_id: id.to_string(),
                                        prompt: self.prompt_input.clone(),
                                    })
                            } else {
                                button("Send").style(iced::widget::button::secondary)
                            }
                        ]
                        .spacing(5)
                        .into()
                    } else {
                        Space::with_height(0).into()
                    }
                ]
                .spacing(5)
                .into()
            })
            .collect();
        
        column![
            row![
                text("Active Conversations").size(20),
                Space::with_width(Length::Fill),
                text(format!("Total: {}", self.conversations.len())).size(12),
            ],
            
            if conversations.is_empty() {
                column![
                    Space::with_height(50),
                    text("No conversations yet")
                        .size(16)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                    text("Start a new conversation from the Dashboard")
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                ]
                .align_x(iced::alignment::Horizontal::Center)
                .into()
            } else {
                scrollable(column(conversations).spacing(10))
                    .height(Length::Fill)
                    .into()
            },
        ]
        .spacing(10)
        .into()
    }
    
    fn view_events(&self) -> Element<Message> {
        // Combine legacy and TEA events
        let mut all_events: Vec<Element<Message>> = Vec::new();
        
        // Add TEA events (prioritized)
        for event in self.tea_events.iter().take(50) {
            let priority_color = match event.priority() {
                255 => iced::Color::from_rgb(1.0, 0.2, 0.2), // Error - Red
                200..=254 => iced::Color::from_rgb(1.0, 0.6, 0.0), // High - Orange
                100..=199 => iced::Color::from_rgb(0.2, 0.8, 0.2), // Normal - Green
                _ => iced::Color::from_rgb(0.5, 0.5, 0.5), // Low - Gray
            };
            
            let conv_info = if let Some(conv_id) = event.conversation_id() {
                if let Some(conv) = self.conversations.get(&conv_id) {
                    format!(" | {}", conv.metadata.title)
                } else {
                    format!(" | Conv: {}", &conv_id.to_string()[..8])
                }
            } else {
                String::new()
            };
            
            all_events.push(
                row![
                    text("●")
                        .size(12)
                        .style(iced::theme::Text::Color(priority_color)),
                    text(format!(
                        "[{}] {} - {}{}",
                        event.timestamp().format("%H:%M:%S"),
                        event.event_type(),
                        if event.should_update_ui() { "UI" } else { "BG" },
                        conv_info
                    ))
                    .size(11),
                ]
                .spacing(5)
                .align_y(iced::alignment::Vertical::Center)
                .into()
            );
        }
        
        // Add legacy events if any
        for event_envelope in self.recent_events.iter().take(10) {
            all_events.push(
                row![
                    text("○")
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.3, 0.3, 0.7))),
                    text(format!(
                        "[{}] {:?} - {}",
                        event_envelope.timestamp.format("%H:%M:%S"),
                        event_envelope.event.event_type(),
                        &event_envelope.correlation_id.as_uuid().to_string()[..8]
                    ))
                    .size(11),
                ]
                .spacing(5)
                .align_y(iced::alignment::Vertical::Center)
                .into()
            );
        }
        
        column![
            row![
                text("Event Stream").size(20),
                Space::with_width(Length::Fill),
                text(format!("TEA: {} | Legacy: {}", self.tea_events.len(), self.recent_events.len())).size(12),
            ],
            
            row![
                text("● High Priority").size(10).style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.6, 0.0))),
                text("● Normal").size(10).style(iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.8, 0.2))),
                text("○ Legacy").size(10).style(iced::theme::Text::Color(iced::Color::from_rgb(0.3, 0.3, 0.7))),
            ]
            .spacing(20),
            
            if all_events.is_empty() {
                column![
                    Space::with_height(50),
                    text("No events yet")
                        .size(16)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                    text("Events will appear here as the system operates")
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                ]
                .align_x(iced::alignment::Horizontal::Center)
                .into()
            } else {
                scrollable(column(all_events).spacing(3))
                    .height(Length::Fill)
                    .into()
            },
        ]
        .spacing(10)
        .into()
    }
    
    fn view_monitoring(&self) -> Element<Message> {
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
    
    fn view_settings(&self) -> Element<Message> {
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
    
    fn view_status_bar(&self) -> Element<Message> {
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
}