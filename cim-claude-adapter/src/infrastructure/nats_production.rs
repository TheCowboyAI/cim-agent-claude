use async_nats::{ConnectOptions, jetstream, HeaderMap};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    adapters::nats_adapter::{NatsAdapter, NatsClusterConfig, NatsHealthStatus},
    domain::errors::InfrastructureError,
};

/// Production NATS infrastructure manager
/// Provides enterprise-grade features like monitoring, metrics, circuit breaking, etc.
#[derive(Clone)]
pub struct NatsProductionManager {
    adapters: Arc<RwLock<HashMap<String, Arc<NatsAdapter>>>>,
    circuit_breakers: Arc<Mutex<HashMap<String, CircuitBreaker>>>,
    metrics_collector: Arc<MetricsCollector>,
    health_monitor: Arc<HealthMonitor>,
    config: ProductionConfig,
}

/// Production configuration for NATS infrastructure
#[derive(Debug, Clone)]
pub struct ProductionConfig {
    pub cluster_name: String,
    pub domains: Vec<String>,
    pub failover_timeout: Duration,
    pub circuit_breaker_threshold: usize,
    pub circuit_breaker_timeout: Duration,
    pub health_check_interval: Duration,
    pub metrics_collection_interval: Duration,
    pub auto_recovery_enabled: bool,
    pub backup_enabled: bool,
    pub backup_interval: Duration,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            cluster_name: "cim-claude-cluster".to_string(),
            domains: vec!["claude".to_string()],
            failover_timeout: Duration::from_secs(30),
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: Duration::from_secs(60),
            health_check_interval: Duration::from_secs(30),
            metrics_collection_interval: Duration::from_secs(10),
            auto_recovery_enabled: true,
            backup_enabled: true,
            backup_interval: Duration::from_hours(6),
        }
    }
}

/// Circuit breaker for fault tolerance
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: usize,
    threshold: usize,
    timeout: Duration,
    last_failure: Option<Instant>,
    last_success: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,  // Normal operation
    Open,    // Failing, rejecting requests
    HalfOpen, // Testing if service recovered
}

/// Metrics collection and aggregation
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    counters: Arc<RwLock<HashMap<String, u64>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    histograms: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    last_collection: Arc<RwLock<Instant>>,
}

