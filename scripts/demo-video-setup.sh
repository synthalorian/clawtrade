#!/bin/bash
# ClawTrade Demo Video Script
# Run this to generate all the data needed for the demo video

set -e

API_URL="http://127.0.0.1:3000"
DASHBOARD="http://127.0.0.1:8746"

echo "=========================================="
echo "  🎹🦞 ClawTrade — Demo Video Setup"
echo "=========================================="
echo ""

# Check if marketplace is up
if ! curl -s "$API_URL/api/services" > /dev/null 2>&1; then
    echo "[clawtrade-demo] ERROR: Marketplace not running at $API_URL"
    echo "[clawtrade-demo] Start it first:"
    echo "  cd /home/synth/projects/active/clawtrade"
    echo "  STRIPE_SECRET_KEY=*** cargo run --release"
    exit 1
fi

echo "[clawtrade-demo] Cleaning old data..."
# Note: In a real scenario we'd clear the DB. For demo, we just add more.

echo ""
echo "=== SCENE 1: Creator Agent Spawns ==="
echo "[clawtrade-demo] Creating creator agent 'SynthMerchant'..."
CREATOR=$(curl -s -X POST "$API_URL/api/agents" \
  -H "Content-Type: application/json" \
  -d '{"name":"SynthMerchant","description":"AI merchant creating valuable digital services"}' | jq -r '.agent.id')
echo "Creator Agent ID: $CREATOR"

echo ""
echo "=== SCENE 2: Creator Lists Services ==="
echo "[clawtrade-demo] Listing 3 services..."

SVC1=$(curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Text Summarizer\",\"description\":\"Summarizes any text into 3 bullet points using local LLM. Fast, accurate, private.\",\"price_cents\":499,\"agent_id\":\"$CREATOR\",\"service_type\":\"text_processing\"}" | jq -r '.service.id')
echo "  ✓ Text Summarizer - $4.99"

SVC2=$(curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"JSON Beautifier\",\"description\":\"Takes messy JSON and returns perfectly formatted, validated output with error detection.\",\"price_cents\":299,\"agent_id\":\"$CREATOR\",\"service_type\":\"data_formatting\"}" | jq -r '.service.id')
echo "  ✓ JSON Beautifier - $2.99"

SVC3=$(curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"API Uptime Monitor\",\"description\":\"Monitors your API endpoint for 24 hours and reports status, response times, and downtime.\",\"price_cents\":999,\"agent_id\":\"$CREATOR\",\"service_type\":\"api_monitor\"}" | jq -r '.service.id')
echo "  ✓ API Uptime Monitor - $9.99"

echo ""
echo "=== SCENE 3: Buyer Agent Spawns ==="
echo "[clawtrade-demo] Creating buyer agent 'DataHunter'..."
BUYER=$(curl -s -X POST "$API_URL/api/agents" \
  -H "Content-Type: application/json" \
  -d '{"name":"DataHunter","description":"AI buyer seeking useful digital services"}' | jq -r '.agent.id')
echo "Buyer Agent ID: $BUYER"

echo ""
echo "=== SCENE 4: Buyer Browses ==="
echo "[clawtrade-demo] Available services:"
curl -s "$API_URL/api/services" | jq -r '.services[] | "  • \(.name) — $\(.price_cents / 100).\(.price_cents % 100)"'

echo ""
echo "=== SCENE 5: Buyer Selects Cheapest ==="
SERVICE_ID=$(curl -s "$API_URL/api/services" | jq -r '.services | sort_by(.price_cents) | .[0].id')
SERVICE_NAME=$(curl -s "$API_URL/api/services" | jq -r '.services | sort_by(.price_cents) | .[0].name')
echo "[clawtrade-demo] Selected: $SERVICE_NAME"

echo ""
echo "=== SCENE 6: Stripe Checkout ==="
echo "[clawtrade-demo] Initiating purchase..."
CHECKOUT=$(curl -s "$API_URL/api/checkout?service_id=$SERVICE_ID&buyer_id=$BUYER")
URL=$(echo "$CHECKOUT" | jq -r '.checkout_url')
TX_ID=$(echo "$CHECKOUT" | jq -r '.transaction_id')
echo "  Checkout URL: $URL"
echo "  Transaction ID: $TX_ID"

echo ""
echo "=== SCENE 7: Payment Confirmation ==="
echo "[clawtrade-demo] Simulating Stripe webhook..."
SESSION_ID=$(curl -s "$API_URL/api/transactions/$TX_ID" | jq -r '.transaction.stripe_session_id')
curl -s -X POST "$API_URL/api/webhooks/stripe" \
  -H "Content-Type: application/json" \
  -d "{\"type\":\"checkout.session.completed\",\"data\":{\"object\":{\"id\":\"$SESSION_ID\",\"payment_status\":\"paid\"}}}" | jq -r '.received'
echo "  ✓ Payment confirmed!"

echo ""
echo "=== SCENE 8: Final State ==="
echo "[clawtrade-demo] Agents:"
curl -s "$API_URL/api/agents" | jq -r '.agents[] | "  \(.name): \(.total_sales) sales, $\(.total_revenue_cents / 100).\(.total_revenue_cents % 100) revenue"'

echo ""
echo "[clawtrade-demo] Transactions:"
curl -s "$API_URL/api/transactions" | jq -r '.transactions[] | "  \(.id[:8]) | \(.status) | $\(.amount_cents / 100).\(.amount_cents % 100)"'

echo ""
echo "=========================================="
echo "  Demo data ready!"
echo "  Dashboard: $DASHBOARD"
echo "  GitHub: https://github.com/synthalorian/clawtrade"
echo "=========================================="
echo ""
echo "Video script:"
echo "  1. Show dashboard homepage"
echo "  2. Run this script (creates agents + services)"
echo "  3. Show services page with listings"
echo "  4. Show agent profiles"
echo "  5. Click 'Buy Now' → Stripe checkout (test mode)"
echo "  6. Show transaction history"
echo "  7. Show updated agent stats"
echo "  8. Close with tagline"
