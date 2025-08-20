# CIM Claude NATS Production Cluster

A production-ready NATS cluster implementation for the CIM Claude Adapter with JetStream, Object Store, KV Store, comprehensive monitoring, and enterprise security features.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     CIM Claude NATS Cluster                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │ NATS Node 1 │  │ NATS Node 2 │  │ NATS Node 3 │             │
│  │ (Leader)    │  │ (Follower)  │  │ (Follower)  │             │
│  │ :4222       │  │ :4223       │  │ :4224       │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│         │                 │                 │                  │
│         └─────────────────┼─────────────────┘                  │
│                           │                                    │
│  ┌─────────────────────────┼─────────────────────────────────┐  │
│  │           JetStream Clustering & Replication             │  │
│  │                                                          │  │
│  │  • Streams (3x replicated)    • KV Stores (3x replica)  │  │
│  │  • Object Store (3x replica)  • Consumers (durable)     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                      Monitoring Stack                          │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │ Prometheus  │  │   Grafana   │  │  Surveyor   │             │
│  │   :9090     │  │   :3000     │  │   :7777     │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
└─────────────────────────────────────────────────────────────────┘
```

## Features

### Core NATS Features
- **High Availability**: 3-node cluster with automatic failover
- **JetStream**: Persistent messaging with at-least-once delivery
- **Object Store**: Large file storage with content addressing
- **KV Store**: Fast key-value storage for metadata
- **Account Security**: NSC-based security with JWT authentication
- **TLS Encryption**: End-to-end encryption for all communications

### Production Features
- **Circuit Breakers**: Fault tolerance and cascading failure prevention
- **Health Monitoring**: Comprehensive health checks and status reporting
- **Metrics Collection**: Detailed metrics for all operations
- **Auto-recovery**: Automatic detection and recovery from failures
- **Backup & Restore**: Automated backup scheduling and point-in-time recovery
- **Load Balancing**: Intelligent connection distribution
- **Rate Limiting**: Configurable rate limits per domain/user

### Monitoring & Observability
- **Prometheus Metrics**: Detailed metrics collection and alerting
- **Grafana Dashboards**: Rich visualizations and real-time monitoring
- **NATS Surveyor**: Specialized NATS cluster monitoring
- **Log Aggregation**: Centralized logging with structured output
- **Alert Manager**: Intelligent alerting and notification routing

## Quick Start

### Prerequisites

- Docker and Docker Compose
- NATS CLI (`nats`)
- NSC (NATS Security Configuration)
- curl (for health checks)

### 1. Setup Cluster

```bash
# Run the comprehensive setup script
./scripts/setup-cluster.sh
```

This script will:
- Install required tools (NATS CLI, NSC)
- Generate TLS certificates
- Create NSC security credentials
- Start the 3-node NATS cluster
- Initialize JetStream streams and KV stores
- Create durable consumers
- Set up monitoring stack

### 2. Verify Installation

```bash
# Check cluster health
./scripts/health-check.sh

# View cluster status
./scripts/cluster-management.sh status
```

### 3. Access Monitoring

- **NATS Monitoring**: http://localhost:8222, http://localhost:8223, http://localhost:8224
- **NATS Surveyor**: http://localhost:7777
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)

## Configuration

### Cluster Configuration

The cluster is configured through multiple files:

- `cluster-config.yaml`: Main cluster configuration
- `node-{1,2,3}-config.conf`: Individual node configurations
- `docker-compose.yaml`: Container orchestration
- `monitoring/prometheus.yml`: Metrics collection setup

### Security Configuration

Security is implemented using NSC (NATS Security Configuration):

```bash
# View accounts
nsc list accounts

# View users for an account
nsc list users --account CIM_CLAUDE_ADAPTER

# Generate new user credentials
nsc add user new_user --account CIM_CLAUDE_ADAPTER \
    --allow-pub "claude.>" --allow-sub "claude.>"