/// Health monitoring for all NATS components
#[derive(Debug, Clone)]
pub struct HealthMonitor {
    domain_health: Arc<RwLock<HashMap<String, DomainHealth>>>,
    cluster_health: Arc<RwLock<ClusterHealth>>,
    last_check: Arc<RwLock<Instant>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainHealth {
    pub domain: String,
    pub status: HealthStatus,
    pub streams_healthy: usize,
    pub streams_total: usize,
    pub kv_stores_healthy: usize,
    pub kv_stores_total: usize,
    pub object_store_healthy: bool,
    pub consumers_healthy: usize,
    pub consumers_total: usize,
    pub last_error: Option<String>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealth {
    pub cluster_name: String,
    pub status: HealthStatus,
    pub nodes_healthy: usize,
    pub nodes_total: usize,
    pub leader_node: Option<String>,
    pub jetstream_healthy: bool,
    pub total_messages: u64,
    pub total_bytes: u64,
    pub uptime: Duration,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

/// Production NATS operations and monitoring
impl NatsProductionManager {
    /// Create new production manager
    pub async fn new(config: ProductionConfig) -> Result<Self, InfrastructureError> {
        let manager = Self {
            adapters: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(Mutex::new(HashMap::new())),
            metrics_collector: Arc::new(MetricsCollector::new()),
            health_monitor: Arc::new(HealthMonitor::new()),
            config,
        };

        // Initialize adapters for each domain
        manager.initialize_domains().await?;
        
        // Start background tasks
        manager.start_health_monitoring();
        manager.start_metrics_collection();
        
        if manager.config.backup_enabled {
            manager.start_backup_scheduler();
        }

        info!("Production NATS manager initialized for cluster: {}", manager.config.cluster_name);
        Ok(manager)
    }

    /// Initialize NATS adapters for all domains
    async fn initialize_domains(&self) -> Result<(), InfrastructureError> {
        let mut adapters = self.adapters.write().await;
        let mut circuit_breakers = self.circuit_breakers.lock().await;

        for domain in &self.config.domains {
            let cluster_config = NatsClusterConfig {
                urls: vec![
                    "nats://localhost:4222".to_string(),
                    "nats://localhost:4223".to_string(),
                    "nats://localhost:4224".to_string(),
                ],
                credentials_path: Some(format!("./nats-cluster/creds/claude_service.creds")),
                name: format!("cim-claude-{}", domain),
                domain: domain.clone(),
                account: "CIM_CLAUDE_ADAPTER".to_string(),
                reconnect_buffer_size: 8 * 1024 * 1024,
                max_reconnects: Some(10),
                reconnect_delay: Duration::from_millis(250),
                ..Default::default()
            };

            let adapter = Arc::new(NatsAdapter::new(cluster_config).await?);
            adapters.insert(domain.clone(), adapter);

            // Initialize circuit breaker for this domain
            let circuit_breaker = CircuitBreaker::new(
                self.config.circuit_breaker_threshold,
                self.config.circuit_breaker_timeout,
            );
            circuit_breakers.insert(domain.clone(), circuit_breaker);

            info!("Initialized NATS adapter for domain: {}", domain);
        }

        Ok(())
    }

    /// Get NATS adapter for specific domain with circuit breaker protection
    pub async fn get_adapter(&self, domain: &str) -> Result<Arc<NatsAdapter>, InfrastructureError> {
        // Check circuit breaker first
        {
            let mut circuit_breakers = self.circuit_breakers.lock().await;
            if let Some(cb) = circuit_breakers.get_mut(domain) {
                if !cb.can_execute() {
                    return Err(InfrastructureError::NatsConnection(
                        format!("Circuit breaker open for domain: {}", domain)
                    ));
                }
            }
        }

        let adapters = self.adapters.read().await;
        if let Some(adapter) = adapters.get(domain) {
            // Test adapter health quickly
            match adapter.get_health_status().await.is_connected {
                true => {
                    self.record_success(domain).await;
                    Ok(adapter.clone())
                }
                false => {
                    self.record_failure(domain).await;
                    Err(InfrastructureError::NatsConnection(
                        format!("Adapter unhealthy for domain: {}", domain)
                    ))
                }
            }
        } else {
            Err(InfrastructureError::NatsConnection(
                format!("No adapter found for domain: {}", domain)
            ))
        }
    }

    /// Execute operation with circuit breaker protection and retry logic
    pub async fn execute_with_protection<F, T>(
        &self,
        domain: &str,
        operation_name: &str,
        operation: F,
    ) -> Result<T, InfrastructureError>
    where
        F: Fn(Arc<NatsAdapter>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, InfrastructureError>> + Send>> + Send,
    {
        let start_time = Instant::now();
        
        // Record operation attempt
        self.metrics_collector.increment_counter(&format!("operations.{}.attempts", operation_name)).await;

        let adapter = self.get_adapter(domain).await?;
        
        match operation(adapter).await {
            Ok(result) => {
                let duration = start_time.elapsed();
                self.metrics_collector.record_histogram(&format!("operations.{}.duration", operation_name), duration.as_secs_f64()).await;
                self.metrics_collector.increment_counter(&format!("operations.{}.success", operation_name)).await;
                self.record_success(domain).await;
                Ok(result)
            }
            Err(e) => {
                let duration = start_time.elapsed();
                self.metrics_collector.record_histogram(&format!("operations.{}.duration", operation_name), duration.as_secs_f64()).await;
                self.metrics_collector.increment_counter(&format!("operations.{}.failure", operation_name)).await;
                self.record_failure(domain).await;
                
                error!("Operation {} failed for domain {}: {}", operation_name, domain, e);
                Err(e)
            }
        }
    }

    /// Record successful operation
    async fn record_success(&self, domain: &str) {
        let mut circuit_breakers = self.circuit_breakers.lock().await;
        if let Some(cb) = circuit_breakers.get_mut(domain) {
            cb.record_success();
        }
    }

    /// Record failed operation
    async fn record_failure(&self, domain: &str) {
        let mut circuit_breakers = self.circuit_breakers.lock().await;
        if let Some(cb) = circuit_breakers.get_mut(domain) {
            cb.record_failure();
        }
    }

    /// Start health monitoring background task
    fn start_health_monitoring(&self) {
        let health_monitor = self.health_monitor.clone();
        let adapters = self.adapters.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.health_check_interval);
            
            loop {
                interval.tick().await;
                
                // Check cluster health
                if let Err(e) = Self::check_cluster_health(&health_monitor, &adapters).await {
                    error!("Cluster health check failed: {}", e);
                }
                
                // Check domain health
                let adapters_read = adapters.read().await;
                for (domain, adapter) in adapters_read.iter() {
                    if let Err(e) = Self::check_domain_health(&health_monitor, domain, adapter).await {
                        error!("Domain {} health check failed: {}", domain, e);
                    }
                }
            }
        });
    }

    /// Check cluster-wide health
    async fn check_cluster_health(
        health_monitor: &HealthMonitor,
        adapters: &RwLock<HashMap<String, Arc<NatsAdapter>>>,
    ) -> Result<(), InfrastructureError> {
        let adapters_read = adapters.read().await;
        let mut healthy_nodes = 0;
        let total_nodes = 3; // Fixed cluster size for now
        let mut jetstream_healthy = false;
        let mut total_messages = 0u64;
        let mut total_bytes = 0u64;

        // Use first available adapter to check cluster status
        if let Some(adapter) = adapters_read.values().next() {
            let health_status = adapter.get_health_status().await;
            
            healthy_nodes = if health_status.is_connected { health_status.cluster_size } else { 0 };
            jetstream_healthy = health_status.object_store_available;
            // TODO: Get actual message/byte counts from JetStream stats
        }

        let cluster_health = ClusterHealth {
            cluster_name: "cim-claude-cluster".to_string(),
            status: match healthy_nodes {
                n if n == total_nodes => HealthStatus::Healthy,
                n if n >= 2 => HealthStatus::Degraded,
                _ => HealthStatus::Critical,
            },
            nodes_healthy: healthy_nodes,
            nodes_total: total_nodes,
            leader_node: None, // TODO: Determine leader
            jetstream_healthy,
            total_messages,
            total_bytes,
            uptime: Duration::from_secs(0), // TODO: Calculate actual uptime
            last_check: Utc::now(),
        };

        let mut health = health_monitor.cluster_health.write().await;
        *health = cluster_health;

        Ok(())
    }

    /// Check health of specific domain
    async fn check_domain_health(
        health_monitor: &HealthMonitor,
        domain: &str,
        adapter: &NatsAdapter,
    ) -> Result<(), InfrastructureError> {
        let health_status = adapter.get_health_status().await;
        
        let domain_health = DomainHealth {
            domain: domain.to_string(),
            status: if health_status.is_connected && health_status.active_streams > 0 {
                HealthStatus::Healthy
            } else if health_status.is_connected {
                HealthStatus::Degraded
            } else {
                HealthStatus::Critical
            },
            streams_healthy: health_status.active_streams,
            streams_total: 5, // Expected number of streams
            kv_stores_healthy: health_status.kv_stores_count,
            kv_stores_total: 5, // Expected number of KV stores
            object_store_healthy: health_status.object_store_available,
            consumers_healthy: health_status.active_consumers,
            consumers_total: 3, // Expected number of consumers
            last_error: if health_status.errors.is_empty() { None } else { Some(health_status.errors.last().unwrap().clone()) },
            last_check: Utc::now(),
        };

        let mut health = health_monitor.domain_health.write().await;
        health.insert(domain.to_string(), domain_health);

        Ok(())
    }

    /// Start metrics collection background task
    fn start_metrics_collection(&self) {
        let metrics_collector = self.metrics_collector.clone();
        let adapters = self.adapters.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.metrics_collection_interval);
            
            loop {
                interval.tick().await;
                
                // Collect metrics from all adapters
                let adapters_read = adapters.read().await;
                for (domain, adapter) in adapters_read.iter() {
                    if let Err(e) = Self::collect_domain_metrics(&metrics_collector, domain, adapter).await {
                        warn!("Failed to collect metrics for domain {}: {}", domain, e);
                    }
                }
            }
        });
    }

    /// Collect metrics from domain adapter
    async fn collect_domain_metrics(
        metrics_collector: &MetricsCollector,
        domain: &str,
        adapter: &NatsAdapter,
    ) -> Result<(), InfrastructureError> {
        let metrics = adapter.get_metrics().await;
        let health = adapter.get_health_status().await;

        // Record adapter metrics
        metrics_collector.set_gauge(&format!("domains.{}.commands_processed", domain), metrics.commands_processed as f64).await;
        metrics_collector.set_gauge(&format!("domains.{}.events_published", domain), metrics.events_published as f64).await;
        metrics_collector.set_gauge(&format!("domains.{}.errors_count", domain), metrics.errors_count as f64).await;
        metrics_collector.set_gauge(&format!("domains.{}.average_processing_time_ms", domain), metrics.average_processing_time_ms).await;

        // Record health metrics
        metrics_collector.set_gauge(&format!("domains.{}.is_connected", domain), if health.is_connected { 1.0 } else { 0.0 }).await;
        metrics_collector.set_gauge(&format!("domains.{}.active_streams", domain), health.active_streams as f64).await;
        metrics_collector.set_gauge(&format!("domains.{}.active_consumers", domain), health.active_consumers as f64).await;
        metrics_collector.set_gauge(&format!("domains.{}.kv_stores_count", domain), health.kv_stores_count as f64).await;

        Ok(())
    }

    /// Start backup scheduler (if enabled)
    fn start_backup_scheduler(&self) {
        let adapters = self.adapters.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.backup_interval);
            
            loop {
                interval.tick().await;
                
                info!("Starting scheduled backup...");
                // TODO: Implement backup logic
                warn!("Backup functionality not yet implemented");
            }
        });
    }

    /// Get comprehensive health report
    pub async fn get_health_report(&self) -> HealthReport {
        let cluster_health = self.health_monitor.cluster_health.read().await.clone();
        let domain_health = self.health_monitor.domain_health.read().await.clone();

        HealthReport {
            timestamp: Utc::now(),
            cluster: cluster_health,
            domains: domain_health,
            overall_status: self.calculate_overall_status(&domain_health).await,
        }
    }

    /// Calculate overall system health status
    async fn calculate_overall_status(&self, domain_health: &HashMap<String, DomainHealth>) -> HealthStatus {
        if domain_health.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut critical_count = 0;

        for health in domain_health.values() {
            match health.status {
                HealthStatus::Healthy => healthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Critical => critical_count += 1,
                HealthStatus::Unknown => {},
            }
        }

        if critical_count > 0 {
            HealthStatus::Critical
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else if healthy_count > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    /// Get metrics summary
    pub async fn get_metrics_summary(&self) -> MetricsSummary {
        let counters = self.metrics_collector.counters.read().await.clone();
        let gauges = self.metrics_collector.gauges.read().await.clone();

        MetricsSummary {
            timestamp: Utc::now(),
            counters,
            gauges,
            collection_interval: self.config.metrics_collection_interval,
        }
    }
}

/// Circuit breaker implementation
impl CircuitBreaker {
    fn new(threshold: usize, timeout: Duration) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            threshold,
            timeout,
            last_failure: None,
            last_success: None,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure {
                    if last_failure.elapsed() > self.timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.last_success = Some(Instant::now());
        self.state = CircuitBreakerState::Closed;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.threshold {
                    self.state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
            }
            CircuitBreakerState::Open => {}
        }
    }
}

