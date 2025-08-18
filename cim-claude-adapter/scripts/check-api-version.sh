#!/usr/bin/env bash
# CIM Claude Adapter - API Version Check Script
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

set -euo pipefail

echo "🔍 CIM Claude Adapter - API Version Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if we're in a Nix environment
if [[ -n "${CIM_ANTHROPIC_API_VERSION:-}" ]]; then
    echo "✅ Nix Environment Detected"
    echo "🔒 Hard-locked API Version: $CIM_ANTHROPIC_API_VERSION"
    VERSION_SOURCE="Nix-locked"
else
    echo "⚠️  Development Environment (no Nix lock)"
    echo "📋 Using fallback version: 2023-06-01"
    VERSION_SOURCE="fallback"
fi

echo ""
echo "📊 Version Information:"
echo "  Source: $VERSION_SOURCE"
echo "  Version: ${CIM_ANTHROPIC_API_VERSION:-2023-06-01}"
echo ""

# Test with cargo if available
if command -v cargo >/dev/null 2>&1; then
    echo "🧪 Testing with example..."
    echo ""
    
    # Set the version for testing if not in Nix
    if [[ -z "${CIM_ANTHROPIC_API_VERSION:-}" ]]; then
        export CIM_ANTHROPIC_API_VERSION="2023-06-01"
        echo "🔧 Set CIM_ANTHROPIC_API_VERSION for testing"
    fi
    
    # Run the example
    cargo run --example api_key_usage 2>/dev/null | grep -E "(API Version|anthropic-version)" || true
else
    echo "❌ Cargo not available - skipping test"
fi

echo ""
echo "📋 To update the API version:"
echo "  1. Edit flake.nix: anthropicApiVersion = \"new-version\""
echo "  2. Rebuild: nix build"
echo "  3. Verify: nix run . -- --version"
echo ""

# Check flake version if available
if [[ -f "flake.nix" ]]; then
    FLAKE_VERSION=$(grep -E 'anthropicApiVersion\s*=' flake.nix | sed -E 's/.*"([^"]+)".*/\1/' || echo "unknown")
    echo "📁 Flake Configuration: $FLAKE_VERSION"
    
    if [[ -n "${CIM_ANTHROPIC_API_VERSION:-}" ]]; then
        if [[ "$CIM_ANTHROPIC_API_VERSION" == "$FLAKE_VERSION" ]]; then
            echo "✅ Version consistency verified"
        else
            echo "⚠️  Version mismatch detected!"
            echo "   Environment: $CIM_ANTHROPIC_API_VERSION"
            echo "   Flake: $FLAKE_VERSION"
        fi
    fi
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"