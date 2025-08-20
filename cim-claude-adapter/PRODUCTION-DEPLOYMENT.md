# CIM Claude Adapter - Production Deployment Guide

This comprehensive guide covers production deployment of the CIM Claude Adapter using Nix-based infrastructure, including high availability, security hardening, monitoring, and operational procedures.

## Architecture Overview

The CIM Claude Adapter production deployment implements a multi-tier architecture with comprehensive observability and security:

```
┌─────────────────────────────────────────────────────────────┐
│                      DMZ Zone (10.100.0.0/24)              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │    NGINX    │    │   HAProxy   │    │  Cloudflare │     │
│  │Load Balance │    │   Backup    │    │     CDN     │     │
│  │10.100.0.10  │    │10.100.0.11  │    │   External  │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
└─────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────┐
│                  Application Zone (10.101.0.0/24)          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │CIM Claude 01│    │CIM Claude 02│    │CIM Claude 03│     │
│  │   Master    │    │   Backup    │    │   Backup    │     │
│  │10.101.0.10  │    │10.101.0.11  │    │10.101.0.12  │     │
│  │   VIP: 10.101.0.100 (VRRP)     │    │             │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
└─────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────┐
│                    Data Zone (10.102.0.0/24)               │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  NATS-01    │    │  NATS-02    │    │  NATS-03    │     │
│  │ JetStream   │    │ JetStream   │    │ JetStream   │     │
│  │10.102.0.10  │    │10.102.0.11  │    │10.102.0.12  │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                           │                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │PostgreSQL-01│    │PostgreSQL-02│    │   Redis     │     │
│  │   Primary   │    │  Replica    │    │   Cache     │     │
│  │10.102.0.20  │    │10.102.0.21  │    │10.102.0.30  │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
└─────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────┐
│                 Management Zone (10.103.0.0/24)            │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │ Prometheus  │    │   Grafana   │    │   Loki      │     │
│  │10.103.0.10  │    │10.103.0.20  │    │10.103.0.30  │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Jaeger    │    │Alertmanager │    │  Backup     │     │
│  │10.103.0.40  │    │10.103.0.50  │    │ Storage     │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## Deployment Options

### 1. NixOS Deployment (Recommended)

#### Prerequisites

- NixOS 24.11 or later
- Minimum hardware requirements:
  - CPU: 8 cores (16 threads recommended)
  - RAM: 16GB (32GB recommended)
  - Storage: 500GB NVMe SSD (1TB recommended)
  - Network: Gigabit Ethernet with multiple interfaces

#### Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/TheCowboyAI/cim-claude-adapter.git
cd cim-claude-adapter

# 2. Use production flake
nix develop .#production

# 3. Generate hardware configuration
nixos-generate-config --root /mnt

# 4. Copy production template
cp -r templates/production/* .

# 5. Configure secrets (see Security section)
# Edit secrets/claude-api-key.age with your API key

# 6. Deploy to server
nixos-rebuild switch --flake .#cim-claude-prod-01 --target-host root@your-server

# 7. Verify deployment
nix run .#health-check-production
```

#### Complete Production Configuration

```nix
# flake.nix - Production deployment
{
  nixosConfigurations.cim-claude-prod-01 = nixpkgs.lib.nixosSystem {
    system = "x86_64-linux";
    modules = [
      ./hardware-configuration.nix
      
      # All CIM Claude modules
      cim-claude-adapter.nixosModules.cim-claude-adapter
      cim-claude-adapter.nixosModules.nats-infrastructure
      cim-claude-adapter.nixosModules.security-hardening
      cim-claude-adapter.nixosModules.monitoring
      cim-claude-adapter.nixosModules.backup-restore
      cim-claude-adapter.nixosModules.network-topology
      cim-claude-adapter.nixosModules.high-availability
      
      # Secrets management
      agenix.nixosModules.default
      
      # Production configuration
      {
        services.cim-claude-adapter = {
          enable = true;
          claude = {
            apiKey = "/run/secrets/claude-api-key";
            model = "claude-3-sonnet-20240229";
            maxTokens = 8192;
            temperature = 0.7;
          };
          nats = {
            enable = true;
            infrastructure = {
              enable = true;
              environment = "production";
              replication.replicas = 3;
            };
          };
        };
        
        # Security hardening
        services.cim-claude-security = {
          enable = true;
          tls.enable = true;
          auth.enable = true;
          compliance.enable = true;
        };
        
        # Full monitoring stack
        services.cim-claude-monitoring = {
          enable = true;
          prometheus.enable = true;
          grafana.enable = true;
          alertmanager.enable = true;
          jaeger.enable = true;
          loki.enable = true;
        };
        
        # High availability
        services.cim-claude-ha = {
          enable = true;
          thisNode = {
            name = "cim-claude-prod-01";
            ip = "10.101.0.10";
            priority = 150;
          };
          cluster.virtualIP = "10.101.0.100";
        };
      }
    ];
  };
}
```

