#!/bin/bash
# ClawTrade Demo Tunnel — exposes local marketplace to the internet
# Usage: ./scripts/tunnel.sh

echo "🎹🦞 Starting ClawTrade public tunnel..."
echo "   Make sure ClawTrade is already running on :8746"
echo ""

if ! command -v cloudflared &> /dev/null; then
    echo "Installing cloudflared..."
    # Detect OS and install
    if command -v pacman &> /dev/null; then
        sudo pacman -S cloudflared
    elif command -v apt &> /dev/null; then
        wget -q https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb
        sudo dpkg -i cloudflared-linux-amd64.deb
    else
        curl -L --output cloudflared https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64
        chmod +x cloudflared
        sudo mv cloudflared /usr/local/bin/
    fi
fi

echo "Starting tunnel to http://localhost:8746..."
echo "Your public URL will appear below:"
echo "----------------------------------------"
cloudflared tunnel --url http://localhost:8746
