# Example NixOS Configuration for CIM Agent Claude
# Shows all available services with enable/disable options
{ config, pkgs, ... }:

let
  # Import the CIM Agent Claude flake
  cim-agent-claude = builtins.getFlake "github:TheCowboyAI/cim-agent-claude";
in {
  imports = [
    cim-agent-claude.nixosModules.default
  ];

  # Enable all CIM Agent Claude services
  services.cim-agent-claude = {
    enable = true;
    package = cim-agent-claude.packages.${pkgs.system}.cim-agent-claude;

    # SAGE Orchestrator Service - The intelligent expert agent coordinator
    sage = {
      enable = true;  # Set to false to disable SAGE
      package = cim-agent-claude.packages.${pkgs.system}.cim-sage-service;
      
      # Service configuration
      user = "sage";
      group = "sage";
      
      nats = {
        url = "nats://localhost:4222";
        sageSubject = "cim.sage";
      };
      
      claude = {
        # Store Claude API key in a file readable by sage user
        apiKeyFile = "/var/lib/sage/claude-api-key";
      };
      
      server = {
        host = "127.0.0.1";
        port = 8082;
      };
      
      observability = {
        logLevel = "INFO";  # TRACE, DEBUG, INFO, WARN, ERROR
      };
      
      # Optional: environment file for additional secrets
      environmentFile = "/var/lib/sage/environment";
    };

    # Claude Adapter Service - Direct Claude API integration
    adapter = {
      enable = true;  # Set to false to disable adapter
      
      # Service configuration
      user = "cim-claude";
      group = "cim-claude";
      
      nats = {
        url = "nats://localhost:4222";
        subject_prefix = "cim.claude";
        
        jetstream = {
          maxMemoryStore = 1073741824;  # 1GB
          maxFileStore = 10737418240;   # 10GB
          storeDir = "/var/lib/nats/jetstream";
        };
      };
      
      claude = {
        # Store Claude API key in a file readable by cim-claude user
        apiKeyFile = "/var/lib/cim-claude/claude-api-key";
        baseUrl = "https://api.anthropic.com";
        model = "claude-3-5-sonnet-20241022";
        maxTokens = 4096;
        temperature = 0.7;
      };
      
      server = {
        host = "127.0.0.1";
        port = 8080;
        cleanupIntervalSeconds = 300;
        healthCheckIntervalSeconds = 30;
      };
      
      observability = {
        logLevel = "INFO";
        metricsEnabled = true;
        metricsPort = 9090;
        tracingEnabled = false;
      };
      
      # Optional: environment file for additional secrets
      environmentFile = "/var/lib/cim-claude/environment";
    };

    # Desktop GUI Application
    gui = {
      enable = true;  # Set to false to disable GUI
      package = cim-agent-claude.packages.${pkgs.system}.cim-claude-gui;
      
      # Automatically start GUI on user login
      autostart = false;  # Set to true for autostart
    };

    # Web Interface
    web = {
      enable = true;  # Set to false to disable web interface
      package = cim-agent-claude.packages.${pkgs.system}.cim-claude-gui-wasm;
      
      # Web server configuration
      virtualHost = "cim-claude.local";
      port = 8081;
      
      # SSL configuration (optional)
      enableSSL = false;
      # sslCertificate = "/path/to/cert.pem";
      # sslCertificateKey = "/path/to/key.pem";
    };
  };

  # Example: Minimal configuration (only SAGE)
  # services.cim-agent-claude = {
  #   enable = true;
  #   package = cim-agent-claude.packages.${pkgs.system}.cim-agent-claude;
  #   
  #   sage = {
  #     enable = true;
  #     package = cim-agent-claude.packages.${pkgs.system}.cim-sage-service;
  #     claude.apiKeyFile = "/var/lib/sage/claude-api-key";
  #   };
  #   
  #   # Disable other services
  #   adapter.enable = false;
  #   gui.enable = false;
  #   web.enable = false;
  # };

  # Example: Web-only configuration
  # services.cim-agent-claude = {
  #   enable = true;
  #   package = cim-agent-claude.packages.${pkgs.system}.cim-agent-claude;
  #   
  #   sage = {
  #     enable = true;
  #     package = cim-agent-claude.packages.${pkgs.system}.cim-sage-service;
  #     claude.apiKeyFile = "/var/lib/sage/claude-api-key";
  #   };
  #   
  #   web = {
  #     enable = true;
  #     package = cim-agent-claude.packages.${pkgs.system}.cim-claude-gui-wasm;
  #     virtualHost = "cim.example.com";
  #     port = 80;
  #     enableSSL = true;
  #     sslCertificate = "/path/to/cert.pem";
  #     sslCertificateKey = "/path/to/key.pem";
  #   };
  #   
  #   # Disable other services
  #   adapter.enable = false;
  #   gui.enable = false;
  # };

  # System configuration
  system.stateVersion = "24.11";
  
  # Add some useful tools for CIM development
  environment.systemPackages = with pkgs; [
    natscli  # NATS CLI for debugging
    jq       # JSON processing
    curl     # HTTP client
    htop     # System monitor
  ];
  
  # Optional: Create API key files (you'll need to populate these)
  # These would typically be managed by your deployment system
  # system.activationScripts.setup-api-keys = ''
  #   mkdir -p /var/lib/sage /var/lib/cim-claude
  #   
  #   # Set up SAGE API key (replace with your actual key)
  #   if [ ! -f /var/lib/sage/claude-api-key ]; then
  #     echo "sk-your-sage-claude-api-key-here" > /var/lib/sage/claude-api-key
  #     chown sage:sage /var/lib/sage/claude-api-key
  #     chmod 600 /var/lib/sage/claude-api-key
  #   fi
  #   
  #   # Set up Adapter API key (replace with your actual key)
  #   if [ ! -f /var/lib/cim-claude/claude-api-key ]; then
  #     echo "sk-your-adapter-claude-api-key-here" > /var/lib/cim-claude/claude-api-key
  #     chown cim-claude:cim-claude /var/lib/cim-claude/claude-api-key
  #     chmod 600 /var/lib/cim-claude/claude-api-key
  #   fi
  # '';
}