# Contributing to CIM Claude Adapter

*Copyright 2025 - Cowboy AI, LLC. All rights reserved.*

Thank you for your interest in contributing to the CIM Claude Adapter! This document provides guidelines and information for contributors.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Environment](#development-environment)
4. [Contributing Process](#contributing-process)
5. [Coding Standards](#coding-standards)
6. [Testing Guidelines](#testing-guidelines)
7. [Documentation](#documentation)
8. [Pull Request Process](#pull-request-process)
9. [Issue Reporting](#issue-reporting)
10. [Community](#community)

## Code of Conduct

### Our Pledge

We are committed to making participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, sex characteristics, gender identity and expression, level of experience, education, socio-economic status, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

Examples of behavior that contributes to creating a positive environment include:

- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

Examples of unacceptable behavior include:

- The use of sexualized language or imagery and unwelcome sexual attention or advances
- Trolling, insulting/derogatory comments, and personal or political attacks
- Public or private harassment
- Publishing others' private information without explicit permission
- Other conduct which could reasonably be considered inappropriate in a professional setting

## Getting Started

### Prerequisites

- Rust 1.70 or later
- NATS Server 2.9+ with JetStream
- Claude API access
- Git
- Docker (optional, for containerized development)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/cim-claude-adapter.git
   cd cim-claude-adapter
   ```

3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/cowboy-ai/cim-claude-adapter.git
   ```

## Development Environment

### Local Setup

1. **Install Rust**: Follow instructions at [rustup.rs](https://rustup.rs/)

2. **Install NATS Server**:
   ```bash
   # macOS
   brew install nats-server
   
   # Linux
   wget https://github.com/nats-io/nats-server/releases/latest/download/nats-server-linux-amd64.zip
   unzip nats-server-linux-amd64.zip
   sudo mv nats-server /usr/local/bin/
   ```

3. **Start NATS with JetStream**:
   ```bash
   nats-server -js
   ```

4. **Set Environment Variables**:
   ```bash
   export CLAUDE_API_KEY="your-test-api-key"
   export NATS_URL="nats://localhost:4222"
   export RUST_LOG="debug"
   ```

5. **Build and Test**:
   ```bash
   cargo build
   cargo test
   ```

### Docker Development

```bash
# Start development environment
docker-compose up -d

# Run tests
docker-compose exec adapter cargo test

# View logs
docker-compose logs -f adapter
```

## Contributing Process

### 1. Choose an Issue

- Look for issues labeled `good first issue` for beginners
- Check existing issues or create a new one for discussion
- Comment on the issue to indicate you're working on it

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 3. Make Changes

- Follow the coding standards below
- Write tests for new functionality
- Update documentation as needed
- Ensure all tests pass

### 4. Commit Changes

Use conventional commit messages:

```bash
git commit -m "feat: add conversation timeout handling"
git commit -m "fix: resolve rate limit edge case"
git commit -m "docs: update API documentation"
git commit -m "test: add integration tests for error scenarios"
```

**Commit Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test changes
- `refactor`: Code refactoring
- `style`: Code style changes
- `chore`: Build/tooling changes

## Coding Standards

### Rust Code Style

Follow the official Rust style guide:

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Check for security issues
cargo audit
```

### Code Organization

```
src/
├── lib.rs              # Public API and module exports
├── domain/             # Domain logic (DDD)
│   ├── mod.rs
│   ├── aggregates.rs   # Domain aggregates
│   ├── events.rs       # Domain events
│   ├── commands.rs     # Domain commands
│   ├── value_objects.rs # Value objects
│   └── errors.rs       # Domain errors
├── application/        # Application services
│   ├── mod.rs
│   └── services.rs
├── infrastructure/     # External integrations
│   ├── mod.rs
│   ├── nats.rs        # NATS adapter
│   ├── claude_api.rs  # Claude API client
│   └── config.rs      # Configuration
├── ports/              # Interface definitions
│   └── mod.rs
└── adapters/           # External interface implementations
    └── mod.rs
```

### Documentation Standards

- All public APIs must have doc comments
- Use `///` for doc comments, `//` for implementation notes
- Include examples in doc comments where helpful
- Keep README and docs/ folder updated

```rust
/// Starts a new conversation with Claude AI.
/// 
/// # Arguments
/// 
/// * `command` - The StartConversation command with session ID and prompt
/// 
/// # Returns
/// 
/// Returns a `Result` containing the conversation ID on success.
/// 
/// # Errors
/// 
/// Returns `DomainError::ValidationError` if the prompt is invalid.
/// Returns `DomainError::RateLimitExceeded` if rate limits are hit.
/// 
/// # Example
/// 
/// ```rust
/// let command = StartConversation {
///     session_id: "user-123".to_string(),
///     initial_prompt: "Hello Claude".to_string(),
///     context: ConversationContext::default(),
///     correlation_id: CorrelationId::new(),
/// };
/// 
/// let conversation_id = service.start_conversation(command).await?;
/// ```
pub async fn start_conversation(&self, command: StartConversation) -> Result<ConversationId, DomainError> {
    // Implementation
}
```

## Testing Guidelines

### Test Categories

1. **Unit Tests**: Test individual functions/methods
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflows

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_start_conversation_success() {
        // Arrange
        let service = setup_test_service().await;
        let command = create_test_command();
        
        // Act
        let result = service.start_conversation(command).await;
        
        // Assert
        assert!(result.is_ok());
        let conversation_id = result.unwrap();
        assert!(!conversation_id.as_uuid().is_nil());
    }
    
    #[tokio::test]
    async fn test_start_conversation_invalid_prompt() {
        // Arrange
        let service = setup_test_service().await;
        let command = create_invalid_command();
        
        // Act
        let result = service.start_conversation(command).await;
        
        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => {
                assert!(msg.contains("prompt"));
            },
            _ => panic!("Expected ValidationError"),
        }
    }
}
```

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# With coverage
cargo tarpaulin --out Html

# Performance tests
cargo bench
```

### Test Environment

Use test containers or mocks for external dependencies:

```rust
#[tokio::test]
async fn test_with_mock_claude_api() {
    let mut mock_api = MockClaudeApi::new();
    mock_api
        .expect_send_prompt()
        .returning(|_| Ok(create_test_response()));
    
    let service = ConversationService::new(Box::new(mock_api));
    // Test logic
}
```

## Documentation

### Required Documentation Updates

When contributing, update relevant documentation:

- **README.md**: For significant feature changes
- **docs/API.md**: For API changes
- **docs/USER_GUIDE.md**: For user-facing features
- **docs/DESIGN.md**: For architectural changes
- **Inline documentation**: For all public APIs

### Documentation Style

- Use clear, concise language
- Include practical examples
- Keep it up-to-date with code changes
- Use proper markdown formatting

## Pull Request Process

### Before Submitting

1. **Rebase on latest main**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run full test suite**:
   ```bash
   cargo test
   cargo fmt --check
   cargo clippy -- -D warnings
   ```

3. **Update documentation** as needed

4. **Add entry to CHANGELOG.md** (if applicable)

### PR Requirements

- [ ] Tests pass locally and in CI
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Commits are squashed and have clear messages
- [ ] PR description explains the changes

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or breaking changes documented)
```

### Review Process

1. **Automated checks**: CI must pass
2. **Peer review**: At least one approval required
3. **Maintainer review**: For significant changes
4. **Integration testing**: In staging environment

## Issue Reporting

### Bug Reports

Use the bug report template:

```markdown
## Bug Description
Clear description of the bug

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- OS: [e.g. Linux, macOS, Windows]
- Rust version: [e.g. 1.70.0]
- NATS version: [e.g. 2.9.15]
- Adapter version: [e.g. 0.1.0]

## Additional Context
Any other relevant information
```

### Feature Requests

Use the feature request template:

```markdown
## Feature Description
Clear description of the proposed feature

## Use Case
Why is this feature needed? What problem does it solve?

## Proposed Solution
How should this feature work?

## Alternatives Considered
Other approaches you've considered

## Additional Context
Any other relevant information
```

## Community

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: General questions, ideas
- **Email**: security@cowboy-ai.com (for security issues)

### Getting Help

1. **Check existing documentation**
2. **Search existing issues**
3. **Ask in GitHub Discussions**
4. **Create a new issue** if needed

### Recognition

Contributors will be:
- Listed in the CONTRIBUTORS.md file
- Mentioned in release notes
- Acknowledged in project documentation

## Security

### Reporting Security Issues

**Do not report security vulnerabilities through public GitHub issues.**

Instead, please email: security@cowboy-ai.com

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Security Guidelines

- Never commit secrets or API keys
- Follow secure coding practices
- Keep dependencies updated
- Use proper authentication and authorization

## License

By contributing to this project, you agree that your contributions will be licensed under the MIT License.

## Questions?

If you have questions about contributing, please:

1. Check this document first
2. Look at existing issues and discussions
3. Create a new discussion or issue
4. Contact the maintainers

Thank you for contributing to CIM Claude Adapter!