### 2. Kubernetes Deployment

#### Prerequisites

- Kubernetes 1.28+
- Helm 3.0+
- Ingress controller (NGINX recommended)
- Certificate manager (cert-manager)
- Storage provisioner with SSD support
- Prometheus Operator (optional but recommended)

#### Deploy to Kubernetes

```bash
# 1. Create namespace and secrets
kubectl create namespace cim-claude
kubectl create secret generic claude-api-secret \
  --from-literal=api-key="your-claude-api-key" \
  -n cim-claude

# 2. Build and push container image
nix build .#container
docker load < result
docker tag cim-claude-adapter:v1.0.0 your-registry/cim-claude-adapter:v1.0.0
docker push your-registry/cim-claude-adapter:v1.0.0

# 3. Apply Kubernetes manifests
kubectl apply -f templates/kubernetes/deployment.yaml

# 4. Verify deployment
kubectl get pods -n cim-claude
kubectl get services -n cim-claude
kubectl get ingress -n cim-claude

# 5. Check health
kubectl port-forward service/cim-claude-adapter-service 8080:8080 -n cim-claude &
curl http://localhost:8080/health
```

#### Kubernetes Configuration Details

The Kubernetes deployment includes:

- **StatefulSet for NATS**: 3-node NATS cluster with JetStream persistence
- **Deployment for CIM Claude**: 3+ replicas with horizontal pod autoscaling
- **Services**: ClusterIP for internal communication
- **Ingress**: External access with SSL termination
- **NetworkPolicy**: Security isolation between components
- **PodDisruptionBudget**: Ensures availability during updates
- **ServiceMonitor**: Prometheus metrics collection
- **HPA**: Automatic scaling based on CPU/memory usage

### 3. Container Deployment

#### Build Container Image

```bash
# Build with Nix
nix build .#container

# Load into Docker
docker load < result

# Run container
docker run -d \
  --name cim-claude-adapter \
  -p 8080:8080 \
  -p 9090:9090 \
  -e CLAUDE_API_KEY="your-api-key" \
  -e NATS_URL="nats://nats-server:4222" \
  -e RUST_LOG="info" \
  cim-claude-adapter:v1.0.0
```

#### Docker Compose Deployment

```yaml
version: '3.8'

services:
  nats:
    image: nats:2.10-alpine
    command: [
      "nats-server", 
      "--jetstream", 
      "--store_dir", "/data/jetstream",
      "--port", "4222",
      "--monitor_port", "8222"
    ]
    ports:
      - "4222:4222"
      - "8222:8222"
    volumes:
      - nats_data:/data/jetstream
    restart: unless-stopped

  cim-claude-adapter:
    image: cim-claude-adapter:v1.0.0
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - CLAUDE_API_KEY=${CLAUDE_API_KEY}
      - NATS_URL=nats://nats:4222
      - RUST_LOG=info
    depends_on:
      - nats
    restart: unless-stopped
    deploy:
      replicas: 3
      resources:
        limits:
          memory: 2G
          cpus: '1.0'
        reservations:
          memory: 512M
          cpus: '0.25'

  prometheus:
    image: prom/prometheus:v2.45.0
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'
    restart: unless-stopped

  grafana:
    image: grafana/grafana:10.0.0
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin123  # Change this!
    volumes:
      - grafana_data:/var/lib/grafana
      - ./templates/monitoring/grafana-dashboard.json:/etc/grafana/provisioning/dashboards/cim-claude.json
    restart: unless-stopped

volumes:
  nats_data:
  prometheus_data:
  grafana_data:
```

