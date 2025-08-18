/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! CIM Expert Service Binary
//!
//! A complete service that provides CIM architectural expertise through multiple interfaces:
//! - HTTP API for conversation control
//! - NATS integration for CIM ecosystem communication
//! - WebSocket for real-time conversations
//! - Web interface for interactive consultations

use cim_claude_adapter::{
    CimExpertService, CimExpertTopic, CimExpertQuery, CimExpertResponse,
    infrastructure::{
        claude_client::ClaudeClientConfig,
        nats_client::{NatsClient, NatsClientConfig},
    },
};
use anyhow::{Result, Context};
use axum::{
    extract::{Query, State, WebSocketUpgrade, Path},
    response::{Html, IntoResponse, Json},
    routing::{get, post, delete},
    Router, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    services::ServeDir,
};
use tracing::{info, error, instrument};
use uuid::Uuid;

/// Service configuration loaded from TOML
#[derive(Debug, Clone, Deserialize)]
struct ServiceConfig {
    service: ServiceSettings,
    claude: ClaudeSettings, 
    nats: NatsSettings,
    _expert: ExpertSettings,
    web_interface: WebInterfaceSettings,
}

#[derive(Debug, Clone, Deserialize)]
struct ServiceSettings {
    bind_address: String,
    port: u16,
    _log_level: String,
    _max_concurrent_conversations: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct ClaudeSettings {
    _max_tokens: u32,
    _temperature: f32,
    timeout_seconds: u64,
    max_retries: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct NatsSettings {
    servers: Vec<String>,
    connection_timeout_seconds: u64,
    request_timeout_seconds: u64,
    max_reconnect_attempts: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpertSettings {
    _enable_conversation_history: bool,
    _max_conversation_length: usize,
    _enable_audit_logging: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct WebInterfaceSettings {
    enable: bool,
    static_files_path: String,
    enable_api_docs: bool,
}

/// Conversation session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub messages: Vec<ConversationMessage>,
    pub context: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub topic: Option<CimExpertTopic>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Expert,
    System,
}

/// Application state
#[derive(Clone)]
struct AppState {
    expert_service: Arc<CimExpertService>,
    conversations: Arc<RwLock<HashMap<String, ConversationSession>>>,
    _config: ServiceConfig,
}

/// HTTP API request types
#[derive(Debug, Deserialize)]
struct StartConversationRequest {
    context: Option<String>,
    user_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct StartConversationResponse {
    conversation_id: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct SendMessageRequest {
    message: String,
    topic: Option<CimExpertTopic>,
}

#[derive(Debug, Serialize)]
struct SendMessageResponse {
    response: CimExpertResponse,
    conversation_id: String,
    message_id: String,
}

#[derive(Debug, Deserialize)]
struct ListConversationsQuery {
    user_id: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ConversationSummary {
    id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    last_activity: chrono::DateTime<chrono::Utc>,
    message_count: usize,
    context: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("🚀 Starting CIM Expert Service");

    // Load configuration
    let config_path = std::env::var("CIM_EXPERT_CONFIG")
        .unwrap_or_else(|_| "cim-expert-config.toml".to_string());
    let config = load_config(&config_path).await
        .context("Failed to load service configuration")?;

    info!("📋 Configuration loaded from: {}", config_path);

    // Initialize Claude client
    let claude_api_key = std::env::var("CLAUDE_API_KEY")
        .context("CLAUDE_API_KEY environment variable not set")?;
        
    let claude_config = ClaudeClientConfig {
        api_key: claude_api_key,
        base_url: "https://api.anthropic.com".to_string(),
        timeout: Duration::from_secs(config.claude.timeout_seconds),
        max_retries: config.claude.max_retries,
        retry_delay: Duration::from_secs(2),
        user_agent: "cim-expert-service/0.1.0".to_string(),
    };

    // Initialize NATS client
    let nats_config = NatsClientConfig {
        servers: config.nats.servers.clone(),
        name: "cim-expert-service".to_string(),
        token: None,
        username: None,
        password: None,
        connect_timeout: Duration::from_secs(config.nats.connection_timeout_seconds),
        request_timeout: Duration::from_secs(config.nats.request_timeout_seconds),
        max_reconnect_attempts: config.nats.max_reconnect_attempts,
        reconnect_delay: Duration::from_secs(2),
    };

    let nats_client = NatsClient::new(nats_config).await
        .context("Failed to connect to NATS server")?;

    info!("📡 Connected to NATS servers: {:?}", config.nats.servers);

    // Initialize CIM Expert service
    let expert_service = Arc::new(
        CimExpertService::new(claude_config, nats_client).await
            .context("Failed to initialize CIM Expert service")?
    );

    // Start the expert service background task
    let expert_service_clone = expert_service.clone();
    tokio::spawn(async move {
        if let Err(e) = expert_service_clone.start().await {
            error!("CIM Expert service error: {:?}", e);
        }
    });

    info!("🧠 CIM Expert service initialized");

    // Create application state
    let app_state = AppState {
        expert_service,
        conversations: Arc::new(RwLock::new(HashMap::new())),
        _config: config.clone(),
    };

    // Build the HTTP service
    let app = create_app(app_state, &config).await?;

    // Start the HTTP server
    let addr = format!("{}:{}", config.service.bind_address, config.service.port)
        .parse::<SocketAddr>()
        .context("Invalid bind address format")?;

    info!("🌐 Starting HTTP server on {}", addr);
    info!("📖 API documentation: http://{}/docs", addr);
    info!("🖥️  Web interface: http://{}/", addr);

    let listener = tokio::net::TcpListener::bind(addr).await
        .context("Failed to bind to address")?;

    axum::serve(listener, app).await
        .context("Failed to start HTTP server")?;

    Ok(())
}

/// Load service configuration from TOML file
async fn load_config(path: &str) -> Result<ServiceConfig> {
    let content = tokio::fs::read_to_string(path).await
        .with_context(|| format!("Failed to read config file: {}", path))?;
    
    let config: ServiceConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path))?;
    
    Ok(config)
}

/// Create the Axum application with all routes
async fn create_app(state: AppState, config: &ServiceConfig) -> Result<Router> {
    let mut app = Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        
        // Conversation management API
        .route("/api/v1/conversations", post(start_conversation))
        .route("/api/v1/conversations", get(list_conversations))
        .route("/api/v1/conversations/:id", get(get_conversation))
        .route("/api/v1/conversations/:id/messages", post(send_message))
        .route("/api/v1/conversations/:id", delete(delete_conversation))
        
        // Direct expert queries (stateless)
        .route("/api/v1/expert/ask", post(ask_expert))
        .route("/api/v1/expert/topics", get(list_expert_topics))
        
        // WebSocket for real-time conversations
        .route("/api/v1/conversations/:id/ws", get(conversation_websocket))
        
        .with_state(state);

    // Add static file serving if web interface is enabled
    if config.web_interface.enable {
        app = app.nest_service("/static", ServeDir::new(&config.web_interface.static_files_path));
        app = app.route("/", get(serve_index_html));
        
        if config.web_interface.enable_api_docs {
            app = app.route("/docs", get(serve_api_docs));
        }
    }

    // Add middleware
    app = app.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive())
    );

    Ok(app)
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "cim-expert",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Metrics endpoint
async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let conversations = state.conversations.read().await;
    
    Json(serde_json::json!({
        "active_conversations": conversations.len(),
        "total_messages": conversations.values()
            .map(|c| c.messages.len())
            .sum::<usize>(),
        "uptime_seconds": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }))
}

/// Start a new conversation
#[instrument(skip(state))]
async fn start_conversation(
    State(state): State<AppState>,
    Json(request): Json<StartConversationRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let conversation_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();
    
    let session = ConversationSession {
        id: conversation_id.clone(),
        created_at: now,
        last_activity: now,
        messages: vec![
            ConversationMessage {
                id: Uuid::new_v4().to_string(),
                role: MessageRole::System,
                content: "Welcome to CIM Expert! I'm here to help you understand CIM's cognitive architecture - including Conceptual Spaces, memory engram patterns, graph-based workflows, and emergent intelligence. What would you like to explore?".to_string(),
                timestamp: now,
                topic: None,
                metadata: None,
            }
        ],
        context: request.context,
        user_id: request.user_id,
    };
    
    state.conversations.write().await.insert(conversation_id.clone(), session);
    
    info!("Started new conversation: {}", conversation_id);
    
    Ok(Json(StartConversationResponse {
        conversation_id,
        message: "Conversation started! Ask me anything about CIM architecture.".to_string(),
    }))
}

/// Send a message in a conversation
#[instrument(skip(state))]
async fn send_message(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(request): Json<SendMessageRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get conversation
    let mut conversations = state.conversations.write().await;
    let conversation = conversations.get_mut(&conversation_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Add user message
    let user_message_id = Uuid::new_v4().to_string();
    conversation.messages.push(ConversationMessage {
        id: user_message_id.clone(),
        role: MessageRole::User,
        content: request.message.clone(),
        timestamp: chrono::Utc::now(),
        topic: request.topic.clone(),
        metadata: None,
    });
    
    // Create expert query
    let expert_query = CimExpertQuery {
        question: request.message,
        topic: request.topic.clone().unwrap_or(CimExpertTopic::Architecture),
        domain_context: conversation.context.clone(),
        user_id: conversation.user_id.clone(),
        session_context: Some(cim_claude_adapter::SessionContext {
            session_id: conversation_id.clone(),
            conversation_history: conversation.messages.iter()
                .filter(|m| matches!(m.role, MessageRole::User))
                .map(|m| m.content.clone())
                .collect(),
            domain: conversation.context.clone(),
            user_context: conversation.user_id.clone(),
        }),
    };
    
    // Get expert response
    let expert_response = state.expert_service.handle_expert_query(expert_query).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Add expert message
    let expert_message_id = Uuid::new_v4().to_string();
    conversation.messages.push(ConversationMessage {
        id: expert_message_id.clone(),
        role: MessageRole::Expert,
        content: expert_response.explanation.clone(),
        timestamp: chrono::Utc::now(),
        topic: request.topic,
        metadata: Some(serde_json::to_value(&expert_response.metadata).unwrap()),
    });
    
    conversation.last_activity = chrono::Utc::now();
    
    info!("Processed message in conversation: {}", conversation_id);
    
    Ok(Json(SendMessageResponse {
        response: expert_response,
        conversation_id,
        message_id: expert_message_id,
    }))
}

/// List conversations
async fn list_conversations(
    State(state): State<AppState>,
    Query(params): Query<ListConversationsQuery>,
) -> impl IntoResponse {
    let conversations = state.conversations.read().await;
    
    let mut summaries: Vec<ConversationSummary> = conversations
        .values()
        .filter(|conv| {
            params.user_id.as_ref()
                .map(|uid| conv.user_id.as_ref() == Some(uid))
                .unwrap_or(true)
        })
        .map(|conv| ConversationSummary {
            id: conv.id.clone(),
            created_at: conv.created_at,
            last_activity: conv.last_activity,
            message_count: conv.messages.len(),
            context: conv.context.clone(),
        })
        .collect();
    
    // Sort by last activity (most recent first)
    summaries.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
    
    // Apply limit
    if let Some(limit) = params.limit {
        summaries.truncate(limit);
    }
    
    Json(summaries)
}

/// Get a specific conversation
async fn get_conversation(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let conversations = state.conversations.read().await;
    let conversation = conversations.get(&conversation_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(conversation.clone()))
}

/// Delete a conversation
async fn delete_conversation(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut conversations = state.conversations.write().await;
    conversations.remove(&conversation_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    info!("Deleted conversation: {}", conversation_id);
    Ok(StatusCode::NO_CONTENT)
}

/// Direct expert query (stateless)
async fn ask_expert(
    State(state): State<AppState>,
    Json(query): Json<CimExpertQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let response = state.expert_service.handle_expert_query(query).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(response))
}

/// List available expert topics
async fn list_expert_topics() -> impl IntoResponse {
    Json(vec![
        "Architecture",
        "MathematicalFoundations", 
        "NatsPatterns",
        "EventSourcing",
        "DomainModeling",
        "Implementation",
        "Troubleshooting",
        "ConceptualSpaces",
        "MemoryEngrams", 
        "GraphWorkflows",
        "CognitivePerformance",
        "EmergentIntelligence",
    ])
}

/// WebSocket handler for real-time conversations
async fn conversation_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state, conversation_id))
}

/// Handle WebSocket connections
async fn handle_websocket(
    socket: axum::extract::ws::WebSocket,
    _state: AppState,
    conversation_id: String,
) {
    info!("WebSocket connected for conversation: {}", conversation_id);
    // WebSocket implementation would go here
    // For now, we'll just close the connection
    let _ = socket;
}

/// Serve the main web interface
async fn serve_index_html() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

/// Serve API documentation
async fn serve_api_docs() -> Html<&'static str> {
    Html("<html><body><h1>API Documentation</h1><p>API documentation would be here</p></body></html>")
}