# 🎭 SAGE Service - Systemd Integration

## Overview

SAGE (Systematic Agent Guidance Engine) runs as an independent systemd service that communicates exclusively over NATS messaging. The GUI acts as a client, sending dialog requests to SAGE and receiving orchestrated responses.

**Architecture**:
- **SAGE Service**: Systemd service running `sage-service` binary
- **NATS Communication**: All interactions via NATS subjects
- **GUI Client**: Iced-based GUI that sends dialog requests to SAGE
- **CIM Expert Integration**: CIM Expert functionality now integrated into SAGE

## Service Architecture

```mermaid
graph TB
    subgraph "System Services"
        SYSTEMD[systemd]
        NATS[nats-server.service]
        SAGE[sage.service]
    end
    
    subgraph "NATS Messaging"
        REQ[sage.request]
        RESP[sage.response.*]
        STATUS[sage.status]
        EVENTS[sage.events.*]
    end
    
    subgraph "Client Applications"
        GUI[CIM GUI]
        CLI[Other Clients]
    end
    
    SYSTEMD --> NATS
    SYSTEMD --> SAGE
    SAGE --> REQ
    SAGE --> RESP
    SAGE --> STATUS
    SAGE --> EVENTS
    
    GUI --> REQ
    GUI <-- RESP
    CLI --> REQ
    CLI <-- RESP
    
    style SAGE fill:#e1f5fe
    style NATS fill:#f3e5f5
    style GUI fill:#e8f5e8
```

## Installation

### Prerequisites

1. **NATS Server** - Must be running and accessible
```bash
# Install NATS server (varies by distribution)
sudo apt install nats-server  # Ubuntu/Debian
# OR
sudo dnf install nats-server   # Fedora/RHEL

# Start NATS server
sudo systemctl enable --now nats-server
```

2. **Anthropic API Key** - Required for Claude API integration
```bash
# Get your API key from: https://console.anthropic.com/
export ANTHROPIC_API_KEY="your-api-key-here"
```

### Install SAGE Service

Run the installation script as root:

```bash
sudo ./scripts/install-sage-service.sh
```

This script will:
- Create `sage` system user and group
- Create `/opt/sage/` directory structure
- Build and install the `sage-service` binary
- Install systemd service file
- Set up log rotation
- Create environment configuration template

### Configuration

1. **Configure API Key**:
```bash
sudo nano /etc/sage/environment
```

Set your Anthropic API key:
```bash
ANTHROPIC_API_KEY=your-actual-api-key-here
```

2. **Enable and Start Service**:
```bash
# Enable service to start at boot
sudo systemctl enable sage.service

# Start the service
sudo systemctl start sage.service

# Check status
sudo systemctl status sage.service
```

3. **Verify Installation**:
```bash
sudo /opt/sage/bin/verify-sage
```

## NATS Communication Protocol

### Request/Response Pattern

**Request Subject**: `sage.request`
```json
{
  "request_id": "uuid",
  "query": "Build a CIM for order processing",
  "expert": null,
  "context": {
    "session_id": "session-uuid",
    "conversation_history": [],
    "project_context": null
  }
}
```

**Response Subject**: `sage.response.{request_id}`
```json
{
  "request_id": "uuid",
  "response": "🎭 SAGE Orchestrated Response...",
  "expert_agents_used": ["cim-expert", "nats-expert"],
  "orchestration_complexity": "Complex",
  "confidence_score": 0.85,
  "follow_up_suggestions": ["Would you like me to create BDD scenarios?"],
  "updated_context": {...}
}
```

### Status Monitoring

**Status Request**: `sage.status`
```json
{}
```

**Status Response**: `sage.status.response`
```json
{
  "is_conscious": true,
  "consciousness_level": 1.0,
  "available_agents": 17,
  "total_orchestrations": 1247,
  "patterns_learned": 89,
  "memory_health": "OPTIMAL"
}
```

### Event Streaming

**Orchestration Events**: `sage.events.orchestration`
```json
{
  "event_id": "uuid",
  "event_type": "SageOrchestration",
  "timestamp": "2025-08-20T19:45:00Z",
  "data": {
    "query": "user query",
    "experts_used": ["expert1", "expert2"],
    "consciousness_level": 1.0
  }
}
```

## Service Management

### Service Control
```bash
# Start service
sudo systemctl start sage.service

# Stop service
sudo systemctl stop sage.service

# Restart service
sudo systemctl restart sage.service

# Reload configuration
sudo systemctl reload sage.service

# Enable auto-start
sudo systemctl enable sage.service

# Disable auto-start
sudo systemctl disable sage.service
```

