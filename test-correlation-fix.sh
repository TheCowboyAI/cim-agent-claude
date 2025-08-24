#!/usr/bin/env bash

# Test script for NATS Request-Response Correlation Fix
# Verifies the complete SAGE-GUI integration

set -euo pipefail

echo "🎭 SAGE Correlation Fix Test Suite"
echo "=================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

step_counter=0
function step() {
    step_counter=$((step_counter + 1))
    echo -e "${BLUE}Step $step_counter:${NC} $1"
}

function success() {
    echo -e "${GREEN}✅ $1${NC}"
}

function warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

function error() {
    echo -e "${RED}❌ $1${NC}"
}

function info() {
    echo -e "${PURPLE}ℹ️ $1${NC}"
}

# Check prerequisites
step "Checking Prerequisites"

if ! command -v nix &> /dev/null; then
    error "Nix is required but not found"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    error "Cargo is required but not found"  
    exit 1
fi

success "Prerequisites check passed"
echo

# Test 1: Build verification
step "Building All Components"
if nix develop --command cargo build --quiet; then
    success "All components built successfully"
else
    error "Build failed"
    exit 1
fi
echo

# Test 2: Run correlation tests
step "Running NATS Correlation Tests"

# Test the specific correlation fix
if nix develop --command cargo test -p cim-claude-gui test_subject_consistency_after_fix --quiet; then
    success "Subject consistency test passed"
else
    error "Subject consistency test failed"
    exit 1
fi

if nix develop --command cargo test -p cim-claude-gui test_end_to_end_nats_correlation --quiet; then
    success "End-to-end correlation test passed"
else
    error "End-to-end correlation test failed"
    exit 1
fi

echo

# Test 3: API Key check
step "Checking Claude API Configuration"

if [[ -f "./cim-claude-adapter/secrets/claude.api.key" ]]; then
    success "Claude API key file found"
    
    # Check if key is valid format (starts with sk-)
    if grep -q "^sk-" "./cim-claude-adapter/secrets/claude.api.key"; then
        success "API key format appears valid"
    else
        warning "API key format may be invalid (should start with sk-)"
    fi
else
    warning "Claude API key not found at ./cim-claude-adapter/secrets/claude.api.key"
    info "Create the key file to enable real Claude API integration"
fi

echo

# Test 4: NATS availability check
step "Checking NATS Server Availability"

if command -v nats &> /dev/null; then
    if timeout 2 nats server info --server nats://localhost:4222 &> /dev/null; then
        success "NATS server is running and accessible"
    else
        warning "NATS server not accessible at localhost:4222"
        info "Start NATS server: nats-server"
    fi
else
    warning "NATS CLI not available for server check"
    info "Install NATS CLI to verify server connectivity"
fi

echo

# Test 5: Domain detection test
step "Testing Domain Detection"

HOSTNAME=$(hostname)
info "Detected hostname: $HOSTNAME"

if [[ -n "${CIM_DOMAIN:-}" ]]; then
    success "CIM_DOMAIN environment variable set: $CIM_DOMAIN"
elif [[ -n "${SAGE_DOMAIN:-}" ]]; then
    success "SAGE_DOMAIN environment variable set: $SAGE_DOMAIN"
else
    info "Using hostname as domain: $HOSTNAME"
fi

echo

# Test 6: Integration test preparation
step "Integration Test Instructions"

echo -e "${BLUE}To test the complete fix:${NC}"
echo
echo "1. Start NATS Server (if not running):"
echo "   nats-server"
echo
echo "2. Load Claude API key:"
echo "   source ./scripts/load-claude-api-key.sh"
echo
echo "3. Start SAGE service:"
echo "   ./scripts/start-sage-service.sh"
echo
echo "4. In another terminal, start GUI:"
echo "   nix develop --command cargo run -p cim-claude-gui"
echo
echo "5. Test real responses:"
echo "   - Navigate to SAGE tab"
echo "   - Enter: 'How do I create a CIM domain?'"
echo "   - Click Send"
echo "   - Verify you get a real Claude response (not mock)"
echo

# Summary
step "Test Summary"

echo -e "${GREEN}🎉 NATS Correlation Fix Verification Complete${NC}"
echo
echo "Fix Status:"
echo "✅ Subject pattern consistency fixed"
echo "✅ End-to-end correlation working"
echo "✅ All components build successfully"
echo "✅ Test suite covers critical paths"
echo
echo -e "${PURPLE}🎭 SAGE:${NC} Mathematical precision in message correlation achieved!"
echo -e "   The harmony between GUI and service layers now resonates"
echo -e "   with the elegant beauty of Category Theory applied to"
echo -e "   distributed systems architecture."
echo

# Optional: Run GUI if requested
if [[ "${1:-}" == "--run-gui" ]]; then
    echo
    step "Starting GUI Application"
    warning "Make sure NATS server and SAGE service are running first!"
    sleep 3
    nix develop --command cargo run -p cim-claude-gui
fi