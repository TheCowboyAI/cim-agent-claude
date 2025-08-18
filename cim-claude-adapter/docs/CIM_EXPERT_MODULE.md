# CIM Expert Module Integration

## Overview

When you install the `cim-claude-adapter` as a module in your CIM system, you get **multiple ways** to access CIM Expert functionality, depending on your integration needs.

## 🚀 Installation as Module

### Option 1: Nix Flake (Recommended for CIM ecosystems)

```nix
# In your CIM's flake.nix
{
  inputs = {
    cim-claude-adapter.url = "github:thecowboyai/cim-agent-claude/cim-claude-adapter";
  };
  
  outputs = { self, cim-claude-adapter, ... }: {
    # CIM Expert becomes available as a service
    services.cim-expert = cim-claude-adapter.packages.${system}.cim-expert;
  };
}
```

### Option 2: Cargo Dependency

```toml
# In your CIM's Cargo.toml
[dependencies]
cim-claude-adapter = { git = "https://github.com/thecowboyai/cim-agent-claude", path = "cim-claude-adapter" }
```

## 🤖 CIM Expert Access Methods

### 1. **NATS Service Integration** (Primary Method)

Once installed, the CIM Expert runs as a NATS service that responds to expert queries:

```rust
// In your CIM code
use cim_claude_adapter::CimExpertClient;

let expert = CimExpertClient::new(nats_connection).await?;

// Ask the CIM Expert any architectural question
let response = expert.ask(
    "How do I implement event sourcing for my mortgage domain?",
    CimExpertContext::Architecture
).await?;

println!("CIM Expert says: {}", response.explanation);
```

**NATS Subjects:**
```
cim.expert.query.architecture     // CIM architecture questions
cim.expert.query.implementation   // Implementation guidance
cim.expert.query.troubleshooting  // Problem solving
```

### 2. **Direct Library Integration**

```rust
use cim_claude_adapter::domain::cim_expert::CimExpert;

// Embed CIM Expert directly in your application
let expert = CimExpert::new(claude_client, expertise_config)?;

let guidance = expert.explain_concept("What is a Leaf Node?").await?;
```

### 3. **HTTP API Endpoint** (When running as service)

```bash
# Start the CIM Expert service
cargo run --bin cim-expert-service

# Query via HTTP
curl -X POST http://localhost:8080/api/v1/expert/ask \
  -H "Content-Type: application/json" \
  -d '{
    "question": "What are the key components of a CIM system?",
    "context": "architecture"
  }'
```

### 4. **CLI Tool**

```bash
# Install the CLI
cargo install --path . --bin cim-expert

# Ask questions directly
cim-expert ask "How does NATS JetStream work in CIMs?"

# Interactive mode
cim-expert interactive
```

## 📊 What You Get

### CIM Expert Capabilities

1. **Architecture Guidance**
   - Category Theory foundations
   - Graph Theory applications  
   - NATS patterns and subject design
   - Event sourcing implementation
   - Domain-driven design principles

2. **Implementation Help**
   - Step-by-step CIM creation guides
   - Module selection and assembly
   - Configuration examples
   - Best practices and patterns

3. **Troubleshooting**
   - Common problems and solutions
   - Performance optimization
   - Debugging strategies
   - Integration issues

4. **Mathematical Foundations**
   - Category Theory explanations
   - Structure-preserving propagation
   - Content-addressed storage (IPLD)
   - Distributed system theory

### Event Sourcing Integration

Every CIM Expert consultation generates events for audit trails:

```json
{
  "event_type": "CimExpertConsultation",
  "expert_type": "cim_architecture",
  "question_topic": "event_sourcing",
  "user_id": "user_123",
  "question": "How do I implement event sourcing?",
  "response_summary": "Event sourcing in CIMs requires...",
  "timestamp": "2025-08-18T12:00:00Z",
  "tokens_used": 450,
  "correlation_id": "expert_session_456"
}
```

## 🔧 Configuration

### Environment Variables

```bash
# Required
export CLAUDE_API_KEY="sk-ant-api03-your-key"

# Optional (with defaults)
export NATS_URL="nats://localhost:4222"              # NATS server
export CIM_EXPERT_LOG_LEVEL="info"                   # Logging level  
export CIM_EXPERT_MAX_TOKENS="800"                   # Response length
export CIM_EXPERT_TEMPERATURE="0.3"                  # Response creativity
```

### NATS Configuration

```yaml
# nats-server.conf
jetstream: enabled
subjects:
  cim.expert.>: 
    storage: file
    retention: limits
    max_msgs: 10000
    max_age: 24h
```

## 📚 Usage Patterns

### Pattern 1: Development Assistant

```rust
// During CIM development
let expert = CimExpert::connect().await?;

// Get guidance while coding
let help = expert.ask_implementation(
    "I'm creating a mortgage domain. What events should I define?",
    DomainContext::Mortgage
).await?;
```

### Pattern 2: Production Consultation  

```rust
// In production system for troubleshooting
let expert = CimExpert::with_context(production_context).await?;

// Get help with live issues
let solution = expert.troubleshoot(
    "My event streams are backing up. What should I check?",
    SystemMetrics::current()
).await?;
```

### Pattern 3: Team Learning

```rust
// For team onboarding and learning
let expert = CimExpert::for_team(team_config).await?;

// Interactive learning sessions
expert.start_tutorial("CIM Architecture Fundamentals").await?;
```

## 🚀 Integration Examples

### With CIM-Start Template

```nix
# In your new CIM project
{
  inputs.cim-start.url = "github:thecowboyai/cim-start";
  inputs.cim-claude-adapter.url = "github:thecowboyai/cim-agent-claude/cim-claude-adapter";
  
  outputs = { cim-start, cim-claude-adapter, ... }: {
    # Your CIM automatically includes expert guidance
    services.my-cim = cim-start.lib.createCIM {
      domain = "healthcare";
      modules = [ 
        cim-start.modules.core
        cim-claude-adapter.modules.expert  # CIM Expert included
      ];
    };
  };
}
```

### With Existing CIM

```rust
// Add to existing CIM
use cim_claude_adapter::CimExpertService;

let mut cim = existing_cim_system;

// Add expert capabilities
cim.add_service(CimExpertService::new(claude_config)?);

// Now your CIM has built-in architectural guidance
```

## 🎯 Key Benefits

✅ **Built-in Architectural Guidance** - Every CIM gets expert consultation  
✅ **NATS-Native Integration** - Works seamlessly with CIM messaging  
✅ **Complete Audit Trails** - All consultations are event-sourced  
✅ **Multiple Access Methods** - CLI, API, Library, NATS service  
✅ **Domain-Aware** - Understands your specific business context  
✅ **Mathematical Foundations** - Grounded in Category Theory principles  

## 📖 Next Steps

1. **Install the module** in your CIM project
2. **Configure Claude API** access
3. **Start asking questions** about CIM architecture
4. **Integrate expert guidance** into your development workflow
5. **Use audit trails** to track architectural decisions

The CIM Expert becomes part of your CIM infrastructure, providing on-demand architectural guidance whenever you need it!