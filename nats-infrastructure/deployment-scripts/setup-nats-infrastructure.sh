#!/bin/bash

# NATS Infrastructure Setup Script for Claude API Adapter
# Comprehensive deployment script for production-ready NATS infrastructure

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NATS_CONFIG_DIR="${SCRIPT_DIR}/../"
NATS_DATA_DIR="/data/nats"
NATS_LOGS_DIR="/var/log/nats"
NATS_CREDS_DIR="/etc/nats/creds"
NATS_KEYS_DIR="/etc/nats/keys"
BACKUP_DIR="/backup/nats"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
    exit 1
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if running as root or with sudo
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root or with sudo"
    fi
    
    # Check required commands
    local required_commands=("nats" "nsc" "docker" "curl" "jq" "yq")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            error "Required command '$cmd' is not installed"
        fi
    done
    
    # Check NATS server
    if ! command -v nats-server &> /dev/null; then
        warning "nats-server not found in PATH, will install via Docker"
    fi
    
    success "Prerequisites check completed"
}

# Create directory structure
create_directories() {
    log "Creating directory structure..."
    
    local directories=(
        "$NATS_DATA_DIR"
        "$NATS_DATA_DIR/jetstream"
        "$NATS_LOGS_DIR"
        "$NATS_CREDS_DIR"
        "$NATS_KEYS_DIR"
        "$BACKUP_DIR"
        "$BACKUP_DIR/streams"
        "$BACKUP_DIR/kv"
        "$BACKUP_DIR/configs"
    )
    
    for dir in "${directories[@]}"; do
        if [[ ! -d "$dir" ]]; then
            mkdir -p "$dir"
            log "Created directory: $dir"
        fi
    done
    
    # Set proper permissions
    chown -R nats:nats "$NATS_DATA_DIR" "$NATS_LOGS_DIR" 2>/dev/null || true
    chmod 750 "$NATS_CREDS_DIR" "$NATS_KEYS_DIR"
    
    success "Directory structure created"
}

# Setup NSC (NATS Security Configuration)
setup_nsc() {
    log "Setting up NSC (NATS Security Configuration)..."
    
    # Initialize NSC environment
    export NSC_HOME="$NATS_KEYS_DIR"
    
    # Create operator
    if ! nsc list operators | grep -q "CIM_CLAUDE_OPERATOR"; then
        log "Creating NATS operator..."
        nsc add operator CIM_CLAUDE_OPERATOR
        success "Created operator: CIM_CLAUDE_OPERATOR"
    else
        log "Operator CIM_CLAUDE_OPERATOR already exists"
    fi
    
    # Create system account
    if ! nsc list accounts -A | grep -q "SYS"; then
        log "Creating system account..."
        nsc add account SYS --operator CIM_CLAUDE_OPERATOR
        success "Created system account: SYS"
    fi
    
    # Create Claude service account
    if ! nsc list accounts -A | grep -q "CLAUDE_SERVICE"; then
        log "Creating Claude service account..."
        nsc add account CLAUDE_SERVICE --operator CIM_CLAUDE_OPERATOR
        
        # Configure JetStream for Claude service account
        nsc edit account CLAUDE_SERVICE \
            --js-mem-storage 2G \
            --js-disk-storage 20G \
            --js-streams 20 \
            --js-consumers 100
            
        success "Created Claude service account with JetStream enabled"
    fi
    
    # Create monitoring account
    if ! nsc list accounts -A | grep -q "MONITORING"; then
        log "Creating monitoring account..."
        nsc add account MONITORING --operator CIM_CLAUDE_OPERATOR
        success "Created monitoring account: MONITORING"
    fi
    
    # Create audit account
    if ! nsc list accounts -A | grep -q "AUDIT"; then
        log "Creating audit account..."
        nsc add account AUDIT --operator CIM_CLAUDE_OPERATOR
        success "Created audit account: AUDIT"
    fi
    
    # Create API gateway account
    if ! nsc list accounts -A | grep -q "API_GATEWAY"; then
        log "Creating API gateway account..."
        nsc add account API_GATEWAY --operator CIM_CLAUDE_OPERATOR
        success "Created API gateway account: API_GATEWAY"
    fi
    
    success "NSC setup completed"
}

