// TEA-ECS Bridge Core Implementation
//
// This is the critical bridge that connects:
// - TEA Display Layer: Synchronous Model-View-Update for UI rendering
// - ECS Communication Layer: Asynchronous Entity-Component-System for message bus operations
//
// The bridge ensures clean separation while enabling efficient data flow and state synchronization.

use super::*;
use crate::{
    adapters::NatsAdapter,
    domain::events::DomainEvent,
};
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;
use tracing::{info, warn, error, debug};

/// Core bridge that connects TEA display with ECS communication
pub struct TeaEcsBridge {
    // Entity management
    entity_manager: Arc<RwLock<EntityManager>>,
    
    // TEA to ECS: Commands from display layer
    command_sender: mpsc::UnboundedSender<EcsCommand>,
    command_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<EcsCommand>>>>,
    
    // ECS to TEA: Events to display layer  
    event_sender: mpsc::UnboundedSender<TeaEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<TeaEvent>>>>,
    
    // System registry
    systems: Arc<RwLock<SystemRegistry>>,
    
    // Synchronization manager
    sync_manager: Arc<RwLock<EntitySyncManager>>,
    
    // NATS adapter for ECS layer
    nats_adapter: Arc<NatsAdapter>,
    
    // Bridge state
    is_running: Arc<RwLock<bool>>,
}

impl TeaEcsBridge {
    /// Create new TEA-ECS bridge
    pub fn new(nats_adapter: Arc<NatsAdapter>) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        Self {
            entity_manager: Arc::new(RwLock::new(EntityManager::new())),
            command_sender: command_tx,
            command_receiver: Arc::new(RwLock::new(Some(command_rx))),
            event_sender: event_tx,
            event_receiver: Arc::new(RwLock::new(Some(event_rx))),
            systems: Arc::new(RwLock::new(SystemRegistry::new())),
            sync_manager: Arc::new(RwLock::new(EntitySyncManager::new())),
            nats_adapter,
            is_running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start the bridge (begins async processing)
    pub async fn start(&self) -> Result<(), BridgeError> {
        info!("Starting TEA-ECS bridge");
        
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(BridgeError::SystemError("Bridge already running".to_string()));
            }
            *running = true;
        }
        
        // Register core systems
        self.register_core_systems().await?;
        
        // Start command processing loop
        self.start_command_processor().await?;
        
        // Start synchronization loop
        self.start_sync_loop().await?;
        
        // Start NATS event subscription
        self.start_nats_subscriber().await?;
        
