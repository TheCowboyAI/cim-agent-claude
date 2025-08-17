#!/bin/bash

# Install Claude NATS Adapter as systemd service

set -e

echo "🔧 Installing Claude NATS Adapter Service"
echo "========================================"

# Build the service
echo "📦 Building service..."
cargo build --release

# Create service user
echo "👤 Creating service user..."
sudo useradd --system --no-create-home --shell /bin/false claude-adapter 2>/dev/null || echo "User already exists"

# Create directories
echo "📁 Creating directories..."
sudo mkdir -p /opt/claude-adapter
sudo mkdir -p /etc/claude-adapter
sudo mkdir -p /var/log/claude-adapter

# Copy binary
echo "📋 Installing binary..."
sudo cp target/release/simple-claude-adapter /opt/claude-adapter/
sudo chmod +x /opt/claude-adapter/simple-claude-adapter

# Copy systemd service
echo "⚙️  Installing systemd service..."
sudo cp claude-adapter.service /etc/systemd/system/

# Create config file
echo "📝 Creating config file..."
sudo tee /etc/claude-adapter/config > /dev/null << EOF
# Claude API configuration
CLAUDE_API_KEY=your-api-key-here
NATS_URL=nats://localhost:4222
RUST_LOG=info
EOF

# Set permissions
echo "🔒 Setting permissions..."
sudo chown -R claude-adapter:claude-adapter /opt/claude-adapter
sudo chown -R claude-adapter:claude-adapter /var/log/claude-adapter
sudo chmod 600 /etc/claude-adapter/config

# Reload systemd
echo "🔄 Reloading systemd..."
sudo systemctl daemon-reload

echo ""
echo "✅ Installation complete!"
echo ""
echo "Next steps:"
echo "1. Edit your API key: sudo nano /etc/claude-adapter/config"
echo "2. Start the service:   sudo systemctl start claude-adapter"
echo "3. Enable auto-start:   sudo systemctl enable claude-adapter"
echo "4. Check status:        sudo systemctl status claude-adapter"
echo "5. View logs:           sudo journalctl -u claude-adapter -f"
echo ""
echo "The service listens on: claude.cmd.*"
echo "Publishes events to:    claude.event.*"