nsc generate creds --account CIM_CLAUDE_ADAPTER --name new_user > new_user.creds
```

### Stream Configuration

JetStream streams are automatically configured:

| Stream | Subjects | Retention | Replicas | Purpose |
|--------|----------|-----------|----------|---------|
| `CIM_CLAUDE_CONV_CMD` | `claude.conv.cmd.*` | WorkQueue | 3 | Command processing |
| `CIM_CLAUDE_CONV_EVT` | `claude.conv.evt.*` | Limits | 3 | Event sourcing |
| `CIM_CLAUDE_CONV_RESP` | `claude.conv.resp.*` | Interest | 2 | Response delivery |
| `CIM_CLAUDE_TOOL_OPS` | `claude.tool.*` | WorkQueue | 3 | Tool operations |
| `CIM_CLAUDE_CONFIG` | `claude.config.*` | Limits | 3 | Configuration changes |

### KV Store Configuration

| KV Store | Purpose | TTL | Replicas |
|----------|---------|-----|----------|
| `CIM_CLAUDE_CONV_META` | Conversation metadata | 30d | 3 |
| `CIM_CLAUDE_SESSIONS` | Session information | 1d | 3 |
| `CIM_CLAUDE_CONFIG` | Runtime configuration | 1y | 3 |
| `CIM_CLAUDE_TOOL_STATE` | Tool execution state | 7d | 3 |
| `CIM_CLAUDE_RATE_LIMITS` | Rate limiting counters | 1h | 3 |

## Operations

### Cluster Management

```bash
# Start cluster
./scripts/cluster-management.sh start

# Stop cluster
./scripts/cluster-management.sh stop

# Restart cluster
./scripts/cluster-management.sh restart

# View detailed status
./scripts/cluster-management.sh status

# View logs
./scripts/cluster-management.sh logs [service]

# Run health check
./scripts/cluster-management.sh health
```

### Backup & Restore

```bash
# Create full backup
./scripts/cluster-management.sh backup all

# Create JetStream-only backup
./scripts/cluster-management.sh backup jetstream

# List available backups
ls -la backups/

# Restore from backup
./scripts/cluster-management.sh restore cim_claude_backup_20231201_120000 --force
```

### Monitoring

```bash
# Check cluster health
./scripts/health-check.sh

# View real-time metrics
nats --server=nats://localhost:4222 --creds=creds/claude_admin.creds \
    server ping --count=10

# Monitor specific stream
nats --server=nats://localhost:4222 --creds=creds/claude_admin.creds \
    stream info CIM_CLAUDE_CONV_EVT

# Monitor consumer lag
nats --server=nats://localhost:4222 --creds=creds/claude_service.creds \
    consumer info CIM_CLAUDE_CONV_CMD command_processor
```

## Development Usage

### Using the Production NATS Adapter

```rust
use cim_claude_adapter::infrastructure::nats_production::*;
use cim_claude_adapter::adapters::nats_adapter::NatsClusterConfig;

// Initialize production manager
let config = ProductionConfig {
    cluster_name: "cim-claude-cluster".to_string(),
    domains: vec!["claude".to_string()],
    failover_timeout: Duration::from_secs(30),
    circuit_breaker_threshold: 5,
    auto_recovery_enabled: true,
    ..Default::default()
};

let nats_manager = NatsProductionManager::new(config).await?;

// Execute operations with circuit breaker protection
let result = nats_manager.execute_with_protection(
    "claude",
    "publish_event", 
    |adapter| Box::pin(async move {
        adapter.publish_events(events).await
    })
).await?;

// Get health report
let health = nats_manager.get_health_report().await;
println!("Cluster status: {:?}", health.overall_status);

// Get metrics summary
let metrics = nats_manager.get_metrics_summary().await;
println!("Total operations: {}", metrics.counters.len());
```

### Subject Algebra

The cluster uses hierarchical subjects for message organization:

```
{domain}.{category}.{aggregate}.{event}.{id}

