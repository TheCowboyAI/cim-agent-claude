#!/bin/bash

# CIM Claude NATS Cluster Management Script
# Operations: start, stop, restart, scale, backup, restore, migrate

set -euo pipefail

# Configuration
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CREDS_DIR="${PROJECT_DIR}/creds"
BACKUP_DIR="${PROJECT_DIR}/backups"
COMPOSE_FILE="${PROJECT_DIR}/docker-compose.yaml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO: $1${NC}"
}

# Show usage
show_usage() {
    echo "CIM Claude NATS Cluster Management"
    echo ""
    echo "Usage: $0 COMMAND [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  start               Start the NATS cluster"
    echo "  stop                Stop the NATS cluster"
    echo "  restart             Restart the NATS cluster"
    echo "  status              Show cluster status"
    echo "  logs [service]      Show logs (optionally for specific service)"
    echo "  scale NODE_COUNT    Scale cluster to specified number of nodes"
    echo "  backup [TYPE]       Backup cluster data (jetstream, kv, all)"
    echo "  restore BACKUP_ID   Restore from backup"
    echo "  migrate SOURCE      Migrate data from another cluster"
    echo "  cleanup             Clean up old data and logs"
    echo "  maintenance on|off  Enable/disable maintenance mode"
    echo "  drain NODE          Drain a specific node"
    echo "  health              Run comprehensive health check"
    echo ""
    echo "Options:"
    echo "  --force             Force operation without confirmation"
    echo "  --timeout SECONDS   Set operation timeout (default: 30)"
    echo "  --verbose           Enable verbose output"
    echo ""
    echo "Examples:"
    echo "  $0 start"
    echo "  $0 status"
    echo "  $0 logs nats-node-1"
    echo "  $0 backup jetstream"
    echo "  $0 scale 5"
    echo "  $0 maintenance on"
}

# Parse command line arguments
parse_args() {
    COMMAND=""
    FORCE=false
    TIMEOUT=30
    VERBOSE=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            start|stop|restart|status|logs|scale|backup|restore|migrate|cleanup|maintenance|drain|health)
                COMMAND="$1"
                shift
                ;;
            --force)
                FORCE=true
                shift
                ;;
            --timeout)
                TIMEOUT="$2"
                shift 2
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --help|-h)
                show_usage
                exit 0
                ;;
            *)
                if [[ -z "$COMMAND" ]]; then
                    error "Unknown command: $1. Use --help for usage."
                fi
                break
                ;;
        esac
    done
    
    if [[ -z "$COMMAND" ]]; then
        show_usage
        exit 1
    fi
    
    # Store remaining arguments for command-specific processing
    COMMAND_ARGS=("$@")
}

# Check prerequisites
check_prerequisites() {
    if [[ ! -f "$COMPOSE_FILE" ]]; then
        error "Docker Compose file not found: $COMPOSE_FILE"
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        error "Docker Compose is not installed or not in PATH"
    fi
    
    if [[ "$COMMAND" =~ ^(backup|restore|migrate)$ ]] && ! command -v nats &> /dev/null; then
        error "NATS CLI is required for $COMMAND operations"
    fi
}