## Security Configuration

### 1. TLS/SSL Setup

```bash
# Generate certificates (or use Let's Encrypt)
openssl req -x509 -newkey rsa:4096 -keyout tls-key.pem -out tls-cert.pem -days 365 -nodes

# Encrypt with age
age-encrypt -r $(age-keygen -y secrets.key) -o secrets/tls-cert.age < tls-cert.pem
age-encrypt -r $(age-keygen -y secrets.key) -o secrets/tls-key.age < tls-key.pem
```

### 2. Authentication & Authorization

The system supports multiple authentication methods:

- **NATS JWT**: Recommended for production (default)
- **OAuth2**: For integration with existing identity providers
- **API Keys**: For service-to-service communication
- **mTLS**: For maximum security between components

### 3. Secrets Management

Using age encryption for secure secret storage:

```bash
# Initialize age key
age-keygen > ~/.age/key.txt

# Encrypt Claude API key
echo "your-claude-api-key" | age -r $(age-keygen -y ~/.age/key.txt) > secrets/claude-api-key.age

# Encrypt database password
echo "secure-db-password" | age -r $(age-keygen -y ~/.age/key.txt) > secrets/db-password.age

# JWT signing key
openssl rand -hex 32 | age -r $(age-keygen -y ~/.age/key.txt) > secrets/jwt-key.age
```

### 4. Network Security

- **Zone-based Firewall**: Restricts traffic between network zones
- **Rate Limiting**: Prevents abuse and DoS attacks
- **DDoS Protection**: Multi-layer protection with Cloudflare
- **VPN Access**: Secure remote administration via WireGuard

### 5. Compliance & Auditing

Supports multiple compliance frameworks:

- **SOC 2**: Comprehensive security and availability controls
- **GDPR**: Data protection and privacy controls
- **HIPAA**: Healthcare data protection (when enabled)
- **PCI-DSS**: Payment card industry standards

## Monitoring & Observability

### 1. Metrics Collection

- **Prometheus**: Time-series metrics collection
- **Node Exporter**: System-level metrics
- **Application Metrics**: Custom business and technical metrics
- **NATS Metrics**: Message broker performance metrics

### 2. Logging

- **Structured Logging**: JSON format for all application logs
- **Centralized Collection**: Loki aggregation with Promtail
- **Log Retention**: Configurable retention policies
- **Security Logs**: Separate audit trail for security events

### 3. Distributed Tracing

- **Jaeger**: Complete request tracing across all components
- **OpenTelemetry**: Standards-based instrumentation
- **Performance Analysis**: Bottleneck identification and optimization

### 4. Alerting

Comprehensive alerting for:

- **Service Health**: Uptime and availability monitoring
- **Performance**: Response times and throughput
- **Resource Usage**: CPU, memory, disk, and network utilization
- **Security Events**: Authentication failures and suspicious activity
- **Business Metrics**: Conversation volumes and error rates

## Backup & Disaster Recovery

### 1. Automated Backup

```bash
# Manual backup
systemctl start cim-claude-backup

# Check backup status
systemctl status cim-claude-backup
journalctl -u cim-claude-backup -f

# List available backups
ls -la /var/lib/cim-claude-backup/
```

### 2. Backup Configuration

- **Daily Automated Backups**: All critical data backed up nightly
- **Multiple Retention Policies**: Daily, weekly, monthly backups
- **Remote Storage**: S3-compatible storage for off-site backups
- **Encryption**: All backups encrypted at rest and in transit
- **Compression**: ZSTD compression for optimal storage efficiency

### 3. Disaster Recovery

```bash
# Restore from backup
cim-claude-restore /var/lib/cim-claude-backup/cim-claude-backup-2025-01-20T10-00-00.tar.zst

# Verify restoration
systemctl status cim-claude-adapter
curl http://localhost:8080/health
```

### 4. Recovery Time Objectives

