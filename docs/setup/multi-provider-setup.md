# Multi-Provider LLM Setup Guide

This guide explains how to configure and use multiple LLM providers in the Universal Agent System.

## 🌐 Supported Providers

The system supports multiple LLM providers through a unified interface:

### Paid API Providers
- **Claude (Anthropic)** - Best for complex reasoning and analysis
- **OpenAI (GPT-4)** - Excellent for general tasks and code generation
- **HuggingFace** - Access to various open models (with API key)

### Free/Local Providers
- **Ollama** - Run models locally (Llama2, Mistral, Vicuna, CodeLlama, etc.)
- **LM Studio** - Local model hosting (coming soon)
- **LocalAI** - OpenAI-compatible local API (coming soon)

## 🔑 API Key Configuration

### Claude (Anthropic)
```bash
# Add your Claude API key
echo "sk-ant-api03-YOUR-KEY-HERE" > cim-llm-adapter/secrets/claude.api.key
chmod 600 cim-llm-adapter/secrets/claude.api.key
```

### OpenAI
```bash
# Add your OpenAI API key
echo "sk-YOUR-OPENAI-KEY-HERE" > cim-llm-adapter/secrets/openai.api.key
chmod 600 cim-llm-adapter/secrets/openai.api.key
```

### Environment Variables (Alternative)
```bash
export ANTHROPIC_API_KEY="sk-ant-api03-YOUR-KEY"
export OPENAI_API_KEY="sk-YOUR-OPENAI-KEY"
```

## 🐳 Ollama Setup (Local Models)

### Install Ollama
```bash
# On Linux/Mac
curl -fsSL https://ollama.ai/install.sh | sh

# Or via Nix
nix-shell -p ollama
```

### Start Ollama Service
```bash
ollama serve
```

### Install Models
```bash
# Recommended models for CIM development
ollama pull llama2:7b        # General purpose, 7B parameters
ollama pull mistral:7b       # Fast and efficient
ollama pull vicuna           # Fine-tuned for conversation
ollama pull codellama:7b     # Specialized for code
ollama pull phi-2            # Small but capable (2.7B)

# Larger models (require more RAM)
ollama pull llama2:13b       # Better quality, needs 16GB RAM
ollama pull mixtral:8x7b     # MoE model, very capable
```

### Check Available Models
```bash
ollama list
```

## ⚙️ Provider Configuration

The system uses `providers.toml` for configuration:

```toml
# cim-llm-adapter/providers.toml

[providers.claude]
enabled = true
type = "claude"
api_key_file = "secrets/claude.api.key"
model = "claude-3-5-sonnet-20241022"
max_tokens = 4096

[providers.openai]
enabled = true
type = "openai"
api_key_file = "secrets/openai.api.key"
model = "gpt-4-turbo-preview"
max_tokens = 4096

[providers.ollama]
enabled = true
type = "ollama"
base_url = "http://localhost:11434"

[providers.ollama.models.llama2]
name = "llama2:7b"
context_size = 4096
use_for = ["general", "conversation"]

[providers.ollama.models.mistral]
name = "mistral:7b"
context_size = 8192
use_for = ["code", "analysis", "reasoning"]
```

## 🎯 Agent-to-Provider Mapping

Map specific agents to optimal providers:

```toml
[agent_mapping]
sage = "claude"              # SAGE needs best reasoning
cim-expert = "claude"        # Complex architecture discussions
ddd-expert = "claude"        # Domain modeling
tdd-expert = "openai"        # GPT-4 excellent for tests
git-expert = "codellama"     # Code-specific tasks
qa-expert = "mistral"        # Good balance for QA
iced-ui-expert = "codellama" # UI code generation
```

## 🧪 Testing Your Setup

### 1. Test Individual Providers
```bash
# Run the multi-provider test suite
./scripts/test-multi-provider.sh
```

### 2. Test Claude
```bash
nats request "cim.llm.dialog.turn.request" '{
  "provider": "claude",
  "messages": [{"role": "user", "content": "Hello"}]
}'
```