# Create users and credentials
create_users() {
    log "Creating users and credentials..."
    
    # Claude service users
    create_user_if_not_exists "CLAUDE_SERVICE" "claude-service-primary" \
        --allow-pub "claude.event.>,claude.resp.>,claude.monitor.>,_INBOX.>" \
        --allow-sub "claude.cmd.>,claude.internal.>,_INBOX.>"
        
    create_user_if_not_exists "CLAUDE_SERVICE" "claude-service-worker" \
        --allow-pub "claude.event.>,claude.resp.>,_INBOX.>" \
        --allow-sub "claude.cmd.>,_INBOX.>"
        
    create_user_if_not_exists "CLAUDE_SERVICE" "claude-database-service" \
        --allow-pub "claude.event.stored,db.>,_INBOX.>" \
        --allow-sub "claude.event.>,db.query.>,_INBOX.>"
    
    # API Gateway users
    create_user_if_not_exists "API_GATEWAY" "api-gateway-service" \
        --allow-pub "claude.cmd.>,gateway.>,_INBOX.>" \
        --allow-sub "claude.resp.>,claude.event.*.started,claude.event.*.ended,_INBOX.>" \
        --deny-pub "claude.cmd.system.>" \
        --deny-sub "claude.internal.>"
        
    # Monitoring users
    create_user_if_not_exists "MONITORING" "prometheus-collector" \
        --allow-pub "metrics.>,_INBOX.>" \
        --allow-sub "claude.event.>,claude.monitor.>,\$SYS.>,_INBOX.>"
        
    create_user_if_not_exists "MONITORING" "alert-manager" \
        --allow-pub "alerts.>,notifications.>,_INBOX.>" \
        --allow-sub "metrics.>,claude.monitor.>,_INBOX.>"
    
    # Audit users
    create_user_if_not_exists "AUDIT" "audit-service" \
        --allow-pub "audit.>,compliance.>,_INBOX.>" \
        --allow-sub "claude.event.>,\$SYS.ACCOUNT.>,_INBOX.>"
    
    success "User creation completed"
}

# Helper function to create user if not exists
create_user_if_not_exists() {
    local account="$1"
    local user="$2"
    shift 2
    local args=("$@")
    
    if ! nsc list users -A "$account" | grep -q "$user"; then
        log "Creating user: $user in account: $account"
        nsc add user "$user" --account "$account" "${args[@]}"
        
        # Generate credentials file
        nsc generate creds -A "$account" -n "$user" > "$NATS_CREDS_DIR/${account,,}-${user}.creds"
        chmod 600 "$NATS_CREDS_DIR/${account,,}-${user}.creds"
        
        success "Created user: $user with credentials"
    else
        log "User $user already exists in account $account"
    fi
}

# Generate NATS server configuration
generate_server_config() {
    log "Generating NATS server configuration..."
    
    cat > "$NATS_CONFIG_DIR/nats-server.conf" << EOF
# NATS Server Configuration for Claude API Adapter
server_name: "claude-nats-server"
port: 4222
http_port: 8222

# Logging
log_file: "$NATS_LOGS_DIR/nats-server.log"
log_size_limit: 100MB
max_traced_msg_len: 32768
logtime: true

# JetStream Configuration
jetstream {
    store_dir: "$NATS_DATA_DIR/jetstream"
    max_memory_store: 2GB
    max_file_store: 20GB
}

# TLS Configuration
tls {
    cert_file: "/etc/nats/server-cert.pem"
    key_file: "/etc/nats/server-key.pem"
    ca_file: "/etc/nats/ca-cert.pem"
    verify: true
    timeout: 5
}

# Security - JWT Authentication
authorization {
    resolver: MEMORY
    include "$NATS_KEYS_DIR/accounts/nsc_accounts.conf"
    system_account: "$(nsc describe account SYS --field sub 2>/dev/null || echo 'SYS')"
}

# Connection Limits
max_connections: 1000
max_subscriptions: 10000
max_payload: 10MB
ping_interval: 2m
ping_max: 2
write_deadline: 10s

# Clustering (if needed)
# cluster {
#     name: "claude-cluster"
#     listen: 0.0.0.0:6222
#     routes = [
#         nats-route://nats-2:6222
#         nats-route://nats-3:6222
#     ]
# }

# Monitoring
http: 8222
server_tags: ["claude", "jetstream", "production"]

# Performance Tuning
max_pending: 67108864  # 64MB
max_control_line: 4096
max_connections: 1000
EOF

    success "NATS server configuration generated"
}

