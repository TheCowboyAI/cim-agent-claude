# Universal Agent System

A universal LLM agent orchestration system built on **llm.rs**, providing support for multiple LLM providers (Claude, GPT, Ollama, DeepSeek, etc.) with dynamic agent personality configuration.

## 🚀 Quick Start

```bash
# 1. Enter Nix development shell
nix develop

# 2. Set up your LLM provider API key (e.g., for Claude)
export ANTHROPIC_API_KEY="your-api-key"

# 3. Start the universal agent system
./scripts/start-universal-agent-system.sh
```

## 📚 Documentation

**[View Complete Documentation →](docs/index.md)**

### Key Documentation
- [Quick Start Guide](docs/setup/quickstart.md)
- [Universal Agent System Architecture](docs/architecture/universal-agent-system.md)
- [Implementation Plan & Progress](docs/implementation/implementation-plan.md)

## 🎯 Key Features

### Universal LLM Support
Built on **llm.rs** (NOT Claude-specific), supporting:
- Claude (Anthropic)
- GPT (OpenAI)
- Ollama (Local models)
- DeepSeek
- Any provider supported by llm.rs

### Dynamic Agent System
- **19 specialized expert agents** loaded from markdown files
- **SAGE master orchestrator** for multi-agent coordination
- **No hard-coded logic** - all agents are configurations
- **Universal GUI** that works with any agent personality

### Mathematical Foundations
- Category Theory for agent composition
- Graph Theory for dialog flows
- Monadic transformations for context preservation
- Functors for provider abstraction

## 📊 Current Status

| Component | Status | Description |
|-----------|--------|-------------|
| cim-llm-adapter | ✅ 95% | Universal LLM abstraction service |
| Agent System | ✅ 85% | Dynamic personality loading |
| SAGE V2 | ✅ Complete | Master orchestrator service |
| Universal GUI | ✅ 75% | Iced-based universal interface |
| Documentation | ✅ Complete | Comprehensive docs in `/docs` |

## 🏗️ Architecture

```
Universal Agent System
├── cim-llm-adapter/     # Universal LLM abstraction via llm.rs
├── src/agent_system/     # Core agent personality system
├── .claude/agents/       # 19 expert agent configurations
├── cim-claude-gui/       # Universal GUI (Iced framework)
└── docs/                 # Complete documentation
```

## 🛠️ Development

```bash
# Run tests
cargo test --workspace

# Build all components
cargo build --workspace

# Run specific service
cargo run --bin llm-adapter-service
cargo run --bin sage-service-v2
cargo run --bin cim-claude-gui
```

## 📖 Learn More

- [Complete Documentation](docs/index.md)
- [Implementation Roadmap](docs/implementation/implementation-plan.md)
- [Agent Architecture](docs/architecture/agent-architecture.md)
- [LLM Adapter Design](docs/architecture/llm-adapter-design.md)

## 📄 License

This project is part of the CIM (Composable Information Machine) ecosystem.

---

*Built with llm.rs for universal LLM support - not limited to any single provider*