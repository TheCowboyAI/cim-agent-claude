#!/bin/bash

# CIM Claude NATS Cluster Setup Script
# Production deployment automation

set -euo pipefail

# Configuration
CLUSTER_NAME="cim-claude-cluster"
NATS_VERSION="2.10"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CERTS_DIR="${PROJECT_DIR}/certs"
CREDS_DIR="${PROJECT_DIR}/creds"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed or not in PATH"
    fi
    
    # Check if Docker Compose is available
    if ! command -v docker-compose &> /dev/null; then
        error "Docker Compose is not installed or not in PATH"
    fi
    
    # Check if NATS CLI is available
    if ! command -v nats &> /dev/null; then
        warn "NATS CLI not found. Installing..."
        install_nats_cli
    fi
    
    # Check if NSC is available
    if ! command -v nsc &> /dev/null; then
        warn "NSC (NATS Security Configuration) not found. Installing..."
        install_nsc
    fi
    
    log "Prerequisites check completed"
}

# Install NATS CLI
install_nats_cli() {
    log "Installing NATS CLI..."
    
    case "$(uname -s)" in
        Linux*)
            curl -sf https://binaries.nats.dev/nats-io/natscli/install.sh | sh
            ;;
        Darwin*)
            brew install nats-io/nats-tools/nats
            ;;
        *)
            error "Unsupported operating system for automatic NATS CLI installation"
            ;;
    esac
    
    log "NATS CLI installed successfully"
}

# Install NSC
install_nsc() {
    log "Installing NSC..."
    
    case "$(uname -s)" in
        Linux*)
            curl -sf https://binaries.nats.dev/nats-io/nsc/install.sh | sh
            ;;
        Darwin*)
            brew install nats-io/nats-tools/nsc
            ;;
        *)
            error "Unsupported operating system for automatic NSC installation"
            ;;
    esac
    
    log "NSC installed successfully"
}

# Generate TLS certificates
generate_certificates() {
    log "Generating TLS certificates..."
    
    mkdir -p "${CERTS_DIR}"
    cd "${CERTS_DIR}"
    
    # Generate CA private key
    openssl genrsa -out ca.key 4096
    
    # Generate CA certificate
    openssl req -new -x509 -days 365 -key ca.key -out ca.crt -subj "/C=US/ST=CA/L=SF/O=CIM/OU=Claude/CN=CIM-NATS-CA"
    
    # Generate server private key
    openssl genrsa -out server.key 4096
    
    # Generate server certificate signing request
    openssl req -new -key server.key -out server.csr -subj "/C=US/ST=CA/L=SF/O=CIM/OU=Claude/CN=nats-cluster"
    
    # Create server certificate extensions
    cat > server.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = nats-node-1
DNS.3 = nats-node-2
DNS.4 = nats-node-3
DNS.5 = *.cim.local
IP.1 = 127.0.0.1
IP.2 = 172.20.0.2
IP.3 = 172.20.0.3
IP.4 = 172.20.0.4
EOF
    
    # Generate server certificate
    openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 365 -extensions v3_req -extfile server.ext
    
    # Set appropriate permissions
    chmod 400 *.key
    chmod 444 *.crt
    
    log "TLS certificates generated successfully"
}