# Setup JetStream streams
setup_streams() {
    log "Setting up JetStream streams..."
    
    # Wait for NATS server to be ready
    wait_for_nats
    
    # Create command stream
    create_stream_if_not_exists "CLAUDE_COMMANDS" \
        --subjects="claude.cmd.*.start,claude.cmd.*.prompt,claude.cmd.*.end,claude.cmd.*.cancel" \
        --storage=file \
        --retention=workqueue \
        --max-msgs=100000 \
        --max-age=24h \
        --max-msg-size=1MB \
        --duplicate-window=2m
    
    # Create events stream
    create_stream_if_not_exists "CLAUDE_EVENTS" \
        --subjects="claude.event.*.started,claude.event.*.prompt_sent,claude.event.*.ended,claude.event.*.error,claude.event.*.conversation_updated" \
        --storage=file \
        --retention=limits \
        --max-msgs=1000000 \
        --max-age=720h \
        --max-msg-size=1MB \
        --duplicate-window=5m \
        --deny-delete \
        --deny-purge
    
    # Create responses stream
    create_stream_if_not_exists "CLAUDE_RESPONSES" \
        --subjects="claude.resp.*.content,claude.resp.*.streaming,claude.resp.*.complete,claude.resp.*.error" \
        --storage=file \
        --retention=interest \
        --max-msgs=50000 \
        --max-age=1h \
        --max-msg-size=10MB \
        --duplicate-window=1m
    
    # Create monitoring stream
    create_stream_if_not_exists "CLAUDE_MONITORING" \
        --subjects="claude.monitor.health.>,claude.monitor.metrics.>,claude.monitor.trace.>" \
        --storage=file \
        --retention=limits \
        --max-msgs=100000 \
        --max-age=168h \
        --max-msg-size=512KB \
        --duplicate-window=30s
    
    success "JetStream streams setup completed"
}

# Helper function to create stream if not exists
create_stream_if_not_exists() {
    local stream_name="$1"
    shift
    local args=("$@")
    
    if ! nats stream info "$stream_name" &>/dev/null; then
        log "Creating stream: $stream_name"
        nats stream add "$stream_name" "${args[@]}" --defaults
        success "Created stream: $stream_name"
    else
        log "Stream $stream_name already exists"
    fi
}

# Setup consumers
setup_consumers() {
    log "Setting up JetStream consumers..."
    
    # Command processor consumer
    create_consumer_if_not_exists "CLAUDE_COMMANDS" "claude-cmd-processor-v1" \
        --deliver=new \
        --ack=explicit \
        --wait=30s \
        --max-deliver=3 \
        --filter="claude.cmd.>" \
        --replay=instant \
        --sample=10%
    
    # Response distributor consumer
    create_consumer_if_not_exists "CLAUDE_RESPONSES" "claude-resp-dist-v1" \
        --deliver=all \
        --ack=explicit \
        --wait=15s \
        --max-deliver=2 \
        --filter="claude.resp.>" \
        --replay=instant \
        --sample=5%
    
    # Event processor consumer
    create_consumer_if_not_exists "CLAUDE_EVENTS" "claude-event-proc-v1" \
        --deliver=all \
        --ack=explicit \
        --wait=60s \
        --max-deliver=5 \
        --filter="claude.event.>" \
        --replay=original \
        --sample=100%
    
    # Monitoring consumer
    create_consumer_if_not_exists "CLAUDE_MONITORING" "claude-monitor-v1" \
        --deliver=new \
        --ack=explicit \
        --wait=10s \
        --max-deliver=1 \
        --filter="claude.monitor.>" \
        --replay=instant \
        --sample=1%
    
    success "JetStream consumers setup completed"
}

# Helper function to create consumer if not exists
create_consumer_if_not_exists() {
    local stream_name="$1"
    local consumer_name="$2"
    shift 2
    local args=("$@")
    
    if ! nats consumer info "$stream_name" "$consumer_name" &>/dev/null; then
        log "Creating consumer: $consumer_name for stream: $stream_name"
        nats consumer add "$stream_name" "$consumer_name" "${args[@]}"
        success "Created consumer: $consumer_name"
    else
        log "Consumer $consumer_name already exists for stream $stream_name"
    fi
}

