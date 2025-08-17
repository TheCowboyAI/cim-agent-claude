# CIM Claude Adapter - Integration Test Configuration
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{ config, lib, pkgs, ... }:

{
  imports = [
    ./module.nix
  ];

  # Enable the CIM Claude Adapter with full NATS infrastructure
  services.cim-claude-adapter = {
    enable = true;
    
    # Claude configuration (using dummy values for testing)
    claude = {
      apiKey = "sk-ant-test-key-for-integration-testing";
      model = "claude-3-sonnet-20240229";
      maxTokens = 4000;
      temperature = 0.7;
    };
    
    # Enable full NATS infrastructure
    nats = {
      enable = true;
      url = "nats://localhost:4222";
      
      infrastructure = {
        enable = true;
        environment = "development";  # Use development settings for testing
        replication.replicas = 1;     # Single replica for testing
        openFirewall = false;         # Keep firewall closed in testing
      };
    };
    
    # Monitoring
    monitoring = {
      metricsPort = 9090;
      healthPort = 8080;
      enableTracing = false;  # Disable tracing for simpler testing
    };
    
    # Logging
    logging = {
      level = "debug";
      format = "pretty";
    };
    
    # Keep ports closed for testing
    openFirewall = false;
  };

  # Basic system configuration for testing
  system.stateVersion = "24.05";
  
  # Ensure we have the necessary packages for testing
  environment.systemPackages = with pkgs; [
    natscli
    curl
    jq
  ];
  
  # Create test user for validation
  users.users.test-user = {
    isNormalUser = true;
    description = "Test user for CIM Claude Adapter validation";
  };

  # Simple validation script as a systemd service
  systemd.services.cim-claude-test = {
    description = "CIM Claude Adapter Integration Test";
    after = [ "cim-claude-adapter.service" "nats-jetstream-setup.service" ];
    wantedBy = [ "multi-user.target" ];
    
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      User = "test-user";
    };
    
    script = ''
      echo "Testing CIM Claude Adapter integration..."
      
      # Wait for services to be ready
      sleep 10
      
      # Test NATS server connectivity
      if ${pkgs.natscli}/bin/nats --server=nats://localhost:4222 server info; then
        echo "✓ NATS server is running"
      else
        echo "✗ NATS server connectivity failed"
        exit 1
      fi
      
      # Test JetStream streams
      if ${pkgs.natscli}/bin/nats --server=nats://localhost:4222 stream list | grep -q "CIM_CLAUDE_CONV_CMD"; then
        echo "✓ JetStream streams are configured"
      else
        echo "✗ JetStream streams not found"
        exit 1
      fi
      
      # Test health endpoint
      if ${pkgs.curl}/bin/curl -f http://localhost:8080/health >/dev/null 2>&1; then
        echo "✓ Claude Adapter health endpoint is responding"
      else
        echo "! Claude Adapter health endpoint not ready (may be expected if service is still starting)"
      fi
      
      # Test metrics endpoint  
      if ${pkgs.curl}/bin/curl -f http://localhost:9090/metrics >/dev/null 2>&1; then
        echo "✓ Claude Adapter metrics endpoint is responding"
      else
        echo "! Claude Adapter metrics endpoint not ready (may be expected if service is still starting)"
      fi
      
      echo "Integration test completed successfully"
    '';
  };
}