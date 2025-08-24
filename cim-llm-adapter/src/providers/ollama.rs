//! Ollama Provider Implementation
//!
//! Provides integration with Ollama for running local LLMs
//! Supports models like Llama2, Vicuna, Mistral, CodeLlama, etc.

use super::{LlmProvider, Message, CompletionOptions, ProviderResponse, ProviderError, ProviderConfig, ProviderHealth, HealthStatus, TokenCount};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::collections::HashMap;

/// Default Ollama API base URL
const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Ollama provider implementation
pub struct OllamaProvider {
    config: ProviderConfig,
    client: Client,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, ProviderError> {
        // Get base URL from config or use default
        let base_url = config.base_url.clone()
            .or_else(|| std::env::var("OLLAMA_URL").ok())
            .unwrap_or_else(|| DEFAULT_OLLAMA_URL.to_string());
        
        let client = Client::new();
        
        Ok(Self { 
            config,
            client,
            base_url,
        })
    }
    
    /// List available models from Ollama
    pub async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::HttpError(format!("Failed to list Ollama models: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::ApiError {
                status_code: response.status().as_u16(),
                message: "Failed to list Ollama models".to_string(),
            });
        }
        
        #[derive(Deserialize)]
        struct ModelsResponse {
            models: Vec<ModelInfo>,
        }
        
        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }
        
        let models_response: ModelsResponse = response.json().await
            .map_err(|e| ProviderError::SerializationError(format!("Failed to parse models list: {}", e)))?;
        
        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<Vec<i32>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,  // max_tokens equivalent
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
    #[serde(default)]
    context: Vec<i32>,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    load_duration: Option<u64>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    prompt_eval_duration: Option<u64>,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    eval_duration: Option<u64>,
}

/// Convert messages to Ollama prompt format
fn messages_to_prompt(messages: Vec<Message>) -> (Option<String>, String) {
    let mut system_prompt = None;
    let mut conversation = Vec::new();
    
    for msg in messages {
        match msg.role.as_str() {
            "system" => {
                system_prompt = Some(msg.content);
            }
            "user" => {
                conversation.push(format!("User: {}", msg.content));
            }
            "assistant" => {
                conversation.push(format!("Assistant: {}", msg.content));
            }
            _ => {
                conversation.push(format!("{}: {}", msg.role, msg.content));
            }
        }
    }
    
    // Add prompt for assistant response
    conversation.push("Assistant:".to_string());
    
    (system_prompt, conversation.join("\n\n"))
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    async fn complete(
        &self,
        messages: Vec<Message>,
        options: Option<CompletionOptions>,
    ) -> Result<ProviderResponse, ProviderError> {
        let start = Instant::now();
        
        // Convert messages to Ollama format
        let (system_prompt, prompt) = messages_to_prompt(messages);
        
        // Determine model to use
        let model = options.as_ref()
            .and_then(|o| o.model.clone())
            .unwrap_or_else(|| self.config.model.clone());
        
        // Prepare Ollama options
        let ollama_options = options.as_ref().map(|opts| OllamaOptions {
            temperature: opts.temperature.map(|t| t as f32),
            num_predict: opts.max_tokens,
            top_p: opts.top_p.map(|t| t as f32),
            top_k: opts.top_k,
        });
        
        // Prepare request
        let request = OllamaRequest {
            model: model.clone(),
            prompt,
            system: system_prompt,
            template: None,
            context: None,
            stream: false,
            options: ollama_options,
        };
        
        // Make API request
        let url = format!("{}/api/generate", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::HttpError(format!("Failed to connect to Ollama: {}. Is Ollama running?", e)))?;
        
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ProviderError::ApiError {
                status_code: status.as_u16(),
                message: format!("Ollama error: {}", error_text),
            });
        }
        
        // Parse response
        let ollama_response: OllamaResponse = response.json().await
            .map_err(|e| ProviderError::SerializationError(format!("Failed to parse Ollama response: {}", e)))?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        // Extract token usage if available
        let token_count = if ollama_response.prompt_eval_count.is_some() || ollama_response.eval_count.is_some() {
            Some(TokenCount {
                input_tokens: ollama_response.prompt_eval_count.unwrap_or(0),
                output_tokens: ollama_response.eval_count.unwrap_or(0),
                total_tokens: ollama_response.prompt_eval_count.unwrap_or(0) + 
                    ollama_response.eval_count.unwrap_or(0),
            })
        } else {
            None
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("latency_ms".to_string(), serde_json::json!(latency_ms));
        if let Some(duration) = ollama_response.total_duration {
            metadata.insert("total_duration_ns".to_string(), serde_json::json!(duration));
        }
        
        Ok(ProviderResponse {
            content: ollama_response.response,
            model_used: model,
            token_count,
            finish_reason: if ollama_response.done { Some("complete".to_string()) } else { None },
            metadata,
        })
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, ProviderError> {
        let start = Instant::now();
        
        // Check if Ollama is running by listing models
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        match response {
            Ok(resp) if resp.status().is_success() => {
                // Try to list models to verify Ollama is working
                match self.list_models().await {
                    Ok(models) if !models.is_empty() => {
                        Ok(ProviderHealth {
                            status: HealthStatus::Healthy,
                            latency_ms: Some(latency_ms),
                            error_message: None,
                        })
                    }
                    Ok(_) => {
                        Ok(ProviderHealth {
                            status: HealthStatus::Degraded,
                            latency_ms: Some(latency_ms),
                            error_message: Some("Ollama is running but no models are installed".to_string()),
                        })
                    }
                    Err(e) => {
                        Ok(ProviderHealth {
                            status: HealthStatus::Unhealthy,
                            latency_ms: Some(latency_ms),
                            error_message: Some(format!("Ollama error: {}", e)),
                        })
                    }
                }
            }
            Ok(resp) => {
                Ok(ProviderHealth {
                    status: HealthStatus::Unhealthy,
                    latency_ms: Some(latency_ms),
                    error_message: Some(format!("Ollama returned status: {}", resp.status())),
                })
            }
            Err(e) => {
                Ok(ProviderHealth {
                    status: HealthStatus::Unhealthy,
                    latency_ms: None,
                    error_message: Some(format!("Cannot connect to Ollama at {}: {}. Is Ollama running?", self.base_url, e)),
                })
            }
        }
    }
    
    fn config(&self) -> &ProviderConfig {
        &self.config
    }
}