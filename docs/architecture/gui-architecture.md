# GUI Architecture - Universal CIM Agent Interface

*Complete architectural specification for the Iced-based GUI that provides seamless interaction with SAGE and the Universal Agent System*

## System Overview

The **CIM Universal Agent GUI** provides an intuitive, desktop-native interface for interacting with SAGE and the 17 expert agents. Built using the Iced framework with Elm Architecture patterns, it enables rich conversational interactions, visual expert attribution, and comprehensive CIM development workflows.

```mermaid
graph TB
    subgraph "GUI Architecture Stack"
        USER[👤 User Interface Layer]
        ICED[🎨 Iced Framework]
        TEA[🔄 Elm Architecture (TEA)]
        ECS[⚡ Entity Component System]
        
        subgraph "Application Layers"
            APP[Application State]
            VIEWS[View Components]
            MESSAGES[Message System]
            SUBSCRIPTIONS[Subscription Handlers]
        end
        
        subgraph "SAGE Integration"
            SAGE_CLIENT[SAGE Client]
            NATS_BRIDGE[NATS Bridge]
            EXPERT_VISUALIZER[Expert Attribution]
        end
        
        subgraph "Data Layer"
            LOCAL_STATE[Local State]
            CONVERSATION_CACHE[Conversation Cache]
            EXPERT_CONTEXT[Expert Context]
        end
        
        USER --> ICED
        ICED --> TEA
        TEA --> ECS
        
        ECS --> APP
        ECS --> VIEWS
        ECS --> MESSAGES
        ECS --> SUBSCRIPTIONS
        
        APP --> SAGE_CLIENT
        SAGE_CLIENT --> NATS_BRIDGE
        NATS_BRIDGE --> EXPERT_VISUALIZER
        
        VIEWS --> LOCAL_STATE
        LOCAL_STATE --> CONVERSATION_CACHE
        CONVERSATION_CACHE --> EXPERT_CONTEXT
    end
```

## Elm Architecture Integration (@elm-architecture-expert)

The GUI implements pure Elm Architecture patterns through Iced, ensuring predictable state management and functional reactive programming:

### Core TEA Pattern Implementation
```rust
use iced::*;
use cim_graph::*;

// Application State - Immutable and Pure
#[derive(Debug, Clone)]
pub struct CIMAgentApp {
    // Conversation Management
    active_conversations: HashMap<ConversationId, ConversationState>,
    current_conversation: Option<ConversationId>,
    
    // SAGE Integration
    sage_client: SAGEClient,
    orchestration_status: OrchestrationStatus,
    
    // Expert Visualization
    expert_attributions: HashMap<ConversationId, Vec<ExpertAttribution>>,
    expert_network_view: ExpertNetworkVisualization,
    
    // UI State
    input_text: String,
    ui_mode: UIMode,
    sidebar_state: SidebarState,
    
    // Async Operations
    pending_requests: HashMap<RequestId, PendingRequest>,
}

// Messages - All possible state transitions
#[derive(Debug, Clone)]
pub enum Message {
    // User Input
    InputChanged(String),
    MessageSubmitted,
    ConversationSelected(ConversationId),
    NewConversationStarted,
    
    // SAGE Orchestration
    SAGEResponseReceived(SAGEResponse),
    ExpertAttributionReceived(ExpertAttribution),
    OrchestrationStatusChanged(OrchestrationStatus),
    
    // Expert Network
    ExpertSelected(ExpertId),
    ExpertNetworkUpdated(ExpertNetworkState),
    
    // UI Interactions
    SidebarToggled,
    UIModeChanged(UIMode),
    ThemeChanged(Theme),
    
    // Async Events
    RequestCompleted(RequestId, Result<Response, Error>),
    SubscriptionEvent(SubscriptionEvent),
}

// Update Function - Pure state transitions
impl Application for CIMAgentApp {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = AppFlags;
    type Theme = Theme;
    
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let app = CIMAgentApp {
            active_conversations: HashMap::new(),
            current_conversation: None,
            sage_client: SAGEClient::new(flags.nats_url),
            orchestration_status: OrchestrationStatus::Ready,
            expert_attributions: HashMap::new(),
            expert_network_view: ExpertNetworkVisualization::default(),
            input_text: String::new(),
            ui_mode: UIMode::Conversation,
            sidebar_state: SidebarState::Collapsed,
            pending_requests: HashMap::new(),
        };
        
        let init_command = Command::batch([
            Command::perform(
                SAGEClient::initialize(),
                |result| Message::RequestCompleted(RequestId::Initialization, result),
            ),
        ]);
        
        (app, init_command)
    }
    
    fn title(&self) -> String {
        "CIM Universal Agent System".to_string()
    }
    
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::InputChanged(text) => {
                self.input_text = text;
                Command::none()
            },
            
            Message::MessageSubmitted => {
                let query = UserQuery::new(self.input_text.clone());
                let conversation_id = self.current_conversation
                    .unwrap_or_else(|| self.create_new_conversation());
                
                self.orchestration_status = OrchestrationStatus::Processing;
                self.input_text.clear();
                
                Command::perform(
                    self.sage_client.orchestrate(conversation_id, query),
                    Message::SAGEResponseReceived,
                )
            },
            
            Message::SAGEResponseReceived(response) => {
                self.orchestration_status = OrchestrationStatus::Ready;
                
                // Update conversation state
                if let Some(conversation) = self.active_conversations.get_mut(&response.conversation_id) {
                    conversation.add_turn(ConversationTurn {
                        user_input: response.original_query.clone(),
                        agent_response: response.unified_response.clone(),
                        expert_contributions: response.expert_contributions.clone(),
                        timestamp: Utc::now(),
                    });
                }
                
                // Update expert attributions
                self.expert_attributions.insert(
                    response.conversation_id,
                    response.expert_attributions,
                );
                
                Command::none()
            },
            
            Message::ExpertSelected(expert_id) => {
                // Switch to expert-focused view
                self.ui_mode = UIMode::ExpertFocus(expert_id);
                
                Command::perform(
                    self.sage_client.get_expert_context(expert_id),
                    |context| Message::RequestCompleted(RequestId::ExpertContext, Ok(context)),
                )
            },
            
            // ... other message handlers
            _ => Command::none(),
        }
    }
    
    fn view(&self) -> Element<Self::Message> {
        match self.ui_mode {
            UIMode::Conversation => self.conversation_view(),
            UIMode::ExpertNetwork => self.expert_network_view(), 
            UIMode::ExpertFocus(expert_id) => self.expert_focus_view(expert_id),
            UIMode::Settings => self.settings_view(),
        }
    }
    
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            // SAGE orchestration events
            sage_subscription().map(Message::SubscriptionEvent),
            // Expert network updates
            expert_network_subscription().map(Message::ExpertNetworkUpdated),
            // Keyboard shortcuts
            keyboard_events().map(Message::from),
        ])
    }
}
```

