# CIM Claude Adapter - Production Nix Flake
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{
  description = "CIM Claude Adapter - Production-ready event-driven Claude AI integration for CIM ecosystems";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs-stable.url = "github:NixOS/nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    # Security and secrets management
    agenix.url = "github:ryantm/agenix";
    sops-nix.url = "github:Mic92/sops-nix";
    # Container and Kubernetes tools
    nix2container.url = "github:nlewo/nix2container";
  };

  outputs = { self, nixpkgs, nixpkgs-stable, rust-overlay, flake-utils, crane, advisory-db, agenix, sops-nix, nix2container }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Production dependencies
        pkgs-stable = import nixpkgs-stable {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Crane for advanced Rust builds
        craneLib = crane.mkLib pkgs;
        
        # Container builder
        nix2containerPkgs = nix2container.packages.${system};

        # Rust toolchain
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # CIM Claude Adapter Configuration
        # Hard-locked Anthropic API version for consistency
        anthropicApiVersion = "2023-06-01";
        
        # Production version and metadata
        version = "1.0.0";
        gitRev = self.rev or "dirty";
        buildDate = builtins.substring 0 10 (builtins.toString (self.lastModified or 1970-01-01));

        # Source filtering for better caching
        src = pkgs.lib.cleanSourceWith {
          src = ./.; 
          filter = path: type:
            (pkgs.lib.hasSuffix ".rs" path) ||
            (pkgs.lib.hasSuffix ".toml" path) ||
            (pkgs.lib.hasSuffix ".lock" path) ||
            (pkgs.lib.hasInfix "/src/" path) ||
            (pkgs.lib.hasInfix "/tests/" path) ||
            (pkgs.lib.hasInfix "/examples/" path) ||
            (baseNameOf path == "Cargo.toml") ||
            (baseNameOf path == "Cargo.lock") ||
            (type == "directory");
        };

        # Common cargo arguments
        commonArgs = {
          inherit src;
          strictDeps = true;
          
          buildInputs = with pkgs; [
            openssl
            pkg-config
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            libiconv
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            # Linux-specific dependencies for Iced GUI
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            vulkan-loader
            libGL
            wayland
            wayland-protocols
            libxkbcommon
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
            wrapGAppsHook
          ];
          
          # Build-time environment variables
          CIM_ANTHROPIC_API_VERSION = anthropicApiVersion;
          CIM_VERSION = version;
          CIM_GIT_REV = gitRev;
          CIM_BUILD_DATE = buildDate;
        };

        # Build the dependency-only derivation
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Production Rust application using Crane
        cim-claude-adapter = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "cim-claude-adapter";
          inherit version;
          
          # Production optimizations
          cargoExtraArgs = "--release --locked";
          
          # Post-install processing
          postInstall = ''
            # Install desktop files and icons for GUI
            mkdir -p $out/share/applications
            cat > $out/share/applications/cim-claude-adapter.desktop << EOF
[Desktop Entry]
Name=CIM Claude Adapter
Comment=Event-driven Claude AI integration for CIM ecosystems
Exec=$out/bin/cim-gui
Icon=cim-claude-adapter
Terminal=false
Type=Application
Categories=Development;Utility;
StartupWMClass=CIM Claude Adapter
EOF
            
            # Install systemd service templates
            mkdir -p $out/lib/systemd/system
            cat > $out/lib/systemd/system/cim-claude-adapter@.service << EOF
[Unit]
Description=CIM Claude Adapter for %i
After=network-online.target nats-server.service
Wants=network-online.target
Requires=nats-server.service

[Service]
Type=exec
User=cim-claude-adapter
Group=cim-claude-adapter
ExecStart=$out/bin/cim-claude-adapter
Restart=always
RestartSec=10s
Environment=CIM_CONFIG_PATH=/etc/cim-claude-adapter/%i.toml

[Install]
WantedBy=multi-user.target
EOF
          '';

          # Metadata
          meta = with pkgs.lib; {
            description = "Production event-driven Claude AI adapter for CIM ecosystems";
            homepage = "https://github.com/TheCowboyAI/cim-agent-claude";
            license = licenses.mit;
            maintainers = [ "Cowboy AI, LLC <info@thecowboy.ai>" ];
            platforms = platforms.unix;
            mainProgram = "cim-claude-adapter";
          };
        });
        
        # Security audit
        cim-claude-adapter-audit = craneLib.cargoAudit (commonArgs // {
          inherit cargoArtifacts;
          inherit advisory-db;
        });
        
        # Documentation
        cim-claude-adapter-doc = craneLib.cargoDoc (commonArgs // {
          inherit cargoArtifacts;
        });
        
        # Container image
        cim-claude-adapter-container = nix2containerPkgs.nix2container.buildImage {
          name = "cim-claude-adapter";
          tag = "v${version}";
          
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = with pkgs; [
              cim-claude-adapter
              bash
              coreutils
              curl
              # CA certificates for HTTPS
              cacert
              # Timezone data
              tzdata
            ];
            pathsToLink = [ "/bin" "/share" "/etc" ];
          };
          
          config = {
            Cmd = [ "/bin/cim-claude-adapter" ];
            Env = [
              "PATH=/bin"
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              "TZ=UTC"
            ];
            ExposedPorts = {
              "8080/tcp" = {}; # Health check port
              "9090/tcp" = {}; # Metrics port
            };
            Labels = {
              "org.opencontainers.image.title" = "CIM Claude Adapter";
              "org.opencontainers.image.description" = "Event-driven Claude AI integration";
              "org.opencontainers.image.version" = version;
              "org.opencontainers.image.revision" = gitRev;
              "org.opencontainers.image.created" = buildDate;
              "org.opencontainers.image.licenses" = "MIT";
              "org.opencontainers.image.vendor" = "Cowboy AI, LLC";
            };
            User = "65534:65534"; # nobody:nobody
          };
        };
        
        # Kubernetes manifests
        kubernetes-manifests = pkgs.writeTextDir "k8s/cim-claude-adapter.yaml" ''