# Setup KV stores
setup_kv_stores() {
    log "Setting up KV stores..."
    
    # Session state KV store
    create_kv_if_not_exists "CLAUDE_SESSIONS" \
        --description="Active conversation sessions with state and context" \
        --max-value-size=1MB \
        --history=10 \
        --ttl=24h \
        --storage=file \
        --replicas=1
    
    # Conversation aggregate KV store
    create_kv_if_not_exists "CLAUDE_CONVERSATIONS" \
        --description="Conversation aggregate state and business data" \
        --max-value-size=512KB \
        --history=20 \
        --ttl=720h \
        --storage=file \
        --replicas=1
    
    # Rate limiting KV store
    create_kv_if_not_exists "CLAUDE_RATE_LIMITS" \
        --description="Rate limiting counters and quota tracking" \
        --max-value-size=4KB \
        --history=5 \
        --ttl=1h \
        --storage=memory \
        --replicas=1
    
    # Circuit breaker KV store
    create_kv_if_not_exists "CLAUDE_CIRCUIT_BREAKERS" \
        --description="Circuit breaker states for external API reliability" \
        --max-value-size=8KB \
        --history=3 \
        --ttl=30m \
        --storage=file \
        --replicas=1
    
    # Configuration KV store
    create_kv_if_not_exists "CLAUDE_CONFIG" \
        --description="Dynamic configuration and feature flags" \
        --max-value-size=64KB \
        --history=50 \
        --storage=file \
        --replicas=1
    
    success "KV stores setup completed"
}

# Helper function to create KV store if not exists
create_kv_if_not_exists() {
    local kv_name="$1"
    shift
    local args=("$@")
    
    if ! nats kv status "$kv_name" &>/dev/null; then
        log "Creating KV store: $kv_name"
        nats kv add "$kv_name" "${args[@]}"
        success "Created KV store: $kv_name"
    else
        log "KV store $kv_name already exists"
    fi
}

# Wait for NATS server to be ready
wait_for_nats() {
    log "Waiting for NATS server to be ready..."
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if nats server check connection &>/dev/null; then
            success "NATS server is ready"
            return 0
        fi
        
        log "Waiting for NATS server... (attempt $attempt/$max_attempts)"
        sleep 2
        ((attempt++))
    done
    
    error "NATS server failed to start within expected time"
}

# Setup monitoring
setup_monitoring() {
    log "Setting up monitoring configuration..."
    
    # Copy monitoring configuration files
    if [[ -f "$NATS_CONFIG_DIR/monitoring-config.yml" ]]; then
        cp "$NATS_CONFIG_DIR/monitoring-config.yml" /etc/nats/monitoring.yml
        log "Copied monitoring configuration"
    fi
    
    # Setup Prometheus configuration for NATS metrics
    cat > /etc/prometheus/nats-targets.yml << EOF
- targets:
  - 'nats:8222'
  labels:
    job: 'nats-server'
    environment: 'production'
    service: 'claude-adapter'
EOF
    
    success "Monitoring setup completed"
}

# Setup backup configuration
setup_backup() {
    log "Setting up backup configuration..."
    
    # Create backup script
    cat > "$BACKUP_DIR/backup-nats.sh" << 'EOF'
#!/bin/bash
# NATS Backup Script

set -euo pipefail

BACKUP_DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_BASE_DIR="/backup/nats"
BACKUP_DIR="$BACKUP_BASE_DIR/$BACKUP_DATE"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup JetStream streams
echo "Backing up JetStream streams..."
for stream in $(nats stream list --json | jq -r '.streams[].config.name'); do
    echo "Backing up stream: $stream"
    nats stream backup "$stream" "$BACKUP_DIR/stream_${stream}.tar.gz"
done

# Backup KV stores
echo "Backing up KV stores..."
for kv in $(nats kv list --json | jq -r '.[].bucket'); do
    echo "Backing up KV store: $kv"
    nats kv status "$kv" --json > "$BACKUP_DIR/kv_${kv}_status.json"
done

# Backup NSC configuration
echo "Backing up NSC configuration..."
tar -czf "$BACKUP_DIR/nsc_config.tar.gz" -C /etc/nats/keys .

# Cleanup old backups (keep last 7 days)
find "$BACKUP_BASE_DIR" -name "20*" -type d -mtime +7 -exec rm -rf {} \;

echo "Backup completed: $BACKUP_DIR"
EOF
    
    chmod +x "$BACKUP_DIR/backup-nats.sh"
    
    # Setup cron job for daily backups
    (crontab -l 2>/dev/null; echo "0 2 * * * $BACKUP_DIR/backup-nats.sh") | crontab -
    
    success "Backup configuration completed"
}