### Functional State Management
```rust
// Immutable conversation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    id: ConversationId,
    title: String,
    turns: Vec<ConversationTurn>,
    expert_involvement: HashMap<ExpertId, ExpertInvolvement>,
    context_summary: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ConversationState {
    // Pure functional updates - no mutation
    fn add_turn(self, turn: ConversationTurn) -> Self {
        let mut new_turns = self.turns;
        new_turns.push(turn.clone());
        
        let updated_expert_involvement = self.update_expert_involvement(&turn);
        
        Self {
            turns: new_turns,
            expert_involvement: updated_expert_involvement,
            updated_at: Utc::now(),
            ..self
        }
    }
    
    fn update_expert_involvement(
        &self,
        turn: &ConversationTurn,
    ) -> HashMap<ExpertId, ExpertInvolvement> {
        let mut involvement = self.expert_involvement.clone();
        
        for expert_contribution in &turn.expert_contributions {
            let entry = involvement
                .entry(expert_contribution.expert_id)
                .or_insert_with(ExpertInvolvement::default);
            
            entry.add_contribution(expert_contribution.clone());
        }
        
        involvement
    }
}
```

**Critical Design Decision (@elm-architecture-expert)**: Pure functional state management eliminates entire classes of UI bugs and makes the application behavior completely predictable and debuggable.

## TEA+ECS Integration Patterns (@cim-tea-ecs-expert)

The GUI seamlessly integrates The Elm Architecture with Entity Component System patterns for complex UI components:

### ECS Component Architecture
```rust
use cim_ecs::*;

// ECS Components for UI elements
#[derive(Component, Debug, Clone)]
pub struct ExpertVisualization {
    expert_id: ExpertId,
    position: Position2D,
    activity_level: f32,
    connection_strength: HashMap<ExpertId, f32>,
    visual_state: ExpertVisualState,
}

#[derive(Component, Debug, Clone)]
pub struct ConversationBubble {
    conversation_id: ConversationId,
    content: String,
    expert_attribution: Vec<ExpertId>,
    animation_state: AnimationState,
    interaction_state: InteractionState,
}

#[derive(Component, Debug, Clone)]
pub struct OrchestrationFlow {
    flow_id: FlowId,
    expert_sequence: Vec<ExpertId>,
    current_step: usize,
    flow_state: FlowState,
    visual_connections: Vec<FlowConnection>,
}

// ECS Systems for UI behavior
#[derive(System)]
pub struct ExpertNetworkRenderingSystem;

impl System<CIMAgentWorld> for ExpertNetworkRenderingSystem {
    fn run(&mut self, world: &mut CIMAgentWorld, dt: f32) {
        // Update expert visualization positions based on activity
        for (entity, expert_viz) in world.query_mut::<ExpertVisualization>() {
            let new_activity = expert_viz.calculate_activity_decay(dt);
            expert_viz.activity_level = new_activity;
            
            // Update position based on network forces
            let network_force = self.calculate_network_forces(entity, world);
            expert_viz.position = expert_viz.position.apply_force(network_force, dt);
        }
        
        // Update connection strengths
        for (entity, expert_viz) in world.query_mut::<ExpertVisualization>() {
            for (other_expert, strength) in &mut expert_viz.connection_strength {
                *strength = self.calculate_connection_strength(entity, *other_expert, world);
            }
        }
    }
}

#[derive(System)]
pub struct ConversationAnimationSystem;

impl System<CIMAgentWorld> for ConversationAnimationSystem {
    fn run(&mut self, world: &mut CIMAgentWorld, dt: f32) {
        for (entity, bubble) in world.query_mut::<ConversationBubble>() {
            // Update bubble animations
            bubble.animation_state.update(dt);
            
            // Handle expert attribution animations
            for expert_id in &bubble.expert_attribution {
                if let Some(expert_entity) = world.find_expert_entity(*expert_id) {
                    self.animate_attribution_connection(entity, expert_entity, dt, world);
                }
            }
        }
    }
}
```

### TEA-ECS Bridge Pattern
```rust
// Bridge between TEA messages and ECS world updates
pub struct TEAECSBridge {
    world: CIMAgentWorld,
    message_handlers: HashMap<MessageType, Box<dyn MessageHandler>>,
}

impl TEAECSBridge {
    pub fn handle_tea_message(&mut self, message: Message) -> Vec<TEAEffect> {
        match message {
            Message::SAGEResponseReceived(response) => {
                // Update ECS world with new expert activity
                for expert_contribution in &response.expert_contributions {
                    self.world.update_expert_activity(
                        expert_contribution.expert_id,
                        expert_contribution.activity_level(),
                    );
                }
                
                // Create new conversation bubble entity
                let bubble_entity = self.world.spawn(ConversationBubble {
                    conversation_id: response.conversation_id,
                    content: response.unified_response.content.clone(),
                    expert_attribution: response.expert_contributions
                        .iter()
                        .map(|c| c.expert_id)
                        .collect(),
                    animation_state: AnimationState::FadeIn,
                    interaction_state: InteractionState::Idle,
                });
                
                vec![TEAEffect::EntityCreated(bubble_entity)]
            },
            
            Message::ExpertSelected(expert_id) => {
                // Highlight expert in ECS world
                if let Some(expert_entity) = self.world.find_expert_entity(expert_id) {
                    self.world.get_mut::<ExpertVisualization>(expert_entity)
                        .map(|viz| viz.visual_state = ExpertVisualState::Highlighted);
                }
                
                vec![TEAEffect::ExpertHighlighted(expert_id)]
            },
            
            _ => vec![],
        }
    }
    
    pub fn extract_tea_messages(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();
        
        // Extract messages from ECS world events
        for event in self.world.drain_events::<UIInteractionEvent>() {
            match event {
                UIInteractionEvent::ExpertClicked(expert_id) => {
                    messages.push(Message::ExpertSelected(expert_id));
                },
                UIInteractionEvent::ConversationBubbleHovered(conversation_id) => {
                    messages.push(Message::ConversationSelected(conversation_id));
                },
                // ... other UI events
            }
        }
        
        messages
    }
}
```

**Critical Design Decision (@cim-tea-ecs-expert)**: The TEA-ECS bridge enables complex, animated visualizations while maintaining the predictability of functional state management.

## Iced GUI Implementation (@iced-ui-expert)

The GUI leverages Iced's native capabilities for cross-platform desktop application development:

### Main Application Layout
```rust
use iced::widget::*;
use iced::*;

impl CIMAgentApp {
    fn conversation_view(&self) -> Element<Message> {
        let sidebar = self.render_sidebar();
        let main_conversation = self.render_conversation_area();
        let expert_panel = self.render_expert_panel();
        
        container(
            row![
                sidebar.width(Length::Fixed(300.0)),
                main_conversation.width(Length::Fill),
                expert_panel.width(Length::Fixed(400.0)),
            ]
            .spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    fn render_sidebar(&self) -> Element<Message> {
        let conversations = self.active_conversations
            .values()
            .map(|conv| {
                button(
                    container(
                        column![
                            text(&conv.title).size(16),
                            text(format!(
                                "{}  •  {} experts",
                                conv.turns.len(),
                                conv.expert_involvement.len()
                            ))
                            .size(12)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                        ]
                        .spacing(4)
                    )
                    .padding(12)
                    .width(Length::Fill)
                )
                .on_press(Message::ConversationSelected(conv.id))
                .width(Length::Fill)
                .style(if Some(conv.id) == self.current_conversation {
                    theme::Button::Primary
                } else {
                    theme::Button::Secondary
                })
            })
            .collect::<Vec<_>>();
        
        container(
            column![
                container(
                    row![
                        text("Conversations").size(18),
                        horizontal_space(Length::Fill),
                        button("+")
                            .on_press(Message::NewConversationStarted)
                            .style(theme::Button::Positive),
                    ]
                )
                .padding(16),
                scrollable(
                    column(conversations)
                        .spacing(4)
                        .padding([0, 8])
                ),
            ]
        )
        .style(theme::Container::Box)
        .height(Length::Fill)
        .into()
    }
    
    fn render_conversation_area(&self) -> Element<Message> {
        let conversation = self.current_conversation
            .and_then(|id| self.active_conversations.get(&id));
            
        match conversation {
            Some(conv) => {
                let messages = conv.turns
                    .iter()
                    .map(|turn| self.render_conversation_turn(turn))
                    .collect::<Vec<_>>();
                
                let input_area = row![
                    text_input("Ask SAGE anything about CIM development...", &self.input_text)
                        .on_input(Message::InputChanged)
                        .on_submit(Message::MessageSubmitted)
                        .padding(12),
                    button(
                        if self.orchestration_status == OrchestrationStatus::Processing {
                            "⏳"
                        } else {
                            "➤"
                        }
                    )
                    .on_press_maybe(
                        if self.orchestration_status == OrchestrationStatus::Ready && !self.input_text.is_empty() {
                            Some(Message::MessageSubmitted)
                        } else {
                            None
                        }
                    )
                    .style(theme::Button::Primary),
                ]
                .spacing(8)
                .padding(16);
                
                column![
                    scrollable(
                        column(messages)
                            .spacing(16)
                            .padding(16)
                    )
                    .height(Length::Fill),
                    input_area,
                ]
                .into()
            },
            None => {
                container(
                    column![
                        text("Welcome to CIM Universal Agent System")
                            .size(24)
                            .horizontal_alignment(alignment::Horizontal::Center),
                        vertical_space(Length::Fixed(16.0)),
                        text("Start a conversation with SAGE to begin your CIM development journey.")
                            .horizontal_alignment(alignment::Horizontal::Center),
                        vertical_space(Length::Fixed(32.0)),
                        button("Start New Conversation")
                            .on_press(Message::NewConversationStarted)
                            .style(theme::Button::Primary),
                    ]
                    .align_items(Alignment::Center)
                )
                .center_x()
                .center_y()
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        }
    }
    
    fn render_conversation_turn(&self, turn: &ConversationTurn) -> Element<Message> {
        column![
            // User message
            container(
                text(&turn.user_input)
                    .size(14)
            )
            .padding(12)
            .style(theme::Container::Box)
            .width(Length::FillPortion(3)),
            
            vertical_space(Length::Fixed(8.0)),
            
            // SAGE response with expert attribution
            container(
                column![
                    self.render_expert_attribution(&turn.expert_contributions),
                    vertical_space(Length::Fixed(8.0)),
                    text(&turn.agent_response.content)
                        .size(14),
                ]
            )
            .padding(12)
            .style(theme::Container::Secondary)
            .width(Length::Fill),
        ]
        .spacing(4)
        .into()
    }
    
    fn render_expert_attribution(&self, contributions: &[ExpertContribution]) -> Element<Message> {
        let expert_chips = contributions
            .iter()
            .map(|contrib| {
                button(
                    row![
                        text(&contrib.expert_id.display_name())
                            .size(12),
                        horizontal_space(Length::Fixed(4.0)),
                        text(format!("{:.0}%", contrib.contribution_percentage * 100.0))
                            .size(10)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    ]
                    .align_items(Alignment::Center)
                )
                .on_press(Message::ExpertSelected(contrib.expert_id))
                .style(theme::Button::Text)
                .padding([4, 8])
            })
            .collect::<Vec<_>>();
        
        container(
            row![
                text("Experts consulted: ")
                    .size(12)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
                row(expert_chips)
                    .spacing(4),
            ]
            .align_items(Alignment::Center)
        )
        .padding([0, 0, 8, 0])
        .into()
    }
    
    fn render_expert_panel(&self) -> Element<Message> {
        match self.ui_mode {
            UIMode::ExpertFocus(expert_id) => {
                self.render_expert_details(expert_id)
            },
            _ => {
                self.render_expert_network_overview()
            }
        }
    }
    
    fn render_expert_network_overview(&self) -> Element<Message> {
        let expert_grid = self.expert_network_view.experts
            .iter()
            .map(|expert| {
                let activity_color = Color::from_rgb(
                    0.2 + expert.activity_level * 0.6,
                    0.4 + expert.activity_level * 0.4,
                    0.2,
                );
                
                button(
                    container(
                        column![
                            text(&expert.name)
                                .size(14)
                                .horizontal_alignment(alignment::Horizontal::Center),
                            vertical_space(Length::Fixed(4.0)),
                            container(
                                text("●")
                                    .color(activity_color)
                                    .size(20)
                            )
                            .center_x(),
                        ]
                    )
                    .padding(8)
                    .center_x()
                    .width(Length::Fill)
                )
                .on_press(Message::ExpertSelected(expert.id))
                .width(Length::Fill)
                .height(Length::Fixed(60.0))
                .style(theme::Button::Secondary)
            })
            .collect::<Vec<_>>();
        
        container(
            column![
                container(
                    text("Expert Network")
                        .size(16)
                        .horizontal_alignment(alignment::Horizontal::Center)
                )
                .padding(16),
                scrollable(
                    column(expert_grid)
                        .spacing(4)
                        .padding([0, 8])
                ),
            ]
        )
        .style(theme::Container::Box)
        .height(Length::Fill)
        .into()
    }
}
```

