#!/usr/bin/env bash
# SAGE-Orchestrated Claude Code Wrapper
# Ensures OpenSSL compatibility for Claude Code execution
# Mathematical OpenSSL version resolution through Nix derivation paths

set -euo pipefail

# Detect current development shell environment
if [[ -n "${IN_NIX_SHELL:-}" ]]; then
    echo "🧠 SAGE: Running Claude Code within Nix development shell"
    
    # Use development shell Node.js with compatible OpenSSL
    if [[ -n "${OPENSSL_LIB_DIR:-}" ]]; then
        echo "🔒 SAGE: Using development shell OpenSSL: $OPENSSL_LIB_DIR"
        export LD_LIBRARY_PATH="${OPENSSL_LIB_DIR}:${LD_LIBRARY_PATH:-}"
    fi
    
    # Verify Node.js can load required libraries
    if command -v node >/dev/null 2>&1; then
        NODE_VERSION=$(node --version)
        echo "🟢 SAGE: Node.js version: $NODE_VERSION"
        
        # Test OpenSSL symbol resolution
        if node -e "console.log('OpenSSL compatible:', process.versions.openssl)" 2>/dev/null; then
            echo "✅ SAGE: OpenSSL compatibility verified"
        else
            echo "⚠️  SAGE: OpenSSL compatibility issue detected"
            
            # Attempt to locate compatible OpenSSL
            if [[ -d "/nix/store" ]]; then
                echo "🔍 SAGE: Searching for compatible OpenSSL in Nix store..."
                
                # Find the newest OpenSSL 3.x version in the store
                COMPATIBLE_OPENSSL=$(find /nix/store -name "openssl-3.*" -type d | sort -V | tail -1)
                
                if [[ -n "$COMPATIBLE_OPENSSL" && -d "$COMPATIBLE_OPENSSL/lib" ]]; then
                    echo "🔧 SAGE: Found compatible OpenSSL at $COMPATIBLE_OPENSSL"
                    export LD_LIBRARY_PATH="$COMPATIBLE_OPENSSL/lib:${LD_LIBRARY_PATH:-}"
                fi
            fi
        fi
    else
        echo "❌ SAGE: Node.js not found in development shell"
        exit 1
    fi
else
    echo "🔄 SAGE: Entering Nix development shell for Claude Code compatibility"
    
    # Enter the Rust development shell and re-execute this script
    cd "$(dirname "$(dirname "${BASH_SOURCE[0]}")")" || exit 1
    exec nix develop .#rust --command bash "$0" "$@"
fi

# Execute Claude Code with all arguments
echo "🚀 SAGE: Launching Claude Code..."
exec claude "$@"