# CIM Git Repository Domain Integration

## Overview

This document provides instructions for extending the existing NATS infrastructure (in `nats-infrastructure/`) to support CIM-specific git repository domains. Each CIM is fundamentally a git repository with its own domain structure, IPLD integration, and event sourcing patterns.

**Important**: This extends our existing NATS infrastructure - do NOT create new streams or duplicate existing configuration.

## CIM Domain Architecture

### Git Repository as CIM Identity

Every CIM is a git repository where:
- **Repository URL** determines domain authority and permissions
- **Configuration flake** (`flake.nix`) serves as declarative authority source  
- **Git commits** become IPLD CIDs for content-addressed event correlation
- **Domain isolation** through repository-specific subject hierarchies

### Subject Algebra Extension

Extend our existing subject hierarchy with CIM-specific patterns:

```yaml
# Existing: claude.{category}.{session_id}.{operation}
# CIM Extension: cim.{org}.{repo}.{category}.{entity}.{operation}

cim_subject_patterns:
  # Git Integration Subjects
  git_events:
    pattern: "cim.{org}.{repo}.git.{event_type}.{commit_hash}"
    examples:
      - "cim.acme-corp.order-management.git.commit.abc123def"
      - "cim.acme-corp.order-management.git.branch.created.feature-xyz"
      - "cim.acme-corp.order-management.git.merge.completed.pr-456"
  
  # Configuration Flake Subjects
  config_events:
    pattern: "cim.{org}.{repo}.config.{change_type}.{flake_hash}"
    examples:
      - "cim.acme-corp.order-management.config.flake.updated.nix-789abc"
      - "cim.acme-corp.order-management.config.permissions.changed.auth-def456"
  
  # Domain Event Subjects (extends existing claude.event.*)
  domain_events:
    pattern: "cim.{org}.{repo}.domain.{aggregate}.{event_type}.{event_id}"
    examples:
      - "cim.acme-corp.order-management.domain.order.placed.evt-123"
      - "cim.acme-corp.order-management.domain.payment.processed.evt-456"
  
  # IPLD Object Subjects
  ipld_objects:
    pattern: "cim.{org}.{repo}.ipld.{operation}.{cid}"
    examples:
      - "cim.acme-corp.order-management.ipld.put.bafybeigdyrzt5sf"
      - "cim.acme-corp.order-management.ipld.pin.bafkreiabcd1234"
```

## Extending Existing NATS Infrastructure

### 1. Additional Stream Templates

Add to existing `nats-infrastructure/jetstream-config.yml`:

```yaml
# CIM Domain Stream Templates (ADD to existing stream_templates)
stream_templates:
  CIM_GIT_EVENTS_TEMPLATE:
    name: "CIM_GIT_{{.org}}_{{.repo}}_EVENTS"
    subjects:
      - "cim.{{.org}}.{{.repo}}.git.>"
    storage: "file"
    retention: "limits"
    max_msgs: 100000
    max_age: "2160h"        # 90 days
    max_bytes: 5368709120   # 5GB
    duplicate_window: "120s"
    replicas: 1
    
  CIM_DOMAIN_EVENTS_TEMPLATE:
    name: "CIM_DOMAIN_{{.org}}_{{.repo}}_EVENTS"
    subjects:
      - "cim.{{.org}}.{{.repo}}.domain.>"
      - "cim.{{.org}}.{{.repo}}.events.>"
    storage: "file"
    retention: "limits"
    max_msgs: 1000000
    max_age: "8760h"        # 1 year
    max_bytes: 21474836480  # 20GB
    duplicate_window: "300s"
    replicas: 1
    
  CIM_IPLD_OBJECTS_TEMPLATE:
    name: "CIM_IPLD_{{.org}}_{{.repo}}_OBJECTS"
    subjects:
      - "cim.{{.org}}.{{.repo}}.ipld.>"
    storage: "file"
    retention: "workqueue"
    max_age: "17520h"       # 2 years
    max_bytes: 10737418240  # 10GB
    replicas: 1
```

### 2. Additional KV Store Templates

Add to existing `nats-infrastructure/kv-store-config.yml`:

```yaml
# CIM Domain KV Store Templates (ADD to existing kv_stores)
kv_store_templates:
  CIM_METADATA_TEMPLATE:
    name: "CIM_METADATA_{{.org}}_{{.repo}}"
    description: "CIM domain metadata and authority information"
    max_value_size: 65536   # 64KB
    history: 20
    ttl: "17520h"           # 2 years
    storage: "file"
    replicas: 1
    
  CIM_IPLD_INDEX_TEMPLATE:
    name: "CIM_IPLD_INDEX_{{.org}}_{{.repo}}"
    description: "IPLD CID to git commit mappings"
    max_value_size: 4096    # 4KB per mapping
    history: 100
    ttl: "17520h"           # 2 years
    storage: "file"
    replicas: 1
    
  CIM_STATE_TEMPLATE:
    name: "CIM_STATE_{{.org}}_{{.repo}}"
    description: "Current CIM domain state"
    max_value_size: 524288  # 512KB
    history: 10
    ttl: "720h"             # 30 days
    storage: "file"
    replicas: 1
```

### 3. CIM Account Extension

Add to existing `nats-infrastructure/security-config.yml`:

```yaml
# CIM Domain Account (ADD to existing accounts)
accounts:
  CIM_DOMAINS:
    description: "Account for CIM domain operations with git integration"
    signing_keys:
      - "ACIM123..."
      
    # JetStream for CIM domains
    jetstream:
      memory_storage: 1073741824     # 1GB memory
      disk_storage: 107374182400     # 100GB disk
      streams: 50                    # Support many CIM domains
      consumers: 500
      
    limits:
      connections: 50
      subscriptions: 500
      data: 107374182400             # 100GB total
      payload: 67108864              # 64MB (for large IPLD objects)
      
    # CIM Domain Subjects
    subjects:
      publish:
        allow:
          - "cim.>.git.>"            # Git events
          - "cim.>.config.>"         # Configuration events
          - "cim.>.domain.>"         # Domain events
          - "cim.>.ipld.>"           # IPLD operations
        deny: []
      subscribe:
        allow:
          - "cim.>.>"                # Subscribe to own domain
          - "claude.event.>"         # Access to Claude events
        deny:
          - "claude.cmd.>"           # No command access
```

## Git Integration Implementation

### 1. Git Hook Templates

Create `git-hooks/cim-pre-commit`:

```bash
#!/bin/bash
# CIM Pre-commit Hook - Generate git events
set -e

# Extract CIM identity from git config
REPO_URL=$(git config --get remote.origin.url)
if [[ -z "$REPO_URL" ]]; then
    echo "Warning: No git remote origin configured"
    exit 0
fi

# Parse organization and repository name
if [[ "$REPO_URL" =~ github\.com[:/]([^/]+)/([^/]+)(\.git)?$ ]]; then
    ORG_NAME="${BASH_REMATCH[1]}"
    REPO_NAME="${BASH_REMATCH[2]%.git}"
elif [[ "$REPO_URL" =~ ([^/]+)/([^/]+)\.git$ ]]; then
    ORG_NAME="${BASH_REMATCH[1]}"
    REPO_NAME="${BASH_REMATCH[2]}"
else
    echo "Warning: Cannot parse git remote origin: $REPO_URL"
    exit 0
fi

# Generate pre-commit event
COMMIT_HASH=$(git rev-parse HEAD)
BRANCH=$(git branch --show-current)
TIMESTAMP=$(date -Iseconds)
CORRELATION_ID=$(uuidgen)

# Check if NATS is available
if ! command -v nats &> /dev/null; then
    echo "Warning: NATS CLI not available, skipping event publication"
    exit 0
fi

# Publish pre-commit event
SUBJECT="cim.${ORG_NAME}.${REPO_NAME}.git.pre-commit.${COMMIT_HASH}"
EVENT_JSON=$(cat <<EOF
{
    "event_id": "${CORRELATION_ID}",
    "event_type": "GitPreCommit",
    "repository": "${ORG_NAME}/${REPO_NAME}",
    "commit_hash": "${COMMIT_HASH}",
    "branch": "${BRANCH}",
    "timestamp": "${TIMESTAMP}",
    "staged_files": $(git diff --cached --name-only | jq -R . | jq -s .),
    "correlation_id": "${CORRELATION_ID}"
}
EOF
)

# Publish to NATS (with error handling)
echo "$EVENT_JSON" | nats pub "$SUBJECT" --stdin 2>/dev/null || {
    echo "Warning: Failed to publish pre-commit event to NATS"
}

echo "✅ CIM pre-commit event published"
```

### 2. Post-commit Hook with IPLD Integration

Create `git-hooks/cim-post-commit`:

```bash
#!/bin/bash
# CIM Post-commit Hook - IPLD integration and event correlation
set -e

# Extract CIM identity (same as pre-commit)
REPO_URL=$(git config --get remote.origin.url)
if [[ -z "$REPO_URL" ]]; then exit 0; fi

if [[ "$REPO_URL" =~ github\.com[:/]([^/]+)/([^/]+)(\.git)?$ ]]; then
    ORG_NAME="${BASH_REMATCH[1]}"
    REPO_NAME="${BASH_REMATCH[2]%.git}"
else
    exit 0
fi

# Commit details
COMMIT_HASH=$(git rev-parse HEAD)
COMMIT_MESSAGE=$(git show -s --format=%s HEAD)
COMMIT_AUTHOR=$(git show -s --format=%an HEAD)
BRANCH=$(git branch --show-current)
TIMESTAMP=$(date -Iseconds)

# Generate IPLD CID (simplified - in production use proper IPLD tools)
IPLD_CID="bafybei$(echo "$COMMIT_HASH" | head -c 40 | sha256sum | head -c 32)"

# Store git commit metadata in NATS KV
if command -v nats &> /dev/null; then
    # Update metadata store
    nats kv put "CIM_METADATA_${ORG_NAME}_${REPO_NAME}" "latest_commit" "$COMMIT_HASH" || true
    
    # Update IPLD index
    nats kv put "CIM_IPLD_INDEX_${ORG_NAME}_${REPO_NAME}" "commit.$COMMIT_HASH" "$IPLD_CID" || true
    
    # Publish post-commit event with IPLD correlation
    SUBJECT="cim.${ORG_NAME}.${REPO_NAME}.git.commit.${BRANCH}.${COMMIT_HASH}"
    EVENT_JSON=$(cat <<EOF
{
    "event_id": "$(uuidgen)",
    "event_type": "GitCommit",
    "repository": "${ORG_NAME}/${REPO_NAME}",
    "commit_hash": "${COMMIT_HASH}",
    "branch": "${BRANCH}",
    "message": "$COMMIT_MESSAGE",
    "author": "$COMMIT_AUTHOR",
    "timestamp": "$TIMESTAMP",
    "ipld_cid": "$IPLD_CID",
    "correlation_id": "commit-$COMMIT_HASH",
    "modified_files": $(git diff-tree --name-only HEAD^ HEAD | jq -R . | jq -s .)
}
EOF
    )
    
    echo "$EVENT_JSON" | nats pub "$SUBJECT" --stdin || echo "Warning: NATS publish failed"
fi

echo "✅ CIM post-commit with IPLD correlation completed"
```

### 3. Configuration Flake Monitor

Create `scripts/monitor-flake-changes.sh`:

```bash
#!/bin/bash
# Monitor flake.nix changes and publish configuration events
set -e

REPO_URL=$(git config --get remote.origin.url)
if [[ "$REPO_URL" =~ github\.com[:/]([^/]+)/([^/]+)(\.git)?$ ]]; then
    ORG_NAME="${BASH_REMATCH[1]}"
    REPO_NAME="${BASH_REMATCH[2]%.git}"
else
    echo "Error: Cannot parse repository URL"
    exit 1
fi

# Monitor flake.nix for changes
monitor_flake() {
    local old_hash="$1"
    local new_hash="$2"
    
    if [[ "$old_hash" != "$new_hash" ]]; then
        echo "🔧 Configuration flake changed: $old_hash -> $new_hash"
        
        # Generate configuration change event
        SUBJECT="cim.${ORG_NAME}.${REPO_NAME}.config.flake.updated.${new_hash}"
        EVENT_JSON=$(cat <<EOF
{
    "event_id": "$(uuidgen)",
    "event_type": "ConfigurationFlakeUpdated",
    "repository": "${ORG_NAME}/${REPO_NAME}",
    "old_authority_hash": "$old_hash",
    "new_authority_hash": "$new_hash",
    "timestamp": "$(date -Iseconds)",
    "requires_permission_sync": true,
    "flake_evaluation": $(nix eval --json .#authorityConfig 2>/dev/null || echo 'null')
}
EOF
        )
        
        if command -v nats &> /dev/null; then
            echo "$EVENT_JSON" | nats pub "$SUBJECT" --stdin || echo "Warning: NATS publish failed"
            
            # Update authority hash in metadata
            nats kv put "CIM_METADATA_${ORG_NAME}_${REPO_NAME}" "authority_hash" "$new_hash" || true
        fi
        
        echo "✅ Configuration change event published"
    fi
}

# Watch for changes
if command -v inotifywait &> /dev/null; then
    echo "📡 Monitoring flake.nix for changes..."
    
    # Get initial hash
    CURRENT_HASH=$(nix eval --raw .#authorityHash 2>/dev/null || echo "unknown")
    
    # Watch for file changes
    inotifywait -m flake.nix -e modify | while read path action file; do
        if [[ "$file" == "flake.nix" ]]; then
            sleep 1  # Wait for write to complete
            NEW_HASH=$(nix eval --raw .#authorityHash 2>/dev/null || echo "unknown")
            monitor_flake "$CURRENT_HASH" "$NEW_HASH"
            CURRENT_HASH="$NEW_HASH"
        fi
    done
else
    echo "Warning: inotifywait not available, use polling mode"
    
    # Fallback to polling
    LAST_HASH=$(nix eval --raw .#authorityHash 2>/dev/null || echo "unknown")
    while true; do
        sleep 10
        CURRENT_HASH=$(nix eval --raw .#authorityHash 2>/dev/null || echo "unknown")
        monitor_flake "$LAST_HASH" "$CURRENT_HASH"
        LAST_HASH="$CURRENT_HASH"
    done
fi
```