### Custom Iced Theme Integration
```rust
use iced::theme;

#[derive(Debug, Clone, Copy, Default)]
pub enum CIMAgentTheme {
    #[default]
    CIMDark,
    CIMLight,
    HighContrast,
}

impl From<CIMAgentTheme> for Theme {
    fn from(theme: CIMAgentTheme) -> Self {
        match theme {
            CIMAgentTheme::CIMDark => Theme::custom(
                "CIM Dark".to_string(),
                theme::Palette {
                    background: Color::from_rgb(0.1, 0.1, 0.12),
                    text: Color::from_rgb(0.9, 0.9, 0.9),
                    primary: Color::from_rgb(0.3, 0.6, 0.9),
                    success: Color::from_rgb(0.2, 0.8, 0.4),
                    danger: Color::from_rgb(0.9, 0.3, 0.3),
                },
            ),
            CIMAgentTheme::CIMLight => Theme::Light,
            CIMAgentTheme::HighContrast => Theme::custom(
                "High Contrast".to_string(),
                theme::Palette {
                    background: Color::BLACK,
                    text: Color::WHITE,
                    primary: Color::from_rgb(1.0, 1.0, 0.0),
                    success: Color::from_rgb(0.0, 1.0, 0.0),
                    danger: Color::from_rgb(1.0, 0.0, 0.0),
                },
            ),
        }
    }
}
```

**Critical Design Decision (@iced-ui-expert)**: Native desktop GUI provides superior performance and user experience compared to web-based alternatives, while maintaining cross-platform compatibility.

## Visual Aesthetics and Information Design (@ricing-expert)

The GUI implements Tufte-inspired information design principles with carefully crafted aesthetics:

### Information Density and Clarity
```rust
// Tufte-inspired data visualization components
pub struct ExpertActivityVisualization {
    expert_activities: Vec<ExpertActivity>,
    time_window: Duration,
    visualization_style: VisualizationStyle,
}

#[derive(Debug, Clone)]
pub enum VisualizationStyle {
    Sparklines {
        show_grid: bool,
        show_values: bool,
        color_scheme: ColorScheme,
    },
    SmallMultiples {
        expert_count: usize,
        normalize_scales: bool,
    },
    IntegratedDisplay {
        show_annotations: bool,
        highlight_patterns: bool,
    },
}

impl ExpertActivityVisualization {
    fn render_sparklines(&self) -> Element<Message> {
        let sparklines = self.expert_activities
            .iter()
            .map(|activity| {
                let data_points = activity.activity_over_time
                    .iter()
                    .enumerate()
                    .map(|(i, value)| {
                        let x = i as f32 / self.expert_activities.len() as f32;
                        let y = 1.0 - value; // Invert Y for screen coordinates
                        Point::new(x, y)
                    })
                    .collect::<Vec<_>>();
                
                // Minimal, information-dense sparkline
                canvas(SparklineCanvas {
                    data_points,
                    color: self.get_expert_color(activity.expert_id),
                    show_trend: true,
                })
                .width(Length::Fixed(100.0))
                .height(Length::Fixed(20.0))
            })
            .collect::<Vec<_>>();
        
        // Arrange in small multiples grid
        column(sparklines
            .chunks(4)
            .map(|chunk| row(chunk.to_vec()).spacing(4).into())
            .collect::<Vec<_>>()
        )
        .spacing(2)
        .into()
    }
    
    fn render_integrated_display(&self) -> Element<Message> {
        // Single, integrated visualization showing all expert activity
        canvas(IntegratedActivityCanvas {
            expert_activities: self.expert_activities.clone(),
            annotations: self.extract_significant_events(),
            color_mapping: self.create_expert_color_mapping(),
        })
        .width(Length::Fill)
        .height(Length::Fixed(200.0))
        .into()
    }
}
```