# Validate deployment
validate_deployment() {
    log "Validating NATS deployment..."
    
    local validation_errors=0
    
    # Check NATS server connectivity
    if ! nats server check connection &>/dev/null; then
        error "NATS server connection failed"
        ((validation_errors++))
    fi
    
    # Check JetStream
    if ! nats server check jetstream &>/dev/null; then
        error "JetStream is not available"
        ((validation_errors++))
    fi
    
    # Validate streams
    local expected_streams=("CLAUDE_COMMANDS" "CLAUDE_EVENTS" "CLAUDE_RESPONSES" "CLAUDE_MONITORING")
    for stream in "${expected_streams[@]}"; do
        if ! nats stream info "$stream" &>/dev/null; then
            error "Stream $stream is not available"
            ((validation_errors++))
        fi
    done
    
    # Validate KV stores
    local expected_kvs=("CLAUDE_SESSIONS" "CLAUDE_CONVERSATIONS" "CLAUDE_RATE_LIMITS" "CLAUDE_CIRCUIT_BREAKERS" "CLAUDE_CONFIG")
    for kv in "${expected_kvs[@]}"; do
        if ! nats kv status "$kv" &>/dev/null; then
            error "KV store $kv is not available"
            ((validation_errors++))
        fi
    done
    
    # Test basic pub/sub
    local test_subject="claude.test.validation"
    local test_message="validation test"
    
    # Start subscriber in background
    timeout 10s nats sub "$test_subject" > /tmp/nats_test_output &
    local sub_pid=$!
    sleep 2
    
    # Publish test message
    echo "$test_message" | nats pub "$test_subject" --stdin
    sleep 2
    
    # Kill subscriber and check output
    kill $sub_pid 2>/dev/null || true
    
    if grep -q "$test_message" /tmp/nats_test_output 2>/dev/null; then
        success "Basic pub/sub test passed"
    else
        error "Basic pub/sub test failed"
        ((validation_errors++))
    fi
    
    rm -f /tmp/nats_test_output
    
    if [[ $validation_errors -eq 0 ]]; then
        success "All validation tests passed"
        return 0
    else
        error "Validation failed with $validation_errors errors"
        return 1
    fi
}

# Display deployment summary
show_summary() {
    log "NATS Infrastructure Deployment Summary"
    echo "======================================"
    echo
    echo "Configuration Files:"
    echo "  - NATS Server Config: $NATS_CONFIG_DIR/nats-server.conf"
    echo "  - NSC Keys Directory: $NATS_KEYS_DIR"
    echo "  - Credentials Directory: $NATS_CREDS_DIR"
    echo
    echo "Data Directories:"
    echo "  - JetStream Data: $NATS_DATA_DIR/jetstream"
    echo "  - Logs: $NATS_LOGS_DIR"
    echo "  - Backups: $BACKUP_DIR"
    echo
    echo "Streams Created:"
    nats stream list 2>/dev/null || echo "  Unable to list streams"
    echo
    echo "KV Stores Created:"
    nats kv list 2>/dev/null || echo "  Unable to list KV stores"
    echo
    echo "Monitoring:"
    echo "  - HTTP Port: 8222"
    echo "  - Metrics Endpoint: http://localhost:8222/varz"
    echo "  - Health Endpoint: http://localhost:8222/healthz"
    echo
    echo "Next Steps:"
    echo "  1. Start NATS server: nats-server -c $NATS_CONFIG_DIR/nats-server.conf"
    echo "  2. Configure monitoring dashboards"
    echo "  3. Deploy Claude API adapter application"
    echo "  4. Test end-to-end functionality"
    echo
    success "NATS infrastructure deployment completed successfully!"
}

# Main execution
main() {
    log "Starting NATS infrastructure setup for Claude API Adapter..."
    echo "============================================================="
    
    check_prerequisites
    create_directories
    setup_nsc
    create_users
    generate_server_config
    
    # Start NATS server if not running
    if ! pgrep -f "nats-server" > /dev/null; then
        log "Starting NATS server..."
        nats-server -c "$NATS_CONFIG_DIR/nats-server.conf" -D &
        sleep 5
    fi
    
    setup_streams
    setup_consumers
    setup_kv_stores
    setup_monitoring
    setup_backup
    validate_deployment
    show_summary
    
    success "NATS infrastructure setup completed successfully!"
}

# Cleanup function for script interruption
cleanup() {
    log "Cleaning up..."
    # Add any cleanup code here
    exit 1
}

# Trap signals for cleanup
trap cleanup SIGINT SIGTERM

# Run main function
main "$@"