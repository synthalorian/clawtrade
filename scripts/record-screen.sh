#!/bin/bash
# ClawTrade Screen Recording Helper
# Records the demo using ffmpeg + x11grab

set -e

OUTPUT_DIR="$HOME/projects/active/clawtrade/recordings"
mkdir -p "$OUTPUT_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="$OUTPUT_DIR/clawtrade_demo_$TIMESTAMP.mp4"

echo "=========================================="
echo "  🎹🦞 ClawTrade — Screen Recorder"
echo "=========================================="
echo ""
echo "This will record your screen at 1920x1080 @ 30fps"
echo "Output: $OUTPUT_FILE"
echo ""
echo "Make sure:"
echo "  1. Marketplace is running (STRIPE_SECRET_KEY=*** cargo run --release)"
echo "  2. Demo data is loaded (./scripts/demo-video-setup.sh)"
echo "  3. Dashboard is open at http://127.0.0.1:8746"
echo "  4. Browser is at 1920x1080 or larger"
echo ""
read -p "Press Enter to start recording (Ctrl+C to stop)..."

ffmpeg -f x11grab -r 30 -s 1920x1080 -i :0.0 \
  -c:v libx264 -preset fast -pix_fmt yuv420p \
  -movflags +faststart "$OUTPUT_FILE"

echo ""
echo "Recording saved: $OUTPUT_FILE"
echo ""
echo "Upload to X/Twitter and tag @NousResearch"
