/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

// Adapters module
// Implement external interfaces and integrations

pub mod claude_api_adapter;
pub mod nats_adapter;

pub use claude_api_adapter::ClaudeApiAdapter;
pub use nats_adapter::NatsAdapter;