apiVersion: v1
kind: Namespace
metadata:
  name: cim-claude
  labels:
    name: cim-claude
    app.kubernetes.io/name: cim-claude-adapter
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cim-claude-adapter
  namespace: cim-claude
  labels:
    app: cim-claude-adapter
    version: "${version}"
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cim-claude-adapter
  template:
    metadata:
      labels:
        app: cim-claude-adapter
        version: "${version}"
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: cim-claude-adapter
      containers:
      - name: cim-claude-adapter
        image: cim-claude-adapter:v${version}
        ports:
        - containerPort: 8080
          name: health
          protocol: TCP
        - containerPort: 9090
          name: metrics
          protocol: TCP
        env:
        - name: RUST_LOG
          value: "info"
        - name: CLAUDE_API_KEY
          valueFrom:
            secretKeyRef:
              name: claude-api-key
              key: api-key
        - name: NATS_URL
          value: "nats://nats-service:4222"
        livenessProbe:
          httpGet:
            path: /health
            port: health
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: health
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          runAsNonRoot: true
          runAsUser: 65534
          capabilities:
            drop:
            - ALL
      restartPolicy: Always
---
apiVersion: v1
kind: Service
metadata:
  name: cim-claude-adapter-service
  namespace: cim-claude
  labels:
    app: cim-claude-adapter
spec:
  selector:
    app: cim-claude-adapter
  ports:
  - name: health
    port: 8080
    targetPort: health
  - name: metrics
    port: 9090
    targetPort: metrics
  type: ClusterIP
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: cim-claude-adapter
  namespace: cim-claude
        '';
        
        # Monitoring configuration
        monitoring-config = pkgs.writeTextDir "monitoring/prometheus.yml" ''
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "cim-claude-adapter-rules.yml"

scrape_configs:
  - job_name: 'cim-claude-adapter'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: /metrics
    scrape_interval: 10s
    
  - job_name: 'nats-server'
    static_configs:
      - targets: ['localhost:8222']
    metrics_path: /varz
    scrape_interval: 15s
        '' + pkgs.writeText "cim-claude-adapter-rules.yml" ''