- **RTO (Recovery Time Objective)**: 15 minutes
- **RPO (Recovery Point Objective)**: 1 hour
- **High Availability Failover**: < 30 seconds
- **Full Disaster Recovery**: < 4 hours

## High Availability

### 1. Cluster Configuration

The HA setup provides:

- **Active-Passive Configuration**: One master, multiple backup nodes
- **Automatic Failover**: VRRP-based virtual IP management
- **Split-Brain Protection**: Quorum-based decision making
- **Health Monitoring**: Comprehensive service health checks

### 2. Load Balancing

- **NGINX Load Balancer**: Layer 7 load balancing with health checks
- **Session Affinity**: Optional sticky sessions for stateful operations
- **Circuit Breaker**: Automatic failure detection and routing
- **SSL Termination**: Centralized certificate management

### 3. Data Replication

- **NATS Clustering**: Built-in message replication
- **Database Replication**: PostgreSQL streaming replication
- **File Synchronization**: rsync-based configuration synchronization

## Performance Optimization

### 1. Resource Allocation

**Minimum Production Requirements:**
- **CPU**: 8 cores / 16 threads
- **RAM**: 16GB (32GB recommended)
- **Storage**: 500GB NVMe SSD
- **Network**: 1Gbps with low latency

**Recommended Production Setup:**
- **CPU**: 16 cores / 32 threads
- **RAM**: 64GB
- **Storage**: 2TB NVMe SSD RAID 1
- **Network**: 10Gbps with redundant paths

### 2. Tuning Parameters

```nix
# System-level optimizations
boot.kernel.sysctl = {
  # Network performance
  "net.core.rmem_max" = 134217728;
  "net.core.wmem_max" = 134217728;
  "net.ipv4.tcp_rmem" = "4096 12582912 134217728";
  "net.ipv4.tcp_wmem" = "4096 12582912 134217728";
  "net.core.netdev_max_backlog" = 5000;
  
  # File system performance
  "vm.dirty_ratio" = 15;
  "vm.dirty_background_ratio" = 5;
  "vm.swappiness" = 10;
  
  # Memory management
  "vm.min_free_kbytes" = 131072;
  "kernel.shmmax" = 68719476736;
};
```

### 3. Application Tuning

- **Connection Pooling**: Optimized NATS connection management
- **Message Batching**: Efficient bulk operations
- **Async Processing**: Non-blocking I/O for maximum throughput
- **Memory Management**: Careful buffer management and GC tuning

## Operational Procedures

### 1. Health Checks

```bash
# System health
nix run .#health-check-production

# Individual component health
curl -f http://10.101.0.100:8080/health      # Application health
curl -f http://10.102.0.10:8222/varz         # NATS health
curl -f http://10.103.0.10:9090/-/healthy    # Prometheus health
```

### 2. Log Analysis

```bash
# Application logs
journalctl -u cim-claude-adapter -f --since "1 hour ago"

# Security logs
journalctl -u cim-claude-security -f

# Cluster logs
journalctl -u keepalived -f

# NATS logs
journalctl -u nats-server -f
```

### 3. Performance Monitoring

```bash
# Resource usage
htop
iotop
nethogs

# NATS statistics
nats --server=nats://10.102.0.10:4222 server info
nats --server=nats://10.102.0.10:4222 stream report

# Application metrics
curl http://10.101.0.100:9090/metrics | grep cim_claude
```

### 4. Maintenance Procedures

#### Rolling Updates

```bash
# Update node 1 (backup)
nixos-rebuild switch --flake .#cim-claude-prod-02 --target-host root@10.101.0.11

# Wait for health check
sleep 30 && curl -f http://10.101.0.11:8080/health

# Update node 2 (backup)
nixos-rebuild switch --flake .#cim-claude-prod-03 --target-host root@10.101.0.12

# Wait for health check
sleep 30 && curl -f http://10.101.0.12:8080/health

# Update master (will trigger failover)
nixos-rebuild switch --flake .#cim-claude-prod-01 --target-host root@10.101.0.10
```

#### Certificate Renewal