Examples:
claude.conv.cmd.session123.start
claude.conv.evt.conversation456.prompt_sent
claude.tool.ops.read_file.result
claude.config.rate_limits.update
```

### Message Headers

All messages include correlation headers:

```rust
let mut headers = HeaderMap::new();
headers.insert("correlation-id", correlation_id.to_string());
headers.insert("causation-id", causation_id.to_string());
headers.insert("event-id", event_id.to_string());
headers.insert("timestamp", Utc::now().to_rfc3339());
```

## Troubleshooting

### Common Issues

**1. Cluster Formation Problems**
```bash
# Check if all nodes can reach each other
docker exec cim-nats-1 nats-server --help
docker logs cim-nats-1

# Verify cluster routes
curl http://localhost:8222/routez
```

**2. JetStream Issues**
```bash
# Check JetStream status
nats --server=nats://localhost:4222 --creds=creds/claude_admin.creds \
    server info

# Check stream health
nats --server=nats://localhost:4222 --creds=creds/claude_admin.creds \
    stream report
```

**3. Authentication Problems**
```bash
# Verify credentials file
nsc describe user claude_admin --account CIM_CLAUDE_ADAPTER

# Test connection with credentials
nats --server=nats://localhost:4222 --creds=creds/claude_admin.creds \
    server ping
```

**4. Performance Issues**
```bash
# Check resource usage
docker stats

# Monitor message rates
curl -s http://localhost:8222/varz | jq '.in_msgs, .out_msgs'

# Check for slow consumers
curl -s http://localhost:8222/subsz | jq '.subscriptions[] | select(.slow_consumer == true)'
```

### Log Locations

- NATS Logs: `docker logs cim-nats-{1,2,3}`
- Application Logs: `logs/cim-claude-adapter.log`
- Health Check Logs: `logs/health-check.log`
- Prometheus Logs: `docker logs cim-prometheus`
- Grafana Logs: `docker logs cim-grafana`

### Recovery Procedures

**Node Recovery:**
```bash
# Restart specific node
docker-compose restart nats-node-2

# Force cluster reformation
./scripts/cluster-management.sh restart --force
```

**Data Recovery:**
```bash
# Restore from latest backup
./scripts/cluster-management.sh restore $(ls -t backups/*.tar.gz | head -1 | xargs basename -s .tar.gz)

# Manual stream recreation
nats stream add CIM_CLAUDE_CONV_EVT --subjects="claude.conv.evt.*" --storage=file --retention=limits
```

## Performance Tuning

### Cluster Configuration

For high-throughput workloads:

```yaml
# In node config files
max_connections: 65536
max_pending: 134217728  # 128MB
max_payload: 2097152    # 2MB
write_deadline: "2s"

jetstream:
  max_memory_store: 4GB
  max_file_store: 100GB
```

### JetStream Optimization

```bash
# Increase consumer batch size
nats consumer add STREAM_NAME consumer_name \
    --pull --batch=1000 --max-ack-pending=10000

# Optimize stream for high throughput
nats stream add STREAM_NAME \
    --subjects="subject.*" \
    --storage=file \
    --discard=old \
    --max-age=24h \
    --max-msgs-per-subject=1000000
```

## Security Considerations

### Network Security
- All inter-node communication uses TLS 1.3
- Client connections require valid JWT credentials
- Account isolation prevents cross-domain access
- Subject-level permissions control message access

### Data Security
- Messages encrypted in transit and at rest
- Credentials stored securely with proper file permissions
- Regular credential rotation recommended
- Audit logging enabled for all operations

### Access Control
- Role-based access control through NSC accounts
- Fine-grained permissions at subject level
- Rate limiting prevents abuse
- Connection limits prevent resource exhaustion

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Run health checks and integration tests
5. Submit pull request with detailed description

## Support

For issues and questions:

1. Check the troubleshooting section
2. Review cluster logs
3. Run health check script
4. Create issue with logs and configuration

## License

Copyright 2025 - CowboyAI, LLC. All rights reserved.