# Wait for services to be ready
wait_for_services() {
    local timeout="$1"
    local services=("${@:2}")
    
    log "Waiting for services to be ready (timeout: ${timeout}s)..."
    
    local start_time=$(date +%s)
    local ready_services=0
    
    while [[ $ready_services -lt ${#services[@]} ]]; do
        ready_services=0
        
        for service in "${services[@]}"; do
            if docker-compose -f "$COMPOSE_FILE" ps "$service" | grep -q "Up.*healthy"; then
                ((ready_services++))
            fi
        done
        
        local current_time=$(date +%s)
        if [[ $((current_time - start_time)) -gt $timeout ]]; then
            error "Timeout waiting for services to be ready"
        fi
        
        if [[ $ready_services -lt ${#services[@]} ]]; then
            sleep 2
        fi
    done
    
    log "All services are ready"
}

# Start cluster
start_cluster() {
    log "Starting CIM Claude NATS cluster..."
    
    cd "$PROJECT_DIR"
    
    # Pull latest images
    docker-compose pull
    
    # Start services
    docker-compose up -d
    
    # Wait for NATS nodes to be ready
    wait_for_services "$TIMEOUT" nats-node-1 nats-node-2 nats-node-3
    
    # Wait a bit more for cluster formation
    sleep 5
    
    log "NATS cluster started successfully"
    
    # Show status
    show_status
}

# Stop cluster
stop_cluster() {
    log "Stopping CIM Claude NATS cluster..."
    
    cd "$PROJECT_DIR"
    
    if [[ "$FORCE" == true ]]; then
        docker-compose down --timeout 10
    else
        # Graceful shutdown
        log "Performing graceful shutdown..."
        docker-compose stop --timeout "$TIMEOUT"
        docker-compose down
    fi
    
    log "NATS cluster stopped successfully"
}

# Restart cluster
restart_cluster() {
    log "Restarting CIM Claude NATS cluster..."
    
    stop_cluster
    sleep 2
    start_cluster
    
    log "NATS cluster restarted successfully"
}

# Show cluster status
show_status() {
    log "Getting CIM Claude NATS cluster status..."
    
    cd "$PROJECT_DIR"
    
    echo ""
    echo "=== Docker Container Status ==="
    docker-compose ps
    
    echo ""
    echo "=== NATS Cluster Information ==="
    
    local creds="${CREDS_DIR}/claude_admin.creds"
    if [[ -f "$creds" ]]; then
        for port in 4222 4223 4224; do\n            local node_name=\"Node $(((port-4221)))\"\n            echo \"$node_name (localhost:$port):\"\n            \n            if timeout 5 nats --server=\"nats://localhost:$port\" --creds=\"$creds\" server info --json >/dev/null 2>&1; then\n                local server_info=$(nats --server=\"nats://localhost:$port\" --creds=\"$creds\" server info --json 2>/dev/null)\n                local server_name=$(echo \"$server_info\" | jq -r '.server_name // \"unknown\"')\n                local version=$(echo \"$server_info\" | jq -r '.version // \"unknown\"')\n                local connections=$(echo \"$server_info\" | jq -r '.connections // 0')\n                local subscriptions=$(echo \"$server_info\" | jq -r '.subscriptions // 0')\n                \n                echo \"  Server: $server_name (v$version)\"\n                echo \"  Connections: $connections\"\n                echo \"  Subscriptions: $subscriptions\"\n                \n                # JetStream info\n                local js_enabled=$(echo \"$server_info\" | jq -r '.jetstream // false')\n                if [[ \"$js_enabled\" == \"true\" ]]; then\n                    local js_stats=$(echo \"$server_info\" | jq -r '.jetstream_stats // {}')\n                    local memory_used=$(echo \"$js_stats\" | jq -r '.memory_used // 0')\n                    local storage_used=$(echo \"$js_stats\" | jq -r '.storage_used // 0')\n                    echo \"  JetStream: enabled (mem: $memory_used, storage: $storage_used)\"\n                else\n                    echo \"  JetStream: disabled\"\n                fi\n            else\n                echo \"  Status: not accessible\"\n            fi\n            echo \"\"\n        done\n        \n        # Cluster-wide statistics\n        echo \"=== Cluster-wide Statistics ===\"\n        if timeout 5 nats --server=\"nats://localhost:4222\" --creds=\"$creds\" server info --json >/dev/null 2>&1; then\n            echo \"Streams:\"\n            nats --server=\"nats://localhost:4222\" --creds=\"$creds\" stream list 2>/dev/null || echo \"  Unable to list streams\"\n            \n            echo \"\"\n            echo \"KV Stores:\"\n            nats --server=\"nats://localhost:4222\" --creds=\"$creds\" kv list 2>/dev/null || echo \"  Unable to list KV stores\"\n            \n            echo \"\"\n            echo \"Object Stores:\"\n            nats --server=\"nats://localhost:4222\" --creds=\"$creds\" object list 2>/dev/null || echo \"  Unable to list object stores\"\n        fi\n    else\n        warn \"Admin credentials not found. Limited status information available.\"\n    fi\n    \n    echo \"\"\n    echo \"=== Monitoring Endpoints ===\"\n    echo \"NATS Monitoring:  http://localhost:8222 http://localhost:8223 http://localhost:8224\"\n    echo \"NATS Surveyor:    http://localhost:7777\"\n    echo \"Prometheus:       http://localhost:9090\"\n    echo \"Grafana:          http://localhost:3000\"\n}\n\n# Show logs\nshow_logs() {\n    local service=\"${COMMAND_ARGS[0]:-}\"\n    \n    cd \"$PROJECT_DIR\"\n    \n    if [[ -n \"$service\" ]]; then\n        log \"Showing logs for service: $service\"\n        docker-compose logs -f \"$service\"\n    else\n        log \"Showing logs for all services\"\n        docker-compose logs -f\n    fi\n}\n\n# Scale cluster\nscale_cluster() {\n    local target_nodes=\"${COMMAND_ARGS[0]:-}\"\n    \n    if [[ -z \"$target_nodes\" ]] || ! [[ \"$target_nodes\" =~ ^[0-9]+$ ]]; then\n        error \"Please specify the target number of nodes (e.g., scale 5)\"\n    fi\n    \n    if [[ \"$target_nodes\" -lt 3 ]]; then\n        error \"Minimum cluster size is 3 nodes for high availability\"\n    fi\n    \n    if [[ \"$target_nodes\" -gt 7 ]]; then\n        error \"Maximum supported cluster size is 7 nodes\"\n    fi\n    \n    log \"Scaling cluster to $target_nodes nodes...\"\n    warn \"Cluster scaling is not yet implemented in this version\"\n    warn \"Current cluster size is fixed at 3 nodes\"\n    \n    # TODO: Implement dynamic scaling\n    # This would involve:\n    # 1. Generating new node configurations\n    # 2. Updating docker-compose.yaml\n    # 3. Starting new nodes\n    # 4. Updating cluster routes\n    # 5. Rebalancing data\n}\n\n# Backup cluster data\nbackup_cluster() {\n    local backup_type=\"${COMMAND_ARGS[0]:-all}\"\n    local timestamp=$(date +%Y%m%d_%H%M%S)\n    local backup_id=\"cim_claude_backup_${timestamp}\"\n    \n    mkdir -p \"$BACKUP_DIR\"\n    \n    log \"Creating backup: $backup_id (type: $backup_type)\"\n    \n    local backup_path=\"${BACKUP_DIR}/${backup_id}\"\n    mkdir -p \"$backup_path\"\n    \n    local creds=\"${CREDS_DIR}/claude_admin.creds\"\n    if [[ ! -f \"$creds\" ]]; then\n        error \"Admin credentials not found: $creds\"\n    fi\n    \n    case \"$backup_type\" in\n        jetstream|all)\n            log \"Backing up JetStream data...\"\n            mkdir -p \"${backup_path}/jetstream\"\n            \n            # Backup streams\n            local streams=$(nats --server=\"nats://localhost:4222\" --creds=\"$creds\" stream list --json 2>/dev/null | jq -r '.streams[].config.name' || echo \"\")\n            for stream in $streams; do\n                if [[ -n \"$stream\" ]]; then\n                    log \"Backing up stream: $stream\"\n                    nats --server=\"nats://localhost:4222\" --creds=\"$creds\" stream backup \"$stream\" \"${backup_path}/jetstream/${stream}.backup\" 2>/dev/null || warn \"Failed to backup stream $stream\"\n                fi\n            done\n            ;;\n    esac\n    \n    case \"$backup_type\" in\n        kv|all)\n            log \"Backing up KV stores...\"\n            mkdir -p \"${backup_path}/kv\"\n            \n            # Backup KV stores\n            local kvs=$(nats --server=\"nats://localhost:4222\" --creds=\"$creds\" kv list --json 2>/dev/null | jq -r '.[].bucket' || echo \"\")\n            for kv in $kvs; do\n                if [[ -n \"$kv\" ]]; then\n                    log \"Backing up KV store: $kv\"\n                    # Export all keys from KV store\n                    nats --server=\"nats://localhost:4222\" --creds=\"$creds\" kv get \"$kv\" --history=all > \"${backup_path}/kv/${kv}.json\" 2>/dev/null || warn \"Failed to backup KV store $kv\"\n                fi\n            done\n            ;;\n    esac\n    \n    case \"$backup_type\" in\n        config|all)\n            log \"Backing up configuration...\"\n            mkdir -p \"${backup_path}/config\"\n            \n            # Copy configuration files\n            cp -r \"${PROJECT_DIR}\"/*.conf \"${backup_path}/config/\" 2>/dev/null || warn \"No configuration files to backup\"\n            cp -r \"${PROJECT_DIR}\"/*.yaml \"${backup_path}/config/\" 2>/dev/null || warn \"No YAML files to backup\"\n            cp -r \"$CREDS_DIR\" \"${backup_path}/\" 2>/dev/null || warn \"No credentials to backup\"\n            ;;\n    esac\n    \n    # Create backup metadata\n    cat > \"${backup_path}/metadata.json\" << EOF\n{\n  \"backup_id\": \"$backup_id\",\n  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\n  \"type\": \"$backup_type\",\n  \"cluster_size\": 3,\n  \"version\": \"1.0\",\n  \"created_by\": \"$(whoami)@$(hostname)\"\n}\nEOF\n    \n    # Compress backup\n    log \"Compressing backup...\"\n    cd \"$BACKUP_DIR\"\n    tar -czf \"${backup_id}.tar.gz\" \"$backup_id\"/\n    rm -rf \"$backup_id\"\n    \n    log \"Backup created successfully: ${BACKUP_DIR}/${backup_id}.tar.gz\"\n    \n    # Cleanup old backups (keep last 10)\n    local backup_count=$(ls -1 \"$BACKUP_DIR\"/*.tar.gz 2>/dev/null | wc -l)\n    if [[ $backup_count -gt 10 ]]; then\n        log \"Cleaning up old backups (keeping last 10)...\"\n        ls -1t \"$BACKUP_DIR\"/*.tar.gz | tail -n +11 | xargs rm -f\n    fi\n}\n\n# Restore from backup\nrestore_cluster() {\n    local backup_id=\"${COMMAND_ARGS[0]:-}\"\n    \n    if [[ -z \"$backup_id\" ]]; then\n        error \"Please specify backup ID to restore from\"\n    fi\n    \n    local backup_file=\"${BACKUP_DIR}/${backup_id}.tar.gz\"\n    if [[ ! -f \"$backup_file\" ]]; then\n        error \"Backup file not found: $backup_file\"\n    fi\n    \n    if [[ \"$FORCE\" != true ]]; then\n        read -p \"This will restore from backup and may overwrite existing data. Continue? [y/N] \" -n 1 -r\n        echo\n        if [[ ! $REPLY =~ ^[Yy]$ ]]; then\n            log \"Restore cancelled\"\n            exit 0\n        fi\n    fi\n    \n    log \"Restoring from backup: $backup_id\"\n    \n    # Extract backup\n    local temp_dir=\"/tmp/cim_restore_$$\"\n    mkdir -p \"$temp_dir\"\n    cd \"$temp_dir\"\n    tar -xzf \"$backup_file\"\n    \n    local backup_path=\"${temp_dir}/${backup_id}\"\n    \n    # Verify backup metadata\n    if [[ -f \"${backup_path}/metadata.json\" ]]; then\n        local backup_type=$(jq -r '.type' \"${backup_path}/metadata.json\")\n        local backup_timestamp=$(jq -r '.timestamp' \"${backup_path}/metadata.json\")\n        log \"Backup type: $backup_type, created: $backup_timestamp\"\n    else\n        warn \"Backup metadata not found. Proceeding with caution.\"\n    fi\n    \n    # TODO: Implement actual restore logic\n    warn \"Restore functionality is not yet fully implemented\"\n    warn \"This is a placeholder for restore operations\"\n    \n    # Cleanup\n    rm -rf \"$temp_dir\"\n    \n    log \"Restore completed\"\n}\n\n# Migrate data from another cluster\nmigrate_cluster() {\n    local source=\"${COMMAND_ARGS[0]:-}\"\n    \n    if [[ -z \"$source\" ]]; then\n        error \"Please specify source cluster URL (e.g., nats://source-cluster:4222)\"\n    fi\n    \n    log \"Migrating data from: $source\"\n    warn \"Migration functionality is not yet implemented\"\n    warn \"This would involve connecting to source cluster and transferring streams, KV data, etc.\"\n    \n    # TODO: Implement migration logic\n    # This would involve:\n    # 1. Connecting to source cluster\n    # 2. Enumerating all streams, KV stores, object stores\n    # 3. Creating equivalent structures in target cluster\n    # 4. Copying data with proper consistency guarantees\n    # 5. Verifying data integrity\n}\n\n# Cleanup old data\ncleanup_cluster() {\n    log \"Cleaning up old data and logs...\"\n    \n    if [[ \"$FORCE\" != true ]]; then\n        read -p \"This will remove old logs and temporary data. Continue? [y/N] \" -n 1 -r\n        echo\n        if [[ ! $REPLY =~ ^[Yy]$ ]]; then\n            log \"Cleanup cancelled\"\n            exit 0\n        fi\n    fi\n    \n    # Clean Docker volumes and unused images\n    log \"Cleaning Docker resources...\"\n    docker system prune -f\n    \n    # Clean old logs\n    if [[ -d \"${PROJECT_DIR}/logs\" ]]; then\n        log \"Cleaning old log files...\"\n        find \"${PROJECT_DIR}/logs\" -name \"*.log\" -mtime +30 -delete 2>/dev/null || true\n    fi\n    \n    # Clean old backups (keep last 5)\n    if [[ -d \"$BACKUP_DIR\" ]]; then\n        log \"Cleaning old backups (keeping last 5)...\"\n        ls -1t \"$BACKUP_DIR\"/*.tar.gz 2>/dev/null | tail -n +6 | xargs rm -f 2>/dev/null || true\n    fi\n    \n    log \"Cleanup completed\"\n}\n\n# Enable/disable maintenance mode\nmaintenance_mode() {\n    local mode=\"${COMMAND_ARGS[0]:-}\"\n    \n    if [[ \"$mode\" != \"on\" && \"$mode\" != \"off\" ]]; then\n        error \"Please specify 'on' or 'off' for maintenance mode\"\n    fi\n    \n    log \"Setting maintenance mode: $mode\"\n    \n    # TODO: Implement maintenance mode\n    # This would involve:\n    # 1. Creating a maintenance mode flag in KV store\n    # 2. Updating load balancer configuration\n    # 3. Draining connections gracefully\n    # 4. Preventing new connections (for 'on' mode)\n    \n    warn \"Maintenance mode functionality is not yet implemented\"\n}\n\n# Drain a specific node\ndrain_node() {\n    local node=\"${COMMAND_ARGS[0]:-}\"\n    \n    if [[ -z \"$node\" ]]; then\n        error \"Please specify node to drain (e.g., nats-node-1)\"\n    fi\n    \n    log \"Draining node: $node\"\n    \n    # TODO: Implement node draining\n    # This would involve:\n    # 1. Stopping new connections to the node\n    # 2. Waiting for existing operations to complete\n    # 3. Moving stream leadership to other nodes\n    # 4. Gracefully shutting down the node\n    \n    warn \"Node draining functionality is not yet implemented\"\n}\n\n# Run health check\nrun_health_check() {\n    local health_script=\"${PROJECT_DIR}/scripts/health-check.sh\"\n    \n    if [[ -f \"$health_script\" ]]; then\n        log \"Running comprehensive health check...\"\n        \"$health_script\"\n    else\n        error \"Health check script not found: $health_script\"\n    fi\n}\n\n# Main execution\nmain() {\n    parse_args \"$@\"\n    check_prerequisites\n    \n    case \"$COMMAND\" in\n        start)\n            start_cluster\n            ;;\n        stop)\n            stop_cluster\n            ;;\n        restart)\n            restart_cluster\n            ;;\n        status)\n            show_status\n            ;;\n        logs)\n            show_logs\n            ;;\n        scale)\n            scale_cluster\n            ;;\n        backup)\n            backup_cluster\n            ;;\n        restore)\n            restore_cluster\n            ;;\n        migrate)\n            migrate_cluster\n            ;;\n        cleanup)\n            cleanup_cluster\n            ;;\n        maintenance)\n            maintenance_mode\n            ;;\n        drain)\n            drain_node\n            ;;\n        health)\n            run_health_check\n            ;;\n        *)\n            error \"Unknown command: $COMMAND\"\n            ;;\n    esac\n}\n\n# Execute main function with all arguments\nmain \"$@\""