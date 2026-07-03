#!/usr/bin/env bash
# install.sh — Install HighHarness
# Usage: curl -fsSL https://raw.githubusercontent.com/MAHADEV369/HighHarness/main/scripts/install.sh | bash
set -euo pipefail

echo "╔══════════════════════════════════════════╗"
echo "║     Installing HighHarness               ║"
echo "╚══════════════════════════════════════════╝"

# Check for cargo
if command -v cargo &>/dev/null; then
    echo "📦 Installing via cargo..."
    cargo install highharness 2>/dev/null && {
        echo "   ✅ Installed via cargo"
        echo "   Run 'HighHarness --help' to get started"
        exit 0
    } || {
        echo "   ⚠️  cargo install failed, trying from source..."
        # Fall through to source install
    }
fi

# Install from source
if command -v git &>/dev/null && command -v cargo &>/dev/null; then
    TMPDIR=$(mktemp -d)
    echo "📦 Cloning from GitHub..."
    git clone --depth=1 https://github.com/MAHADEV369/HighHarness.git "$TMPDIR" 2>/dev/null
    cd "$TMPDIR"
    echo "🔨 Building..."
    cargo build --release 2>&1 | tail -1
    cp target/release/HighHarness /usr/local/bin/HighHarness 2>/dev/null || {
        mkdir -p "$HOME/.local/bin"
        cp target/release/HighHarness "$HOME/.local/bin/HighHarness"
        echo "   Add ~/.local/bin to your PATH"
    }
    rm -rf "$TMPDIR"
    echo "   ✅ Installed from source"
else
    echo "❌ Need Rust installed: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo ""
echo "Quick start:"
echo "  HighHarness --help"
echo "  HighHarness mcp serve-http --port 8931"
