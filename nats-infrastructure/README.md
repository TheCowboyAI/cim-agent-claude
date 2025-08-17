# NATS Infrastructure for Claude API Adapter

This directory contains a comprehensive, production-ready NATS infrastructure designed specifically for the Claude API to NATS adapter. The infrastructure supports the hexagonal architecture with proper event-driven patterns, security, monitoring, and reliability.

## 🏗️ Architecture Overview

The NATS infrastructure implements a complete messaging platform that serves as:

- **Message Bus**: Event-driven communication with proper subject algebra
- **Object Store**: IPLD-compatible content-addressed storage (future enhancement)
- **Key-Value Store**: Session state, configuration, and metadata management
- **Security Framework**: NSC-based account and user management
- **Monitoring Platform**: Comprehensive observability and alerting

## 📁 Directory Structure

```
nats-infrastructure/
├── jetstream-config.yml          # JetStream streams and performance config
├── subject-hierarchy.yml         # Complete subject design with wildcards
├── consumer-config.yml           # Durable consumers with retry logic
├── kv-store-config.yml           # KV stores for state management
├── security-config.yml           # NSC accounts, users, and permissions
├── monitoring-config.yml         # Metrics, health checks, and alerting
├── deployment-scripts/
│   └── setup-nats-infrastructure.sh  # Automated deployment script
├── docker/
│   └── docker-compose.yml        # Container orchestration
└── README.md                     # This file
```

## 🚀 Core Components

### 1. JetStream Streams

| Stream | Purpose | Retention | Max Age | Storage |
|--------|---------|-----------|---------|---------|
| `CLAUDE_COMMANDS` | Command processing | WorkQueue | 24h | 10GB |
| `CLAUDE_EVENTS` | Event sourcing & audit | Limits | 30d | 50GB |
| `CLAUDE_RESPONSES` | API response delivery | Interest | 1h | 5GB |
| `CLAUDE_MONITORING` | Health & metrics data | Limits | 7d | 1GB |

### 2. Subject Hierarchy

```
# Command Subjects (Inbound)
claude.cmd.{session_id}.start      # Start new conversation
claude.cmd.{session_id}.prompt     # Send prompt to Claude
claude.cmd.{session_id}.end        # End conversation
claude.cmd.{session_id}.cancel     # Cancel pending request

# Event Subjects (Outbound) 
claude.event.{session_id}.started          # Conversation started
claude.event.{session_id}.prompt_sent      # Prompt sent to Claude API
claude.event.{session_id}.ended            # Conversation ended
claude.event.{session_id}.error            # Error occurred

# Response Subjects (Claude API Results)
claude.resp.{session_id}.content     # Claude response content
claude.resp.{session_id}.streaming   # Streaming response chunks
claude.resp.{session_id}.complete    # Response completed
claude.resp.{session_id}.error       # API error response

# Monitoring Subjects
claude.monitor.health.{component}    # Component health status
claude.monitor.metrics.{type}       # Performance metrics
claude.monitor.trace.{trace_id}     # Distributed tracing
```

### 3. Durable Consumers

| Consumer | Stream | Purpose | Max Deliver | Ack Wait |
|----------|--------|---------|-------------|----------|
| `claude-cmd-processor-v1` | CLAUDE_COMMANDS | Process all commands | 3 | 30s |
| `claude-resp-dist-v1` | CLAUDE_RESPONSES | Distribute responses | 2 | 15s |
| `claude-event-proc-v1` | CLAUDE_EVENTS | Event processing & audit | 5 | 60s |
| `claude-monitor-v1` | CLAUDE_MONITORING | Health & metrics processing | 1 | 10s |

### 4. KV Stores

| Store | Purpose | Max Value | TTL | Storage |
|-------|---------|-----------|-----|---------|
| `CLAUDE_SESSIONS` | Active conversation sessions | 1MB | 24h | File |
| `CLAUDE_CONVERSATIONS` | Conversation aggregate state | 512KB | 30d | File |
| `CLAUDE_RATE_LIMITS` | Rate limiting counters | 4KB | 1h | Memory |
| `CLAUDE_CIRCUIT_BREAKERS` | Circuit breaker states | 8KB | 30m | File |
| `CLAUDE_CONFIG` | Dynamic configuration | 64KB | ∞ | File |

### 5. Security Model

**Account Structure:**
- `SYS`: System account for NATS operations
- `CLAUDE_SERVICE`: Main service operations with JetStream
- `API_GATEWAY`: External client access with limited permissions
- `MONITORING`: Metrics collection and observability
- `AUDIT`: Compliance and audit logging

