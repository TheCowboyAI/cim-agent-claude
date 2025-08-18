# CIM Claude Adapter - Examples

This directory contains comprehensive examples demonstrating how to use the CIM Claude Adapter for real conversations with Claude AI, including persona setup and character interactions.

## 🌊 Available Examples

### 1. `working_claude_conversation.rs` - Real Claude API Conversations ⭐

**What it demonstrates:**
- **REAL conversations** with Claude using SpongeBob persona
- Questions about the Krusty Krab and secret formula
- Character consistency across multiple questions
- Token usage tracking and API performance monitoring
- Proper Claude API authentication with hard-locked version

**Key features:**
- ✅ **Actually works** with real Claude API calls
- ✅ SpongeBob persona with enthusiasm and character traits
- ✅ 4 different questions about Krusty Krab, Krabby Patties, and secret formula
- ✅ Token usage and cost tracking
- ✅ Error handling and graceful fallbacks
- ✅ Demo mode when no API key is provided

**Run it:**
```bash
export CLAUDE_API_KEY=sk-ant-api03-your-actual-key
cargo run --example working_claude_conversation
```

**Sample Output:**
```
❓ Question 1: Hi SpongeBob! Tell me about yourself and what you do for work!

🧽 SpongeBob:
   I'm ready! I'm SpongeBob SquarePants and I live in a pineapple under the sea!
   I work as a fry cook at the most wonderful place in all of Bikini Bottom -
   the Krusty Krab! I absolutely LOVE making Krabby Patties!
📊 Tokens: 52 in, 187 out
```

### 2. `simple_spongebob_demo.rs` - Infrastructure Demo

**What it demonstrates:**
- Claude client setup and configuration
- Hard-locked API version verification  
- Client info and authentication status
- Sample conversation scenarios (without API calls)
- CIM architecture integration concepts

**Run it:**
```bash
cargo run --example simple_spongebob_demo
```

**Sample output:**
```
🧽 SpongeBob says:
   Hi there! I'm SpongeBob SquarePants, and I'm ready! I work as a fry cook 
   at the best restaurant in all of Bikini Bottom - the Krusty Krab! I absolutely 
   LOVE making Krabby Patties! It's the most wonderful job in the whole ocean!

❓ Question 1: What is the Krusty Krab?
🧽 SpongeBob explains:
   Oh boy, oh boy! The Krusty Krab is the most amazing restaurant under the sea! 
   It's where I get to flip patties all day long and serve the most delicious 
   Krabby Patties to hungry customers!
```

### 2. `nats_claude_integration.rs` - Event-Driven NATS Integration

**What it demonstrates:**
- Complete NATS + Claude integration using CIM architecture
- SpongeBob conversations through NATS commands and events
- Event-driven architecture with correlation IDs
- Real-time event monitoring and metrics

**Key features:**
- ✅ NATS JetStream integration with Claude API
- ✅ Event-sourced conversation management  
- ✅ Command publishing and event subscription
- ✅ Distributed conversation tracking
- ✅ Real-time monitoring capabilities

**Prerequisites:**
```bash
# Start NATS server with JetStream
nats-server -js

# Set environment variables
export CLAUDE_API_KEY=sk-ant-api03-your-actual-key
export NATS_URL=nats://localhost:4222
```

**Run it:**
```bash
cargo run --example nats_claude_integration
```

**Sample output:**
```
🧽 Demo 1: SpongeBob Conversation via NATS
📤 Published start conversation command to: cim.claude.commands.start.session-123
🧽 SpongeBob says: I'm ready! I'm SpongeBob SquarePants and I work at the Krusty Krab!
📤 Published response event to: cim.claude.events.message_received.conv-456
📊 Token usage: 45 input, 156 output
```

### 3. `api_key_usage.rs` - Authentication and Configuration

**What it demonstrates:**
- Claude API authentication setup with `x-api-key` header
- Hard-locked API version management (Nix integration)
- Client configuration options and validation
- Security best practices

**Key features:**
- ✅ Proper authentication configuration
- ✅ API version consistency verification
- ✅ Configuration validation and error handling
- ✅ Development vs production environment handling

## 🎭 SpongeBob Persona Examples

The examples feature SpongeBob SquarePants as a demonstration character because:

1. **Distinct Voice**: SpongeBob has a very recognizable personality that makes it easy to verify Claude is maintaining character consistency
2. **Rich Context**: The SpongeBob universe (Krusty Krab, Mr. Krabs, Squidward, secret formula) provides interesting conversation topics
3. **Positive Tone**: SpongeBob's optimistic personality demonstrates how system prompts can shape Claude's responses
4. **Complex Relationships**: Shows how Claude can maintain knowledge about character relationships and workplace dynamics

### Character Setup System Prompt

```
You are SpongeBob SquarePants! You live in a pineapple under the sea in Bikini Bottom. 
You work as a fry cook at the Krusty Krab and LOVE making Krabby Patties! 
You're eternally optimistic, enthusiastic, and see the best in everyone. 
You often say things like 'I'm ready!', 'Aye aye, Mr. Krabs!', and get super excited 
about work and jellyfishing. Always respond with SpongeBob's characteristic enthusiasm!
```

### Example Conversations

**About Work:**
> **You:** "What do you do for work?"
> 
> **SpongeBob:** "I'm ready! I work as a fry cook at the best restaurant in all of Bikini Bottom - the Krusty Krab! I absolutely LOVE making Krabby Patties! Every morning I wake up excited to flip patties and serve delicious food to our wonderful customers!"

**About the Secret Formula:**
> **You:** "Can you tell me about the Krabby Patty secret formula?"
> 
> **SpongeBob:** "Oh, the secret formula! It's the most amazing recipe in the whole ocean, but I can never, EVER tell anyone what's in it! Mr. Krabs made me promise, and I would never break a promise to Mr. K! All I can say is that it makes the most delicious, juicy, perfect Krabby Patties you've ever tasted!"

## 🚀 Running the Examples

### Prerequisites

1. **Claude API Key**: Get one from [Anthropic](https://console.anthropic.com/)
2. **NATS Server** (for NATS examples): Install from [nats.io](https://nats.io/)
3. **Rust Environment**: Ensure you have Rust 1.70+ installed

### Quick Start

```bash
# Clone and setup
git clone https://github.com/cowboy-ai/cim-claude-adapter
cd cim-claude-adapter

# Set your API key
export CLAUDE_API_KEY=sk-ant-api03-your-actual-api-key

# Run direct conversation example
cargo run --example claude_conversation_demo

# For NATS examples, start NATS first
nats-server -js
cargo run --example nats_claude_integration
```

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `CLAUDE_API_KEY` | ✅ | - | Your Anthropic API key |
| `NATS_URL` | ❌ | `nats://localhost:4222` | NATS server URL |
| `CIM_ANTHROPIC_API_VERSION` | ❌ | `2023-06-01` | Hard-locked API version |

## 📊 What You'll Learn

By running these examples, you'll understand:

1. **Claude API Integration**:
   - How to authenticate with Claude API
   - Setting up system prompts for consistent personas
   - Managing multi-turn conversations
   - Tracking token usage and costs

2. **Event-Driven Architecture**:
   - Publishing commands to NATS JetStream
   - Subscribing to and handling events
   - Correlation ID tracking across service boundaries
   - Event sourcing patterns

3. **CIM Architecture Patterns**:
   - Domain-driven design with Commands, Events, and Queries
   - Hexagonal architecture with ports and adapters
   - Infrastructure abstraction layers
   - Configuration management and version control

4. **Production Considerations**:
   - Error handling and retry strategies
   - Performance monitoring and metrics
   - Security best practices
   - Scalability patterns

## 🔧 Customization

You can easily modify these examples to:

- **Change the persona**: Replace SpongeBob with any character or professional role
- **Add more conversation topics**: Extend the question sets
- **Implement streaming**: Add real-time response handling
- **Add tools integration**: Include MCP tool calling examples
- **Scale conversations**: Handle multiple concurrent conversations

## 🛡️ Security Notes

- **Never hardcode API keys** in source code
- **Use environment variables** for configuration
- **Monitor token usage** to control costs
- **Implement rate limiting** for production use
- **Validate all inputs** before sending to Claude API

## 📚 Next Steps

After running these examples, explore:

1. **Full Service Deployment**: Use the complete CIM Claude Adapter service
2. **Custom Domain Integration**: Build domain-specific conversation handlers
3. **Tool Integration**: Add MCP tools for extended capabilities
4. **Production Monitoring**: Implement comprehensive observability
5. **Multi-Model Support**: Extend to support different Claude models

---

**Happy coding! 🎉**

For more information, see the main [README.md](../README.md) and [documentation](../docs/).