## CIM Domain Setup Script

Create `scripts/setup-cim-domain.sh`:

```bash
#!/bin/bash
# Setup CIM domain with git integration
set -e

CIM_NAME="${1:-dev-demo}"
ORG_NAME="${2:-local}"
ADMIN_EMAIL="${3:-admin@local.dev}"

if [[ -z "$1" ]]; then
    echo "Usage: $0 <cim-name> [org-name] [admin-email]"
    echo "Example: $0 order-management acme-corp admin@acme.com"
    exit 1
fi

echo "🚀 Setting up CIM domain: $ORG_NAME/$CIM_NAME"

# 1. Create NATS streams for this CIM domain
create_cim_streams() {
    echo "📊 Creating NATS streams..."
    
    # Create git events stream
    nats stream add "CIM_GIT_${ORG_NAME^^}_${CIM_NAME^^}_EVENTS" \
        --subjects="cim.${ORG_NAME}.${CIM_NAME}.git.>" \
        --storage=file --retention=limits \
        --max-age=2160h --max-msgs=100000 \
        --max-bytes=5368709120 || echo "Stream may already exist"
    
    # Create domain events stream  
    nats stream add "CIM_DOMAIN_${ORG_NAME^^}_${CIM_NAME^^}_EVENTS" \
        --subjects="cim.${ORG_NAME}.${CIM_NAME}.domain.>,cim.${ORG_NAME}.${CIM_NAME}.events.>" \
        --storage=file --retention=limits \
        --max-age=8760h --max-msgs=1000000 \
        --max-bytes=21474836480 || echo "Stream may already exist"
    
    # Create IPLD objects stream
    nats stream add "CIM_IPLD_${ORG_NAME^^}_${CIM_NAME^^}_OBJECTS" \
        --subjects="cim.${ORG_NAME}.${CIM_NAME}.ipld.>" \
        --storage=file --retention=workqueue \
        --max-age=17520h --max-bytes=10737418240 || echo "Stream may already exist"
}

# 2. Create KV stores for this CIM domain
create_cim_kv_stores() {
    echo "🗃️ Creating KV stores..."
    
    # Metadata store
    nats kv add "CIM_METADATA_${ORG_NAME^^}_${CIM_NAME^^}" \
        --description="CIM domain metadata and authority" \
        --max-value-size=65536 --history=20 \
        --ttl=17520h || echo "KV store may already exist"
    
    # IPLD index store
    nats kv add "CIM_IPLD_INDEX_${ORG_NAME^^}_${CIM_NAME^^}" \
        --description="IPLD CID to git commit mappings" \
        --max-value-size=4096 --history=100 \
        --ttl=17520h || echo "KV store may already exist"
    
    # State store
    nats kv add "CIM_STATE_${ORG_NAME^^}_${CIM_NAME^^}" \
        --description="Current CIM domain state" \
        --max-value-size=524288 --history=10 \
        --ttl=720h || echo "KV store may already exist"
}

# 3. Initialize CIM metadata
initialize_cim_metadata() {
    echo "⚙️ Initializing CIM metadata..."
    
    # Create domain manifest
    DOMAIN_MANIFEST=$(cat <<EOF
{
    "domain": {
        "name": "$CIM_NAME",
        "organization": "$ORG_NAME", 
        "administrator": "$ADMIN_EMAIL",
        "created_at": "$(date -Iseconds)",
        "repository": "https://github.com/${ORG_NAME}/cim-${CIM_NAME}.git"
    },
    "authority": {
        "hash": "initial",
        "source": "flake.nix"
    },
    "nats": {
        "streams": [
            "CIM_GIT_${ORG_NAME^^}_${CIM_NAME^^}_EVENTS",
            "CIM_DOMAIN_${ORG_NAME^^}_${CIM_NAME^^}_EVENTS",
            "CIM_IPLD_${ORG_NAME^^}_${CIM_NAME^^}_OBJECTS"
        ],
        "kv_stores": [
            "CIM_METADATA_${ORG_NAME^^}_${CIM_NAME^^}",
            "CIM_IPLD_INDEX_${ORG_NAME^^}_${CIM_NAME^^}",
            "CIM_STATE_${ORG_NAME^^}_${CIM_NAME^^}"
        ]
    }
}
EOF
    )
    
    # Store in KV
    echo "$DOMAIN_MANIFEST" | nats kv put "CIM_METADATA_${ORG_NAME^^}_${CIM_NAME^^}" "domain_manifest" --stdin
    
    echo "✅ CIM metadata initialized"
}

# 4. Install git hooks
install_git_hooks() {
    echo "🔗 Installing git hooks..."
    
    if [[ ! -d ".git" ]]; then
        echo "Warning: Not in a git repository, skipping git hooks"
        return
    fi
    
    # Copy hook templates and make executable
    cp git-hooks/cim-pre-commit .git/hooks/pre-commit
    cp git-hooks/cim-post-commit .git/hooks/post-commit
    chmod +x .git/hooks/pre-commit .git/hooks/post-commit
    
    echo "✅ Git hooks installed"
}

# Execute setup steps
echo "🔄 Executing CIM domain setup..."

create_cim_streams
create_cim_kv_stores  
initialize_cim_metadata
install_git_hooks

echo ""
echo "✅ CIM domain setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Create flake.nix with authority configuration"
echo "2. Start flake monitoring: ./scripts/monitor-flake-changes.sh"
echo "3. Test git integration: git commit -m 'test: CIM integration'"
echo "4. Monitor events: nats sub 'cim.$ORG_NAME.$CIM_NAME.>'"
echo ""
echo "🎯 Your CIM domain is ready for git-integrated development!"
```