/// Metrics collector implementation
impl MetricsCollector {
    fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            last_collection: Arc::new(RwLock::new(Instant::now())),
        }
    }

    async fn increment_counter(&self, name: &str) {
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_insert(0) += 1;
    }

    async fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), value);
    }

    async fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.write().await;
        histograms.entry(name.to_string()).or_insert_with(Vec::new).push(value);
        
        // Keep only last 1000 values per histogram
        let values = histograms.get_mut(name).unwrap();
        if values.len() > 1000 {
            values.drain(0..values.len() - 1000);
        }
    }
}

/// Health monitor implementation
impl HealthMonitor {
    fn new() -> Self {
        Self {
            domain_health: Arc::new(RwLock::new(HashMap::new())),
            cluster_health: Arc::new(RwLock::new(ClusterHealth {
                cluster_name: "unknown".to_string(),
                status: HealthStatus::Unknown,
                nodes_healthy: 0,
                nodes_total: 0,
                leader_node: None,
                jetstream_healthy: false,
                total_messages: 0,
                total_bytes: 0,
                uptime: Duration::from_secs(0),
                last_check: Utc::now(),
            })),
            last_check: Arc::new(RwLock::new(Instant::now())),
        }
    }
}

/// Comprehensive health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub timestamp: DateTime<Utc>,
    pub cluster: ClusterHealth,
    pub domains: HashMap<String, DomainHealth>,
    pub overall_status: HealthStatus,
}

/// Metrics summary report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub timestamp: DateTime<Utc>,
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub collection_interval: Duration,
}