```bash
# Update certificates
age-encrypt -r $(cat ~/.age/key.txt | age-keygen -y) -o secrets/tls-cert.age < new-cert.pem
age-encrypt -r $(cat ~/.age/key.txt | age-keygen -y) -o secrets/tls-key.age < new-key.pem

# Deploy updated certificates
nixos-rebuild switch --flake .#cim-claude-prod-01
```

### 5. Troubleshooting

#### Common Issues

1. **Service Won't Start**
   ```bash
   # Check service status
   systemctl status cim-claude-adapter
   
   # Check logs
   journalctl -u cim-claude-adapter --since "10 minutes ago"
   
   # Verify configuration
   nix build .#cim-claude-adapter
   ```

2. **NATS Connection Issues**
   ```bash
   # Test NATS connectivity
   nats --server=nats://10.102.0.10:4222 account info
   
   # Check NATS cluster status
   nats --server=nats://10.102.0.10:4222 server check cluster
   ```

3. **High CPU Usage**
   ```bash
   # Profile application
   cargo flamegraph --bin cim-claude-adapter
   
   # Check system resources
   htop
   vmstat 1
   ```

4. **Memory Leaks**
   ```bash
   # Memory profiling
   valgrind --tool=massif ./target/release/cim-claude-adapter
   
   # Heap analysis
   cargo profiling --bin cim-claude-adapter --heap
   ```

## Security Incident Response

### 1. Incident Classification

- **P1 Critical**: Service outage, data breach, security compromise
- **P2 High**: Performance degradation, minor security issue
- **P3 Medium**: Non-critical functionality affected
- **P4 Low**: Cosmetic issues, enhancement requests

### 2. Response Procedures

1. **Detection & Analysis**: Automated alerts and manual monitoring
2. **Containment**: Isolate affected systems
3. **Eradication**: Remove threat and vulnerabilities
4. **Recovery**: Restore normal operations
5. **Post-Incident**: Document lessons learned

### 3. Emergency Contacts

- **On-call Engineer**: +1-XXX-XXX-XXXX
- **Security Team**: security@cowboy-ai.com
- **Management**: alerts@cowboy-ai.com

## Cost Optimization

### 1. Resource Right-Sizing

- **CPU**: Start with 4 cores, scale based on usage
- **Memory**: Start with 8GB, monitor for actual requirements
- **Storage**: Use tiered storage for different data types
- **Network**: Optimize for actual bandwidth requirements

### 2. Monitoring Costs

- **Cloud Usage**: Track resource consumption
- **License Costs**: Monitor API usage and optimize
- **Operational Overhead**: Automate routine tasks

## Compliance & Certification

### 1. SOC 2 Type II

- **Security**: Access controls, encryption, monitoring
- **Availability**: High availability, disaster recovery
- **Processing Integrity**: Data validation, error handling
- **Confidentiality**: Data protection, secure communications
- **Privacy**: Data minimization, consent management

### 2. GDPR Compliance

- **Data Mapping**: Understand data flows and storage
- **Consent Management**: User consent tracking
- **Right to be Forgotten**: Data deletion capabilities
- **Data Portability**: Export functionality
- **Breach Notification**: Automated alerting within 72 hours

## Future Roadmap

### 1. Short-term (3 months)

- **Multi-region Deployment**: Active-active across regions
- **Advanced Monitoring**: ML-based anomaly detection
- **Performance Optimization**: Response time improvements
- **Security Enhancements**: Zero-trust architecture

### 2. Medium-term (6 months)

- **Kubernetes Migration**: Full containerization
- **Service Mesh**: Istio integration for advanced networking
- **GitOps**: Automated deployments with ArgoCD
- **Chaos Engineering**: Resilience testing with Chaos Monkey

### 3. Long-term (12 months)

- **Edge Deployment**: CDN integration and edge computing
- **AI/ML Integration**: Intelligent auto-scaling and optimization
- **Multi-cloud**: AWS, Azure, GCP deployment options
- **Compliance Automation**: Automated compliance reporting

---

**Support**: For deployment assistance, contact support@cowboy-ai.com
**Documentation**: Full documentation at https://docs.cowboy-ai.com/cim-claude-adapter
**Source Code**: https://github.com/TheCowboyAI/cim-claude-adapter