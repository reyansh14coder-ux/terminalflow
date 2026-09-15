#!/bin/bash

set -e

echo "🚀 Installing TerminalFlow..."

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

BINARY_URL=""

case $OS in
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_URL="https://github.com/reyansh14coder-ux/terminalflow/releases/latest/download/terminalflow-linux-amd64"
        elif [ "$ARCH" = "aarch64" ]; then
            BINARY_URL="https://github.com/reyansh14coder-ux/terminalflow/releases/latest/download/terminalflow-linux-arm64"
        fi
        ;;
    Darwin)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_URL="https://github.com/reyansh14coder-ux/terminalflow/releases/latest/download/terminalflow-macos-amd64"
        elif [ "$ARCH" = "arm64" ]; then
            BINARY_URL="https://github.com/reyansh14coder-ux/terminalflow/releases/latest/download/terminalflow-macos-arm64"
        fi
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

if [ -z "$BINARY_URL" ]; then
    echo "❌ Unsupported architecture: $ARCH"
    echo ""
    echo "You can build from source instead:"
    echo "  git clone https://github.com/reyansh14coder-ux/terminalflow.git"
    echo "  cd terminalflow"
    echo "  cargo build --release"
    echo "  sudo cp target/release/terminalflow /usr/local/bin/"
    exit 1
fi

echo "📥 Downloading TerminalFlow from: $BINARY_URL"

if ! curl -fsSL --fail --connect-timeout 30 --max-time 120 "$BINARY_URL" -o /tmp/terminalflow; then
    echo "❌ Download failed"
    echo ""
    echo "This may be because no release binaries are available yet."
    echo "You can build from source instead:"
    echo "  git clone https://github.com/reyansh14coder-ux/terminalflow.git"
    echo "  cd terminalflow"
    echo "  cargo build --release"
    echo "  sudo cp target/release/terminalflow /usr/local/bin/"
    exit 1
fi

# Verify download
FILE_SIZE=$(stat -f%z /tmp/terminalflow 2>/dev/null || stat -c%s /tmp/terminalflow 2>/dev/null || echo 0)
if [ "$FILE_SIZE" -lt 1048576 ]; then
    echo "❌ Downloaded file is missing or too small"
    rm -f /tmp/terminalflow
    exit 1
fi

echo "📦 Installing..."
chmod +x /tmp/terminalflow
sudo mv /tmp/terminalflow /usr/local/bin/terminalflow

echo "✅ TerminalFlow installed successfully!"
echo ""
echo "🚀 Run 'terminalflow' or 'tf' to start!"
echo "📚 Documentation: https://github.com/reyansh14coder-ux/terminalflow#readme"