### 3. Test OpenAI
```bash
nats request "cim.llm.dialog.turn.request" '{
  "provider": "openai",
  "model": "gpt-4-turbo-preview",
  "messages": [{"role": "user", "content": "Hello"}]
}'
```

### 4. Test Ollama
```bash
nats request "cim.llm.dialog.turn.request" '{
  "provider": "ollama",
  "model": "llama2:7b",
  "messages": [{"role": "user", "content": "Hello"}]
}'
```

## 📊 Performance Profiles

### Fast Profile (Quick responses)
- Primary: Phi-2 (local, 2.7B)
- Fallback: Mistral (local, 7B)
- Use for: Simple queries, quick feedback

### Balanced Profile
- Primary: Mistral (local, 7B)
- Fallback: GPT-3.5-turbo (API)
- Use for: Most development tasks

### Quality Profile
- Primary: Claude 3.5 Sonnet (API)
- Fallback: GPT-4 (API)
- Use for: Complex reasoning, architecture

### Local-Only Profile
- Primary: Mistral (local)
- Fallback: Llama2 (local)
- Use for: Privacy-sensitive work, offline

## 🔄 Fallback Strategy

The system automatically falls back to alternative providers:

```
Primary Provider Fails → Fallback Chain:
Claude → OpenAI → Mistral → Llama2
```

Configure in `providers.toml`:
```toml
[strategy]
default = "claude"
fallback_chain = ["claude", "openai", "mistral", "llama2"]
```

## 💰 Cost Optimization

### Minimize API Costs
1. Use local models for development/testing
2. Reserve Claude/GPT-4 for complex tasks
3. Cache responses in NATS KV store
4. Batch similar requests

### Recommended Setup
- **Development**: Ollama with Mistral/Llama2
- **Testing**: Mix of local and API providers
- **Production**: Claude for SAGE, local for simple agents

## 🚀 Advanced Features

### Model Comparison
```bash
# Compare responses from different models
./scripts/compare-models.sh "Your query here"
```

### Load Balancing
```toml
[load_balancing]
strategy = "round_robin"  # or "least_latency", "weighted"
providers = ["ollama", "openai", "claude"]
```

### Custom Providers
Add new providers by implementing the `LlmProvider` trait:
```rust
impl LlmProvider for YourProvider {
    async fn complete(...) -> Result<ProviderResponse, ProviderError>
    async fn health_check(...) -> Result<ProviderHealth, ProviderError>
}
```

## 📈 Monitoring

### Check Provider Health
```bash
# Health status of all providers
nats request "cim.llm.health.check.all" '{}'
```

### View Provider Metrics
```bash
# Get usage statistics
nats request "cim.llm.metrics" '{}'
```

## 🔧 Troubleshooting

### Ollama Not Responding
```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# Restart Ollama
ollama serve

# Check logs
journalctl -u ollama -f
```

### API Key Issues
```bash
# Verify key files exist
ls -la cim-llm-adapter/secrets/

# Test API keys directly
curl -H "x-api-key: $(cat cim-llm-adapter/secrets/claude.api.key)" \
     https://api.anthropic.com/v1/messages
```

### Provider Selection Issues
```bash
# Check provider configuration
cat cim-llm-adapter/providers.toml

# View active providers
nats request "cim.llm.providers.list" '{}'
```

## 📝 Best Practices

1. **Start with Local Models**: Test with Ollama before using paid APIs
2. **Monitor Usage**: Track token usage to control costs
3. **Cache Aggressively**: Use NATS KV for response caching
4. **Profile Selection**: Choose appropriate profile for task complexity
5. **Regular Health Checks**: Monitor provider availability
6. **Fallback Testing**: Regularly test fallback chains

## 🎉 Next Steps

1. Install and configure your preferred providers
2. Run the multi-provider test suite
3. Configure agent-to-provider mappings
4. Test multi-agent workflows with different providers
5. Optimize for your specific use cases

The multi-provider system gives you flexibility to:
- Use the best model for each task
- Control costs with local models
- Ensure availability with fallbacks
- Scale with load balancing