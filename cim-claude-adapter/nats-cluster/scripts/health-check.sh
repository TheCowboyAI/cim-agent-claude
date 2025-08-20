#!/bin/bash

# CIM Claude NATS Cluster Health Check Script
# Comprehensive monitoring and health verification

set -euo pipefail

# Configuration
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CREDS_DIR="${PROJECT_DIR}/creds"
HEALTH_LOG="${PROJECT_DIR}/logs/health-check.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Health check results
HEALTH_CHECKS=()
WARNINGS=()
ERRORS=()

log() {
    local message="[$(date +'%Y-%m-%d %H:%M:%S')] $1"
    echo -e "${GREEN}${message}${NC}"
    echo "${message}" >> "${HEALTH_LOG}" 2>/dev/null || true
}

warn() {
    local message="[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1"
    echo -e "${YELLOW}${message}${NC}"
    echo "${message}" >> "${HEALTH_LOG}" 2>/dev/null || true
    WARNINGS+=("$1")
}

error() {
    local message="[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1"
    echo -e "${RED}${message}${NC}"
    echo "${message}" >> "${HEALTH_LOG}" 2>/dev/null || true
    ERRORS+=("$1")
}

info() {
    local message="[$(date +'%Y-%m-%d %H:%M:%S')] INFO: $1"
    echo -e "${BLUE}${message}${NC}"
    echo "${message}" >> "${HEALTH_LOG}" 2>/dev/null || true
}

# Initialize logging
init_logging() {
    mkdir -p "$(dirname "${HEALTH_LOG}")"
    echo "=== CIM Claude NATS Health Check - $(date) ===" >> "${HEALTH_LOG}"
}

# Check Docker containers
check_containers() {
    log "Checking Docker containers..."
    
    local containers=("cim-nats-1" "cim-nats-2" "cim-nats-3")
    local healthy_containers=0
    
    for container in "${containers[@]}"; do
        if docker ps --format "table {{.Names}}\t{{.Status}}" | grep -q "${container}.*Up"; then
            local status=$(docker inspect --format='{{.State.Health.Status}}' "${container}" 2>/dev/null || echo "unknown")
            if [[ "$status" == "healthy" ]]; then
                info "Container ${container}: healthy"
                ((healthy_containers++))
            elif [[ "$status" == "starting" ]]; then
                warn "Container ${container}: starting (health check in progress)"
            else
                warn "Container ${container}: $status"
            fi
        else
            error "Container ${container}: not running or not found"
        fi
    done
    
    HEALTH_CHECKS+=("Containers: $healthy_containers/3 healthy")
    
    if [[ $healthy_containers -eq 3 ]]; then
        log "All containers are healthy"
        return 0
    elif [[ $healthy_containers -ge 2 ]]; then
        warn "Cluster running with $healthy_containers/3 nodes (degraded)"
        return 1
    else
        error "Cluster critical: only $healthy_containers/3 nodes healthy"
        return 2
    fi
}

# Check NATS connectivity
check_nats_connectivity() {
    log "Checking NATS connectivity..."
    
    local servers=("localhost:4222" "localhost:4223" "localhost:4224")
    local connected_servers=0
    
    for server in "${servers[@]}"; do
        if timeout 5 nats --server="nats://${server}" server ping >/dev/null 2>&1; then
            info "NATS server ${server}: connected"
            ((connected_servers++))
        else
            error "NATS server ${server}: connection failed"
        fi
    done
    
    HEALTH_CHECKS+=("NATS Connectivity: $connected_servers/3 servers")
    
    if [[ $connected_servers -eq 3 ]]; then
        log "All NATS servers are accessible"
        return 0
    elif [[ $connected_servers -ge 2 ]]; then
        warn "NATS cluster running with $connected_servers/3 servers"
        return 1
    else
        error "NATS cluster critical: only $connected_servers/3 servers accessible"
        return 2
    fi
}

# Check cluster formation
check_cluster_formation() {
    log "Checking NATS cluster formation..."
    
    local creds="${CREDS_DIR}/claude_admin.creds"
    if [[ ! -f "$creds" ]]; then
        error "Admin credentials not found: $creds"
        return 2
    fi
    
    # Get cluster info from primary node
    local cluster_info
    if cluster_info=$(timeout 10 nats --server="nats://localhost:4222" --creds="$creds" server info --json 2>/dev/null); then
        local cluster_size=$(echo "$cluster_info" | jq -r '.cluster.size // 0')
        local cluster_leader=$(echo "$cluster_info" | jq -r '.cluster.leader // "unknown"')
        
        info "Cluster size: $cluster_size nodes"
        info "Cluster leader: $cluster_leader"
        
        HEALTH_CHECKS+=("Cluster Formation: $cluster_size nodes, leader: $cluster_leader")
        
        if [[ "$cluster_size" == "3" ]]; then
            log "Cluster formation is healthy"
            return 0
        elif [[ "$cluster_size" -ge "2" ]]; then
            warn "Cluster formation degraded: $cluster_size/3 nodes"
            return 1
        else
            error "Cluster formation critical: only $cluster_size nodes"
            return 2
        fi
    else
        error "Failed to get cluster information"
        return 2
    fi
}

