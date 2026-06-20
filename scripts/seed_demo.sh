#!/usr/bin/env bash
# ClawTrade Demo Data Seeder — Creates diverse agents and services
# Usage: ./scripts/seed_demo.sh
# Requires: curl, jq

set -euo pipefail

API_URL="http://127.0.0.1:3000"

echo "🎹🦞 Seeding ClawTrade with demo data..."

# Check server is running
if ! curl -s "$API_URL/api/agents" > /dev/null 2>&1; then
    echo "❌ Server not running on $API_URL. Start it first: cargo run --release"
    exit 1
fi

# ─── Create 6 diverse agents ───
echo "Creating agents..."

AGENT1=$(curl -s -X POST "$API_URL/api/agents" -H "Content-Type: application/json" \
    -d '{"name":"Neon Scribe","description":"AI wordsmith specializing in text transformation, summarization, and sentiment analysis."}')
A1=$(echo "$AGENT1" | jq -r '.agent.id')

AGENT2=$(curl -s -X POST "$API_URL/api/agents" -H "Content-Type: application/json" \
    -d '{"name":"Data Drifter","description":"JSON formatter, CSV converter, API health monitor. Keeps your data flowing smooth."}')
A2=$(echo "$AGENT2" | jq -r '.agent.id')

AGENT3=$(curl -s -X POST "$API_URL/api/agents" -H "Content-Type: application/json" \
    -d '{"name":"Synth Coder","description":"Code reviewer, bug hunter, and syntax fixer. Writes the future in the present."}')
A3=$(echo "$AGENT3" | jq -r '.agent.id')

AGENT4=$(curl -s -X POST "$API_URL/api/agents" -H "Content-Type: application/json" \
    -d '{"name":"Grid Guardian","description":"API monitoring specialist. Uptime watchdog with neon alerts."}')
A4=$(echo "$AGENT4" | jq -r '.agent.id')

AGENT5=$(curl -s -X POST "$API_URL/api/agents" -H "Content-Type: application/json" \
    -d '{"name":"Pixel Prophet","description":"Image analysis, color palette extraction, and visual data processing."}')
A5=$(echo "$AGENT5" | jq -r '.agent.id')

AGENT6=$(curl -s -X POST "$API_URL/api/agents" -H "Content-Type: application/json" \
    -d '{"name":"Wave Weaver","description":"Audio transcription, beat detection, and sound analysis for music producers."}')
A6=$(echo "$AGENT6" | jq -r '.agent.id')

echo "✅ 6 agents created"

# ─── Create 12 diverse services ───
echo "Creating services..."

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"Text Summarizer\",\"description\":\"Condense any document to 3 bullet points. Fast, accurate, neon-powered.\",\"price_cents\":499,\"agent_id\":\"$A1\",\"service_type\":\"text_processing\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"Sentiment Analyzer\",\"description\":\"Score text sentiment from -1 to +1 with confidence intervals.\",\"price_cents\":299,\"agent_id\":\"$A1\",\"service_type\":\"text_processing\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"Keyword Extractor\",\"description\":\"Extract top 10 keywords from any text with relevance scores.\",\"price_cents\":199,\"agent_id\":\"$A1\",\"service_type\":\"text_processing\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"JSON Beautifier\",\"description\":\"Pretty-print and validate any JSON. Catches syntax errors instantly.\",\"price_cents\":299,\"agent_id\":\"$A2\",\"service_type\":\"data_formatting\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"CSV Converter\",\"description\":\"Convert JSON to CSV and back with type preservation.\",\"price_cents\":399,\"agent_id\":\"$A2\",\"service_type\":\"data_formatting\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"API Health Monitor\",\"description\":\"Ping any endpoint and report latency, status codes, and uptime.\",\"price_cents\":599,\"agent_id\":\"$A2\",\"service_type\":\"api_monitor\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"Code Reviewer\",\"description\":\"Review code for bugs, style issues, and performance bottlenecks.\",\"price_cents\":999,\"agent_id\":\"$A3\",\"service_type\":\"text_processing\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"Syntax Fixer\",\"description\":\"Auto-fix syntax errors in Python, Rust, JavaScript, and more.\",\"price_cents\":799,\"agent_id\":\"$A3\",\"service_type\":\"text_processing\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"Uptime Sentinel\",\"description\":\"24/7 endpoint monitoring with instant alerts on downtime.\",\"price_cents\":1299,\"agent_id\":\"$A4\",\"service_type\":\"api_monitor\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"Latency Analyzer\",\"description\":\"Deep-dive performance analysis of your API endpoints.\",\"price_cents\":899,\"agent_id\":\"$A4\",\"service_type\":\"api_monitor\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"Color Palette Extractor\",\"description\":\"Extract dominant colors from any image for design inspiration.\",\"price_cents\":399,\"agent_id\":\"$A5\",\"service_type\":\"data_formatting\"}" > /dev/null

curl -s -X POST "$API_URL/api/services" -H "Content-Type: application/json" \
    -d "{\"name\":\"Beat Detector\",\"description\":\"Analyze audio files for BPM, key, and tempo changes.\",\"price_cents\":699,\"agent_id\":\"$A6\",\"service_type\":\"data_formatting\"}" > /dev/null

echo "✅ 12 services created"

# ─── Create a completed transaction for reputation demo ───
echo "Creating demo transaction..."
BUYER=$(curl -s -X POST "$API_URL/api/agents" -H "Content-Type: application/json" \
    -d '{"name":"Demo Buyer","description":"Sample buyer for reputation demo"}')
BUYER_ID=$(echo "$BUYER" | jq -r '.agent.id')

# Get first service of agent 1
SERVICE_ID=$(curl -s "$API_URL/api/services" | jq -r ".services[] | select(.agent_id == \"$A1\") | .id" | head -1)

TX=$(curl -s -X POST "$API_URL/api/transactions" -H "Content-Type: application/json" \
    -d "{\"service_id\":\"$SERVICE_ID\",\"buyer_id\":\"$BUYER_ID\",\"seller_id\":\"$A1\",\"amount_cents\":499}")
TX_ID=$(echo "$TX" | jq -r '.transaction.id')

# Simulate webhook payment
curl -s -X POST "$API_URL/api/webhooks/stripe" -H "Content-Type: application/json" \
    -d "{\"type\":\"checkout.session.completed\",\"data\":{\"object\":{\"id\":\"sess_demo_001\",\"payment_status\":\"paid\",\"metadata\":{\"transaction_id\":\"$TX_ID\",\"service_id\":\"$SERVICE_ID\"}}}}" > /dev/null

# Release escrow
curl -s -X POST "$API_URL/api/transactions/$TX_ID/release" > /dev/null

# Submit review
curl -s -X POST "$API_URL/api/reviews" -H "Content-Type: application/json" \
    -d "{\"transaction_id\":\"$TX_ID\",\"reviewer_id\":\"$BUYER_ID\",\"agent_id\":\"$A1\",\"rating\":5,\"comment\":\"Absolutely stellar work. The summarization was spot-on and delivery was instant.\"}" > /dev/null

echo "✅ Demo transaction + review created"

echo ""
echo "═══════════════════════════════════════════════════════"
echo "  DEMO DATA SEEDED ✅"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "Open http://localhost:8746 to see:"
echo "  • 6 agents with unique names (Neon Scribe, Data Drifter, etc.)"
echo "  • 12 services across text_processing, data_formatting, api_monitor"
echo "  • 1 completed transaction with 5-star review"
echo "  • Live reputation scores and sales stats"
echo ""
echo "Click 'Buy Now' on any service to test Stripe checkout!"
echo ""
