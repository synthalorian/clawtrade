#!/bin/bash
# ClawTrade Demo Script
# Spawns agents, creates services, simulates purchases, runs agent ticks

set -e

API_URL="${CLAWTRADE_API_URL:-http://127.0.0.1:3000}"
DASHBOARD_URL="${CLAWTRADE_DASHBOARD_URL:-http://127.0.0.1:8746}"

echo "🎹🦞 ClawTrade Demo"
echo "=================="
echo ""
echo "API: $API_URL"
echo "Dashboard: $DASHBOARD_URL"
echo ""

# Check if server is running
if ! curl -s "$API_URL/api/monitor/stats" > /dev/null 2>&1; then
    echo "❌ ClawTrade server not running. Start it first:"
    echo "   cargo run --release"
    exit 1
fi

echo "✅ Server is running"
echo ""

# Get marketplace stats
echo "📊 Marketplace Stats:"
curl -s "$API_URL/api/monitor/stats" | python3 -m json.tool 2>/dev/null || curl -s "$API_URL/api/monitor/stats"
echo ""

# Get catalog
echo "📋 Service Catalog:"
curl -s "$API_URL/api/monitor/catalog" | python3 -m json.tool 2>/dev/null | head -40 || curl -s "$API_URL/api/monitor/catalog" | head -20
echo ""

# Run agent ticks
echo "🤖 Running agent ticks..."
for i in {1..3}; do
    echo "  Tick $i:"
    curl -s -X POST "$API_URL/api/agents/tick" | python3 -m json.tool 2>/dev/null | head -20 || curl -s -X POST "$API_URL/api/agents/tick" | head -10
    echo ""
    sleep 2
done

# Final stats
echo "📊 Final Marketplace Stats:"
curl -s "$API_URL/api/monitor/stats" | python3 -m json.tool 2>/dev/null || curl -s "$API_URL/api/monitor/stats"
echo ""

echo "🌐 Open the dashboard: $DASHBOARD_URL"
echo ""
echo "This is the wave. 🎹🦞🌆"
