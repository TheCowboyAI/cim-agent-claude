# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Context

This is **cim-agent-claude**, a specialized Claude agent configuration repository for CIM (Composable Information Machine) development. It contains expert agents, instructions, and templates to guide CIM creation and domain modeling.

**Key Point**: This is NOT a traditional codebase with source code, but rather a configuration repository containing:
- Specialized expert agents (in `.claude/agents/`)
- CIM development instructions and patterns
- Domain creation templates and workflows
- NATS infrastructure configurations

## Common Commands

### Development Commands
```bash
# Check repository type and context
ls -la .claude/

# View available expert agents
ls .claude/agents/

# Start CIM development workflow
# Use the /sage command through Claude to orchestrate complete CIM development

# Access specific expert guidance
# Use /cim, /ddd, /eventstorming, /nix, /nats, /network, or /domain commands
```

### Agent System Commands
The repository uses Claude's Task tool to invoke specialized agents:
- `@sage` - Master orchestrator for complete CIM development
- `@cim-expert` - CIM architecture and mathematical foundations
- `@ddd-expert` - Domain-driven design and boundary analysis
- `@event-storming-expert` - Collaborative domain discovery
- `@nix-expert` - System configuration and infrastructure
- `@nats-expert` - Event streaming and message infrastructure  
- `@network-expert` - Network topology and infrastructure
- `@domain-expert` - Domain creation and validation

## Architecture Overview

### CIM Framework Structure
```
CIM Ecosystem:
├── Client (Local NATS) → Development environment
├── Leaf Node → Single server with multiple services
├── Cluster → 3+ leaf nodes for high availability  
└── Super-cluster → 3+ clusters for global distribution

CIM Development Approach:
├── ASSEMBLE existing cim-* modules (37+ available)
├── EXTEND with domain-specific functionality
├── CONFIGURE for specific business domains
└── DEPLOY using NATS-first architecture
```

### Agent Orchestration Workflow
1. **SAGE Orchestration**: Master agent routes requests to appropriate experts
2. **Multi-Agent Coordination**: Complex tasks involve multiple specialized agents
3. **Domain Discovery**: Event Storming → DDD Analysis → Domain Creation
4. **Infrastructure Setup**: NATS → Network → Nix Configuration
5. **Integration**: Unified guidance synthesized from all experts

## Key Principles

### CIM Development Standards
- **Assembly-First**: Use existing cim-* modules, don't rebuild from scratch
- **Event-Driven**: All state changes through immutable events, NO CRUD operations
- **NATS-First**: All communication via NATS messaging patterns
- **Domain-Focused**: Each CIM targets ONE specific business domain
- **Mathematical Foundation**: Based on Category Theory, Graph Theory, and IPLD

### Agent Interaction Patterns
- **PROACTIVE Guidance**: Agents automatically guide through CIM development journey
- **Collaborative Sessions**: Multiple agents work together on complex tasks
- **Context-Aware**: Agents understand current development stage and adapt guidance
- **Validation-Driven**: All outputs validated against CIM architectural principles

## File Structure

### Core Configuration Files
```
.claude/
├── CLAUDE.md                  # This file - enhanced development guidance
├── instructions.md            # Primary CIM development instructions
├── commands.md               # Slash commands for expert access
├── agents/                   # Specialized expert agent configurations
│   ├── sage.md              # Master orchestrator agent
│   ├── cim-expert.md        # Architecture and foundations
│   ├── ddd-expert.md        # Domain-driven design
│   ├── event-storming-expert.md  # Collaborative discovery
│   ├── nix-expert.md        # System configuration
│   ├── nats-expert.md       # Event infrastructure
│   ├── network-expert.md    # Network topology
│   └── domain-expert.md     # Domain creation
├── instructions/            # Detailed operational guidance
├── contexts/               # Context-specific instructions
└── settings.local.json     # Claude Code permissions configuration
```

## Getting Started

### For New Users
1. Start with `/sage` command to begin complete CIM development journey
2. SAGE will assess your needs and coordinate appropriate expert agents
3. Follow the guided workflow through domain discovery, infrastructure setup, and implementation

### For Experienced Users  
1. Use direct expert commands (`/cim`, `/ddd`, `/nats`, etc.) for specific needs
2. Invoke multiple agents via SAGE for complex multi-domain tasks
3. Reference existing instructions in `.claude/instructions/` for detailed guidance

### For Development Teams
1. Use `/eventstorming` to facilitate collaborative domain discovery sessions
2. SAGE coordinates multi-agent workflows for team-based CIM development  
3. Agents provide structured guidance for distributed team collaboration

## Special Considerations

### Date Handling
- **NEVER generate dates from memory**
- Always use system date: `$(date -I)`
- Use git commit dates: `$(git log -1 --format=%cd --date=short)`
- Capture current date in variables before JSON updates

### Context Detection
- Repository type determines development approach
- Use `.claude/scripts/detect-context.sh` if available
- `cim-agent-claude` = Agent configuration repository
- `cim-*` = Module repository  
- `cim-domain-*` = Domain implementation repository

### Security and Permissions
- Configured permissions in `.claude/settings.local.json`
- Specific bash commands allowed for CIM development workflows
- WebFetch limited to github.com domain for module research

This repository serves as the intelligent orchestration layer for CIM development, providing expert guidance through specialized agents that coordinate the complete journey from domain discovery to production deployment.