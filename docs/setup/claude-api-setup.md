# Claude API Setup Guide

This guide walks you through configuring the Claude API key for SAGE service integration.

## 🔑 API Key Configuration

### Step 1: Add Your Claude API Key

The SAGE service expects your Claude API key to be stored in a secure location:

```bash
# Create the config directory if it doesn't exist
mkdir -p ~/.config/claude

# Add your Claude API key to the file
echo "sk-ant-api03-your-actual-api-key-here" > ~/.config/claude/api-key

# Secure the file permissions
chmod 600 ~/.config/claude/api-key
```

**Important**: Replace `sk-ant-api03-your-actual-api-key-here` with your actual Claude API key from [Anthropic Console](https://console.anthropic.com/).

### Step 2: Verify API Key Format

Your API key should:
- Start with `sk-ant-api03-`
- Be approximately 120+ characters long
- Contain only alphanumeric characters, hyphens, and underscores

## 🧪 Testing the Integration

### Quick Test

Run the automated integration test:

```bash
./scripts/test-claude-integration.sh
```

This test will:
1. ✅ Validate your API key
2. ✅ Check NATS connectivity  
3. ✅ Build and start SAGE service
4. ✅ Test Claude API integration
5. ✅ Verify expert agent coordination

### Manual Test

1. **Start SAGE service:**
   ```bash
   ./scripts/start-sage-service.sh
   ```

2. **Test with NATS CLI:**
   ```bash
   # In another terminal
   nix develop --command nats request commands.sage.request '{
     "request_id": "test-123",
     "query": "Explain CIM architecture principles",
     "expert": null,
     "context": {
       "session_id": "test",
       "conversation_history": [],
       "project_context": null
     }
   }' --timeout=30s
   ```

3. **Test with GUI:**
   ```bash
   nix develop --command cargo run --bin cim-claude-gui
   ```

## 🚨 Troubleshooting

### API Key Issues

**Problem**: "ANTHROPIC_API_KEY environment variable required"
**Solution**: Ensure your API key file exists and contains a valid key:
```bash
# Check if file exists and has content
ls -la ~/.config/claude/api-key
cat ~/.config/claude/api-key
```

**Problem**: "Claude API error: 401 Unauthorized"
**Solutions**:
- Verify your API key is correct
- Check if your Anthropic account has available credits
- Ensure the API key hasn't expired

### NATS Issues

**Problem**: "Failed to connect to NATS"
**Solutions**:
```bash
# Start local NATS server
nix develop --command nats-server --port 4222

# Or connect to existing server
export NATS_URL="nats://your-nats-server:4222"
```

### Service Issues

**Problem**: SAGE service not responding
**Solutions**:
1. Check service logs for errors
2. Verify all dependencies are built:
   ```bash
   nix develop --command cargo build --bin sage-service
   ```
3. Test NATS connectivity separately

## 🔒 Security Best Practices

1. **File Permissions**: Keep your API key file private:
   ```bash
   chmod 600 ~/.config/claude/api-key
   ```

2. **Environment Variables**: Never commit API keys to git:
   ```bash
   # Add to .gitignore
   echo "*.env" >> .gitignore
   echo ".env*" >> .gitignore
   ```

3. **Production Deployment**: Use proper secret management:
   - Kubernetes secrets
   - HashiCorp Vault
   - Cloud provider secret managers

## 📊 Expected Performance

With a properly configured Claude API:
- **Response Time**: 2-15 seconds (depending on query complexity)
- **Rate Limits**: Anthropic API limits apply
- **Fallback Mode**: Service continues with template responses if API is unavailable

## 🎭 SAGE Features with Claude API

When properly configured, SAGE provides:
- **Real Claude Responses**: Actual AI-powered guidance
- **Expert Agent Coordination**: Multi-agent synthesis
- **Context Awareness**: Conversation history and project context
- **CIM-Specific Knowledge**: Specialized CIM architectural guidance
- **Mathematical Foundations**: Category Theory and Graph Theory integration

## 📞 Support

If you encounter issues:
1. Run the integration test: `./scripts/test-claude-integration.sh`
2. Check SAGE consciousness: `ls -la .sage/`
3. Verify project structure: `ls -la .claude/agents/`

The integration is working correctly when you see real Claude responses with the SAGE orchestration formatting.