## Integration with Existing Infrastructure

### 1. Extend Monitoring

Add to existing `nats-infrastructure/monitoring-config.yml`:

```yaml
# CIM Domain Monitoring (ADD to existing monitoring)
monitoring:
  cim_domains:
    - name: "cim_git_events_rate"
      description: "Rate of git events per CIM domain"
      query: 'rate(nats_stream_messages_total{stream_name=~"CIM_GIT_.*"}[5m])'
      
    - name: "cim_domain_health"
      description: "Health of CIM domain streams"
      query: 'nats_stream_state{stream_name=~"CIM_.*"} == 1'
      
    - name: "cim_ipld_object_storage"
      description: "IPLD object storage usage per domain"
      query: 'nats_stream_bytes{stream_name=~"CIM_IPLD_.*"}'
```

### 2. Backup Integration

Add to existing backup procedures:

```bash
# Add to existing backup script in nats-infrastructure/
backup_cim_domains() {
    echo "Backing up CIM domains..."
    
    # Backup all CIM streams
    for stream in $(nats stream list --json | jq -r '.streams[].config.name' | grep "^CIM_"); do
        nats stream backup "$stream" "$BACKUP_DIR/cim_stream_${stream}.tar.gz"
    done
    
    # Backup CIM KV stores  
    for kv in $(nats kv list --json | jq -r '.[].bucket' | grep "^CIM_"); do
        nats kv export "$kv" > "$BACKUP_DIR/cim_kv_${kv}.json"
    done
}
```

## Usage Examples

### 1. Setting up a new CIM domain
```bash
# Setup new order management CIM
./scripts/setup-cim-domain.sh order-management acme-corp admin@acme.com

# Create git repository structure
mkdir -p cim-order-management/{.cim,src/domain,tests}
cd cim-order-management
git init
git remote add origin https://github.com/acme-corp/cim-order-management.git
```

### 2. Monitoring CIM events
```bash
# Monitor all git events for a specific CIM
nats sub "cim.acme-corp.order-management.git.>"

# Monitor domain events
nats sub "cim.acme-corp.order-management.domain.>"

# Monitor IPLD operations
nats sub "cim.acme-corp.order-management.ipld.>"
```

### 3. Querying CIM metadata
```bash
# Get domain manifest
nats kv get "CIM_METADATA_ACME_CORP_ORDER_MANAGEMENT" "domain_manifest"

# Get latest commit
nats kv get "CIM_METADATA_ACME_CORP_ORDER_MANAGEMENT" "latest_commit"

# Get IPLD CID for commit
nats kv get "CIM_IPLD_INDEX_ACME_CORP_ORDER_MANAGEMENT" "commit.abc123def"
```

This approach extends our existing comprehensive NATS infrastructure with CIM-specific capabilities while avoiding duplication and maintaining consistency with the established patterns.