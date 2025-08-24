# Universal Agent System Documentation Index

Welcome to the Universal Agent System documentation. This system provides a universal LLM abstraction layer using llm.rs, supporting multiple providers (Claude, GPT, Ollama, DeepSeek, etc.) with dynamic agent personalities loaded from markdown files.

## 📚 Documentation Structure

### 🏗️ Architecture Documentation
Core system architecture and design documents.

- **[Universal Agent System](architecture/universal-agent-system.md)** - Complete system overview built on llm.rs (NOT Claude-specific)
- **[Agent Architecture](architecture/agent-architecture.md)** - Dynamic agent personality system design
- **[LLM Adapter Design](architecture/llm-adapter-design.md)** - Universal LLM abstraction using llm.rs
- **[GUI Architecture](architecture/gui-architecture.md)** - Iced-based universal GUI design

### 🚀 Setup & Getting Started
Quick start guides and configuration instructions.

- **[Quick Start Guide](setup/quickstart.md)** - Get running in 5 minutes
- **[Multi-Provider Setup](setup/multi-provider-setup.md)** - Configure Claude, OpenAI, Ollama, and more
- **[Claude API Setup](setup/claude-api-setup.md)** - Configure Anthropic API key (one of many supported providers)
- **[End-to-End Setup](setup/end-to-end-setup.md)** - Complete system setup with SAGE

### 📋 Implementation Plans
Detailed implementation roadmaps and progress tracking.

- **[Implementation Plan](implementation/implementation-plan.md)** - Master implementation roadmap with daily progress
- **[Agent Implementation Plan](implementation/agent-implementation-plan.md)** - Detailed agent system implementation

### 🔧 API & Technical References
Technical documentation for developers.

- **[Claude API Comprehensive Design](claude-api-comprehensive-design.md)** - Detailed API integration patterns
- **[Conversation State Machine](conversation-state-machine.md)** - Dialog flow and state management

### 📦 Component Documentation

#### cim-llm-adapter
Universal LLM adapter service supporting multiple providers via llm.rs.

- Core service that abstracts Claude, GPT, Ollama, DeepSeek, and other LLMs
- NATS-based event-driven communication
- Dialog management with context preservation
- Provider-agnostic API design

#### cim-claude-gui  
Universal agent GUI supporting any agent personality.

- **[Message Rendering Implementation](../cim-claude-gui/MESSAGE_RENDERING_IMPLEMENTATION.md)** - Message display system
- Dynamic agent switching with preserved context
- Real-time NATS integration
- Iced framework with Elm Architecture

#### Agent System
Core agent personality and orchestration system.

- 19 specialized expert agents as markdown configurations
- SAGE master orchestrator (also just a configuration)
- Dynamic agent loading from `.claude/agents/*.md`
- Mathematical composition patterns using Category Theory

### 🗂️ Legacy Documentation
Older documentation kept for reference (may be outdated).

- [Architecture Refactor](legacy/architecture-refactor.md)
- [SAGE Implementation Complete](legacy/sage-implementation-complete.md)
- [SAGE Orchestration Patterns](legacy/sage-orchestration-patterns.md)
- [Universal Agent System Synthesis](legacy/universal-agent-system-synthesis.md)
- [NATS Infrastructure Overview](legacy/nats-infrastructure-overview.md)
- [Validation Summary](legacy/validation-summary.md)
- [Implementation Guide](legacy/implementation-guide.md)
- [SAGE Orchestration BDD](legacy/sage-orchestration.bdd.md)

## 🎯 Key Concepts

### Universal LLM Support via llm.rs
This system is **NOT** Claude-specific. It uses the llm.rs library to provide universal support for:
- **Claude** (Anthropic)
- **GPT** (OpenAI) 
- **Ollama** (Local models)
- **DeepSeek** (DeepSeek AI)
- **Any provider** supported by llm.rs

### Agent Personalities as Configuration
- SAGE and all subagents are **just markdown files** with YAML frontmatter
- No hard-coded agent logic - everything is dynamic
- New agents added by creating a `.md` file
- Agent capabilities extracted from metadata

### Mathematical Foundations
- **Category Theory** for clean agent composition
- **Graph Theory** for dialog flows and agent networks
- **Monadic transformations** for context preservation
- **Functors** for LLM provider abstraction

## 🚦 Current Status

| Component | Status | Documentation |
|-----------|--------|---------------|
| cim-llm-adapter | ✅ 95% Complete | [Design](architecture/llm-adapter-design.md) |
| Universal Agent System | ✅ 85% Complete | [Architecture](architecture/universal-agent-system.md) |
| Agent Loader | ✅ Complete | [Implementation](implementation/agent-implementation-plan.md) |
| SAGE V2 Service | ✅ Complete | [Setup Guide](setup/end-to-end-setup.md) |
| Universal GUI | ✅ 75% Complete | [Architecture](architecture/gui-architecture.md) |
| Documentation | ✅ Complete | This index |

## 🔍 Quick Navigation

### For New Users
1. Start with [Quick Start Guide](setup/quickstart.md)
2. Read [Universal Agent System Overview](architecture/universal-agent-system.md)
3. Follow [End-to-End Setup](setup/end-to-end-setup.md)

### For Developers
1. Review [LLM Adapter Design](architecture/llm-adapter-design.md)
2. Study [Agent Architecture](architecture/agent-architecture.md)
3. Check [Implementation Plan](implementation/implementation-plan.md)

### For System Administrators
1. Configure with [Claude API Setup](setup/claude-api-setup.md) (or other providers)
2. Deploy using scripts in `/scripts/`
3. Monitor with NATS observability tools

## 📝 Documentation Standards

All documentation follows these standards:
- **Lowercase filenames** (e.g., `quickstart.md` not `QUICKSTART.md`)
- **Organized in `/docs`** subdirectories by category
- **Cross-linked** for easy navigation
- **Accurate** - reflecting llm.rs foundation, not Claude-specific
- **Up-to-date** with current implementation status

## 🔗 External Resources

- [llm.rs Library](https://github.com/llm-rs) - Universal LLM library
- [NATS Documentation](https://docs.nats.io) - Message bus documentation
- [Iced Framework](https://github.com/iced-rs/iced) - GUI framework
- [Anthropic API](https://docs.anthropic.com) - Claude provider documentation (one of many supported)

---

*Last Updated: 2025-08-24*
*Documentation Version: 2.0.0*
*System: Universal Agent System with llm.rs*