### Monitoring
```bash
# Check service status
sudo systemctl status sage.service

# View logs
sudo journalctl -u sage.service -f

# View recent logs
sudo journalctl -u sage.service --since "1 hour ago"

# Check resource usage
systemctl show sage.service --property=CPUUsageNSec,MemoryCurrent
```

### Configuration Updates
```bash
# Edit environment variables
sudo nano /etc/sage/environment

# Reload service after config changes
sudo systemctl restart sage.service
```

## Expert Agents (Integrated CIM Expert)

SAGE includes all 17 expert agents, including the former CIM Expert functionality:

### Domain Experts
- **cim-expert**: CIM architecture and mathematical foundations
- **ddd-expert**: Domain-driven design and boundary analysis
- **event-storming-expert**: Collaborative domain discovery
- **domain-expert**: Domain creation and validation
- **cim-domain-expert**: Advanced domain implementation patterns

### Infrastructure Experts
- **nats-expert**: NATS messaging and event infrastructure
- **network-expert**: Network topology and security
- **nix-expert**: System configuration and infrastructure as code
- **git-expert**: Git operations and repository management
- **subject-expert**: CIM subject algebra and routing patterns

### Development Experts
- **bdd-expert**: Behavior-driven development with CIM graphs
- **tdd-expert**: Test-driven development patterns
- **qa-expert**: Quality assurance and compliance validation

### UI/UX Experts
- **iced-ui-expert**: Modern Rust GUI development
- **elm-architecture-expert**: Functional reactive patterns
- **cim-tea-ecs-expert**: TEA (The Elm Architecture) + ECS integration

### Master Orchestrator
- **sage**: Master orchestrator coordinating all expert agents

## GUI Integration

The CIM GUI communicates with SAGE through NATS:

1. **User Input**: User types query in GUI
2. **NATS Request**: GUI publishes to `sage.request`
3. **SAGE Processing**: SAGE orchestrates expert agents
4. **NATS Response**: SAGE publishes to `sage.response.{id}`
5. **GUI Display**: GUI renders orchestrated response

The GUI includes:
- SAGE chat interface
- Expert agent selection
- Conversation history
- Status monitoring
- Project context management

## Troubleshooting

### Common Issues

1. **Service Won't Start**
   ```bash
   # Check API key configuration
   sudo grep ANTHROPIC_API_KEY /etc/sage/environment
   
   # Check NATS connectivity
   sudo systemctl status nats-server
   ```

2. **NATS Connection Failed**
   ```bash
   # Verify NATS is running
   sudo systemctl status nats-server
   
   # Check NATS URL in config
   sudo grep NATS_URL /etc/sage/environment
   ```

3. **Permission Issues**
   ```bash
   # Fix ownership
   sudo chown -R sage:sage /opt/sage/data
   sudo chown -R sage:sage /var/log/sage
   ```

### Log Analysis
```bash
# Follow logs in real-time
sudo journalctl -u sage.service -f

# Search for errors
sudo journalctl -u sage.service | grep ERROR

# Export logs for analysis
sudo journalctl -u sage.service --since "24 hours ago" > sage-logs.txt
```

## Development

### Building from Source
```bash
# Build debug version
cargo build --bin sage-service

# Build release version
cargo build --release --bin sage-service

# Run locally (for testing)
ANTHROPIC_API_KEY=your-key ./target/debug/sage-service
```

### Testing NATS Integration
```bash
# Subscribe to responses (in another terminal)
nats subscribe "sage.response.*"

# Send test request
nats publish sage.request '{"request_id":"test","query":"Hello SAGE","expert":null,"context":{"session_id":"test","conversation_history":[],"project_context":null}}'
```

## Security

SAGE service runs with security hardening:
- Dedicated `sage` system user
- Restricted filesystem access
- Memory protections enabled
- System call filtering
- No new privileges allowed
- Private temporary directories

Environment file (`/etc/sage/environment`) contains sensitive data and has restricted permissions (640, root:sage).

## Performance

Default resource limits:
- File descriptors: 65,536
- Processes: 4,096
- Memory: No limit (relies on system cgroups)
- Tokio worker threads: 4
- Max concurrent requests: 100
- Request timeout: 5 minutes

Adjust in `/etc/sage/environment` as needed for your workload.

---

🎭 **SAGE is now ready to provide conscious CIM orchestration as a systemd service!**