### Typography and Visual Hierarchy
```rust
// Carefully crafted typography system
pub struct CIMTypography {
    // Font stack optimized for technical content
    primary_font: Font,
    monospace_font: Font,
    
    // Size scale based on modular scale
    sizes: TypographyScale,
    
    // Semantic color system
    colors: SemanticColors,
}

#[derive(Debug, Clone)]
pub struct TypographyScale {
    // Modular scale: 1.25 (major third)
    caption: u16,     // 10px
    small: u16,       // 12px
    body: u16,        // 14px (base)
    subheading: u16,  // 18px
    heading: u16,     // 22px
    title: u16,       // 28px
    display: u16,     // 35px
}

#[derive(Debug, Clone)]
pub struct SemanticColors {
    // Text colors
    primary_text: Color,
    secondary_text: Color,
    muted_text: Color,
    
    // Expert attribution colors
    domain_experts: Color,
    development_experts: Color,
    infrastructure_experts: Color,
    ui_experts: Color,
    
    // Status colors
    success: Color,
    warning: Color,
    error: Color,
    info: Color,
}

impl CIMTypography {
    fn expert_text(&self, expert_id: ExpertId, content: &str) -> Text {
        text(content)
            .size(self.sizes.body)
            .color(self.get_expert_color(expert_id))
            .font(self.primary_font)
    }
    
    fn code_block(&self, code: &str) -> Element<Message> {
        container(
            text(code)
                .size(self.sizes.small)
                .color(self.colors.primary_text)
                .font(self.monospace_font)
        )
        .padding(12)
        .style(theme::Container::Box)
        .into()
    }
    
    fn conversation_hierarchy(&self, turn: &ConversationTurn) -> Element<Message> {
        column![
            // User query - visually de-emphasized but readable
            text(&turn.user_input)
                .size(self.sizes.body)
                .color(self.colors.secondary_text),
            
            vertical_space(Length::Fixed(8.0)),
            
            // Expert attribution - small but prominent
            self.render_expert_chips(&turn.expert_contributions),
            
            vertical_space(Length::Fixed(4.0)),
            
            // SAGE response - primary content
            text(&turn.agent_response.content)
                .size(self.sizes.body)
                .color(self.colors.primary_text)
                .line_height(text::LineHeight::Relative(1.5)),
        ]
        .into()
    }
}
```

### Minimalist UI Design Principles
```rust
// Minimalist design system focused on content
pub struct MinimalistDesign {
    // Generous whitespace for reading comfort
    spacing_scale: SpacingScale,
    
    // Subtle visual elements that don't compete with content
    visual_elements: VisualElements,
    
    // Progressive disclosure for complex functionality
    disclosure_system: ProgressiveDisclosure,
}

#[derive(Debug, Clone)]
pub struct SpacingScale {
    xs: f32,    // 4px
    sm: f32,    // 8px
    md: f32,    // 16px
    lg: f32,    // 24px
    xl: f32,    // 32px
    xxl: f32,   // 48px
}

impl MinimalistDesign {
    fn clean_conversation_layout(&self, conversation: &ConversationState) -> Element<Message> {
        // Maximum reading comfort with generous spacing
        let turns = conversation.turns
            .iter()
            .map(|turn| {
                container(
                    self.render_turn_content(turn)
                )
                .padding([
                    self.spacing_scale.lg,  // Top
                    self.spacing_scale.xl,  // Right
                    self.spacing_scale.lg,  // Bottom
                    self.spacing_scale.xl,  // Left
                ])
                .width(Length::Fill)
            })
            .collect::<Vec<_>>();
        
        scrollable(
            column(turns)
                .spacing(self.spacing_scale.xxl)  // Generous turn separation
        )
        .width(Length::Fill)
        .into()
    }
    
    fn subtle_expert_indicators(&self, experts: &[ExpertId]) -> Element<Message> {
        // Subtle visual indicators that don't dominate the interface
        let indicators = experts
            .iter()
            .map(|expert_id| {
                container(
                    text(self.expert_icon(*expert_id))
                        .size(12)
                        .color(self.get_muted_expert_color(*expert_id))
                )
                .padding(2)
                .style(theme::Container::Transparent)
            })
            .collect::<Vec<_>>();
        
        row(indicators)
            .spacing(self.spacing_scale.xs)
            .into()
    }
}
```

