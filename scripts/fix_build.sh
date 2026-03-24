#!/bin/bash

# Build Issue Resolution Script
# Attempts to resolve compilation issues across platforms

set -e

# Project root (auto-detect from script location)
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🔧 AI Security RelayNode Build Issue Resolution"
echo "================================================"
echo ""
echo "📂 Working in: $PROJECT_ROOT"
echo ""

OS_TYPE="$(uname -s)"
echo "🖥️  Platform: $OS_TYPE"
echo ""

# Platform-specific environment checks
if [[ "$OS_TYPE" == "Darwin" ]]; then
    echo "🍎 MACOS DEVELOPMENT ENVIRONMENT CHECK"
    echo "──────────────────────────────────────"

    if xcode-select -p &> /dev/null; then
        echo "✅ Xcode command line tools installed at: $(xcode-select -p)"
    else
        echo "❌ Xcode command line tools not installed"
        echo "Run: xcode-select --install"
        exit 1
    fi

    MACOS_SDK=$(xcrun --show-sdk-path 2>/dev/null || echo "Not found")
    if [[ "$MACOS_SDK" != "Not found" ]]; then
        echo "✅ macOS SDK found: $MACOS_SDK"
        export SDKROOT="$MACOS_SDK"
        export CPATH="$SDKROOT/usr/include"
        export LIBRARY_PATH="$SDKROOT/usr/lib"
    else
        echo "❌ macOS SDK not found"
    fi
    echo ""
elif [[ "$OS_TYPE" == "Linux" ]]; then
    echo "🐧 LINUX DEVELOPMENT ENVIRONMENT CHECK"
    echo "──────────────────────────────────────"

    if command -v gcc &> /dev/null; then
        echo "✅ GCC found: $(gcc --version | head -1)"
    else
        echo "❌ GCC not found — install with: sudo apt install build-essential"
    fi

    if pkg-config --exists openssl 2>/dev/null; then
        echo "✅ OpenSSL development headers found"
    else
        echo "⚠️  OpenSSL dev headers may be missing — try: sudo apt install libssl-dev pkg-config"
    fi
    echo ""
fi

# Check Rust toolchain
echo "🦀 RUST TOOLCHAIN CHECK"
echo "───────────────────────"

if ! command -v cargo &> /dev/null; then
    echo "❌ cargo not found"
    echo "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ cargo $(cargo --version | cut -d' ' -f2)"
echo "✅ rustc $(rustc --version | cut -d' ' -f2)"
echo ""

# Build resolution attempts
echo "🚀 BUILD RESOLUTION ATTEMPTS"
echo "─────────────────────────────"

# Attempt 1: Clean build
echo "Attempt 1: Clean build"
cargo clean
if cargo check 2>/dev/null; then
    echo "✅ Clean build successful"
    exit 0
else
    echo "❌ Clean build failed"
fi
echo ""

# Attempt 2: Update dependencies
echo "Attempt 2: Update dependencies"
cargo update
if cargo check 2>/dev/null; then
    echo "✅ Updated dependencies build successful"
    exit 0
else
    echo "❌ Updated dependencies build failed"
fi
echo ""

# Attempt 3: Try stable toolchain
echo "Attempt 3: Try stable Rust toolchain"
if command -v rustup &> /dev/null; then
    CURRENT_TOOLCHAIN=$(rustup show active-toolchain | cut -d' ' -f1)
    echo "Current toolchain: $CURRENT_TOOLCHAIN"

    if rustup toolchain list | grep -q "stable"; then
        rustup default stable
        if cargo check 2>/dev/null; then
            echo "✅ Stable toolchain build successful"
            exit 0
        else
            echo "❌ Stable toolchain build failed"
            rustup default "$CURRENT_TOOLCHAIN"
        fi
    fi
else
    echo "⚠️  rustup not available, skipping toolchain switch"
fi
echo ""

# Show errors for diagnostics
echo "🔴 BUILD RESOLUTION FAILED"
echo "───────────────────────────"
echo "Showing build errors for diagnosis:"
echo ""
cargo check 2>&1 | head -40
echo ""
echo "MANUAL RESOLUTION STEPS:"
echo "1. Review the errors above"
echo "2. Ensure all system dependencies are installed"
echo "3. Check Tauri prerequisites: https://tauri.app/v1/guides/getting-started/prerequisites"
echo ""
exit 1
