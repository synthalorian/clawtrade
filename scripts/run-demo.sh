#!/bin/bash
# ClawTrade Full Demo — Creator + Buyer Agents
# Run this after starting the marketplace server

set -e

API_URL="http://127.0.0.1:3000"
DASHBOARD="http://127.0.0.1:8746"

echo "=========================================="
echo "  🎹🦞 ClawTrade — Full Marketplace Demo"
echo "=========================================="
echo ""
echo "This script simulates two Hermes agents:"
echo "  1. Creator agent — creates services"
echo "  2. Buyer agent — discovers and purchases"
echo ""
echo "Make sure the marketplace is running:"
echo "  cd /home/synth/projects/active/clawtrade"
echo "  STRIPE_SECRET_KEY=*** cargo run"
echo ""
echo "Dashboard: $DASHBOARD"
echo ""

# Check if marketplace is up
if ! curl -s "$API_URL/api/services" > /dev/null 2>&1; then
    echo "[clawtrade-demo] ERROR: Marketplace not running at $API_URL"
    echo "[clawtrade-demo] Start it first with: STRIPE_SECRET_KEY=*** cargo run"
    exit 1
fi

echo "[clawtrade-demo] Step 1: Creating creator agent..."
CREATOR=$(curl -s -X POST "$API_URL/api/agents" \
  -H "Content-Type: application/json" \
  -d '{"name":"SynthMerchant","description":"AI merchant creating valuable digital services"}' | jq -r '.agent.id')
echo "[clawtrade-demo] Creator agent: $CREATOR"

echo "[clawtrade-demo] Step 2: Creator listing services..."
SVC1=$(curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Text Summarizer\",\"description\":\"Summarizes any text into 3 bullet points using local LLM.\",\"price_cents\":499,\"agent_id\":\"$CREATOR\",\"service_type\":\"text_processing\"}" | jq -r '.service.id')
echo "[clawtrade-demo]   - Text Summarizer ($4.99) -> $SVC1"

SVC2=$(curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"JSON Beautifier\",\"description\":\"Formats and validates messy JSON.\",\"price_cents\":299,\"agent_id\":\"$CREATOR\",\"service_type\":\"data_formatting\"}" | jq -r '.service.id')
echo "[clawtrade-demo]   - JSON Beautifier ($2.99) -> $SVC2"

SVC3=$(curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"API Uptime Monitor\",\"description\":\"24-hour API monitoring with status reports.\",\"price_cents\":999,\"agent_id\":\"$CREATOR\",\"service_type\":\"api_monitor\"}" | jq -r '.service.id')
echo "[clawtrade-demo]   - API Uptime Monitor ($9.99) -> $SVC3"

echo "[clawtrade-demo] Step 3: Creating buyer agent..."
BUYER=$(curl -s -X POST "$API_URL/api/agents" \
  -H "Content-Type: application/json" \
  -d '{"name":"DataHunter","description":"AI buyer seeking useful services"}' | jq -r '.agent.id')
echo "[clawtrade-demo] Buyer agent: $BUYER"

echo "[clawtrade-demo] Step 4: Buyer browsing marketplace..."
SERVICES=$(curl -s "$API_URL/api/services")
echo "$SERVICES" | jq -r '.services[] | "  - \(.name): $\(.price_cents / 100).\(.price_cents % 100 | tostring | if length == 1 then "0" + . else . end)"'

echo "[clawtrade-demo] Step 5: Buyer selecting cheapest service..."
SERVICE_ID=$(echo "$SERVICES" | jq -r '.services | sort_by(.price_cents) | .[0].id')
SERVICE_NAME=$(echo "$SERVICES" | jq -r '.services | sort_by(.price_cents) | .[0].name')
echo "[clawtrade-demo] Selected: $SERVICE_NAME"

echo "[clawtrade-demo] Step 6: Initiating purchase..."
CHECKOUT=$(curl -s "$API_URL/api/checkout?service_id=$SERVICE_ID\&buyer_id=$BUYER")

if echo "$CHECKOUT" | jq -e '.checkout_url' > /dev/null 2>&1; then
    URL=$(echo "$CHECKOUT" | jq -r '.checkout_url')
    TX_ID=$(echo "$CHECKOUT" | jq -r '.transaction_id')
    echo "[clawtrade-demo] Checkout URL: $URL"
    echo "[clawtrade-demo] Transaction ID: $TX_ID"
    echo ""
    echo "[clawtrade-demo] Step 7: Simulate webhook payment confirmation..."
    curl -s -X POST "$API_URL/api/webhooks/stripe" \
      -H "Content-Type: application/json" \
      -d "{\"type\":\"checkout.session.completed\",\"data\":{\"object\":{\"id\":\"$(curl -s "$API_URL/api/transactions/$TX_ID" | jq -r '.transaction.stripe_session_id')\",\"payment_status\":\"paid\"}}}" | jq .
    echo "[clawtrade-demo] Transaction marked as paid!"
else
    echo "[clawtrade-demo] Checkout failed (Stripe key missing?):"
    echo "$CHECKOUT" | jq .
fi

echo ""
echo "[clawtrade-demo] Step 8: Final state..."
echo "Agents:"
curl -s "$API_URL/api/agents" | jq -r '.agents[] | "  \(.name): \(.total_sales) sales, $\(.total_revenue_cents / 100).\(.total_revenue_cents % 100) revenue"'
echo "Transactions:"
curl -s "$API_URL/api/transactions" | jq -r '.transactions[] | "  \(.id[:8]) | \(.status) | $\(.amount_cents / 100).\(.amount_cents % 100)"'

echo ""
echo "=========================================="
echo "  Demo complete! Open dashboard:"
echo "  $DASHBOARD"
echo "=========================================="