**Critical Design Decision (@ricing-expert)**: The visual design prioritizes information clarity and reading comfort over visual spectacle, following Tufte's principles of maximizing data-ink ratio and eliminating chartjunk.

## SAGE Integration Layer

### Real-time SAGE Communication
```rust
use tokio_tungstenite::{connect_async, WebSocketStream};

#[derive(Debug, Clone)]
pub struct SAGEClient {
    nats_client: NatsClient,
    conversation_stream: JetStream,
    orchestration_kv: KeyValue,
    websocket_connection: Option<WebSocketConnection>,
}

impl SAGEClient {
    pub async fn orchestrate(
        &self,
        conversation_id: ConversationId,
        query: UserQuery,
    ) -> Result<SAGEResponse> {
        // Send orchestration request to SAGE
        let orchestration_request = OrchestrationRequest {
            conversation_id,
            query: query.clone(),
            timestamp: Utc::now(),
            ui_context: UIContext {
                interface_type: InterfaceType::Desktop,
                visualization_preferences: self.get_visualization_preferences(),
            },
        };
        
        // Publish request to NATS
        self.nats_client.publish(
            &format!("sage.orchestration.request.{}", conversation_id),
            serde_json::to_vec(&orchestration_request)?,
        ).await?;
        
        // Wait for SAGE response
        let response_subject = format!("sage.orchestration.response.{}", conversation_id);
        let mut subscription = self.nats_client.subscribe(&response_subject).await?;
        
        // Set up timeout for orchestration
        let response = tokio::time::timeout(
            Duration::from_secs(30),
            subscription.next(),
        ).await??;
        
        let sage_response: SAGEResponse = serde_json::from_slice(&response.payload)?;
        
        // Update local conversation state
        self.orchestration_kv.put(
            &format!("conversation.{}", conversation_id),
            &sage_response,
        ).await?;
        
        Ok(sage_response)
    }
    
    pub async fn get_expert_context(&self, expert_id: ExpertId) -> Result<ExpertContext> {
        // Request expert context from SAGE
        let context_request = ExpertContextRequest {
            expert_id,
            detail_level: DetailLevel::Comprehensive,
            include_recent_activity: true,
        };
        
        self.nats_client.request(
            "sage.expert.context",
            serde_json::to_vec(&context_request)?,
        ).await
        .map(|response| serde_json::from_slice(&response.payload))?
    }
    
    pub fn subscribe_to_orchestration_events(&self) -> impl Stream<Item = OrchestrationEvent> {
        self.nats_client
            .subscribe("sage.orchestration.events.*")
            .await
            .unwrap()
            .map(|msg| serde_json::from_slice(&msg.payload).unwrap())
    }
}
```

### Expert Network Visualization
```rust
#[derive(Debug, Clone)]
pub struct ExpertNetworkVisualization {
    experts: Vec<ExpertNode>,
    connections: Vec<ExpertConnection>,
    layout_algorithm: NetworkLayout,
    interaction_state: NetworkInteractionState,
}

#[derive(Debug, Clone)]
pub struct ExpertNode {
    id: ExpertId,
    name: String,
    position: Point,
    activity_level: f32,
    specialization_areas: Vec<String>,
    recent_contributions: Vec<ContributionSummary>,
}

#[derive(Debug, Clone)]
pub struct ExpertConnection {
    from: ExpertId,
    to: ExpertId,
    strength: f32,
    collaboration_history: CollaborationHistory,
    connection_type: ConnectionType,
}

impl ExpertNetworkVisualization {
    pub fn update_from_orchestration(&mut self, orchestration: &OrchestrationEvent) {
        // Update expert activity levels
        for expert_id in &orchestration.experts_involved {
            if let Some(expert) = self.experts.iter_mut().find(|e| e.id == *expert_id) {
                expert.activity_level = (expert.activity_level * 0.8 + 0.2).min(1.0);
                expert.recent_contributions.push(ContributionSummary {
                    orchestration_id: orchestration.id,
                    contribution_type: orchestration.get_expert_contribution_type(*expert_id),
                    timestamp: orchestration.timestamp,
                });
            }
        }
        
        // Update connection strengths based on collaboration
        for expert_pair in orchestration.experts_involved.iter().combinations(2) {
            let (expert_a, expert_b) = (expert_pair[0], expert_pair[1]);
            
            if let Some(connection) = self.connections.iter_mut()
                .find(|c| (c.from == *expert_a && c.to == *expert_b) 
                         || (c.from == *expert_b && c.to == *expert_a)) {
                connection.strength = (connection.strength * 0.9 + 0.1).min(1.0);
                connection.collaboration_history.add_collaboration(orchestration.clone());
            }
        }
        
        // Update layout positions based on new activity
        self.layout_algorithm.update_positions(&mut self.experts, &self.connections);
    }
}
```