**Key Users:**
- `claude-service-primary`: Main service with full permissions
- `claude-service-worker`: Worker instances with limited scope
- `api-gateway-service`: External API access
- `prometheus-collector`: Metrics collection
- `audit-service`: Audit logging

## 🔧 Quick Start

### Option 1: Automated Setup Script

```bash
# Make script executable
chmod +x deployment-scripts/setup-nats-infrastructure.sh

# Run the setup (requires root/sudo)
sudo ./deployment-scripts/setup-nats-infrastructure.sh
```

### Option 2: Docker Compose

```bash
# Start the full infrastructure
cd docker/
docker-compose up -d

# Verify services are running
docker-compose ps

# View logs
docker-compose logs -f nats
```

### Option 3: Manual Configuration

1. **Install NATS Server and CLI Tools**:
```bash
# Install NATS server
curl -sf https://binaries.nats.dev/nats-io/nats-server/v2@latest | sh

# Install NATS CLI
curl -sf https://binaries.nats.dev/nats-io/natscli/v0@latest | sh

# Install NSC (NATS Security Configuration)
curl -sf https://binaries.nats.dev/nats-io/nsc/v2@latest | sh
```

2. **Configure Security**:
```bash
# Create operator
nsc add operator CIM_CLAUDE_OPERATOR

# Create accounts
nsc add account CLAUDE_SERVICE --operator CIM_CLAUDE_OPERATOR
nsc add account MONITORING --operator CIM_CLAUDE_OPERATOR
nsc add account AUDIT --operator CIM_CLAUDE_OPERATOR

# Enable JetStream for Claude service
nsc edit account CLAUDE_SERVICE --js-mem-storage 2G --js-disk-storage 20G
```

3. **Start NATS Server**:
```bash
# Start with JetStream enabled
nats-server --jetstream --store_dir ./jetstream_data --config nats-server.conf
```

4. **Create Infrastructure**:
```bash
# Create streams
nats stream add CLAUDE_COMMANDS --subjects="claude.cmd.*" --storage=file --retention=workqueue
nats stream add CLAUDE_EVENTS --subjects="claude.event.*" --storage=file --retention=limits
nats stream add CLAUDE_RESPONSES --subjects="claude.resp.*" --storage=file --retention=interest

# Create consumers
nats consumer add CLAUDE_COMMANDS claude-cmd-processor-v1 --deliver=new --ack=explicit

# Create KV stores
nats kv add CLAUDE_SESSIONS --max-value-size=1MB --ttl=24h
nats kv add CLAUDE_CONVERSATIONS --max-value-size=512KB --ttl=720h
```

## 📊 Monitoring & Observability

### Built-in Monitoring

The infrastructure includes comprehensive monitoring:

- **NATS Server Metrics**: Connection, message, and resource metrics
- **JetStream Metrics**: Stream and consumer performance
- **Application Metrics**: Claude API interaction metrics
- **Security Metrics**: Authentication and authorization events

### Monitoring Endpoints

| Service | Port | Endpoint | Purpose |
|---------|------|----------|---------|
| NATS Server | 8222 | `/varz` | Server variables and stats |
| NATS Server | 8222 | `/connz` | Connection details |
| NATS Server | 8222 | `/jsz` | JetStream details |
| Prometheus | 9090 | `/metrics` | Metrics collection |
| Grafana | 3000 | `/` | Visualization dashboards |

### Pre-configured Dashboards

1. **NATS Overview**: High-level cluster health
2. **JetStream Streams**: Stream performance and health
3. **Consumer Performance**: Lag, throughput, and errors
4. **Claude API Integration**: API interaction metrics
5. **Security Monitoring**: Authentication events

### Alerting Rules

Critical alerts included for:
- NATS server downtime
- High consumer lag
- JetStream storage issues
- Claude API error rates
- Security violations

## 🔒 Security Features

### NSC-Based Security
- **Operator-level**: CIM_CLAUDE_OPERATOR manages all accounts
- **Account-level**: Isolated namespaces with specific permissions
- **User-level**: Fine-grained publish/subscribe permissions
- **JWT Authentication**: Token-based authentication with expiration

### Data Protection
- **TLS Encryption**: All communication encrypted in transit
- **Access Control**: Subject-level permissions
- **Audit Logging**: Complete audit trail for compliance
- **Key Rotation**: Automatic key rotation policies

### Compliance Features
- **GDPR Compliance**: Data retention and deletion policies
- **SOC 2 Requirements**: Comprehensive audit logging
- **Access Reviews**: Quarterly permission reviews

## 🔄 High Availability & Disaster Recovery