        info!("TEA-ECS bridge started successfully");
        Ok(())
    }
    
    /// Stop the bridge
    pub async fn stop(&self) -> Result<(), BridgeError> {
        info!("Stopping TEA-ECS bridge");
        
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }
        
        // Perform final sync
        let sync_manager = self.sync_manager.read().await;
        if let Err(e) = sync_manager.final_sync().await {
            warn!("Error during final sync: {}", e);
        }
        
        info!("TEA-ECS bridge stopped");
        Ok(())
    }
    
    /// Get entity manager for TEA layer access
    pub fn entity_manager(&self) -> Arc<RwLock<EntityManager>> {
        Arc::clone(&self.entity_manager)
    }
    
    /// Send command from TEA to ECS (non-blocking)
    pub fn send_command(&self, command: EcsCommand) -> Result<(), BridgeError> {
        self.command_sender.send(command)
            .map_err(|e| BridgeError::CommandDispatchError(e.to_string()))
    }
    
    /// Get event receiver for TEA layer
    pub async fn take_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<TeaEvent>> {
        let mut receiver_guard = self.event_receiver.write().await;
        receiver_guard.take()
    }
    
    /// Register a system in the ECS layer
    pub async fn register_system<S>(&self, name: String, system: S) -> Result<(), BridgeError>
    where
        S: System + Send + Sync + 'static,
    {
        let mut systems = self.systems.write().await;
        systems.register(name, Box::new(system));
        Ok(())
    }
    
    /// Register core systems for Claude adapter functionality
    async fn register_core_systems(&self) -> Result<(), BridgeError> {
        let mut systems = self.systems.write().await;
        
        // NATS message system for Claude API communication
        let nats_system = NatsMessageSystem::new(Arc::clone(&self.nats_adapter));
        systems.register("nats_messages".to_string(), Box::new(nats_system));
        
        // Conversation management system
        let conversation_system = ConversationManagementSystem::new();
        systems.register("conversations".to_string(), Box::new(conversation_system));
        
        // Configuration management system  
        let config_system = ConfigurationSystem::new();
        systems.register("configuration".to_string(), Box::new(config_system));
        
        // Tool invocation system
        let tool_system = ToolInvocationSystem::new(Arc::clone(&self.nats_adapter));
        systems.register("tools".to_string(), Box::new(tool_system));
        
        info!("Registered {} core systems", systems.system_count());
        Ok(())
    }
    
    /// Start async command processing loop
    async fn start_command_processor(&self) -> Result<(), BridgeError> {
        let command_receiver = {
            let mut receiver_guard = self.command_receiver.write().await;
            receiver_guard.take()
                .ok_or_else(|| BridgeError::SystemError("Command receiver already taken".to_string()))?
        };
        
        let systems = Arc::clone(&self.systems);
        let entity_manager = Arc::clone(&self.entity_manager);
        let event_sender = self.event_sender.clone();
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            Self::command_processor_loop(
                command_receiver,
                systems,
                entity_manager,
                event_sender,
                is_running,
            ).await;
        });
        
        Ok(())
    }
    
    /// Command processing loop (runs in background task)
    async fn command_processor_loop(
        mut command_receiver: mpsc::UnboundedReceiver<EcsCommand>,
        systems: Arc<RwLock<SystemRegistry>>,
        entity_manager: Arc<RwLock<EntityManager>>,
        event_sender: mpsc::UnboundedSender<TeaEvent>,
        is_running: Arc<RwLock<bool>>,
    ) {
        info!("Starting command processor loop");
        
        while *is_running.read().await {
            tokio::select! {
                command = command_receiver.recv() => {
                    match command {
                        Some(cmd) => {
                            debug!("Processing command: {:?}", cmd);
                            
                            // Execute command through appropriate system
                            let systems_guard = systems.read().await;
                            let mut entity_guard = entity_manager.write().await;
                            
                            match Self::execute_command(&cmd, &*systems_guard, &mut *entity_guard).await {
                                Ok(events) => {
                                    // Send resulting events to TEA layer
                                    for event in events {
                                        if let Err(e) = event_sender.send(event) {
                                            error!("Failed to send event to TEA layer: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Command execution failed: {}", e);
                                    
                                    // Send error event to TEA layer
                                    let error_event = TeaEvent::ErrorOccurred {
                                        error: e.to_string(),
                                        timestamp: Utc::now(),
                                    };
                                    
                                    if let Err(send_err) = event_sender.send(error_event) {
                                        error!("Failed to send error event: {}", send_err);
                                    }
                                }
                            }
                        }
                        None => {
                            info!("Command channel closed, stopping processor");
                            break;
                        }
                    }
                }
                
                // Graceful shutdown check
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    // Continue loop
                }
            }
        }
        
        info!("Command processor loop stopped");
    }
    
    /// Execute command through appropriate system
    async fn execute_command(
        command: &EcsCommand,
        systems: &SystemRegistry,
        entity_manager: &mut EntityManager,
    ) -> Result<Vec<TeaEvent>, BridgeError> {
        match command {
            EcsCommand::SendMessageToNats { conversation_id, content, timestamp } => {
                if let Some(system) = systems.get_system("nats_messages") {
                    system.execute(command.clone(), entity_manager).await
                } else {
                    Err(BridgeError::SystemError("NATS message system not found".to_string()))
                }
            }
            
            EcsCommand::CreateConversation { title } => {
                if let Some(system) = systems.get_system("conversations") {
                    system.execute(command.clone(), entity_manager).await
                } else {
                    Err(BridgeError::SystemError("Conversation system not found".to_string()))
                }
            }
            
            EcsCommand::UpdateConfiguration { config_key, value } => {
                if let Some(system) = systems.get_system("configuration") {
                    system.execute(command.clone(), entity_manager).await
                } else {
                    Err(BridgeError::SystemError("Configuration system not found".to_string()))
                }
            }
            
            EcsCommand::InvokeTool { tool_id, parameters, conversation_id } => {
                if let Some(system) = systems.get_system("tools") {
                    system.execute(command.clone(), entity_manager).await
                } else {
                    Err(BridgeError::SystemError("Tool system not found".to_string()))
                }
            }
        }
    }
    
    /// Start entity synchronization loop
    async fn start_sync_loop(&self) -> Result<(), BridgeError> {
        let sync_manager = Arc::clone(&self.sync_manager);
        let entity_manager = Arc::clone(&self.entity_manager);
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            Self::sync_loop(sync_manager, entity_manager, is_running).await;
        });
        
        Ok(())
    }
    
    /// Synchronization loop (runs periodically)
    async fn sync_loop(
        sync_manager: Arc<RwLock<EntitySyncManager>>,
        entity_manager: Arc<RwLock<EntityManager>>,
        is_running: Arc<RwLock<bool>>,
    ) {
        info!("Starting entity synchronization loop");
        
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        
        while *is_running.read().await {
            interval.tick().await;
            
            // Perform synchronization
            {
                let mut sync_guard = sync_manager.write().await;
                let mut entity_guard = entity_manager.write().await;
                
                if let Err(e) = sync_guard.sync_entities(&mut *entity_guard).await {
                    error!("Entity synchronization failed: {}", e);
                }
            }
        }
        
        info!("Entity synchronization loop stopped");
    }
    
    /// Start NATS event subscriber
    async fn start_nats_subscriber(&self) -> Result<(), BridgeError> {
        let nats_adapter = Arc::clone(&self.nats_adapter);
        let event_sender = self.event_sender.clone();
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            Self::nats_subscriber_loop(nats_adapter, event_sender, is_running).await;
        });
        
        Ok(())
    }
    
    /// NATS event subscription loop
    async fn nats_subscriber_loop(
        nats_adapter: Arc<NatsAdapter>,
        event_sender: mpsc::UnboundedSender<TeaEvent>,
        is_running: Arc<RwLock<bool>>,
    ) {
        info!("Starting NATS event subscriber");
        
        // Subscribe to relevant event streams
        let subjects = vec![
            "cim.claude.conv.evt.>",
            "cim.claude.config.evt.>", 
            "cim.core.tools.evt.>",
        ];
        
        for subject in subjects {
            let adapter = Arc::clone(&nats_adapter);
            let sender = event_sender.clone();
            let running = Arc::clone(&is_running);
            let subject_owned = subject.to_string();
            
            tokio::spawn(async move {
                if let Err(e) = adapter.subscribe_to_events(
                    &subject_owned,
                    Box::new(move |event| {
                        let sender = sender.clone();
                        Box::pin(async move {
                            // Convert NATS event to TEA event
                            if let Some(tea_event) = Self::convert_nats_to_tea_event(event) {
                                if let Err(e) = sender.send(tea_event) {
                                    error!("Failed to send converted event: {}", e);
                                }
                            }
                        })
                    })
                ).await {
                    error!("NATS subscription failed for {}: {}", subject_owned, e);
                }
            });
        }
        
        // Keep subscriber alive
        while *is_running.read().await {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        info!("NATS event subscriber stopped");
    }
    
    /// Convert NATS domain event to TEA event
    fn convert_nats_to_tea_event(domain_event: DomainEvent) -> Option<TeaEvent> {
        match domain_event {
            DomainEvent::ConversationStarted { conversation_id, .. } => {
                Some(TeaEvent::ConversationCreated {
                    conversation_id,
                    timestamp: Utc::now(),
                })
            }
            
            DomainEvent::MessageSent { conversation_id, content, .. } => {
                Some(TeaEvent::MessageAdded {
                    conversation_id,
                    message_content: content,
                    timestamp: Utc::now(),
                })
            }
            
            DomainEvent::ClaudeResponseReceived { conversation_id, content, .. } => {
                Some(TeaEvent::ClaudeResponseReceived {
                    conversation_id,
                    response_content: content,
                    timestamp: Utc::now(),
                })
            }
            
            _ => None, // Other events not relevant to UI
        }
    }
}

/// System registry for managing ECS systems
pub struct SystemRegistry {
    systems: HashMap<String, Box<dyn System>>,
}

impl SystemRegistry {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, name: String, system: Box<dyn System>) {
        self.systems.insert(name, system);
    }
    
    pub fn get_system(&self, name: &str) -> Option<&dyn System> {
        self.systems.get(name).map(|s| s.as_ref())
    }
    
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }
}

impl std::fmt::Debug for SystemRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SystemRegistry {{ systems: {} }}", self.systems.len())
    }
}