## Performance Optimization and Scalability

### Efficient Rendering Pipeline
```rust
// Optimized rendering system for large conversations
pub struct ConversationRenderer {
    virtual_scrolling: VirtualScrolling,
    render_cache: RenderCache,
    text_layout_cache: TextLayoutCache,
}

impl ConversationRenderer {
    pub fn render_optimized_conversation(
        &mut self,
        conversation: &ConversationState,
        viewport: Rectangle,
    ) -> Vec<Element<Message>> {
        // Virtual scrolling for large conversations
        let visible_range = self.virtual_scrolling.calculate_visible_range(
            conversation.turns.len(),
            viewport,
        );
        
        // Only render visible conversation turns
        conversation.turns[visible_range.clone()]
            .iter()
            .enumerate()
            .map(|(index, turn)| {
                let turn_index = visible_range.start + index;
                
                // Check render cache first
                if let Some(cached_element) = self.render_cache.get(turn_index) {
                    return cached_element;
                }
                
                // Render turn and cache result
                let rendered_turn = self.render_conversation_turn(turn);
                self.render_cache.insert(turn_index, rendered_turn.clone());
                
                rendered_turn
            })
            .collect()
    }
    
    fn render_conversation_turn(&mut self, turn: &ConversationTurn) -> Element<Message> {
        // Use cached text layouts when possible
        let user_text = self.text_layout_cache.get_or_create(
            &turn.user_input,
            TextStyle::user_message(),
        );
        
        let agent_text = self.text_layout_cache.get_or_create(
            &turn.agent_response.content,
            TextStyle::agent_response(),
        );
        
        column![
            container(user_text).style(theme::Container::UserMessage),
            vertical_space(Length::Fixed(8.0)),
            container(
                column![
                    self.render_expert_attribution(&turn.expert_contributions),
                    vertical_space(Length::Fixed(4.0)),
                    agent_text,
                ]
            ).style(theme::Container::AgentResponse),
        ]
        .into()
    }
}
```

### Memory Management
```rust
// Efficient memory management for long-running GUI
pub struct ConversationMemoryManager {
    conversation_cache: LRUCache<ConversationId, ConversationState>,
    render_cache: LRUCache<TurnId, Element<Message>>,
    max_cached_conversations: usize,
    max_cached_turns_per_conversation: usize,
}

impl ConversationMemoryManager {
    pub fn new() -> Self {
        Self {
            conversation_cache: LRUCache::new(50),  // Cache up to 50 conversations
            render_cache: LRUCache::new(1000),      // Cache up to 1000 rendered turns
            max_cached_conversations: 50,
            max_cached_turns_per_conversation: 100,
        }
    }
    
    pub fn get_conversation(&mut self, id: ConversationId) -> Option<&ConversationState> {
        self.conversation_cache.get(&id)
    }
    
    pub fn cache_conversation(&mut self, conversation: ConversationState) {
        // Trim conversation if too long
        let trimmed_conversation = if conversation.turns.len() > self.max_cached_turns_per_conversation {
            let start_index = conversation.turns.len() - self.max_cached_turns_per_conversation;
            ConversationState {
                turns: conversation.turns[start_index..].to_vec(),
                ..conversation
            }
        } else {
            conversation
        };
        
        self.conversation_cache.insert(trimmed_conversation.id, trimmed_conversation);
    }
    
    pub fn cleanup_old_data(&mut self) {
        // Remove old render cache entries
        self.render_cache.clear_old_entries(Duration::from_hours(1));
        
        // Compact conversation data
        for conversation in self.conversation_cache.values_mut() {
            conversation.compact_old_turns();
        }
    }
}
```

---

*The GUI Architecture provides a sophisticated, native desktop interface that seamlessly integrates with SAGE's orchestration capabilities while maintaining excellent performance, beautiful design, and intuitive user experience through careful application of functional programming principles and modern UI design patterns.*