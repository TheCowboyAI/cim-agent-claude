# CIM Claude Adapter - Security Hardening Module
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.cim-claude-security;
in {
  ###### Interface
  options.services.cim-claude-security = {
    enable = mkEnableOption "CIM Claude Adapter security hardening";

    # Firewall configuration
    firewall = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable advanced firewall rules for CIM Claude Adapter.";
      };

      allowedClientIPs = mkOption {
        type = types.listOf types.str;
        default = [];
        description = "List of allowed client IP addresses/ranges.";
        example = [ "10.0.0.0/8" "172.16.0.0/12" "192.168.0.0/16" ];
      };

      rateLimiting = {
        enable = mkOption {
          type = types.bool;
          default = true;
          description = "Enable rate limiting using iptables.";
        };

        requestsPerMinute = mkOption {
          type = types.int;
          default = 1000;
          description = "Maximum requests per minute per IP.";
        };
      };
    };

    # TLS/SSL configuration
    tls = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable TLS for all communications.";
      };

      certificatePath = mkOption {
        type = types.path;
        description = "Path to TLS certificate file.";
        example = "/run/secrets/cim-claude-cert.pem";
      };

      keyPath = mkOption {
        type = types.path;
        description = "Path to TLS private key file.";
        example = "/run/secrets/cim-claude-key.pem";
      };

      ciphers = mkOption {
        type = types.listOf types.str;
        default = [
          "ECDHE-ECDSA-AES256-GCM-SHA384"
          "ECDHE-RSA-AES256-GCM-SHA384"
          "ECDHE-ECDSA-CHACHA20-POLY1305"
          "ECDHE-RSA-CHACHA20-POLY1305"
          "ECDHE-ECDSA-AES128-GCM-SHA256"
          "ECDHE-RSA-AES128-GCM-SHA256"
        ];
        description = "Allowed TLS cipher suites.";
      };

      protocols = mkOption {
        type = types.listOf types.str;
        default = [ "TLSv1.2" "TLSv1.3" ];
        description = "Allowed TLS protocol versions.";
      };
    };

    # Authentication and authorization
    auth = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable authentication and authorization.";
      };

      method = mkOption {
        type = types.enum [ "nats-jwt" "oauth2" "api-key" "mTLS" ];
        default = "nats-jwt";
        description = "Authentication method to use.";
      };

      jwtConfig = {
        issuer = mkOption {
          type = types.str;
          default = "cim-claude-adapter";
          description = "JWT token issuer.";
        };

        audience = mkOption {
          type = types.str;
          default = "cim-claude-api";
          description = "JWT token audience.";
        };

        keyPath = mkOption {
          type = types.path;
          description = "Path to JWT signing key.";
          example = "/run/secrets/jwt-signing-key";
        };
      };

      roleMapping = mkOption {
        type = types.attrsOf (types.listOf types.str);
        default = {
          "admin" = [ "*" ];
          "user" = [ "cim.claude.conv.*" "cim.claude.attach.qry.*" ];
          "readonly" = [ "cim.claude.*.qry.*" ];
        };
        description = "Role-based permission mapping.";
      };
    };

    # Secrets management
    secrets = {
      backend = mkOption {
        type = types.enum [ "agenix" "sops-nix" "kubernetes" "vault" ];
        default = "agenix";
        description = "Secrets management backend.";
      };

      autoRotation = {
        enable = mkOption {
          type = types.bool;
          default = true;
          description = "Enable automatic secret rotation.";
        };

        interval = mkOption {
          type = types.str;
          default = "24h";
          description = "Secret rotation interval.";
        };
      };
    };

    # Security monitoring
    monitoring = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable security monitoring and alerting.";
      };

      failedAuthThreshold = mkOption {
        type = types.int;
        default = 10;
        description = "Failed authentication attempts before alert.";
      };

      suspiciousActivityPatterns = mkOption {
        type = types.listOf types.str;
        default = [
          "rapid_requests"
          "unusual_endpoints"
          "privilege_escalation"
          "data_exfiltration"
        ];
        description = "Security patterns to monitor.";
      };

      alerting = {
        webhook = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Webhook URL for security alerts.";
        };

        email = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Email address for security alerts.";
        };
      };
    };

    # Compliance and auditing
    compliance = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable compliance logging and auditing.";
      };

      standards = mkOption {
        type = types.listOf (types.enum [ "SOC2" "GDPR" "HIPAA" "PCI-DSS" ]);
        default = [ "SOC2" "GDPR" ];
        description = "Compliance standards to enforce.";
      };

      auditLogPath = mkOption {
        type = types.str;
        default = "/var/log/cim-claude-audit";
        description = "Path for audit logs.";
      };

      retentionPeriod = mkOption {
        type = types.str;
        default = "7y";
        description = "Audit log retention period.";
      };
    };
  };

  ###### Implementation
  config = mkIf cfg.enable {
    # Advanced firewall configuration
    networking.firewall = mkIf cfg.firewall.enable {
      extraCommands = ''
        # Rate limiting rules
        ${optionalString cfg.firewall.rateLimiting.enable ''
          iptables -A nixos-fw -p tcp --dport 8080 -m limit --limit ${toString cfg.firewall.rateLimiting.requestsPerMinute}/min -j ACCEPT
          iptables -A nixos-fw -p tcp --dport 9090 -m limit --limit ${toString cfg.firewall.rateLimiting.requestsPerMinute}/min -j ACCEPT
        ''}
        
        # IP allowlisting
        ${concatMapStringsSep "\n" (ip: ''
          iptables -A nixos-fw -s ${ip} -j ACCEPT
        '') cfg.firewall.allowedClientIPs}
        
        # Drop packets from suspicious sources
        iptables -A nixos-fw -m string --string "malware" --algo bm -j DROP
        iptables -A nixos-fw -m string --string "injection" --algo bm -j DROP
        
        # Log dropped packets for analysis
        iptables -A nixos-fw -j LOG --log-prefix "CIM-CLAUDE-DROP: " --log-level 4
      '';
    };

    # TLS configuration
    services.cim-claude-adapter = mkIf cfg.tls.enable {
      extraConfig = ''
        [tls]
        enabled = true
        cert_file = "${cfg.tls.certificatePath}"
        key_file = "${cfg.tls.keyPath}"
        ciphers = [${concatMapStringsSep ", " (c: ''"${c}"'') cfg.tls.ciphers}]
        protocols = [${concatMapStringsSep ", " (p: ''"${p}"'') cfg.tls.protocols}]
        
        [security]
        auth_method = "${cfg.auth.method}"
        
        ${optionalString (cfg.auth.method == "nats-jwt") ''
        [jwt]
        issuer = "${cfg.auth.jwtConfig.issuer}"
        audience = "${cfg.auth.jwtConfig.audience}"
        key_file = "${cfg.auth.jwtConfig.keyPath}"
        ''}
        
        [compliance]
        enabled = ${if cfg.compliance.enable then "true" else "false"}
        standards = [${concatMapStringsSep ", " (s: ''"${s}"'') cfg.compliance.standards}]
        audit_log_path = "${cfg.compliance.auditLogPath}"
        retention_period = "${cfg.compliance.retentionPeriod}"
      '';
    };

    # Security monitoring service
    systemd.services.cim-claude-security-monitor = mkIf cfg.monitoring.enable {
      description = "CIM Claude Adapter Security Monitoring";
      after = [ "cim-claude-adapter.service" ];
      wants = [ "cim-claude-adapter.service" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "exec";
        User = "cim-claude-security";
        Group = "cim-claude-security";
        ExecStart = pkgs.writeShellScript "security-monitor" ''
          set -euo pipefail
          
          # Monitor authentication failures
          tail -f /var/log/cim-claude-adapter/audit.log | while read -r line; do
            if echo "$line" | grep -q "auth_failed"; then
              failed_count=$(echo "$line" | jq -r '.failed_count // 0')
              if [ "$failed_count" -gt ${toString cfg.monitoring.failedAuthThreshold} ]; then
                # Send alert
                ${optionalString (cfg.monitoring.alerting.webhook != null) ''
                  curl -X POST "${cfg.monitoring.alerting.webhook}" \
                    -H "Content-Type: application/json" \
                    -d "{\"alert\": \"High authentication failures\", \"count\": $failed_count}"
                ''}
                ${optionalString (cfg.monitoring.alerting.email != null) ''
                  echo "Security Alert: High authentication failures ($failed_count)" | \
                    mail -s "CIM Claude Security Alert" "${cfg.monitoring.alerting.email}"
                ''}
              fi
            fi
          done
        '';
        
        Restart = "always";
        RestartSec = "10s";
      };
    };

    # Create security monitoring user
    users.users.cim-claude-security = {
      description = "CIM Claude Security Monitor";
      group = "cim-claude-security";
      isSystemUser = true;
    };

    users.groups.cim-claude-security = {};

    # Audit log directory
    systemd.tmpfiles.rules = [
      "d ${cfg.compliance.auditLogPath} 0750 cim-claude-adapter cim-claude-adapter -"
    ];

    # Logrotate configuration for audit logs
    services.logrotate.settings."${cfg.compliance.auditLogPath}/*.log" = {
      frequency = "daily";
      rotate = 2557; # ~7 years
      compress = true;
      delaycompress = true;
      missingok = true;
      notifempty = true;
      create = "640 cim-claude-adapter cim-claude-adapter";
      postrotate = ''
        systemctl reload cim-claude-adapter || true
      '';
    };

    # Security kernel parameters
    boot.kernel.sysctl = {
      # Network security
      "net.ipv4.tcp_syncookies" = 1;
      "net.ipv4.ip_forward" = 0;
      "net.ipv4.conf.all.accept_redirects" = 0;
      "net.ipv4.conf.all.send_redirects" = 0;
      "net.ipv4.conf.all.accept_source_route" = 0;
      "net.ipv4.conf.all.log_martians" = 1;
      
      # Memory protection
      "kernel.randomize_va_space" = 2;
      "kernel.exec-shield" = 1;
      "kernel.kptr_restrict" = 2;
      "kernel.dmesg_restrict" = 1;
      
      # Process restrictions
      "fs.suid_dumpable" = 0;
      "kernel.core_uses_pid" = 1;
    };

    # AppArmor profile for additional protection
    security.apparmor = {
      enable = true;
      profiles = {
        cim-claude-adapter = {
          profile = ''
            #include <tunables/global>
            
            /bin/cim-claude-adapter {
              #include <abstractions/base>
              #include <abstractions/nameservice>
              #include <abstractions/ssl_certs>
              
              # Network access
              network inet stream,
              network inet dgram,
              network inet6 stream,
              network inet6 dgram,
              
              # File system access (restricted)
              /etc/cim-claude-adapter/** r,
              /var/lib/cim-claude-adapter/** rw,
              /var/log/cim-claude-adapter/** rw,
              /tmp/cim-claude-** rw,
              
              # Deny access to sensitive locations
              deny /etc/shadow r,
              deny /etc/passwd w,
              deny /etc/gshadow r,
              deny /etc/group w,
              deny /root/** rw,
              deny /home/** rw,
              deny /var/log/auth.log rw,
              
              # Required binaries
              /bin/cim-claude-adapter mr,
              /usr/bin/curl Cx -> curl,
              
              # Transition rules
              profile curl {
                #include <abstractions/base>
                #include <abstractions/nameservice>
                #include <abstractions/ssl_certs>
                
                network inet stream,
                network inet6 stream,
                
                /usr/bin/curl mr,
                /tmp/curl-** rw,
              }
            }
          '';
        };
      };
    };

    # Fail2ban configuration for brute force protection
    services.fail2ban = {
      enable = true;
      jails = {
        cim-claude-auth = ''
          enabled = true
          filter = cim-claude-auth
          logpath = ${cfg.compliance.auditLogPath}/auth.log
          maxretry = 5
          findtime = 600
          bantime = 3600
          action = iptables[name=cim-claude, port="8080,9090", protocol=tcp]
        '';
      };
    };

    # Custom fail2ban filter
    environment.etc."fail2ban/filter.d/cim-claude-auth.conf".text = ''
      [Definition]
      failregex = ^.*"auth_failed".*"ip":"<HOST>".*$
      ignoreregex =
    '';

    # Assertions
    assertions = [
      {
        assertion = cfg.tls.enable -> (cfg.tls.certificatePath != null && cfg.tls.keyPath != null);
        message = "TLS certificate and key paths must be specified when TLS is enabled";
      }
      {
        assertion = cfg.auth.enable && cfg.auth.method == "nats-jwt" -> cfg.auth.jwtConfig.keyPath != null;
        message = "JWT key path must be specified when using JWT authentication";
      }
      {
        assertion = length cfg.firewall.allowedClientIPs > 0 -> cfg.firewall.enable;
        message = "Firewall must be enabled when specifying allowed client IPs";
      }
    ];

    # Warnings for security configuration
    warnings = 
      optional (!cfg.tls.enable) "TLS is disabled - consider enabling for production deployments" ++
      optional (!cfg.auth.enable) "Authentication is disabled - this is not recommended for production" ++
      optional (!cfg.compliance.enable) "Compliance logging is disabled - may not meet regulatory requirements";
  };

  # Meta information
  meta = {
    maintainers = [ "Cowboy AI, LLC <security@cowboy-ai.com>" ];
    doc = ./security-hardening.md;
  };
}