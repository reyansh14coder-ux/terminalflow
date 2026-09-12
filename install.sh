#!/bin/bash

set -e

echo "🚀 Installing TerminalFlow..."

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

case $OS in
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_URL="https://github.com/YOUR_USERNAME/terminalflow/releases/latest/download/terminalflow-linux-amd64"
        elif [ "$ARCH" = "aarch64" ]; then
            BINARY_URL="https://github.com/YOUR_USERNAME/terminalflow/releases/latest/download/terminalflow-linux-arm64"
        fi
        ;;
    Darwin)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_URL="https://github.com/YOUR_USERNAME/terminalflow/releases/latest/download/terminalflow-macos-amd64"
        elif [ "$ARCH" = "arm64" ]; then
            BINARY_URL="https://github.com/YOUR_USERNAME/terminalflow/releases/latest/download/terminalflow-macos-arm64"
        fi
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "📥 Downloading TerminalFlow..."
curl -fsSL "$BINARY_URL" -o /tmp/terminalflow

echo "📦 Installing..."
chmod +x /tmp/terminalflow
sudo mv /tmp/terminalflow /usr/local/bin/terminalflow

echo "✅ TerminalFlow installed successfully!"
echo ""
echo "🚀 Run 'terminalflow' or 'tf' to start!"
echo "📚 Documentation: https://github.com/YOUR_USERNAME/terminalflow#readme"