# Generate NSC credentials
generate_nsc_credentials() {
    log "Generating NSC credentials..."
    
    mkdir -p "${CREDS_DIR}"
    cd "${CREDS_DIR}"
    
    # Initialize NSC environment
    export NSC_HOME="${CREDS_DIR}/nsc"
    mkdir -p "${NSC_HOME}"
    
    # Add operator
    nsc add operator CIM_OPERATOR --generate-signing-key
    
    # Add account
    nsc add account CIM_CLAUDE_ADAPTER --operator CIM_OPERATOR
    
    # Add users
    nsc add user claude_admin --account CIM_CLAUDE_ADAPTER \\\n        --allow-pub \"claude.>\" \\\n        --allow-sub \"claude.>,_INBOX.>\"\n    \n    nsc add user claude_service --account CIM_CLAUDE_ADAPTER \\\n        --allow-pub \"claude.conv.>,claude.tool.>,claude.config.>\" \\\n        --allow-sub \"claude.conv.>,claude.tool.>,claude.config.>,_INBOX.>\"\n    \n    nsc add user claude_readonly --account CIM_CLAUDE_ADAPTER \\\n        --allow-sub \"claude.conv.evt.>,claude.conv.resp.>,claude.tool.results.>\"\n    \n    # Generate credentials files\n    nsc generate creds --account CIM_CLAUDE_ADAPTER --name claude_admin > claude_admin.creds\n    nsc generate creds --account CIM_CLAUDE_ADAPTER --name claude_service > claude_service.creds\n    nsc generate creds --account CIM_CLAUDE_ADAPTER --name claude_readonly > claude_readonly.creds\n    \n    # Create surveyor user for monitoring\n    nsc add user surveyor --account CIM_CLAUDE_ADAPTER \\\n        --allow-sub \"\\$SYS.>,claude.>\"\n    nsc generate creds --account CIM_CLAUDE_ADAPTER --name surveyor > surveyor.creds\n    \n    log \"NSC credentials generated successfully\"\n}\n\n# Setup monitoring\nsetup_monitoring() {\n    log \"Setting up monitoring configuration...\"\n    \n    mkdir -p \"${PROJECT_DIR}/monitoring/grafana/dashboards\"\n    mkdir -p \"${PROJECT_DIR}/monitoring/grafana/datasources\"\n    \n    # Create Prometheus configuration\n    cat > \"${PROJECT_DIR}/monitoring/prometheus.yml\" << EOF\nglobal:\n  scrape_interval: 15s\n  evaluation_interval: 15s\n\nscrape_configs:\n  - job_name: 'nats-cluster'\n    static_configs:\n      - targets: ['nats-node-1:8222', 'nats-node-2:8222', 'nats-node-3:8222']\n    metrics_path: /varz\n    scrape_interval: 30s\n    \n  - job_name: 'nats-jetstream'\n    static_configs:\n      - targets: ['nats-node-1:8222', 'nats-node-2:8222', 'nats-node-3:8222']\n    metrics_path: /jsz\n    scrape_interval: 30s\nEOF\n\n    # Create Grafana datasource configuration\n    cat > \"${PROJECT_DIR}/monitoring/grafana/datasources/prometheus.yml\" << EOF\napiVersion: 1\n\ndatasources:\n  - name: Prometheus\n    type: prometheus\n    access: proxy\n    url: http://prometheus:9090\n    isDefault: true\n    editable: true\nEOF\n\n    log \"Monitoring configuration created\"\n}\n\n# Start the cluster\nstart_cluster() {\n    log \"Starting NATS cluster...\"\n    \n    cd \"${PROJECT_DIR}\"\n    \n    # Pull latest images\n    docker-compose pull\n    \n    # Start the cluster\n    docker-compose up -d\n    \n    # Wait for cluster to be ready\n    log \"Waiting for cluster to be ready...\"\n    sleep 10\n    \n    # Verify cluster health\n    verify_cluster_health\n    \n    log \"NATS cluster started successfully\"\n}\n\n# Verify cluster health\nverify_cluster_health() {\n    log \"Verifying cluster health...\"\n    \n    local max_attempts=30\n    local attempt=1\n    \n    while [ $attempt -le $max_attempts ]; do\n        if curl -f http://localhost:8222/healthz > /dev/null 2>&1; then\n            log \"Node 1 is healthy\"\n            break\n        fi\n        \n        log \"Attempt $attempt/$max_attempts: Waiting for node 1...\"\n        sleep 2\n        ((attempt++))\n    done\n    \n    if [ $attempt -gt $max_attempts ]; then\n        error \"Node 1 failed to become healthy\"\n    fi\n    \n    # Check cluster status\n    if nats --server=nats://localhost:4222 --creds=\"${CREDS_DIR}/claude_admin.creds\" server ping > /dev/null 2>&1; then\n        log \"Cluster connectivity verified\"\n    else\n        warn \"Could not verify cluster connectivity with credentials\"\n    fi\n    \n    log \"Cluster health verification completed\"\n}\n\n# Initialize JetStream streams\ninitialize_streams() {\n    log \"Initializing JetStream streams...\"\n    \n    local server=\"nats://localhost:4222\"\n    local creds=\"${CREDS_DIR}/claude_admin.creds\"\n    \n    # Create conversation command stream\n    nats --server=\"$server\" --creds=\"$creds\" stream add CIM_CLAUDE_CONV_CMD \\\n        --subjects=\"claude.conv.cmd.*\" \\\n        --storage=file \\\n        --retention=workqueue \\\n        --max-msgs=100000 \\\n        --max-age=24h \\\n        --replicas=3 \\\n        --discard=old\n    \n    # Create conversation event stream\n    nats --server=\"$server\" --creds=\"$creds\" stream add CIM_CLAUDE_CONV_EVT \\\n        --subjects=\"claude.conv.evt.*\" \\\n        --storage=file \\\n        --retention=limits \\\n        --max-msgs=1000000 \\\n        --max-age=90d \\\n        --replicas=3 \\\n        --discard=old\n    \n    # Create conversation response stream\n    nats --server=\"$server\" --creds=\"$creds\" stream add CIM_CLAUDE_CONV_RESP \\\n        --subjects=\"claude.conv.resp.*\" \\\n        --storage=memory \\\n        --retention=interest \\\n        --max-msgs=50000 \\\n        --max-age=1h \\\n        --replicas=2 \\\n        --discard=old\n    \n    # Create tool operations stream\n    nats --server=\"$server\" --creds=\"$creds\" stream add CIM_CLAUDE_TOOL_OPS \\\n        --subjects=\"claude.tool.*\" \\\n        --storage=file \\\n        --retention=workqueue \\\n        --max-msgs=200000 \\\n        --max-age=7d \\\n        --replicas=3 \\\n        --discard=old\n    \n    # Create configuration stream\n    nats --server=\"$server\" --creds=\"$creds\" stream add CIM_CLAUDE_CONFIG \\\n        --subjects=\"claude.config.*\" \\\n        --storage=file \\\n        --retention=limits \\\n        --max-msgs=10000 \\\n        --max-age=365d \\\n        --replicas=3 \\\n        --discard=old\n    \n    log \"JetStream streams initialized successfully\"\n}\n\n# Initialize KV stores\ninitialize_kv_stores() {\n    log \"Initializing KV stores...\"\n    \n    local server=\"nats://localhost:4222\"\n    local creds=\"${CREDS_DIR}/claude_admin.creds\"\n    \n    # Conversation metadata KV\n    nats --server=\"$server\" --creds=\"$creds\" kv add CIM_CLAUDE_CONV_META \\\n        --description=\"Conversation metadata and state\" \\\n        --max-value-size=1048576 \\\n        --history=5 \\\n        --ttl=2592000 \\\n        --replicas=3\n    \n    # Session data KV\n    nats --server=\"$server\" --creds=\"$creds\" kv add CIM_CLAUDE_SESSIONS \\\n        --description=\"Active session information\" \\\n        --max-value-size=524288 \\\n        --history=3 \\\n        --ttl=86400 \\\n        --replicas=3\n    \n    # Configuration KV\n    nats --server=\"$server\" --creds=\"$creds\" kv add CIM_CLAUDE_CONFIG \\\n        --description=\"Runtime configuration\" \\\n        --max-value-size=1048576 \\\n        --history=10 \\\n        --ttl=31536000 \\\n        --replicas=3\n    \n    # Tool state KV\n    nats --server=\"$server\" --creds=\"$creds\" kv add CIM_CLAUDE_TOOL_STATE \\\n        --description=\"Tool execution state\" \\\n        --max-value-size=2097152 \\\n        --history=5 \\\n        --ttl=604800 \\\n        --replicas=3\n    \n    # Rate limiting KV\n    nats --server=\"$server\" --creds=\"$creds\" kv add CIM_CLAUDE_RATE_LIMITS \\\n        --description=\"Rate limiting counters\" \\\n        --max-value-size=4096 \\\n        --history=1 \\\n        --ttl=3600 \\\n        --replicas=3\n    \n    log \"KV stores initialized successfully\"\n}\n\n# Initialize object store\ninitialize_object_store() {\n    log \"Initializing object store...\"\n    \n    local server=\"nats://localhost:4222\"\n    local creds=\"${CREDS_DIR}/claude_admin.creds\"\n    \n    # Create object store for attachments\n    nats --server=\"$server\" --creds=\"$creds\" object add CIM_CLAUDE_ATTACHMENTS \\\n        --description=\"CIM Claude Adapter attachments and large objects\" \\\n        --max-bucket-size=53687091200 \\\n        --storage=file \\\n        --replicas=3\n    \n    log \"Object store initialized successfully\"\n}\n\n# Create durable consumers\ncreate_consumers() {\n    log \"Creating durable consumers...\"\n    \n    local server=\"nats://localhost:4222\"\n    local creds=\"${CREDS_DIR}/claude_service.creds\"\n    \n    # Command processor consumer\n    nats --server=\"$server\" --creds=\"$creds\" consumer add CIM_CLAUDE_CONV_CMD command_processor \\\n        --filter=\"claude.conv.cmd.*\" \\\n        --ack=explicit \\\n        --pull \\\n        --deliver=all \\\n        --max-deliver=3 \\\n        --wait=30s \\\n        --replay=instant\n    \n    # Tool operation consumer\n    nats --server=\"$server\" --creds=\"$creds\" consumer add CIM_CLAUDE_TOOL_OPS tool_processor \\\n        --filter=\"claude.tool.*\" \\\n        --ack=explicit \\\n        --pull \\\n        --deliver=all \\\n        --max-deliver=3 \\\n        --wait=30s \\\n        --replay=instant\n    \n    # Configuration change consumer\n    nats --server=\"$server\" --creds=\"$creds\" consumer add CIM_CLAUDE_CONFIG config_processor \\\n        --filter=\"claude.config.*\" \\\n        --ack=explicit \\\n        --pull \\\n        --deliver=all \\\n        --max-deliver=3 \\\n        --wait=30s \\\n        --replay=instant\n    \n    log \"Durable consumers created successfully\"\n}\n\n# Display cluster information\nshow_cluster_info() {\n    log \"Cluster information:\"\n    echo \"\"\n    echo \"NATS Cluster Endpoints:\"\n    echo \"  Node 1: nats://localhost:4222 (HTTP: http://localhost:8222)\"\n    echo \"  Node 2: nats://localhost:4223 (HTTP: http://localhost:8223)\"\n    echo \"  Node 3: nats://localhost:4224 (HTTP: http://localhost:8224)\"\n    echo \"\"\n    echo \"Monitoring:\"\n    echo \"  NATS Surveyor: http://localhost:7777\"\n    echo \"  Prometheus: http://localhost:9090\"\n    echo \"  Grafana: http://localhost:3000 (admin/admin)\"\n    echo \"\"\n    echo \"Credentials:\"\n    echo \"  Admin: ${CREDS_DIR}/claude_admin.creds\"\n    echo \"  Service: ${CREDS_DIR}/claude_service.creds\"\n    echo \"  Readonly: ${CREDS_DIR}/claude_readonly.creds\"\n    echo \"\"\n    echo \"Test connection:\"\n    echo \"  nats --server=nats://localhost:4222 --creds=${CREDS_DIR}/claude_admin.creds server ping\"\n    echo \"\"\n}\n\n# Main execution\nmain() {\n    log \"Starting CIM Claude NATS cluster setup...\"\n    \n    check_prerequisites\n    generate_certificates\n    generate_nsc_credentials\n    setup_monitoring\n    start_cluster\n    \n    # Wait a moment for cluster stabilization\n    sleep 5\n    \n    initialize_streams\n    initialize_kv_stores\n    initialize_object_store\n    create_consumers\n    \n    show_cluster_info\n    \n    log \"CIM Claude NATS cluster setup completed successfully!\"\n}\n\n# Execute main function\nmain \"$@\""