### Backup Strategy
- **Daily Backups**: Automated backup of streams and KV stores
- **Point-in-Time Recovery**: 15-minute RPO
- **Configuration Backup**: NSC keys and configuration files
- **7-Day Retention**: Automatic cleanup of old backups

### Failover Configuration
- **Consumer Groups**: Load-balanced message processing
- **Auto-scaling**: Dynamic scaling based on load
- **Health Checks**: Continuous health monitoring
- **Circuit Breakers**: Automatic failure isolation

## 🚀 Performance Characteristics

### Throughput Expectations
- **Commands**: 1,000 messages/second
- **Events**: 500 messages/second  
- **Responses**: 2,000 messages/second
- **Monitoring**: 5,000 messages/second

### Latency Targets
- **Command Processing**: < 100ms (95th percentile)
- **Event Delivery**: < 50ms (95th percentile)
- **KV Operations**: < 10ms (95th percentile)
- **End-to-End**: < 500ms (conversation start to response)

### Resource Requirements

**Minimum Production Setup:**
- CPU: 4 cores
- Memory: 8GB
- Storage: 100GB SSD
- Network: 1Gbps

**Recommended Production Setup:**
- CPU: 8 cores
- Memory: 16GB
- Storage: 500GB NVMe SSD
- Network: 10Gbps

## 🧪 Testing

### Validation Commands

```bash
# Test basic connectivity
nats server check connection

# Test pub/sub functionality
echo "test message" | nats pub claude.test.validation
nats sub "claude.test.validation" --count=1

# Test JetStream
nats stream info CLAUDE_COMMANDS
nats consumer info CLAUDE_COMMANDS claude-cmd-processor-v1

# Test KV operations
nats kv put CLAUDE_SESSIONS test.key "test value"
nats kv get CLAUDE_SESSIONS test.key

# Load testing
nats bench claude.cmd.test --pub 10 --sub 1 --msgs 1000
```

### Integration Tests

The infrastructure supports the following integration test scenarios:

1. **Conversation Lifecycle**: Complete conversation flow testing
2. **Error Handling**: Failure scenarios and recovery
3. **Load Testing**: High-throughput performance validation
4. **Security Testing**: Authentication and authorization validation
5. **Failover Testing**: High availability validation

## 📚 Integration with Claude Adapter

### Application Configuration

The Claude API adapter should use the following NATS configuration:

```yaml
nats:
  url: "nats://localhost:4222"
  credentials_file: "/etc/nats/creds/claude_service-claude-service-primary.creds"
  
  # Connection settings
  max_reconnects: 10
  reconnect_wait: 2s
  ping_interval: 2m
  
  # JetStream settings
  jetstream:
    domain: "claude"
    
  # Subject configuration
  subjects:
    commands: "claude.cmd.>"
    events: "claude.event.{session_id}.>"
    responses: "claude.resp.{session_id}.>"
```

### Consumer Implementation

```go
// Example consumer setup in Go
consumer, err := js.PullSubscribe("claude.cmd.>", "claude-cmd-processor-v1")
if err != nil {
    log.Fatal(err)
}

for {
    msgs, err := consumer.Fetch(10, nats.MaxWait(30*time.Second))
    if err != nil {
        continue
    }
    
    for _, msg := range msgs {
        if err := processCommand(msg); err != nil {
            msg.Nak()
        } else {
            msg.Ack()
        }
    }
}
```

## 🤝 Support & Troubleshooting

### Common Issues

1. **Consumer Lag**: Check processing capacity and scale workers
2. **Storage Issues**: Monitor disk usage and cleanup policies  
3. **Connection Problems**: Verify network connectivity and credentials
4. **Performance Issues**: Review resource allocation and tuning

### Diagnostic Commands

```bash
# Check server status
nats server check

# Monitor consumer lag
nats consumer report

# View stream information
nats stream report

# Check account limits
nats account info CLAUDE_SERVICE

# Monitor real-time metrics
watch -n 1 'nats stream info CLAUDE_COMMANDS | grep "Messages:"'
```

### Log Locations

- **NATS Server**: `/var/log/nats/nats-server.log`
- **Application**: Configured via application logging
- **Monitoring**: Accessible via Grafana dashboards

## 📝 Contributing

To contribute improvements to the NATS infrastructure:

1. Review the existing configuration files
2. Test changes in a development environment
3. Update documentation as needed
4. Validate all security and performance requirements
5. Submit changes with comprehensive testing

## 📄 License

This NATS infrastructure configuration is part of the CIM-Start Claude adapter project and follows the same licensing terms.