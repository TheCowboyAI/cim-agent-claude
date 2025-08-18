# CIM Claude Adapter - API Version Management

## 🔒 Hard-Locked Anthropic API Version

The CIM Claude Adapter implements **hard-locked API version management** to ensure consistency across all deployments and prevent version drift.

## 📋 Implementation

### 1. Nix Flake Configuration

The API version is defined once in the Nix flake as the **single source of truth**:

```nix
# In flake.nix
# Hard-locked Anthropic API version for consistency
anthropicApiVersion = "2023-06-01";
```

### 2. Build-Time Injection

The version is injected at build time as a compile-time environment variable:

```nix
# Build-time environment variables
CIM_ANTHROPIC_API_VERSION = anthropicApiVersion;
```

### 3. Runtime Usage

The Rust code consumes this version at compile time:

```rust
// Anthropic API version - hard-locked via Nix flake, fallback for development
let api_version = option_env!("CIM_ANTHROPIC_API_VERSION").unwrap_or("2023-06-01");
headers.insert(
    "anthropic-version",
    HeaderValue::from_str(api_version)?
);
```

### 4. Version Verification

The version can be verified at runtime:

```rust
let version = ClaudeClient::anthropic_api_version();
println!("Using Anthropic API version: {}", version);
```

## 🎯 Benefits

### ✅ Consistency Guarantees

- **Same version everywhere**: All builds use identical API version
- **No version drift**: Prevents accidental upgrades breaking compatibility
- **Reproducible builds**: Identical builds across environments
- **Single point of control**: Change version in one place

### ✅ Development Safety

- **Explicit upgrades**: Version changes require intentional flake updates
- **Build-time binding**: Version compiled into binary
- **Runtime verification**: Can verify which version is active
- **Fallback support**: Graceful fallback for non-Nix builds

### ✅ Operations Excellence  

- **Deployment consistency**: Same API version across all environments
- **Rollback safety**: Version pinned with infrastructure code
- **Audit trail**: Version changes tracked in git
- **CI/CD integration**: Automated version validation

## 🚀 Usage

### Building with Nix (Production)

```bash
# Build with hard-locked version
nix build

# Version injected automatically
# Headers: anthropic-version: 2023-06-01
```

### Development (Cargo)

```bash
# Set version explicitly for development
export CIM_ANTHROPIC_API_VERSION="2023-06-01"
cargo run

# Or use fallback (development only)
cargo run  # Uses hardcoded fallback
```

### Docker/Container Builds

```dockerfile
# In Dockerfile
FROM nixos/nix:latest AS builder
COPY . /app
WORKDIR /app
RUN nix build
# Version automatically locked via flake
```

## 📊 Verification

### Runtime Version Check

```rust
use cim_claude_adapter::infrastructure::claude_client::ClaudeClient;

let version = ClaudeClient::anthropic_api_version();
let client_info = claude_client.config_info();
println!("API Version: {} ({})", 
    client_info.anthropic_api_version,
    if option_env!("CIM_ANTHROPIC_API_VERSION").is_some() { 
        "Nix-locked" 
    } else { 
        "fallback" 
    }
);
```

### CLI Verification

```bash
# Check version in running service
curl http://localhost:8080/health | jq '.anthropic_api_version'

# Check in logs
journalctl -u cim-claude-adapter | grep "API Version"
```

## 🔄 Updating API Version

### Step 1: Update Flake

```nix
# In flake.nix - change this one line
anthropicApiVersion = "2023-12-01";  # New version
```

### Step 2: Test & Validate

```bash
# Test the change
nix build
nix run . -- --version

# Run tests with new version
nix flake check
```

### Step 3: Deploy

```bash
# Rebuild and deploy
nix-rebuild switch

# Or redeploy containers
docker build . && docker push registry/cim-claude-adapter:latest
```

## 🛡️ Security Considerations

### Version Consistency

- **No version leakage**: Version controlled at build time
- **Audit compliance**: All version changes tracked
- **Security updates**: Coordinated API version updates

### Compatibility Management

- **Breaking change protection**: Explicit version updates only
- **Feature flag safety**: New API features require version bump
- **Deprecation handling**: Controlled migration between API versions

## 📚 References

- [Anthropic API Documentation](https://docs.anthropic.com/en/api)
- [Nix Flakes Manual](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html)
- [Rust Environment Variables](https://doc.rust-lang.org/std/macro.env.html)

---

**Last Updated**: January 2025  
**API Version**: 2023-06-01 (hard-locked)