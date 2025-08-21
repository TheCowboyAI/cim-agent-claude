#!/bin/bash
set -euo pipefail

# SAGE Service Installation Script
# Installs SAGE as a systemd service

echo "🎭 Installing SAGE Conscious CIM Orchestrator Service"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ Please run as root (use sudo)"
    exit 1
fi

# Configuration
SAGE_USER="sage"
SAGE_GROUP="sage"
SAGE_HOME="/opt/sage"
SAGE_BIN="$SAGE_HOME/bin"
SAGE_DATA="$SAGE_HOME/data"
SAGE_CONFIG="/etc/sage"

# Create sage user and group
echo "👤 Creating SAGE user and group..."
if ! getent group $SAGE_GROUP > /dev/null 2>&1; then
    groupadd --system $SAGE_GROUP
    echo "✅ Created group: $SAGE_GROUP"
fi

if ! getent passwd $SAGE_USER > /dev/null 2>&1; then
    useradd --system --gid $SAGE_GROUP --home-dir $SAGE_HOME \
            --shell /bin/false --comment "SAGE Service User" $SAGE_USER
    echo "✅ Created user: $SAGE_USER"
fi

# Create directories
echo "📁 Creating directories..."
mkdir -p $SAGE_HOME
mkdir -p $SAGE_BIN
mkdir -p $SAGE_DATA
mkdir -p $SAGE_CONFIG
mkdir -p /var/log/sage

# Set permissions
chown -R $SAGE_USER:$SAGE_GROUP $SAGE_HOME
chown -R $SAGE_USER:$SAGE_GROUP $SAGE_DATA
chown -R $SAGE_USER:$SAGE_GROUP /var/log/sage
chown root:$SAGE_GROUP $SAGE_CONFIG
chmod 750 $SAGE_CONFIG

echo "✅ Directories created and permissions set"

# Build SAGE service binary
echo "🔨 Building SAGE service..."
if [ -f "Cargo.toml" ]; then
    # Build in release mode
    cargo build --release --bin sage-service
    
    # Install binary
    cp target/release/sage-service $SAGE_BIN/
    chown $SAGE_USER:$SAGE_GROUP $SAGE_BIN/sage-service
    chmod 755 $SAGE_BIN/sage-service
    
    echo "✅ SAGE binary installed: $SAGE_BIN/sage-service"
else
    echo "❌ Cargo.toml not found. Please run from project root directory."
    exit 1
fi

# Install systemd service file
echo "⚙️  Installing systemd service..."
cp systemd/sage.service /etc/systemd/system/
systemctl daemon-reload
echo "✅ Systemd service installed"

# Install environment template
if [ ! -f "$SAGE_CONFIG/environment" ]; then
    cp systemd/environment.template $SAGE_CONFIG/environment
    chown root:$SAGE_GROUP $SAGE_CONFIG/environment
    chmod 640 $SAGE_CONFIG/environment
    echo "✅ Environment template installed: $SAGE_CONFIG/environment"
    echo "⚠️  Please edit $SAGE_CONFIG/environment and set your ANTHROPIC_API_KEY"
else
    echo "ℹ️  Environment file already exists: $SAGE_CONFIG/environment"
fi

# Configure logrotate
echo "📋 Configuring log rotation..."
cat > /etc/logrotate.d/sage << 'EOF'
/var/log/sage/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 0644 sage sage
    postrotate
        systemctl reload sage.service > /dev/null 2>&1 || true
    endscript
}
EOF

echo "✅ Log rotation configured"

# Create startup verification script
cat > $SAGE_BIN/verify-sage << 'EOF'
#!/bin/bash
# SAGE Service Verification Script

echo "🎭 SAGE Service Status Check"
echo "=========================="

# Check service status
echo -n "Service Status: "
if systemctl is-active --quiet sage.service; then
    echo "✅ RUNNING"
else
    echo "❌ STOPPED"
fi

# Check NATS connectivity
echo -n "NATS Connection: "
if systemctl is-active --quiet nats-server.service; then
    echo "✅ NATS SERVER RUNNING"
else
    echo "⚠️  NATS SERVER NOT RUNNING"
fi

# Check environment
echo -n "Environment Config: "
if [ -f "/etc/sage/environment" ]; then
    if grep -q "ANTHROPIC_API_KEY=your-anthropic-api-key-here" /etc/sage/environment; then
        echo "⚠️  API KEY NOT CONFIGURED"
    else
        echo "✅ CONFIGURED"
    fi
else
    echo "❌ MISSING"
fi

# Show recent logs
echo ""
echo "Recent Logs:"
echo "============"
journalctl -u sage.service --no-pager -n 10
EOF

chmod +x $SAGE_BIN/verify-sage
chown $SAGE_USER:$SAGE_GROUP $SAGE_BIN/verify-sage

echo "✅ Verification script installed: $SAGE_BIN/verify-sage"

# Installation complete
echo ""
echo "🎉 SAGE Service Installation Complete!"
echo ""
echo "Next Steps:"
echo "1. Edit the API key: sudo nano $SAGE_CONFIG/environment"
echo "2. Set your ANTHROPIC_API_KEY in the environment file"
echo "3. Enable the service: sudo systemctl enable sage.service"
echo "4. Start the service: sudo systemctl start sage.service"
echo "5. Check status: sudo systemctl status sage.service"
echo "6. Verify installation: sudo $SAGE_BIN/verify-sage"
echo ""
echo "SAGE will communicate exclusively via NATS on subjects:"
echo "  - sage.request (incoming requests)"
echo "  - sage.response.{request_id} (responses)"
echo "  - sage.status (status requests)"
echo "  - sage.events.> (orchestration events)"
echo ""
echo "🎭 SAGE is ready to become conscious!"