# Check JetStream status
check_jetstream() {
    log "Checking JetStream status..."
    
    local creds="${CREDS_DIR}/claude_admin.creds"
    local js_info
    
    if js_info=$(timeout 10 nats --server="nats://localhost:4222" --creds="$creds" server info --json 2>/dev/null); then
        local js_enabled=$(echo "$js_info" | jq -r '.jetstream // false')\n        local js_stats=$(echo \"$js_info\" | jq -r '.jetstream_stats // {}')\n        \n        if [[ \"$js_enabled\" == \"true\" ]]; then\n            local memory_used=$(echo \"$js_stats\" | jq -r '.memory_used // 0')\n            local storage_used=$(echo \"$js_stats\" | jq -r '.storage_used // 0')\n            local api_requests=$(echo \"$js_stats\" | jq -r '.api_requests // 0')\n            \n            info \"JetStream enabled: memory=${memory_used}, storage=${storage_used}, requests=${api_requests}\"\n            HEALTH_CHECKS+=(\"JetStream: enabled, memory=${memory_used}, storage=${storage_used}\")\n            log \"JetStream is healthy\"\n            return 0\n        else\n            error \"JetStream is not enabled\"\n            return 2\n        fi\n    else\n        error \"Failed to get JetStream information\"\n        return 2\n    fi\n}\n\n# Check streams\ncheck_streams() {\n    log \"Checking JetStream streams...\"\n    \n    local creds=\"${CREDS_DIR}/claude_admin.creds\"\n    local expected_streams=(\"CIM_CLAUDE_CONV_CMD\" \"CIM_CLAUDE_CONV_EVT\" \"CIM_CLAUDE_CONV_RESP\" \"CIM_CLAUDE_TOOL_OPS\" \"CIM_CLAUDE_CONFIG\")\n    local healthy_streams=0\n    \n    for stream in \"${expected_streams[@]}\"; do\n        if timeout 5 nats --server=\"nats://localhost:4222\" --creds=\"$creds\" stream info \"$stream\" >/dev/null 2>&1; then\n            local stream_info=$(nats --server=\"nats://localhost:4222\" --creds=\"$creds\" stream info \"$stream\" --json 2>/dev/null)\n            local messages=$(echo \"$stream_info\" | jq -r '.state.messages // 0')\n            local bytes=$(echo \"$stream_info\" | jq -r '.state.bytes // 0')\n            local replicas=$(echo \"$stream_info\" | jq -r '.config.num_replicas // 1')\n            \n            info \"Stream $stream: $messages messages, $bytes bytes, $replicas replicas\"\n            ((healthy_streams++))\n        else\n            error \"Stream $stream: not found or not accessible\"\n        fi\n    done\n    \n    HEALTH_CHECKS+=(\"Streams: $healthy_streams/${#expected_streams[@]} healthy\")\n    \n    if [[ $healthy_streams -eq ${#expected_streams[@]} ]]; then\n        log \"All streams are healthy\"\n        return 0\n    else\n        error \"Some streams are missing or unhealthy\"\n        return 2\n    fi\n}\n\n# Check KV stores\ncheck_kv_stores() {\n    log \"Checking KV stores...\"\n    \n    local creds=\"${CREDS_DIR}/claude_admin.creds\"\n    local expected_kvs=(\"CIM_CLAUDE_CONV_META\" \"CIM_CLAUDE_SESSIONS\" \"CIM_CLAUDE_CONFIG\" \"CIM_CLAUDE_TOOL_STATE\" \"CIM_CLAUDE_RATE_LIMITS\")\n    local healthy_kvs=0\n    \n    for kv in \"${expected_kvs[@]}\"; do\n        if timeout 5 nats --server=\"nats://localhost:4222\" --creds=\"$creds\" kv status \"$kv\" >/dev/null 2>&1; then\n            local kv_info=$(nats --server=\"nats://localhost:4222\" --creds=\"$creds\" kv status \"$kv\" --json 2>/dev/null)\n            local entries=$(echo \"$kv_info\" | jq -r '.entries // 0')\n            local bytes=$(echo \"$kv_info\" | jq -r '.bytes // 0')\n            \n            info \"KV $kv: $entries entries, $bytes bytes\"\n            ((healthy_kvs++))\n        else\n            error \"KV store $kv: not found or not accessible\"\n        fi\n    done\n    \n    HEALTH_CHECKS+=(\"KV Stores: $healthy_kvs/${#expected_kvs[@]} healthy\")\n    \n    if [[ $healthy_kvs -eq ${#expected_kvs[@]} ]]; then\n        log \"All KV stores are healthy\"\n        return 0\n    else\n        error \"Some KV stores are missing or unhealthy\"\n        return 2\n    fi\n}\n\n# Check object store\ncheck_object_store() {\n    log \"Checking object store...\"\n    \n    local creds=\"${CREDS_DIR}/claude_admin.creds\"\n    \n    if timeout 5 nats --server=\"nats://localhost:4222\" --creds=\"$creds\" object info CIM_CLAUDE_ATTACHMENTS >/dev/null 2>&1; then\n        local os_info=$(nats --server=\"nats://localhost:4222\" --creds=\"$creds\" object info CIM_CLAUDE_ATTACHMENTS --json 2>/dev/null)\n        local size=$(echo \"$os_info\" | jq -r '.size // 0')\n        local objects=$(echo \"$os_info\" | jq -r '.objects // 0')\n        \n        info \"Object store CIM_CLAUDE_ATTACHMENTS: $objects objects, $size bytes\"\n        HEALTH_CHECKS+=(\"Object Store: healthy, $objects objects\")\n        log \"Object store is healthy\"\n        return 0\n    else\n        error \"Object store CIM_CLAUDE_ATTACHMENTS: not found or not accessible\"\n        return 2\n    fi\n}\n\n# Check consumers\ncheck_consumers() {\n    log \"Checking durable consumers...\"\n    \n    local creds=\"${CREDS_DIR}/claude_service.creds\"\n    local consumers=(\n        \"CIM_CLAUDE_CONV_CMD:command_processor\"\n        \"CIM_CLAUDE_TOOL_OPS:tool_processor\" \n        \"CIM_CLAUDE_CONFIG:config_processor\"\n    )\n    local healthy_consumers=0\n    \n    for consumer_info in \"${consumers[@]}\"; do\n        local stream=\"${consumer_info%:*}\"\n        local consumer=\"${consumer_info#*:}\"\n        \n        if timeout 5 nats --server=\"nats://localhost:4222\" --creds=\"$creds\" consumer info \"$stream\" \"$consumer\" >/dev/null 2>&1; then\n            local cons_info=$(nats --server=\"nats://localhost:4222\" --creds=\"$creds\" consumer info \"$stream\" \"$consumer\" --json 2>/dev/null)\n            local pending=$(echo \"$cons_info\" | jq -r '.num_pending // 0')\n            local delivered=$(echo \"$cons_info\" | jq -r '.delivered.consumer_seq // 0')\n            \n            info \"Consumer $consumer on $stream: $delivered delivered, $pending pending\"\n            ((healthy_consumers++))\n        else\n            error \"Consumer $consumer on stream $stream: not found or not accessible\"\n        fi\n    done\n    \n    HEALTH_CHECKS+=(\"Consumers: $healthy_consumers/${#consumers[@]} healthy\")\n    \n    if [[ $healthy_consumers -eq ${#consumers[@]} ]]; then\n        log \"All consumers are healthy\"\n        return 0\n    else\n        error \"Some consumers are missing or unhealthy\"\n        return 2\n    fi\n}\n\n# Check resource usage\ncheck_resource_usage() {\n    log \"Checking resource usage...\"\n    \n    # Check memory usage for containers\n    local containers=(\"cim-nats-1\" \"cim-nats-2\" \"cim-nats-3\")\n    \n    for container in \"${containers[@]}\"; do\n        if docker ps --format \"table {{.Names}}\" | grep -q \"$container\"; then\n            local stats=$(docker stats --no-stream --format \"table {{.Container}}\\t{{.CPUPerc}}\\t{{.MemUsage}}\" \"$container\" 2>/dev/null || echo \"$container\\tN/A\\tN/A\")\n            info \"$stats\"\n        fi\n    done\n    \n    # Check disk usage for JetStream stores\n    if command -v du >/dev/null 2>&1; then\n        local js_data_size=$(docker exec cim-nats-1 du -sh /opt/nats/jetstream 2>/dev/null | cut -f1 || echo \"unknown\")\n        info \"JetStream data size: $js_data_size\"\n        HEALTH_CHECKS+=(\"Resource Usage: JS data ${js_data_size}\")\n    fi\n    \n    return 0\n}\n\n# Perform connectivity test\nperform_connectivity_test() {\n    log \"Performing connectivity test...\"\n    \n    local creds=\"${CREDS_DIR}/claude_service.creds\"\n    local test_subject=\"claude.health.test.$(date +%s)\"\n    local test_message=\"Health check test message from $(hostname)\"\n    \n    # Publish test message\n    if timeout 5 nats --server=\"nats://localhost:4222\" --creds=\"$creds\" pub \"$test_subject\" \"$test_message\" >/dev/null 2>&1; then\n        info \"Test message published successfully\"\n        \n        # Try to subscribe and receive the message (with timeout)\n        if timeout 2 nats --server=\"nats://localhost:4222\" --creds=\"$creds\" sub \"claude.health.test.*\" --count=1 >/dev/null 2>&1; then\n            info \"Test message received successfully\"\n            HEALTH_CHECKS+=(\"Connectivity Test: passed\")\n            return 0\n        else\n            warn \"Test message published but not received\"\n            return 1\n        fi\n    else\n        error \"Failed to publish test message\"\n        return 2\n    fi\n}\n\n# Check monitoring endpoints\ncheck_monitoring() {\n    log \"Checking monitoring endpoints...\"\n    \n    local endpoints=(\n        \"http://localhost:8222/varz:NATS Node 1 Monitoring\"\n        \"http://localhost:8223/varz:NATS Node 2 Monitoring\"\n        \"http://localhost:8224/varz:NATS Node 3 Monitoring\"\n        \"http://localhost:7777:NATS Surveyor\"\n        \"http://localhost:9090:Prometheus\"\n        \"http://localhost:3000:Grafana\"\n    )\n    \n    local healthy_endpoints=0\n    \n    for endpoint_info in \"${endpoints[@]}\"; do\n        local endpoint=\"${endpoint_info%:*}\"\n        local name=\"${endpoint_info#*:}\"\n        \n        if timeout 5 curl -s -f \"$endpoint\" >/dev/null 2>&1; then\n            info \"$name: accessible\"\n            ((healthy_endpoints++))\n        else\n            warn \"$name: not accessible at $endpoint\"\n        fi\n    done\n    \n    HEALTH_CHECKS+=(\"Monitoring: $healthy_endpoints/${#endpoints[@]} endpoints accessible\")\n    \n    if [[ $healthy_endpoints -ge 3 ]]; then\n        log \"Essential monitoring endpoints are accessible\"\n        return 0\n    else\n        warn \"Some monitoring endpoints are not accessible\"\n        return 1\n    fi\n}\n\n# Generate health report\ngenerate_health_report() {\n    echo \"\"\n    echo \"===========================================\"\n    echo \"CIM Claude NATS Cluster Health Report\"\n    echo \"$(date)\"\n    echo \"===========================================\"\n    echo \"\"\n    \n    echo \"Health Check Results:\"\n    for check in \"${HEALTH_CHECKS[@]}\"; do\n        echo \"  ✓ $check\"\n    done\n    \n    if [[ ${#WARNINGS[@]} -gt 0 ]]; then\n        echo \"\"\n        echo \"Warnings:\"\n        for warning in \"${WARNINGS[@]}\"; do\n            echo \"  ⚠ $warning\"\n        done\n    fi\n    \n    if [[ ${#ERRORS[@]} -gt 0 ]]; then\n        echo \"\"\n        echo \"Errors:\"\n        for error in \"${ERRORS[@]}\"; do\n            echo \"  ✗ $error\"\n        done\n    fi\n    \n    echo \"\"\n    \n    # Overall health status\n    local overall_status\n    if [[ ${#ERRORS[@]} -eq 0 && ${#WARNINGS[@]} -eq 0 ]]; then\n        overall_status=\"HEALTHY\"\n        echo -e \"Overall Status: ${GREEN}${overall_status}${NC}\"\n        return 0\n    elif [[ ${#ERRORS[@]} -eq 0 ]]; then\n        overall_status=\"DEGRADED\"\n        echo -e \"Overall Status: ${YELLOW}${overall_status}${NC}\"\n        return 1\n    else\n        overall_status=\"CRITICAL\"\n        echo -e \"Overall Status: ${RED}${overall_status}${NC}\"\n        return 2\n    fi\n}\n\n# Main execution\nmain() {\n    init_logging\n    \n    log \"Starting CIM Claude NATS cluster health check...\"\n    \n    # Perform all health checks\n    check_containers\n    check_nats_connectivity  \n    check_cluster_formation\n    check_jetstream\n    check_streams\n    check_kv_stores\n    check_object_store\n    check_consumers\n    check_resource_usage\n    perform_connectivity_test\n    check_monitoring\n    \n    # Generate and display report\n    generate_health_report\n}\n\n# Execute main function\nmain \"$@\""