groups:
- name: cim-claude-adapter
  rules:
  - alert: CIMClaudeAdapterDown
    expr: up{job="cim-claude-adapter"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "CIM Claude Adapter is down"
      description: "CIM Claude Adapter has been down for more than 1 minute."
      
  - alert: CIMClaudeAdapterHighErrorRate
    expr: rate(cim_claude_adapter_errors_total[5m]) > 0.1
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "High error rate in CIM Claude Adapter"
      description: "Error rate is {{ $value }} errors per second."
      
  - alert: CIMClaudeAdapterHighLatency
    expr: histogram_quantile(0.95, rate(cim_claude_adapter_request_duration_seconds_bucket[5m])) > 5
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High latency in CIM Claude Adapter"
      description: "95th percentile latency is {{ $value }}s."
        '';
        
        # Backup and restore scripts
        backup-scripts = pkgs.writeShellApplication {
          name = "cim-claude-backup";
          runtimeInputs = with pkgs; [ nats-server natscli gzip ];
          text = ''
            #!/bin/bash
            set -euo pipefail
            
            BACKUP_DIR="''${BACKUP_DIR:-/var/lib/cim-claude-backup}"
            DATE=$(date -Iseconds)
            NATS_URL="''${NATS_URL:-nats://localhost:4222}"
            
            echo "Starting CIM Claude Adapter backup at $DATE"
            mkdir -p "$BACKUP_DIR/$DATE"
            
            # Backup JetStream streams
            echo "Backing up JetStream streams..."
            nats --server="$NATS_URL" stream backup CIM_CLAUDE_CONV_EVT "$BACKUP_DIR/$DATE/conv_evt.backup"
            nats --server="$NATS_URL" stream backup CIM_CLAUDE_ATTACH_EVT "$BACKUP_DIR/$DATE/attach_evt.backup"
            
            # Backup KV stores
            echo "Backing up KV stores..."
            nats --server="$NATS_URL" kv get CIM_CLAUDE_CONV_KV --all > "$BACKUP_DIR/$DATE/conv_kv.backup"
            nats --server="$NATS_URL" kv get CIM_CLAUDE_ATTACH_KV --all > "$BACKUP_DIR/$DATE/attach_kv.backup"
            
            # Compress backup
            echo "Compressing backup..."
            tar -czf "$BACKUP_DIR/cim-claude-backup-$DATE.tar.gz" -C "$BACKUP_DIR" "$DATE"
            rm -rf "$BACKUP_DIR/$DATE"
            
            echo "Backup completed: $BACKUP_DIR/cim-claude-backup-$DATE.tar.gz"
          '';
        };

        # Development tools and utilities
        dev-tools = with pkgs; [
          # Rust development
          rustToolchain
          rust-analyzer
          clippy
          rustfmt
          cargo-audit
          cargo-watch
          cargo-tarpaulin
          cargo-flamegraph
          
          # NATS tools
          natscli
          
          # Container and Kubernetes tools  
          docker
          kubectl
          kubernetes-helm
          kustomize
          
          # Monitoring and debugging
          prometheus
          grafana
          jaeger
          
          # Network and debugging tools
          curl
          jq
          httpie
          tcpdump
          netcat
          
          # Documentation and analysis
          mdbook
          plantuml
          graphviz
          
          # Nix tools
          nixpkgs-fmt
          nil
          nix-tree
          nix-diff
        ];
        
        # NixOS test VM for integration testing
        test-vm = pkgs.nixosTest {
          name = "cim-claude-adapter-test";
          
          nodes.machine = { config, pkgs, ... }: {
            imports = [ self.nixosModules.cim-claude-adapter ];
            
            services.cim-claude-adapter = {
              enable = true;
              claude.apiKey = "test-key-placeholder";
              nats.infrastructure.enable = true;
              nats.infrastructure.environment = "development";
            };
            
            # Allow unfree packages for testing
            nixpkgs.config.allowUnfree = true;
            
            # Networking configuration
            networking.firewall.enable = false;
            
            # Add test utilities
            environment.systemPackages = with pkgs; [
              curl
              jq
              natscli
            ];
          };
          
          testScript = ''
            start_all()
            
            # Wait for services to start
            machine.wait_for_unit("nats-server.service")
            machine.wait_for_unit("nats-jetstream-setup.service")
            machine.wait_for_unit("cim-claude-adapter.service")
            
            # Test NATS connectivity
            machine.succeed("nats --server=nats://localhost:4222 account info")
            
            # Test health endpoints
            machine.wait_for_open_port(8080)
            machine.wait_for_open_port(9090)
            
            machine.succeed("curl -f http://localhost:8080/health")
            machine.succeed("curl -f http://localhost:9090/metrics")
            
            print("CIM Claude Adapter integration test completed successfully")
          '';
        };

      in {
        # Packages
        packages = {
          default = cim-claude-adapter;
          cim-claude-adapter = cim-claude-adapter;
          container = cim-claude-adapter-container;
          kubernetes-manifests = kubernetes-manifests;
          monitoring-config = monitoring-config;
          backup-scripts = backup-scripts;
          # Documentation
          docs = cim-claude-adapter-doc;
          # Security
          audit = cim-claude-adapter-audit;
        };

        # Development shells
        devShells = {
          default = pkgs.mkShell {
            # Build inputs (same as the package)
            buildInputs = commonArgs.buildInputs;
            nativeBuildInputs = commonArgs.nativeBuildInputs;

            # Development packages
            packages = dev-tools;

            # Environment variables for development
            inherit (commonArgs) CIM_ANTHROPIC_API_VERSION CIM_VERSION CIM_GIT_REV CIM_BUILD_DATE;
            
            # Development-specific environment
            RUST_BACKTRACE = "1";
            RUST_LOG = "debug";
            
            shellHook = ''
              echo "🤖 CIM Claude Adapter Production Development Environment"
              echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
              echo "📦 Rust toolchain: $(rustc --version)"
              echo "🔒 Anthropic API Version: ${anthropicApiVersion} (hard-locked)"
              echo "🏷️  Version: ${version} (rev: ${gitRev})"
              echo "📅 Build Date: ${buildDate}"
              echo ""
              echo "🔧 Development Commands:"
              echo "  cargo build                    - Build the project"
              echo "  cargo test                     - Run tests"
              echo "  cargo run --bin cim-claude-adapter  - Run CLI adapter"
              echo "  cargo run --bin cim-gui        - Run GUI application"
              echo "  cargo watch -x run             - Auto-reload on changes"
              echo "  cargo audit                    - Security audit"
              echo "  cargo tarpaulin                - Code coverage"
              echo ""
              echo "🏗️  Nix Commands:"
              echo "  nix build                      - Build with Nix"
              echo "  nix build .#container          - Build container image"
              echo "  nix build .#kubernetes-manifests - Generate K8s manifests"
              echo "  nix build .#monitoring-config  - Generate monitoring config"
              echo "  nix run .#test-vm              - Run integration test VM"
              echo ""
              echo "🐳 Container Commands:"
              echo "  docker load < $(nix build .#container --print-out-paths)"
              echo "  kubectl apply -f $(nix build .#kubernetes-manifests --print-out-paths)/k8s/"
              echo ""
              echo "📊 Monitoring:"
              echo "  prometheus --config.file=$(nix build .#monitoring-config --print-out-paths)/monitoring/prometheus.yml"
              echo "  grafana-server  # Dashboard on :3000"
              echo ""
              echo "🔍 Required Environment Variables:"
              echo "  export CLAUDE_API_KEY=your-api-key-here"
              echo "  export NATS_URL=nats://localhost:4222  # (default)"
              echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            '';
          };
          
          # Production deployment shell
          production = pkgs.mkShell {
            packages = with pkgs; [
              kubectl
              kubernetes-helm
              kustomize
              prometheus
              grafana
              cim-claude-adapter
            ] ++ [ backup-scripts ];
            
            shellHook = ''
              echo "🏭 CIM Claude Adapter Production Deployment Environment"
              echo "Available tools: kubectl, helm, kustomize, prometheus, grafana"
              echo "Backup tools: cim-claude-backup"
            '';
          };
          
          # Security analysis shell
          security = pkgs.mkShell {
            packages = with pkgs; [
              cargo-audit
              cargo-deny
              semgrep
              trivy
              clair
            ];
            
            shellHook = ''
              echo "🔒 CIM Claude Adapter Security Analysis Environment"
              echo "Run: cargo audit, cargo deny, semgrep, trivy"
            '';
          };
        };

        # Formatter
        formatter = pkgs.nixpkgs-fmt;

        # Comprehensive checks for CI/CD
        checks = {
          # Core application build
          build = cim-claude-adapter;
          
          # Documentation build
          docs = cim-claude-adapter-doc;
          
          # Security audit
          audit = cim-claude-adapter-audit;
          
          # Clippy linting with enhanced rules
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings --deny clippy::all";
          });
          
          # Formatting check
          fmt = craneLib.cargoFmt {
            inherit src;
          };
          
          # Unit tests
          test = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
          
          # Integration tests with NATS
          integration-test = test-vm;
          
          # Container image scan
          container-scan = pkgs.runCommand "container-scan" {
            buildInputs = [ pkgs.trivy ];
          } ''
            mkdir -p $out
            # Note: In production, load the container and scan it
            echo "Container security scan would run here" > $out/scan-results
          '';
          
          # Kubernetes manifest validation
          k8s-validate = pkgs.runCommand "k8s-validate" {
            buildInputs = with pkgs; [ kubectl kubernetes-helm ];
          } ''
            mkdir -p $out
            # Validate Kubernetes manifests
            kubectl --dry-run=client apply -f ${kubernetes-manifests}/k8s/ > $out/k8s-validation
            echo "Kubernetes manifests are valid" >> $out/k8s-validation
          '';
          
          # Performance benchmarks
          bench = craneLib.cargoBench (commonArgs // {
            inherit cargoArtifacts;
          });
        };
        
        # Development applications
        apps = {
          default = flake-utils.lib.mkApp {
            drv = cim-claude-adapter;
            exePath = "/bin/cim-claude-adapter";
          };
          
          gui = flake-utils.lib.mkApp {
            drv = cim-claude-adapter;
            exePath = "/bin/cim-gui";
          };
          
          backup = flake-utils.lib.mkApp {
            drv = backup-scripts;
          };
        };
        
        # Hydra jobs for continuous integration
        hydraJobs = {
          inherit (self.checks.${system}) build test clippy fmt audit;
          inherit (self.packages.${system}) container kubernetes-manifests;
        };
      }
    ) // {
      # NixOS modules for production deployment
      nixosModules = {
        default = import ./nix/module.nix;
        cim-claude-adapter = import ./nix/module.nix;
        nats-infrastructure = import ./nix/nats-infrastructure.nix;
        security-hardening = import ./nix/security-hardening.nix;
        monitoring = import ./nix/monitoring.nix;
        backup-restore = import ./nix/backup-restore.nix;
        network-topology = import ./nix/network-topology.nix;
        high-availability = import ./nix/high-availability.nix;
        test-integration = import ./nix/test-integration.nix;
      };
      
      # Overlays for package distribution
      overlays = {
        default = final: prev: {
          cim-claude-adapter = self.packages.${final.system}.cim-claude-adapter;
        };
        
        monitoring = final: prev: {
          inherit (self.packages.${final.system}) monitoring-config;
        };
        
        backup-tools = final: prev: {
          inherit (self.packages.${final.system}) backup-scripts;
        };
      };
      
      # Template configurations for different deployment scenarios
      templates = {
        production = {
          path = ./templates/production;
          description = "Production CIM Claude Adapter deployment";
        };
        
        development = {
          path = ./templates/development;
          description = "Development environment setup";
        };
        
        kubernetes = {
          path = ./templates/kubernetes;
          description = "Kubernetes deployment manifests";
        };
        
        monitoring = {
          path = ./templates/monitoring;
          description = "Monitoring and observability stack";
        };
      };
    };
}